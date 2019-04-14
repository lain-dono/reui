#![allow(improper_ctypes)]

use core::ptr::null_mut;

use arrayvec::ArrayVec;

use crate::{
    backend::{BackendGL, Image},
    cache::{PathCache, LineCap, LineJoin},
    transform,
    vg::*,
    fons::*,
};

extern "C" {
    fn nvgFontFace(ctx: *mut Context, face: *const u8);

    fn nvgText(ctx: *mut Context, x: f32, y: f32, start: *const u8, end: *const u8) -> f32;
    fn nvgTextBox(ctx: *mut Context, x: f32, y: f32, break_row_width: f32, start: *const u8, end: *const u8);

    fn nvgTextBounds(ctx: *mut Context, x: f32, y: f32, s: *const u8, end: *const u8, bounds: *mut f32) -> f32;
    fn nvgTextBoxBounds(
        ctx: *mut Context, x: f32, y: f32, break_row_width: f32,
        start: *const u8, end: *const u8, bounds: *mut f32);
    fn nvgTextMetrics(ctx: *mut Context, ascender: *mut f32, descender: *mut f32, lineh: *mut f32);
    fn nvgTextBreakLines(
        ctx: *mut Context,
        start: *const u8, end: *const u8,
        break_row_width: f32, rows: *mut TextRow, max_rows: usize) -> usize;
    fn nvgTextGlyphPositions(
        ctx: *mut Context, x: f32, y: f32,
        start: *const u8, end: *const u8,
        positions: *mut GlyphPosition, max_positions: usize,
    ) -> usize;

    fn nvgAddFallbackFontId(ctx: *mut Context, a: i32, b: i32);
}

pub fn itoa10(dst: &mut [u8], mut value: isize) -> &[u8] {
    if value == 0 {
        dst[0] = b'0';
        return &dst[..1];
    }

    let mut count = 0;

    if value < 0 {
        value *= -1;
        dst[0] = b'-';
        count += 1;
    }

    let mut tmp = value;
    while tmp>0 {
        dst[count] = b'0' + (tmp%10) as u8;
        count += 1;
        tmp /= 10;
    }
    &dst[..count]
}

pub fn slice_start_end(b: &[u8]) -> (*const u8, *const u8) {
    unsafe {
        let start = b.as_ptr();
        let end = start.add(b.len());
        (start, end)
    }
}

fn str_start_end(s: &str) -> (*const u8, *const u8) {
    slice_start_end(s.as_bytes())
}

bitflags::bitflags!(
    #[repr(C)]
    pub struct Align: i32 {
        const LEFT      = 1<<0;    // Default, align text horizontally to left.
        const CENTER    = 1<<1;    // Align text horizontally to center.
        const RIGHT     = 1<<2;    // Align text horizontally to right.

        const TOP       = 1<<3;    // Align text vertically to top.
        const MIDDLE    = 1<<4;    // Align text vertically to middle.
        const BOTTOM    = 1<<5;    // Align text vertically to bottom.
        const BASELINE  = 1<<6; // Default, align text vertically to baseline.
    }
);

#[repr(C)]
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

#[repr(C)]
pub struct GlyphPosition {
    pub s: *const u8,   // Position of the glyph in the input string.
    pub x: f32,         // The x-coordinate of the logical glyph position.
    // The bounds of the glyph shape.
    pub minx: f32,
    pub maxx: f32,
}

#[repr(C)]
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
    pub xform: [f32; 6],
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
            fill: Paint::color(Color::new(0xFF_FFFFFF)),
            stroke: Paint::color(Color::new(0xFF_000000)),

            composite: CompositeOp::SOURCE_OVER.into(),
            shape_aa: 1,
            stroke_width: 1.0,
            miter_limit: 10.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            alpha: 1.0,
            xform: transform::identity(),

            scissor: Scissor {
                extent: [-1.0, -1.0],
                xform: transform::identity()
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

#[repr(C)]
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
        self.states.last().expect("last state") //[(self.states.len()-1) as usize]
    }
    pub(crate) fn last_mut(&mut self) -> &mut State {
        self.states.last_mut().expect("last_mut state")
    }

    pub(crate) fn clear(&mut self) {
        self.states.clear();
    }
}

#[repr(C)]
pub struct Context {
    pub commands: Vec<f32>,

    pub commandx: f32,
    pub commandy: f32,

    pub states: States,
    pub cache: PathCache,
    pub device_px_ratio: f32,

    pub fs: Box<FONScontext>,
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
        if let Some(state) = self.states.states.last() {
            self.states.states.push(state.clone());
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

    /*
    pub fn create_font(&mut self, name: &str, path: &str) -> i32 {
        let name = name.as_bytes().as_ptr();
        let path = path.as_bytes().as_ptr();
        unsafe { nvgCreateFont(self, name, path) }
    }
    */

    pub fn add_fallback_font_id(&mut self, a: i32, b: i32) {
        unsafe { nvgAddFallbackFontId(self, a, b) }
    }

    pub fn font_face(&mut self, face: &[u8]) {
        unsafe { nvgFontFace(self, face.as_ptr()); }
    }
}

impl Context {
    pub fn text_bounds_raw_simple(&mut self, x: f32, y: f32, start: *const u8, end: *const u8) -> f32 {
        unsafe { nvgTextBounds(self, x, y, start, end, null_mut()) }
    }
    pub fn text_bounds_raw(&mut self, x: f32, y: f32, start: *const u8, end: *const u8) -> (f32, [f32; 4]) {
        let mut bounds = [0f32; 4];
        let uw = unsafe { nvgTextBounds(self, x, y, start, end, bounds.as_mut_ptr()) };
        (uw, bounds)
    }

    pub fn text_bounds_simple(&mut self, x: f32, y: f32, text: &str) -> f32 {
        let (a, b) = str_start_end(text);
        self.text_bounds_raw_simple(x, y, a, b)
    }

    pub fn text_bounds(&mut self, x: f32, y: f32, text: &str) -> (f32, [f32; 4]) {
        let (a, b) = str_start_end(text);
        self.text_bounds_raw(x, y, a, b)
    }

    pub fn text_box_raw(&mut self, x: f32, y: f32, break_row_width: f32, start: *const u8, end: *const u8) {
        unsafe { nvgTextBox(self, x, y, break_row_width, start, end) }
    }

    pub fn text_box_bounds_raw(
        &mut self, x: f32, y: f32, break_row_width: f32, start: *const u8, end: *const u8,
    ) -> [f32; 4] {
        let mut bounds = [0f32; 4];
        unsafe { nvgTextBoxBounds(self, x, y, break_row_width, start, end, bounds.as_mut_ptr()) }
        bounds
    }

    pub fn text_break_lines<'a>(
        &mut self, start: *const u8, end: *const u8, break_row_width: f32,rows: &'a mut [TextRow]
    ) -> &'a [TextRow] {
        let n = unsafe { nvgTextBreakLines(self, start, end, break_row_width, rows.as_mut_ptr(), rows.len()) };
        &rows[..n]
    }

    pub fn text_glyph_positions<'a>(
        &mut self, x: f32, y: f32, start: *const u8, end: *const u8, positions: &'a mut [GlyphPosition]
    ) -> &'a [GlyphPosition] {
        let n = unsafe {
            nvgTextGlyphPositions(self, x, y, start, end, positions.as_mut_ptr(), positions.len())
        };
        &positions[..n]
    }

    pub fn text_metrics(&mut self) -> (f32, f32, f32) {
        let mut ascender = 0.0;
        let mut descender = 0.0;
        let mut lineh = 0.0;

        unsafe { nvgTextMetrics(self, &mut ascender, &mut descender, &mut lineh) }
        (ascender, descender, lineh)
    }
}


impl Context {
    pub fn text_raw(&mut self, x: f32, y: f32, start: *const u8, end: *const u8) -> f32 {
        unsafe { nvgText(self, x, y, start, end) }
    }
    pub fn text(&mut self, x: f32, y: f32, text: &str) -> f32 {
        let (a, b) = str_start_end(text);
        self.text_raw(x, y, a, b)
    }
    pub fn text_slice(&mut self, x: f32, y: f32, text: &[u8]) -> f32 {
        let (a, b) = slice_start_end(text);
        self.text_raw(x, y, a, b)
    }
}

// State setting
impl Context {
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
        transform::mul(&mut state.stroke.xform, &state.xform);
    }
    pub fn fill_paint(&mut self, paint: Paint) {
        let state = self.states.last_mut();
        state.fill = paint;
        transform::mul(&mut state.fill.xform, &state.xform);
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

/*
#[lang = "eh_personality"] extern fn rust_eh_personality() {}
#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
*/
