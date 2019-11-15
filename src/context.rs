use arrayvec::ArrayVec;
use slotmap::Key;
use std::ptr::null;

use crate::{
    backend::{BackendGL, Image, ImageFlags, TEXTURE_ALPHA, TEXTURE_RGBA},
    cache::{PathCache, LineCap, LineJoin},
    vg::*,
    font::*,
    math::{Offset, Transform, Color},
    canvas::Picture,
};

const INIT_COMMANDS_SIZE: usize = 256;

const INIT_FONTIMAGE_SIZE: usize = 512;
pub const MAX_FONTIMAGE_SIZE: u32 = 2048;
pub const MAX_FONTIMAGES: usize = 6;

pub struct States {
    states: ArrayVec<[State; 32]>,
}

impl States {
    pub(crate) fn new() -> Self {
        let mut states = ArrayVec::<_>::new();
        states.push(State::default());
        Self { states }
    }
    pub(crate) fn last(&self) -> &State {
        self.states.last().expect("last state")
    }
    pub(crate) fn last_mut(&mut self) -> &mut State {
        self.states.last_mut().expect("last_mut state")
    }
    pub(crate) fn clear(&mut self) {
        self.states.clear();
    }
    fn save(&mut self) {
        if self.states.len() >= self.states.capacity() {
            return;
        }
        if let Some(state) = self.states.last().cloned() {
            self.states.push(state);
        }
    }
    fn restore(&mut self) {
        self.states.pop();
    }
    fn reset(&mut self) {
        let state = if let Some(state) = self.states.last_mut() {
            state
        } else {
            self.states.push(unsafe { std::mem::zeroed() });
            self.states.last_mut().expect("last mut state (reset)")
        };
        *state = State::default();
    }
}

pub struct Context {
    pub picture: Picture,

    pub states: States,
    pub cache: PathCache,
    pub device_px_ratio: f32,

    pub fs: Box<Stash>,
    pub font_images: [Image; MAX_FONTIMAGES],
    pub font_image_idx: i32,

    pub params: BackendGL,
}

// FIXME
unsafe impl Sync for Context {}
unsafe impl Send for Context {}

impl Context {
    pub fn save(&mut self) {
        self.states.save();
    }
    pub fn restore(&mut self) {
        self.states.restore();
    }
    pub fn reset(&mut self) {
        self.states.reset();
    }

    // State setting

    pub fn shape_anti_alias(&mut self, enabled: bool) {
        self.states.last_mut().shape_aa = enabled;
    }
    pub fn stroke_width(&mut self, width: f32) {
        self.states.last_mut().stroke_width = width;
    }
    pub fn miter_limit(&mut self, limit: f32) {
        self.states.last_mut().miter_limit = limit;
    }
    pub fn line_cap(&mut self, cap: LineCap) {
        self.states.last_mut().line_cap = cap;
    }
    pub fn line_join(&mut self, join: LineJoin) {
        self.states.last_mut().line_join = join;
    }
    pub fn global_alpha(&mut self, alpha: f32) {
        self.states.last_mut().alpha = alpha;
    }
    pub fn stroke_color(&mut self, color: u32) {
        self.states.last_mut().stroke = Paint::color(Color::new(color))
    }
    pub fn fill_color(&mut self, color: u32) {
        self.states.last_mut().fill = Paint::color(Color::new(color))
    }
    pub fn stroke_paint(&mut self, paint: Paint) {
        let state = self.states.last_mut();
        state.stroke = paint;
        state.stroke.xform.prepend_mut(state.xform);
    }
    pub fn fill_paint(&mut self, paint: Paint) {
        let state = self.states.last_mut();
        state.fill = paint;
        state.fill.xform.prepend_mut(state.xform);
    }

    pub fn font_size(&mut self, size: f32) {
        self.states.last_mut().font_size = size;
    }
    pub fn font_blur(&mut self, blur: f32) {
        self.states.last_mut().font_blur = blur;
    }
    pub fn letter_spacing(&mut self, spacing: f32) {
        self.states.last_mut().letter_spacing = spacing;
    }
    pub fn line_height(&mut self, line_height: f32) {
        self.states.last_mut().line_height = line_height;
    }
    pub fn text_align(&mut self, align: Align) {
        self.states.last_mut().text_align = align;
    }
    pub fn font_face_id(&mut self, font: i32) {
        self.states.last_mut().font_id = font;
    }
    /*
    pub fn font_face(&mut self, name: &str) {
        self.state_mut().font_id = fonsGetFontByName(self.fs, font);
    }
    */
}

impl Context {
    pub fn begin_frame(&mut self, width: f32, height: f32, dpi: f32) {
        self.states.clear();
        self.save();
        self.reset();
        self.set_dpi(dpi);

        self.params.set_viewport(width, height, dpi);
    }

    pub fn cancel_frame(&mut self) {
        self.params.reset()
    }

    pub fn end_frame(&mut self) {
        self.params.flush();

        if self.font_image_idx == 0 {
            return;
        }

        let font_image = self.font_images[self.font_image_idx as usize];

        // delete images that smaller than current one
        if font_image.is_null() {
            return;
        }

        let (iw, ih) = self.image_size(font_image).expect("font image in end_frame (1)");
        let mut j = 0;
        let font_images = self.font_images;
        for &m in &font_images {
            if !m.is_null() {
                let (nw, nh) = self.image_size(m).expect("font image in end_frame (2)");
                if nw < iw || nh < ih {
                    self.delete_image(m);
                } else {
                    self.font_images[j] = m;
                    j += 1;
                }
            }
        }

        // wtf lol?
        j = j.saturating_sub(1);

        // make current font image to first
        self.font_images[j] = self.font_images[0];
        self.font_images[0] = font_image;
        self.font_image_idx = 0;
        j += 1;

        // clear all images after j
        for m in &mut self.font_images[j..] {
            *m = Image::null();
        }
    }

    pub fn set_dpi(&mut self, ratio: f32) {
        self.cache.set_dpi(ratio);
        self.device_px_ratio = ratio;
    }

    pub fn new(mut params: BackendGL) -> Self {
        let fs = Stash::new(INIT_FONTIMAGE_SIZE as i32, INIT_FONTIMAGE_SIZE as i32);

        let font_image = params.create_texture(
            TEXTURE_ALPHA,
            INIT_FONTIMAGE_SIZE as u32,
            INIT_FONTIMAGE_SIZE as u32,
            Default::default(),
            null(),
        );

        Self {
            params, fs,

            states: States::new(),

            font_images: [
                font_image,
                Image::null(),
                Image::null(),
                Image::null(),
                Image::null(),
                Image::null(),
            ],

            cache: PathCache::new(),

            picture: Picture {
                commands: Vec::with_capacity(INIT_COMMANDS_SIZE),
                cmd: Offset::zero(),
                xform: Transform::identity(),
            },

            device_px_ratio: 1.0,

            font_image_idx: 0,
        }
    }
}

impl Context {
    pub fn current_transform(&self) -> &Transform {
        &self.states.last().xform
    }
    pub fn pre_transform(&mut self, m: Transform) {
        self.states.last_mut().xform.append_mut(m);
    }
    pub fn post_transform(&mut self, m: Transform) {
        self.states.last_mut().xform.prepend_mut(m);
    }
    pub fn reset_transform(&mut self) {
        self.states.last_mut().xform = Transform::identity();
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.pre_transform(Transform::translation(x, y));
    }
    pub fn rotate(&mut self, angle: f32) {
        self.pre_transform(Transform::rotation(angle));
    }
    pub fn scale(&mut self, scale: f32) {
        self.pre_transform(Transform::scale(scale));
    }
}

/// Images
///
/// NanoVG allows you to load jpg, png, psd, tga, pic and gif files to be used for rendering.
/// In addition you can upload your own image. The image loading is provided by stb_image.
/// The parameter imageFlags is combination of flags defined in NVGimageFlags.
impl Context {
    /// Creates image by loading it from the disk from specified file name.
    /// Returns handle to the image.
    pub fn create_image(&mut self, name: &str, flags: ImageFlags) -> Image {
        match image::open(name) {
            Ok(m) => {
                let m = m.to_rgba();
                let (w, h) = m.dimensions();
                let data = m.into_raw();
                self.create_image_rgba(w, h, flags, data.as_ptr())
            }
            Err(err) => {
                log::warn!("Failed to load image - {:?}", err);
                Image::null()
            }
        }
    }

    /// Creates image by loading it from the specified chunk of memory.
    /// Returns handle to the image.
    pub fn create_image_mem(&mut self, flags: ImageFlags, data: &[u8]) -> Image {
        match image::load_from_memory(data) {
            Ok(m) => {
                let m = m.to_rgba();
                let (w, h) = m.dimensions();
                let data = m.into_raw();
                self.create_image_rgba(w, h, flags, data.as_ptr())
            }
            Err(err) => {
                log::warn!("Failed to load image - {:?}", err);
                Image::null()
            }
        }
    }

    /// Creates image from specified image data.
    /// Returns handle to the image.
    pub fn create_image_rgba(&mut self, w: u32, h: u32, flags: ImageFlags, data: *const u8) -> Image {
        self.params.create_texture(TEXTURE_RGBA, w, h, flags, data)
    }

    /// Updates image data specified by image handle.
    pub fn update_image(&mut self, image: Image, data: &[u8]) {
        let (w, h) = self.params.texture_size(image).expect("update_image available");
        self.params.update_texture(image, 0, 0, w, h, data);
    }

    /// Returns the dimensions of a created image.
    pub fn image_size(&mut self, image: Image) -> Option<(u32, u32)> {
        self.params.texture_size(image)
    }

    /// Deletes created image.
    pub fn delete_image(&mut self, image: Image) {
        self.params.delete_texture(image);
    }
}