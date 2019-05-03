use std::{
    ffi::{c_void, CString},
    os::raw::c_char,
    mem::transmute,
    ptr::null_mut,
};

use crate::context::Align;
use crate::context::State;

pub use super::stash::Metrics;

use super::atlas::Atlas;
use super::stash::{
    Stash,
    Font,
    Quad,
    TextIter,

    fonsAddFallbackFont,
    fonsAddFont,
    fonsAddFontMem,
    fonsGetFontByName,
    fonsGetTextureData,

    fonsLineBounds,

    fonsTextBounds,
    fonsTextIterInit,
    fonsVertMetrics,
};

pub const FONS_INVALID: i32 = -1;

//const SCRATCH_BUF_SIZE: usize = 96000;
//const HASH_LUT_SIZE: usize = 256;
//const INIT_FONTS: usize = 4;
//const INIT_GLYPHS: usize = 256;
//const INIT_ATLAS_NODES: usize = 256;
//const MAX_FALLBACKS: usize = 20;

pub const GLYPH_BITMAP_OPTIONAL: i32 = 1;
pub const GLYPH_BITMAP_REQUIRED: i32 = 2;

impl Stash {
    pub fn add_font(&mut self, name: &str, path: &str) -> i32 {
        let name = CString::new(name).expect("add_font cstring");
        let path = CString::new(path).expect("add_font cstring");
        unsafe { fonsAddFont(transmute(self), name.as_ptr(), path.as_ptr()) }
    }

    pub fn add_font_mem(&mut self, name: &str, data: *mut u8, ndata: i32, free_data: i32) -> i32 {
        let name = CString::new(name).expect("add_font_mem cstring");
        unsafe { fonsAddFontMem(transmute(self), name.as_ptr(), data, ndata, free_data) }
    }

    pub fn font_by_name(&mut self, name: *const u8) -> i32 {
        unsafe { fonsGetFontByName(transmute(self), name as *const i8) }
    }

    pub fn add_fallback_font(&mut self, base: i32, fallback: i32) -> i32 {
        unsafe { fonsAddFallbackFont(transmute(self), base, fallback) }
    }

    pub fn texture_data(&mut self) -> (i32, i32, *const u8) {
        let mut w = 0;
        let mut h = 0;
        let data = unsafe { fonsGetTextureData(transmute(self), &mut w, &mut h) };
        (w, h, data)
    }

    pub fn validate_texture(&mut self, dirty: &mut [i32; 4]) -> bool {
        let dr = self.dirty_rect;
        let ok = dr[0] < dr[2] && dr[1] < dr[3];
	if ok {
            *dirty = dr;
            // Reset dirty rect
            self.dirty_rect = [
                self.width,
                self.height,
                0,
                0,
            ];
        }
        ok
    }

    pub fn sync_state(&mut self, state: &State, scale: f32) {
        unsafe {
            let fs: &mut Stash = transmute(self);
            let _state = fs.state_mut();
            _state.size = state.font_size*scale;
            _state.spacing = state.letter_spacing*scale;
            _state.blur = state.font_blur*scale;
            _state.align = state.text_align.bits();
            _state.font = state.font_id;
        }
    }

    pub fn metrics(&mut self) -> Metrics {
        unsafe {
            fonsVertMetrics(transmute(self)).unwrap()
        }
    }

    pub fn line_bounds(&mut self, y: f32) -> (f32, f32) {
        let (mut miny, mut maxy) = (0.0, 0.0);
        unsafe {
            fonsLineBounds(transmute(self), y, &mut miny, &mut maxy);
        }
        (miny, maxy)
    }

    pub fn text_bounds(&mut self, x: f32, y: f32, start: *const u8, end: *const u8, bounds: *mut f32) -> f32 {
        unsafe {
            fonsTextBounds(transmute(self), x, y,
                start as *const i8, end as *const i8,
                bounds)
        }
    }

    pub fn text_iter_optional(&mut self,
        x: f32, y: f32,
        start: *const u8, end: *const u8,
    ) -> TextIter {
        let mut iter: TextIter = unsafe { std::mem::zeroed() };
        unsafe {
            fonsTextIterInit(self, &mut iter, x, y,
                start, end,
                GLYPH_BITMAP_OPTIONAL);
            iter.fs = self;
        }
        iter
    }
    
    pub fn text_iter_required(&mut self,
        x: f32, y: f32,
        start: *const u8, end: *const u8,
    ) -> TextIter {
        let mut iter: TextIter = unsafe { std::mem::zeroed() };
        unsafe {
            fonsTextIterInit(self, &mut iter, x, y,
                start, end,
                GLYPH_BITMAP_REQUIRED);
            iter.fs = self;
        }
        iter
    }
}
