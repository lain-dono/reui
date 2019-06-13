use arrayvec::ArrayVec;

use crate::{
    backend::{BackendGL, Image, TEXTURE_ALPHA},
    cache::{PathCache, LineCap, LineJoin},
    vg::Counters,
    transform::Transform,
    vg::*,
    font::*,
    vg::utils::raw_str,
};

use std::ptr::null;
use slotmap::Key;

const INIT_COMMANDS_SIZE: usize = 256;

const INIT_FONTIMAGE_SIZE: usize = 512;
pub const MAX_FONTIMAGE_SIZE: u32 = 2048;
pub const MAX_FONTIMAGES: usize = 4;

bitflags::bitflags!(
    pub struct Align: i32 {
        const LEFT      = 1;       // Default, align text horizontally to left.
        const CENTER    = 1<<1;    // Align text horizontally to center.
        const RIGHT     = 1<<2;    // Align text horizontally to right.

        const TOP       = 1<<3;    // Align text vertically to top.
        const MIDDLE    = 1<<4;    // Align text vertically to middle.
        const BOTTOM    = 1<<5;    // Align text vertically to bottom.
        const BASELINE  = 1<<6; // Default, align text vertically to baseline.
    }
);

#[derive(Clone, Copy)]
pub struct TextRow {
    pub start: *const u8,   // Pointer to the input text where the row starts.
    pub end: *const u8,     // Pointer to the input text where the row ends (one past the last character).
    pub next: *const u8,    // Pointer to the beginning of the next row.
    pub width: f32,         // Logical width of the row.

    // Actual bounds of the row.
    // Logical with and bounds can differ because of kerning and some parts over extending.
    pub minx: f32,
    pub maxx: f32,
}

impl TextRow {
    pub fn text(&self) -> &str {
        unsafe { raw_str(self.start, self.end) }
    }
}

pub struct GlyphPosition {
    pub s: *const u8,   // Position of the glyph in the input string.
    pub x: f32,         // The x-coordinate of the logical glyph position.
    // The bounds of the glyph shape.
    pub minx: f32,
    pub maxx: f32,
}

#[derive(Clone)]
pub struct State {
    pub composite: CompositeState,
    pub shape_aa: i32,

    pub fill: Paint,
    pub stroke: Paint,

    pub stroke_width: f32,
    pub miter_limit: f32,
    pub line_join: LineJoin,
    pub line_cap: LineCap,
    pub alpha: f32,
    pub xform: Transform,
    pub scissor: Scissor,

    pub font_size: f32,
    pub letter_spacing: f32,
    pub line_height: f32,
    pub font_blur: f32,
    pub text_align: Align,
    pub font_id: i32,
}

impl State {
    fn new() -> Self {
        Self {
            fill: Paint::with_color(Color::new(0xFF_FFFFFF)),
            stroke: Paint::with_color(Color::new(0xFF_000000)),

            composite: CompositeOp::SrcOver.into(),
            shape_aa: 1,
            stroke_width: 1.0,
            miter_limit: 10.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            alpha: 1.0,
            xform: Transform::identity(),

            scissor: Scissor {
                extent: [-1.0, -1.0],
                xform: Transform::identity(),
            },

            font_size: 16.0,
            letter_spacing: 0.0,
            line_height: 1.0,
            font_blur: 0.0,
            text_align: Align::LEFT | Align::BASELINE,
            font_id: 0,
        }
    }
}

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
}

pub struct Context {
    pub commands: Vec<f32>,

    pub commandx: f32,
    pub commandy: f32,

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
        if self.states.states.len() >= self.states.states.capacity() {
            return;
        }
        if let Some(state) = self.states.states.last().cloned() {
            self.states.states.push(state);
        }
    }

    pub fn restore(&mut self) {
        self.states.states.pop();
    }

    pub fn reset(&mut self) {
        let state = if let Some(state) = self.states.states.last_mut() {
            state
        } else {
            self.states.states.push(unsafe { std::mem::zeroed() });
            self.states.states.last_mut().expect("last mut state (reset)")
        };
        *state = State::new();
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
        self.states.last_mut().shape_aa = enabled as i32;
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

            commandx: 0.0,
            commandy: 0.0,
            counters: Counters::default(),
            cache: PathCache::new(),
            commands: Vec::with_capacity(INIT_COMMANDS_SIZE),

            device_px_ratio: 1.0,

            font_image_idx: 0,
        }
    }

    pub(crate) fn append_commands(&mut self, vals: &mut [f32]) {
        use crate::draw_api::{MOVETO, LINETO, BEZIERTO, CLOSE, WINDING};
        use crate::transform::transform_pt;

        let xform = &self.states.last().xform;

        if vals[0] as i32 != CLOSE && vals[0] as i32 != WINDING {
            self.commandx = vals[vals.len()-2];
            self.commandy = vals[vals.len()-1];
        }

        // transform commands
        let mut i = 0;
        while i < vals.len() {
            let cmd = vals[i] as i32;
            match cmd {
            MOVETO => {
                transform_pt(&mut vals[i+1..], xform);
                i += 3;
            }
            LINETO => {
                transform_pt(&mut vals[i+1..], xform);
                i += 3;
            }
            BEZIERTO => {
                transform_pt(&mut vals[i+1..], xform);
                transform_pt(&mut vals[i+3..], xform);
                transform_pt(&mut vals[i+5..], xform);
                i += 7;
            }
            CLOSE => i += 1,
            WINDING => i += 2,
            _ => unreachable!(),
            }
        }

        self.commands.extend_from_slice(vals);
    }
}
