#![allow(unused_unsafe)]
#![allow(dead_code)] 

use std::{
    ffi::{c_void, CString},
    os::raw::c_char,
    mem::transmute,
    ptr::null_mut,
};

use crate::context::Align;
use crate::context::State;

#[link(name = "nvg")]
extern "C" {
    fn fonsAddFont(s: *mut FONScontext, name: *const c_char, path: *const c_char) -> i32;
    fn fonsAddFontMem(s: *mut FONScontext, name: *const c_char, data: *mut u8, ndata: i32, free_data: i32) -> i32;
    fn fonsGetFontByName(s: *mut FONScontext, name: *const u8) -> i32;

    fn fonsAddFallbackFont(s: *mut FONScontext, base: i32, fallback: i32) -> i32;

    fn fonsGetTextureData(s: *mut FONScontext, width: *mut i32, height: *mut i32) -> *const u8;
    //fn fonsValidateTexture(s: *mut FONScontext, dirty: *mut i32) -> bool;

    fn fonsResetAtlas(s: *mut FONScontext, width: u32, height: u32) -> i32;

    fn fonsSetSize(s: *mut FONScontext, size: f32);
    fn fonsSetColor(s: *mut FONScontext, color: u32);
    fn fonsSetSpacing(s: *mut FONScontext, spacing: f32);
    fn fonsSetBlur(s: *mut FONScontext, blur: f32);
    fn fonsSetAlign(s: *mut FONScontext, align: Align);
    fn fonsSetFont(s: *mut FONScontext, font: i32);

    fn fonsTextIterInit(
        s: *mut FONScontext, iter: *mut FONStextIter,
        x: f32, y: f32, str: *const u8, end: *const u8, bitmap_option: i32) -> i32;
    fn fonsTextIterNext(s: *mut FONScontext, iter: *mut FONStextIter, quad: *mut FONSquad) -> bool;

    fn fonsTextBounds(s: *mut FONScontext, x: f32, y: f32, start: *const u8, end: *const u8, bounds: *mut f32) -> f32;
    fn fonsLineBounds(s: *mut FONScontext, y: f32, miny: *mut f32, maxy: *mut f32);
    fn fonsVertMetrics(s: *mut FONScontext, ascender: *mut f32, descender: *mut f32, lineh: *mut f32);
}

pub struct Metrics {
    pub ascender: f32,
    pub descender: f32,
    pub line_height: f32,
}

pub const FONS_INVALID: i32 = -1;

const SCRATCH_BUF_SIZE: usize = 96000;
const HASH_LUT_SIZE: usize = 256;
const INIT_FONTS: usize = 4;
const INIT_GLYPHS: usize = 256;
const INIT_ATLAS_NODES: usize = 256;
const VERTEX_COUNT: usize = 1024;
const MAX_STATES: usize = 20;
const MAX_FALLBACKS: usize = 20;

pub const ZERO_TOPLEFT: u8 = 1;
pub const ZERO_BOTTOMLEFT: u8 = 2;

pub const GLYPH_BITMAP_OPTIONAL: i32 = 1;
pub const GLYPH_BITMAP_REQUIRED: i32 = 2;

#[repr(C)]
pub struct FONSatlas {
    _stub: usize
}

#[repr(C)]
pub struct FONSstate {
    _stub: usize
}

#[repr(C)]
pub struct FONSfont {
    _stub: usize
}

#[repr(C)]
#[derive(Default)]
pub struct FONSquad {
    pub x0: f32,
    pub y0: f32,
    pub s0: f32,
    pub t0: f32,
    pub x1: f32,
    pub y1: f32,
    pub s1: f32,
    pub t1: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FONStextIter {
    pub x: f32,
    pub y: f32,
    pub nextx: f32,
    pub nexty: f32,
    pub scale: f32,
    pub spacing: f32,
    pub codepoint: u32,
    pub isize: i16,
    pub iblur: i16,
    pub font: *mut FONSfont,
    pub prev_glyph_index: i32,
    pub str: *const u8,
    pub next: *const u8,
    pub end: *const u8,
    pub utf8state: u32,
    pub bitmap_option: i32,
}

#[repr(C)]
pub struct FONSparams {
    pub width: i32,
    pub height: i32,
    pub flags: u8,

    pub user_ptr: *mut c_void,

    pub render_create: unsafe extern fn(uptr: *mut c_void, width: i32, height: i32) -> i32,
    pub render_resize: unsafe extern fn(uptr: *mut c_void, width: i32, height: i32) -> i32,
    pub render_update: unsafe extern fn(uptr: *mut c_void, rect: *mut i32, data: *const u8),
    pub render_draw:   unsafe extern fn(uptr: *mut c_void, verts: *const f32, tcoords: *const f32, colors: *const i32, nverts: i32),
    pub render_delete: unsafe extern fn(uptr: *mut c_void),
}

impl FONSparams {
    pub fn simple(width: i32, height: i32) -> Self {
        Self {
            width, height,
            flags: ZERO_TOPLEFT,
            render_create: unsafe { transmute(0usize) },
            render_resize: unsafe { transmute(0usize) },
            render_update: unsafe { transmute(0usize) },
            render_draw:   unsafe { transmute(0usize) },
            render_delete: unsafe { transmute(0usize) },
            user_ptr: null_mut(),
        }
    }
}

#[repr(C)]
pub struct FONScontext {
    params: FONSparams,

    itw: f32,
    ith: f32,

    tex_data: *mut u8,
    dirty_rect: [i32; 4],
    fonts: *mut *mut FONSfont,
    atlas: *mut FONSatlas,
    cfonts: i32,
    nfonts: i32,

    verts: [f32; VERTEX_COUNT*2],
    tcoords: [f32; VERTEX_COUNT*2],
    colors: [u32; VERTEX_COUNT],
    nverts: i32,

    scratch: *mut u8,
    nscratch: i32,

    states: [FONSstate; MAX_STATES],
    nstates: i32,

    handle_error: extern fn(uptr: *mut c_void, error: i32, val: i32),
    error_uptr: *mut c_void,
}

impl FONScontext {
    pub fn add_font(&mut self, name: &str, path: &str) -> i32 {
        let name = CString::new(name).expect("add_font cstring");
        let path = CString::new(path).expect("add_font cstring");
        unsafe { fonsAddFont(self, name.as_ptr(), path.as_ptr()) }
    }

    pub fn add_font_mem(&mut self, name: &str, data: *mut u8, ndata: i32, free_data: i32) -> i32 {
        let name = CString::new(name).expect("add_font_mem cstring");
        unsafe { fonsAddFontMem(self, name.as_ptr(), data, ndata, free_data) }
    }

    pub fn font_by_name(&mut self, name: *const u8) -> i32 {
        unsafe { fonsGetFontByName(self, name) }
    }

    pub fn add_fallback_font(&mut self, base: i32, fallback: i32) -> i32 {
        unsafe { fonsAddFallbackFont(self, base, fallback) }
    }

    pub fn texture_data(&mut self) -> (i32, i32, *const u8) {
        let mut w = 0;
        let mut h = 0;
        let data = unsafe { fonsGetTextureData(self, &mut w, &mut h) };
        (w, h, data)
    }

    pub fn validate_texture(&mut self, dirty: &mut [i32; 4]) -> bool {
        let dr = self.dirty_rect;
        let ok = dr[0] < dr[2] && dr[1] < dr[3];
	if ok {
            *dirty = dr;
            // Reset dirty rect
            self.dirty_rect = [
                self.params.width,
                self.params.height,
                0,
                0,
            ];
        }
        ok
    }

    pub fn reset_atlas(&mut self, width: u32, height: u32) -> i32 {
        unsafe { fonsResetAtlas(self, width, height) }
    }

    pub fn sync_state(&mut self, state: &State, scale: f32) {
        unsafe {
            fonsSetSize(self, state.font_size*scale);
            fonsSetSpacing(self, state.letter_spacing*scale);
            fonsSetBlur(self, state.font_blur*scale);
            fonsSetAlign(self, state.text_align);
            fonsSetFont(self, state.font_id);
        }
    }

    pub fn metrics(&mut self) -> Metrics {
        let (mut ascender, mut descender, mut line_height) = (0.0, 0.0, 0.0);
        unsafe {
            fonsVertMetrics(self, &mut ascender, &mut descender, &mut line_height);
        }
        Metrics { ascender, descender, line_height }
    }

    pub fn line_bounds(&mut self, y: f32) -> (f32, f32) {
        let (mut miny, mut maxy) = (0.0, 0.0);
        unsafe {
            fonsLineBounds(self, y, &mut miny, &mut maxy);
        }
        (miny, maxy)
    }

    pub fn text_bounds(&mut self, x: f32, y: f32, start: *const u8, end: *const u8, bounds: *mut f32) -> f32 {
        unsafe {
            fonsTextBounds(self, x, y, start, end, bounds)
        }
    }

    pub fn text_iter_optional(&mut self,
        x: f32, y: f32,
        start: *const u8, end: *const u8,
    ) -> TextIter {
        let mut iter: FONStextIter = unsafe { std::mem::zeroed() };
        unsafe {
            fonsTextIterInit(self, &mut iter, x, y, start, end, GLYPH_BITMAP_OPTIONAL);
        }
        TextIter { fs: self, iter }
    }
    
    pub fn text_iter_required(&mut self,
        x: f32, y: f32,
        start: *const u8, end: *const u8,
    ) -> TextIter {
        let mut iter: FONStextIter = unsafe { std::mem::zeroed() };
        unsafe {
            fonsTextIterInit(self, &mut iter, x, y, start, end, GLYPH_BITMAP_REQUIRED);
        }
        TextIter { fs: self, iter }
    }
}

#[derive(Clone, Copy)]
pub struct TextIter {
    fs: *mut FONScontext,
    iter: FONStextIter,
}

impl TextIter {
    pub fn iter(&self) -> &FONStextIter { &self.iter }
}

impl Iterator for TextIter {
    type Item = FONSquad;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut q = std::mem::uninitialized();
            let ok = fonsTextIterNext(self.fs, &mut self.iter, &mut q);
            if ok {
                Some(q)
            } else {
                None
            }
        }
    }
}