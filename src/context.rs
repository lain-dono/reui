use arrayvec::ArrayVec;

use crate::{
    backend::{BackendGL, Image, TEXTURE_ALPHA},
    cache::{PathCache, LineCap, LineJoin},
    counters::Counters,
    vg::*,
    font::*,
    picture::Picture,
    Point,
    Transform,
};

use std::ptr::null;
use slotmap::Key;

const INIT_COMMANDS_SIZE: usize = 256;

const INIT_FONTIMAGE_SIZE: usize = 512;
pub const MAX_FONTIMAGE_SIZE: u32 = 2048;
pub const MAX_FONTIMAGES: usize = 4;

pub struct States {
    states: ArrayVec<[State; 32]>,
}

impl States {
    pub(crate) fn new() -> Self {
        let mut states = ArrayVec::<_>::new();
        states.push(State::new());
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
        *state = State::new();
    }
}

pub struct Context {
    pub picture: Picture,

    pub states: States,
    pub cache: PathCache,
    pub device_px_ratio: f32,

    pub fs: Box<Stash>,
    pub font_images: [Image; 4],
    pub font_image_idx: i32,

    pub counters: Counters,

    pub params: BackendGL,
}

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

    pub fn add_fallback_font_id(&mut self, base: i32, fallback: i32) -> bool{
        if base == -1 || fallback == -1 {
            false
        } else {
            self.fs.add_fallback_font(base, fallback) != 0
        }
    }

    pub fn font_face(&mut self, name: &[u8]) {
        self.states.last_mut().font_id = self.fs.font_by_name(name.as_ptr());
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
    pub fn stroke_color(&mut self, color: Color) {
        self.states.last_mut().stroke.set_color(color)
    }
    pub fn fill_color(&mut self, color: Color) {
        self.states.last_mut().fill.set_color(color)
    }
    pub fn stroke_paint(&mut self, paint: Paint) {
        let state = self.states.last_mut();
        state.stroke = paint;
        state.stroke.xform = state.stroke.xform.post_mul(&state.xform);
    }
    pub fn fill_paint(&mut self, paint: Paint) {
        let state = self.states.last_mut();
        state.fill = paint;
        state.fill.xform = state.fill.xform.post_mul(&state.xform);
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
        log::trace!("draws:{}  fill:{}  stroke:{}  text:{}  TOTAL:{}",
            self.counters.draw_call_count,
            self.counters.fill_tri_count,
            self.counters.stroke_tri_count,
            self.counters.text_tri_count,

            self.counters.fill_tri_count +
            self.counters.stroke_tri_count +
            self.counters.text_tri_count,
        );

        self.states.clear();
        self.save();
        self.reset();
        self.set_dpi(dpi);

        self.params.set_viewport(width, height, dpi);

        self.counters.clear();
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
            ],

            counters: Counters::default(),
            cache: PathCache::new(),

            picture: Picture {
                commands: Vec::with_capacity(INIT_COMMANDS_SIZE),
                cmd: Point::zero(),
                xform: Transform::identity(),
            },

            device_px_ratio: 1.0,

            font_image_idx: 0,
        }
    }
}