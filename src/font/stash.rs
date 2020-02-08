#![allow(/*dead_code,*/
         //mutable_transmutes,
         non_camel_case_types,
         non_snake_case,
         non_upper_case_globals,
         clippy::collapsible_if,
         //unused_assignments,
         //unused_mut,
         )]

#![deny(dead_code)]

use crate::math::{point2, Rect};
use super::{atlas::Atlas, font_info::*, utils::*};

pub struct Metrics {
    pub ascender: f32,
    pub descender: f32,
    pub line_gap: f32,
}

use std::{cmp::{max, min}, mem::size_of, ptr::null_mut};

extern crate libc;

//
// Copyright (c) 2009-2013 Mikko Mononen memon@inside.org
//
// This software is provided 'as-is', without any express or implied
// warranty.  In no event will the authors be held liable for any damages
// arising from the use of this software.
// Permission is granted to anyone to use this software for any purpose,
// including commercial applications, and to alter it and redistribute it
// freely, subject to the following restrictions:
// 1. The origin of this software must not be misrepresented; you must not
//    claim that you wrote the original software. If you use this software
//    in a product, an acknowledgment in the product documentation would be
//    appreciated but is not required.
// 2. Altered source versions must be plainly marked as such, and must not be
//    misrepresented as being the original software.
// 3. This notice may not be removed or altered from any source distribution.
//

pub type Align = u32;
// Default
pub const FONS_ALIGN_BASELINE: Align = 64;
pub const FONS_ALIGN_BOTTOM: Align = 32;
pub const FONS_ALIGN_MIDDLE: Align = 16;
// Vertical align
pub const FONS_ALIGN_TOP: Align = 8;
pub const FONS_ALIGN_RIGHT: Align = 4;
pub const FONS_ALIGN_CENTER: Align = 2;
// Horizontal align
// Default
pub const FONS_ALIGN_LEFT: Align = 1;

pub type GlyphBitmap = u32;
pub const FONS_GLYPH_BITMAP_REQUIRED: GlyphBitmap = 2;
pub const FONS_GLYPH_BITMAP_OPTIONAL: GlyphBitmap = 1;

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct Quad {
    pub x0: f32,
    pub y0: f32,
    pub s0: f32,
    pub t0: f32,

    pub x1: f32,
    pub y1: f32,
    pub s1: f32,
    pub t1: f32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TextIter {
    pub x: f32,
    pub y: f32,
    pub nextx: f32,
    pub nexty: f32,
    pub scale: f32,
    pub spacing: f32,
    pub codepoint: u32,
    pub isize_0: i16,
    pub iblur: i16,
    pub font: *mut Font,
    pub prev_glyph_index: i32,
    pub str_0: *const u8,
    pub next: *const u8,
    pub end: *const u8,
    pub utf8state: u32,
    pub bitmapOption: i32,

    pub fs: *mut Stash,
}

impl Iterator for TextIter {
    type Item = Quad;
    fn next(&mut self) -> Option<Self::Item> {
        let mut s: *const u8 = self.next;
        self.str_0 = self.next;
        if s == self.end {
            return None;
        }
        let mut quad = Quad::default();
        unsafe {
            while s != self.end {
                if 0 == decutf8(&mut self.utf8state, &mut self.codepoint, u32::from(*s)) {
                    s = s.offset(1);
                    self.x = self.nextx;
                    self.y = self.nexty;
                    let glyph = (*self.fs).get_glyph(
                        self.font,
                        self.codepoint,
                        self.isize_0,
                        self.iblur,
                        self.bitmapOption,
                    );
                    if !glyph.is_null() {
                        (*self.fs).get_quad(
                            self.font,
                            self.prev_glyph_index,
                            &*glyph,
                            self.scale,
                            self.spacing,
                            &mut self.nextx,
                            &mut self.nexty,
                            &mut quad,
                        );
                    }
                    self.prev_glyph_index = if !glyph.is_null() { (*glyph).index } else { -1 };
                    break;
                }

                s = s.offset(1)
            }
        }
        self.next = s;
        Some(quad)
    }
}

#[derive(Clone)]
pub struct LutI32 {
    lut: [i32; 256],
}

impl std::ops::Index<usize> for LutI32 {
    type Output = i32;
    #[inline(always)]
    fn index(&self, idx: usize) -> &i32 { &self.lut[idx] }
}

impl std::ops::IndexMut<usize> for LutI32 {
    #[inline(always)]
    fn index_mut(&mut self, idx: usize) -> &mut i32 { &mut self.lut[idx] }
}

impl Default for LutI32 {
    fn default() -> Self {
        Self {
            lut: [0; 256],
        }
    }
}

#[derive(Clone, Default)]
#[repr(C)]
pub struct Font {
    pub data: Vec<u8>,

    pub font: FontInfo,
    pub name: arrayvec::ArrayString<[u8; 64]>,

    pub ascender: f32,
    pub descender: f32,
    pub lineh: f32,

    pub glyphs: Vec<Glyph>,

    pub lut: LutI32,

    pub fallbacks: [i32; 20],
    pub nfallbacks: i32,
}

#[derive(Clone)]
#[repr(C)]
pub struct Glyph {
    pub codepoint: u32,
    pub index: i32,
    pub next: i32,
    pub size: i16,
    pub blur: i16,
    pub x0: i16,
    pub y0: i16,
    pub x1: i16,
    pub y1: i16,
    pub xadv: i16,
    pub xoff: i16,
    pub yoff: i16,
}

const VERTEX_COUNT: usize = 1024;
const MAX_STATES: usize = 20;

#[derive(Clone)]
pub struct Stash {
    pub width: i32,
    pub height: i32,

    pub itw: f32,
    pub ith: f32,
    pub tex_data: Vec<u8>,
    pub dirty_rect: [i32; 4],

    pub atlas: Atlas,

    pub fonts: Vec<Font>,

    pub verts: [f32; VERTEX_COUNT * 2],
    pub tcoords: [f32; VERTEX_COUNT * 2],
    pub colors: [u32; VERTEX_COUNT],

    pub nverts: i32,

    pub scratch: Vec<u8>,

    pub states: [State; MAX_STATES],
    pub nstates: i32,
}

#[derive(Clone, Default)]
pub struct State {
    pub font: i32,
    pub align: i32,
    pub size: f32,
    pub color: u32,
    pub blur: f32,
    pub spacing: f32,
}

// you can predefine this to use different values
// (we share this with other code at RAD)
// can't use i16 because that's not visible in the header file
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub x: i16,
    pub y: i16,
    pub cx: i16,
    pub cy: i16,
    pub type_0: u8,
    pub padding: u8,
}

pub const VMOVE: u32 = 1;
pub const VLINE: u32 = 2;
pub const VCURVE: u32 = 3;

// @TODO: don't expose this structure
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Bitmap {
    pub w: i32,
    pub h: i32,
    pub stride: i32,
    pub pixels: *mut u8,
}

#[derive(Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Default)]
pub struct Edge {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub invert: i32,
}

pub struct Heap {
    pub head: *mut HeapChunk,
    pub first_free: *mut libc::c_void,
    pub num_remaining_in_head_chunk: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HeapChunk {
    pub next: *mut HeapChunk,
}

impl Heap {
    pub unsafe fn cleanup(&mut self) {
        let mut c = self.head;
        while !c.is_null() {
            c = (*c).next
        }
    }

    pub unsafe fn free(&mut self, p: *mut libc::c_void) {
        *(p as *mut *mut libc::c_void) = self.first_free;
        self.first_free = p;
    }

    pub unsafe fn alloc(&mut self, size: u64, stash: &mut Stash) -> *mut libc::c_void {
        if !self.first_free.is_null() {
            let p: *mut libc::c_void = self.first_free;
            self.first_free = *(p as *mut *mut libc::c_void);
            p
        } else {
            if self.num_remaining_in_head_chunk == 0 {
                let count = if size < 32 {
                    2000
                } else if size < 128 {
                    800
                } else {
                    100
                };
                let size = (size_of::<HeapChunk>() as u64)
                    .wrapping_add(size.wrapping_mul(count as u64));
                let mut c = stash.tmpalloc(size) as *mut HeapChunk;
                if c.is_null() {
                    return null_mut();
                }
                (*c).next = self.head;
                self.head = c;
                self.num_remaining_in_head_chunk = count
            }
            self.num_remaining_in_head_chunk -= 1;
            (self.head as *mut i8)
                .offset(size.wrapping_mul(self.num_remaining_in_head_chunk as u64) as isize)
                as *mut libc::c_void
        }
    }
}


// ////////////////////////////////////////////////////////////////////////////
//
//  Rasterizer

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ActiveEdge {
    pub next: *mut ActiveEdge,
    pub fx: f32,
    pub fdx: f32,
    pub fdy: f32,
    pub direction: f32,
    pub sy: f32,
    pub ey: f32,
}


impl Stash {
    pub fn new(width: i32, height: i32) -> Box<Stash> {
        let mut stash = Box::new(Stash {
            width,
            height,

            scratch: Vec::with_capacity(96000),

            atlas: Atlas::new(width, height, 256),

            fonts: Vec::with_capacity(4),

            itw: 1.0 / width as f32,
            ith: 1.0 / height as f32,

            tex_data: vec![0; (width * height) as usize],

            dirty_rect: [width, height, 0, 0],

            verts: [0.0; VERTEX_COUNT * 2],
            tcoords: [0.0; VERTEX_COUNT * 2],
            colors: [0; VERTEX_COUNT],
            nverts: 0,

            states: Default::default(),
            nstates: 0,
        });

        stash.add_white_rect(2, 2);
        stash.push_state();
        stash.clear_state();
        stash
    }

    pub fn state(&self) -> &State {
        &self.states[self.nstates as usize - 1]
    }
    pub fn state_mut(&mut self) -> &mut State {
        &mut self.states[self.nstates as usize - 1]
    }

    fn clear_state(&mut self) {
        *self.state_mut() = State {
            size: 12.0,
            color: 0xffff_ffff,
            font: 0,
            blur: 0.0,
            spacing: 0.0,
            align: FONS_ALIGN_LEFT as i32 | FONS_ALIGN_BASELINE as i32,
        };
    }

    fn push_state(&mut self) {
        if self.nstates >= 20 {
            return;
        }
        if self.nstates > 0 {
            self.states[self.nstates as usize] = self.states[self.nstates as usize - 1].clone();
        }
        self.nstates += 1;
    }


    fn add_white_rect(&mut self, w: i32, h: i32) {
        let (gx, gy) = if let Some(p) = self.atlas.add_rect(w, h) {
            p
        } else {
            return;
        };

        let mut dst = &mut self.tex_data[(gx + gy * self.width) as usize..];

        let mut y = 0;
        while y < h {
            let mut x = 0;
            while x < w {
                dst[x as usize] = 0xff;
                x += 1
            }
            dst = &mut dst[self.width as usize..];
            y += 1
        }

        self.dirty_rect = [
            min(self.dirty_rect[0], gx),
            min(self.dirty_rect[1], gy),
            max(self.dirty_rect[2], gx + w),
            max(self.dirty_rect[3], gy + h),
        ];
    }

    fn flush(&mut self) {
        if self.dirty_rect[0] < self.dirty_rect[2] && self.dirty_rect[1] < self.dirty_rect[3] {
            self.dirty_rect = [self.width, self.height, 0, 0];
        }

        if self.nverts > 0 {
            self.nverts = 0
        }
    }

    /// Resets the whole stash.
    pub fn reset_atlas(&mut self, width: u32, height: u32) -> bool {
        let (width, height) = (width as i32, height as i32);

        self.flush();

        self.atlas.reset(width, height);
        self.tex_data.clear();
        self.tex_data.resize((width * height) as usize, 0);

        self.dirty_rect = [width, height, 0, 0];

        for font in &mut self.fonts {
            font.glyphs.clear();
            for j in 0..256 {
                font.lut[j] = -1;
            }
        }

        self.width = width;
        self.height = height;
        self.itw = 1.0 / self.width as f32;
        self.ith = 1.0 / self.height as f32;
        self.add_white_rect(2, 2);

        true
    }

    pub fn add_font<P: AsRef<std::path::Path>>(&mut self, name: &str, path: P) -> std::io::Result<i32> {
        use std::fs::File;
        use std::io::BufReader;
        use std::io::prelude::*;

        let file = File::open(path)?;
        let mut buf = BufReader::new(file);
        let mut contents = Vec::new();
        buf.read_to_end(&mut contents)?;

        Ok(self.add_font_mem(name, contents))
    }

    pub fn add_font_mem(&mut self, name: &str, mut data: Vec<u8>) -> i32 {
        let mut font = Font::default();
        font.glyphs = Vec::with_capacity(256);
        self.fonts.push(font);
        let idx = self.fonts.len() as i32 - 1;

        let stash: *mut Self = self;
        let font = &mut self.fonts[idx as usize];
        unsafe {
            font.name.push_str(name);

            for i in 0..256 {
                font.lut[i] = -1;
            }

            let data_ptr = data.as_mut_ptr();
            font.data = data;
            self.scratch.clear();
            if 0 == fons__tt_loadFont(stash, &mut (*font).font, data_ptr) {
                self.fonts.pop();
                -1
            } else {
                let info = &mut font.font;
                let hhea = info.hhea as isize;

                let ascent = read_i16(info.data.offset(hhea).offset(4));
                let descent = read_i16(info.data.offset(hhea).offset(6));
                let line_gap = read_i16(info.data.offset(hhea).offset(8));

                let fh = ascent - descent;
                font.ascender = ascent as f32 / fh as f32;
                font.descender = descent as f32 / fh as f32;
                font.lineh = (fh + line_gap) as f32 / fh as f32;
                idx
            }
        }
    }
}

// computes a scale factor to produce a font whose EM size is mapped to
// 'pixels' tall. This is probably what traditional APIs compute, but
// I'm not positive.

pub unsafe fn fons__tt_loadFont(
    stash: *mut Stash,
    info: &mut FontInfo,
    data: *mut u8,
) -> i32 {
    let fontstart = 0;
    let cmap = find_table(data, fontstart, *b"cmap");

    info.userdata = stash;
    info.data = data;
    info.fontstart = fontstart as i32;
    info.loca = find_table(data, fontstart, *b"loca") as i32;
    info.head = find_table(data, fontstart, *b"head") as i32;
    info.glyf = find_table(data, fontstart, *b"glyf") as i32;
    info.hhea = find_table(data, fontstart, *b"hhea") as i32;
    info.hmtx = find_table(data, fontstart, *b"hmtx") as i32;
    info.kern = find_table(data, fontstart, *b"kern") as i32;

    if 0 == cmap || 0 == info.loca || 0 == info.head || 0 == info.glyf || 0 == info.hhea || 0 == info.hmtx {
        return 0;
    }

    let maxp = find_table(data, fontstart, *b"maxp");
    info.num_glyphs = if 0 != maxp {
        read_u16(data.offset(maxp as isize + 4)) as i32
    } else {
        0xffff
    };

    let num_tables = read_u16(data.offset(cmap as isize + 2)) as isize;
    info.index_map = 0;

    for i in 0..num_tables {
        let encoding_record = cmap as isize + 4 + (8 * i);
        match read_u16(data.offset(encoding_record)) {
            3 => match read_u16(data.offset(encoding_record + 2)) {
                1 | 10 => {
                    info.index_map =
                        cmap.wrapping_add(read_u32(data.offset(encoding_record + 4))) as i32
                }
                _ => {}
            },
            0 => {
                info.index_map =
                    cmap.wrapping_add(read_u32(data.offset(encoding_record + 4))) as i32
            }
            _ => {}
        }
    }

    if info.index_map == 0 {
        return 0;
    }
    info.index2loc_format = read_u16(data.offset(info.head as isize + 50)) as i32;
    1
}

// @OPTIMIZE: binary search
unsafe fn find_table(data: *mut u8, fontstart: u32, tag: [u8; 4]) -> u32 {
    let num_tables: i32 = read_u16(data.offset(fontstart as isize).offset(4)) as i32;
    let tabledir: u32 = fontstart.wrapping_add(12 as u32);
    for i in 0..num_tables {
        let loc = tabledir.wrapping_add((16 * i) as u32) as isize;
        if *(data.offset(loc) as *const [u8; 4]) == tag {
            return read_u32(data.offset(loc + 8));
        }
    }
    0
}

// Draw text

impl Stash {
    pub unsafe fn get_quad(
        &mut self,
        font: *mut Font,
        prev_glyph: i32,
        glyph: &Glyph,
        scale: f32,
        spacing: f32,
        x: &mut f32,
        y: &mut f32,
        mut q: &mut Quad,
    ) {
        if prev_glyph != -1 {
            let adv: f32 = (*font)
                .font
                .glyph_kern_advance(prev_glyph, (*glyph).index)
                as f32
                * scale;
            *x += (adv + spacing + 0.5) as i32 as f32
        }

        let xoff = ((*glyph).xoff as i32 + 1) as i16 as f32;
        let yoff = ((*glyph).yoff as i32 + 1) as i16 as f32;
        let x0 = ((*glyph).x0 as i32 + 1) as f32;
        let y0 = ((*glyph).y0 as i32 + 1) as f32;
        let x1 = ((*glyph).x1 as i32 - 1) as f32;
        let y1 = ((*glyph).y1 as i32 - 1) as f32;

        let rx = (*x + xoff) as i32 as f32;
        let ry = (*y + yoff) as i32 as f32;

        q.x0 = rx;
        q.y0 = ry;
        q.x1 = rx + x1 - x0;
        q.y1 = ry + y1 - y0;
        q.s0 = x0 * self.itw;
        q.t0 = y0 * self.ith;
        q.s1 = x1 * self.itw;
        q.t1 = y1 * self.ith;

        *x += ((*glyph).xadv as f32 / 10.0 + 0.5) as i32 as f32;
    }

    unsafe fn get_glyph(
        &mut self,
        font: *mut Font,
        codepoint: u32,
        isize: i16,
        mut iblur: i16,
        bitmapOption: i32,
    ) -> *mut Glyph {
        let mut advance: i32 = 0;
        let mut lsb: i32 = 0;
        let mut glyph: *mut Glyph = null_mut();
        let size: f32 = isize as f32 / 10.0;

        if isize < 2 {
            return null_mut();
        }
        if iblur > 20 {
            iblur = 20
        }

        let pad = iblur as i32 + 2;
        self.scratch.clear();
        let h = hashint(codepoint) & (256 - 1);
        let mut i = (*font).lut[h as usize];
        while i != -1 {
            let glyph = &mut (*font).glyphs[i as usize];
            if glyph.codepoint == codepoint
                && glyph.size as i32 == isize as i32
                && glyph.blur as i32 == iblur as i32
            {
                if bitmapOption == FONS_GLYPH_BITMAP_OPTIONAL as i32
                    || glyph.x0 as i32 >= 0 && glyph.y0 as i32 >= 0
                {
                    return glyph;
                }
                // At this point, glyph exists but the bitmap data is not yet created.
                break;
            } else {
                i = glyph.next;
            }
        }

        let mut g = (*font).font.glyph_index(codepoint as i32);
        let mut renderFont: *mut Font = font;
        if g == 0 {
            i = 0;
            while i < (*font).nfallbacks {
                let fallbackFont = &mut self.fonts[(*font).fallbacks[i as usize] as usize];
                let fallbackIndex: i32 = (*fallbackFont).font.glyph_index(codepoint as i32);
                if fallbackIndex != 0 {
                    g = fallbackIndex;
                    renderFont = fallbackFont;
                    break;
                } else {
                    i += 1
                }
            }
        }

        let scale = (*renderFont).font.pixel_height_scale(size);
        let [x0, y0, x1, y1] = (*renderFont).font.build_glyph_bitmap(
            g,
            size,
            scale,
            &mut advance,
            &mut lsb,
        );

        let gw = x1 - x0 + pad * 2;
        let gh = y1 - y0 + pad * 2;
        let (gx, gy) = if bitmapOption == FONS_GLYPH_BITMAP_REQUIRED as i32 {
            if let Some(p) = self.atlas.add_rect(gw, gh) {
                p
            } else {
                return null_mut();
            }
        } else {
            (-1, -1)
        };

        if glyph.is_null() {
            glyph = (*font).alloc_glyph();
            (*glyph).codepoint = codepoint;
            (*glyph).size = isize;
            (*glyph).blur = iblur;
            (*glyph).next = 0;
            (*glyph).next = (*font).lut[h as usize];
            (*font).lut[h as usize] = (*font).glyphs.len() as i32 - 1;
        }
        (*glyph).index = g;
        (*glyph).x0 = gx as i16;
        (*glyph).y0 = gy as i16;
        (*glyph).x1 = ((*glyph).x0 as i32 + gw) as i16;
        (*glyph).y1 = ((*glyph).y0 as i32 + gh) as i16;
        (*glyph).xadv = (scale * advance as f32 * 10.0) as i16;
        (*glyph).xoff = (x0 - pad) as i16;
        (*glyph).yoff = (y0 - pad) as i16;
        if bitmapOption == FONS_GLYPH_BITMAP_OPTIONAL as i32 {
            return glyph;
        }

        let dst = &mut self.tex_data
            [((*glyph).x0 as i32 + pad + ((*glyph).y0 as i32 + pad) * self.width) as usize..];
        (*renderFont).font.render_glyph_bitmap(
            dst.as_mut_ptr(),
            gw - pad * 2,
            gh - pad * 2,
            self.width,
            [scale, scale],
            g,
        );

        let dst = &mut self.tex_data
            [(*glyph).x0 as usize + (*glyph).y0 as usize * self.width as usize..];

        for y in 0..gh {
            dst[(y * self.width) as usize] = 0;
            dst[(gw - 1 + y * self.width) as usize] = 0;
        }
        for x in 0..gw {
            dst[x as usize] = 0;
            dst[(x + (gh - 1) * self.width) as usize] = 0;
        }

        if iblur as i32 > 0 {
            self.scratch.clear();
            let bdst = &mut self.tex_data
                [(*glyph).x0 as usize + (*glyph).y0 as usize * self.width as usize..];
            blur(bdst, gw, gh, self.width, iblur as i32);
        }
        self.dirty_rect = [
            self.dirty_rect[0].min((*glyph).x0 as i32),
            self.dirty_rect[1].min((*glyph).y0 as i32),
            self.dirty_rect[2].max((*glyph).x1 as i32),
            self.dirty_rect[3].max((*glyph).y1 as i32),
        ];
        glyph

    }
}


impl Stash {
    pub(crate) fn calloc<T>(&mut self, count: usize) -> *mut T {
        let size = count * size_of::<T>();
        self.tmpalloc(size as u64)
    }

    fn tmpalloc<T>(&mut self, size: u64) -> *mut T {
        let size = (size.wrapping_add(0xf) & !0xf) as usize;
        if self.scratch.len() + size > 96000 {
            null_mut()
        } else {
            let ptr = unsafe { self.scratch.as_mut_ptr().add(self.scratch.len()) };
            self.scratch.resize_with(self.scratch.len() + size, Default::default);
            ptr as *mut T
        }
    }
}

// rasterize a shape with quadratic beziers into a bitmap
// 1-channel bitmap to draw into

pub unsafe fn stbtt_Rasterize(
    result: *mut Bitmap,
    flatness_in_pixels: f32,
    vertices: *mut Vertex,
    num_verts: i32,
    scale_x: f32,
    scale_y: f32,
    shift_x: f32,
    shift_y: f32,
    x_off: i32,
    y_off: i32,
    invert: i32,
    stash: &mut Stash,
) {
    let scale: f32 = if scale_x > scale_y { scale_y } else { scale_x };
    let mut winding_count: i32 = 0;
    let mut winding_lengths: *mut i32 = std::ptr::null_mut();
    let windings: *mut Point = stbtt_FlattenCurves(
        vertices,
        num_verts,
        flatness_in_pixels / scale,
        &mut winding_lengths,
        &mut winding_count,
        stash,
    );
    if !windings.is_null() {
        rasterize(
            result,
            windings,
            winding_lengths,
            winding_count,
            [scale_x, scale_y],
            [shift_x, shift_y],
            [x_off, y_off],
            invert,
            stash,
        );
    };
}

// returns number of contours
unsafe fn stbtt_FlattenCurves(
    vertices: *mut Vertex,
    num_verts: i32,
    objspace_flatness: f32,
    contour_lengths: *mut *mut i32,
    num_contours: *mut i32,
    stash: &mut Stash,
) -> *mut Point {
    let current_block: u64;
    let mut points: *mut Point = null_mut();
    let mut num_points: i32 = 0;
    let objspace_flatness_squared: f32 = objspace_flatness * objspace_flatness;
    let mut n: i32 = 0;
    let mut start: i32 = 0;
    let mut i = 0;
    while i < num_verts {
        if (*vertices.offset(i as isize)).type_0 as i32 == VMOVE as i32 {
            n += 1
        }
        i += 1
    }
    *num_contours = n;
    if n == 0 {
        return null_mut();
    }
    *contour_lengths = stash.calloc(n as usize);
    if (*contour_lengths).is_null() {
        *num_contours = 0;
        return null_mut();
    }
    // make two passes through the points so we don't need to realloc
    let mut pass = 0;
    loop {
        if pass >= 2 {
            current_block = 8_845_338_526_596_852_646;
            break;
        }
        let mut x: f32 = 0 as f32;
        let mut y: f32 = 0 as f32;
        if pass == 1 {
            points = stash.calloc(num_points as usize);
            if points.is_null() {
                current_block = 9_535_040_653_783_544_971;
                break;
            }
        }
        num_points = 0;
        n = -1;
        i = 0;
        while i < num_verts {
            match (*vertices.offset(i as isize)).type_0 as i32 {
                1 => {
                    if n >= 0 {
                        *(*contour_lengths).offset(n as isize) = num_points - start
                    }
                    n += 1;
                    start = num_points;
                    x = (*vertices.offset(i as isize)).x as f32;
                    y = (*vertices.offset(i as isize)).y as f32;
                    Points(points).add_point(num_points, x, y);
                    num_points += 1;
                }
                2 => {
                    x = (*vertices.offset(i as isize)).x as f32;
                    y = (*vertices.offset(i as isize)).y as f32;
                    Points(points).add_point(num_points, x, y);
                    num_points += 1;
                }
                3 => {
                    tesselate_curve(
                        points,
                        &mut num_points,
                        [
                            x,
                            y,
                            (*vertices.offset(i as isize)).cx as f32,
                            (*vertices.offset(i as isize)).cy as f32,
                            (*vertices.offset(i as isize)).x as f32,
                            (*vertices.offset(i as isize)).y as f32,
                        ],
                        objspace_flatness_squared,
                        0,
                    );
                    x = (*vertices.offset(i as isize)).x as f32;
                    y = (*vertices.offset(i as isize)).y as f32
                }
                _ => {}
            }
            i += 1
        }
        *(*contour_lengths).offset(n as isize) = num_points - start;
        pass += 1
    }

    match current_block {
        8_845_338_526_596_852_646 => points,
        _ => {
            *contour_lengths = null_mut();
            *num_contours = 0;
            null_mut()
        }
    }
}
// tesselate until threshhold p is happy... @TODO warped to compensate for non-linear stretching
unsafe fn tesselate_curve(
    points: *mut Point,
    num_points: *mut i32,

    curve: [f32; 6],

    objspace_flatness_squared: f32,
    n: i32,
) {
    let [x0, y0, x1, y1, x2, y2] = curve;

    // midpoint
    let mx = (x0 + 2.0 * x1 + x2) / 4.0;
    let my = (y0 + 2.0 * y1 + y2) / 4.0;
    // versus directly drawn line
    let dx = (x0 + x2) / 2.0 - mx;
    let dy = (y0 + y2) / 2.0 - my;
    if n > 16 { return; }

    if dx * dx + dy * dy > objspace_flatness_squared {
        tesselate_curve(
            points,
            num_points,
            [
                x0,
                y0,
                (x0 + x1) / 2.0,
                (y0 + y1) / 2.0,
                mx,
                my,
            ],
            objspace_flatness_squared,
            n + 1,
        );
        tesselate_curve(
            points,
            num_points,
            [
                mx,
                my,
                (x1 + x2) / 2.0,
                (y1 + y2) / 2.0,
                x2,
                y2,
            ],
            objspace_flatness_squared,
            n + 1,
        );
    } else {
        Points(points).add_point(*num_points, x2, y2);
        *num_points += 1;
    }
}

struct Points(*mut Point);

impl Points {
    fn add_point(self, n: i32, x: f32, y: f32) {
        unsafe {
            if self.0.is_null() { return; }
            (*self.0.offset(n as isize)).x = x;
            (*self.0.offset(n as isize)).y = y;
        }
    }
}

unsafe fn rasterize(
    result: *mut Bitmap,
    pts: *mut Point,
    wcount: *mut i32,
    windings: i32,
    scale: [f32; 2],
    shift: [f32; 2],
    off: [i32; 2],
    invert: i32,
    stash: &mut Stash,
) {
    let invert = invert != 0;

    let y_scale_inv: f32 = if invert { -scale[1] } else { scale[1] };
    let vsubsample = 1;

    let mut n = 0;
    for i in 0..windings {
        n += *wcount.offset(i as isize);
    }

    let edge: *mut Edge = stash.calloc((n + 1) as usize);
    if edge.is_null() {
        return;
    }

    let mut n = 0isize;
    let mut m = 0isize;
    for i in 0..windings as isize {
        let point: *mut Point = pts.offset(m);
        m += *wcount.offset(i) as isize;
        let mut j = *wcount.offset(i) as isize - 1;
        for k in 0..*wcount.offset(i) as isize {
            let mut a = k;
            let mut b = j;
            let (j_point, k_point) = (*point.offset(j), *point.offset(k));
            // skip the edge if horizontal
            if j_point.y != k_point.y {
                (*edge.offset(n)).invert = 0;

                if invert && j_point.y > k_point.y || !invert && j_point.y < k_point.y {
                    (*edge.offset(n)).invert = 1;
                    a = j;
                    b = k;
                }
                let a = *point.offset(a);
                let b = *point.offset(b);

                let edge = &mut *edge.offset(n);
                edge.x0 =  a.x * scale[0] + shift[0];
                edge.y0 = (a.y * y_scale_inv + shift[1]) * vsubsample as f32;
                edge.x1 =  b.x * scale[0] + shift[0];
                edge.y1 = (b.y * y_scale_inv + shift[1]) * vsubsample as f32;
                n += 1
            }
            j = k; 
        }
    }

    sort_edges(edge, n as i32);
    rasterize_sorted_edges(result, edge, n as i32, vsubsample, off[0], off[1], stash);
}
// directly AA rasterize edges w/o supersampling
unsafe fn rasterize_sorted_edges(
    result: *mut Bitmap,
    mut edge: *mut Edge,
    len: i32,
    _vsubsample: i32,
    off_x: i32,
    off_y: i32,
    stash: &mut Stash,
) {
    let mut hh: Heap = Heap {
        head: null_mut(),
        first_free: null_mut(),
        num_remaining_in_head_chunk: 0,
    };
    let mut active: *mut ActiveEdge = null_mut();
    let mut scanline_data: [f32; 129] = [0.; 129];

    let scanline = if (*result).w > 64 {
        stash.calloc(((*result).w * 2 + 1) as usize)
    } else {
        scanline_data.as_mut_ptr()
    };

    let scanline2 = scanline.offset((*result).w as isize);
    let mut y = off_y;
    (*edge.offset(len as isize)).y0 = (off_y + (*result).h) as f32 + 1.0;
    for j in 0..(*result).h {
        let scan_y_top: f32 = y as f32 + 0.0;
        let scan_y_bottom: f32 = y as f32 + 1.0;
        let mut step: *mut *mut ActiveEdge = &mut active;
        scanline.write_bytes(0, (*result).w as usize);
        scanline2.write_bytes(0, (*result).w as usize + 1);
        while !(*step).is_null() {
            let z: *mut ActiveEdge = *step;
            if (*z).ey <= scan_y_top {
                *step = (*z).next;
                assert!(0. != (*z).direction);
                (*z).direction = 0.0;
                hh.free(z as *mut libc::c_void);
            } else {
                step = &mut (**step).next
            }
        }
        while (*edge).y0 <= scan_y_bottom {
            if (*edge).y0 != (*edge).y1 {
                let mut z_0 = stbtt__new_active(&mut hh, edge, off_x, scan_y_top, stash);
                if !z_0.is_null() {
                    assert!((*z_0).ey >= scan_y_top);
                    (*z_0).next = active;
                    active = z_0
                }
            }
            edge = edge.offset(1)
        }
        if !active.is_null() {
            fill_active_edges_new(
                scanline,
                scanline2.offset(1),
                (*result).w,
                active,
                scan_y_top,
            );
        }
        let mut sum: f32 = 0 as f32;
        for i in 0..(*result).w {
            sum += *scanline2.offset(i as isize);
            let k = *scanline.offset(i as isize) + sum;
            let k = k.abs() * 255.0 + 0.5;
            let mut m = k as i32;
            if m > 255 {
                m = 255
            }
            *(*result).pixels.offset((j * (*result).stride + i) as isize) = m as u8;
        }
        step = &mut active;
        while !(*step).is_null() {
            let mut z_1: *mut ActiveEdge = *step;
            (*z_1).fx += (*z_1).fdx;
            step = &mut (**step).next
        }
        y += 1;
    }

    hh.cleanup();
}
unsafe fn fill_active_edges_new(
    scanline: *mut f32,
    scanline_fill: *mut f32,
    len: i32,
    mut e: *mut ActiveEdge,
    y_top: f32,
) {
    let y_bottom: f32 = y_top + 1.0;
    while !e.is_null() {
        assert!((*e).ey >= y_top);

        if (*e).fdx == 0.0 {
            let x0: f32 = (*e).fx;
            if x0 < len as f32 {
                if x0 >= 0.0 {
                    handle_clipped_edge(scanline, x0 as i32, e, x0, y_top, x0, y_bottom);
                    handle_clipped_edge(
                        scanline_fill.offset(-1),
                        x0 as i32 + 1,
                        e,
                        x0,
                        y_top,
                        x0,
                        y_bottom,
                    );
                } else {
                    handle_clipped_edge(scanline_fill.offset(-1), 0, e, x0, y_top, x0, y_bottom);
                }
            }
        } else {
            let mut x0_0: f32 = (*e).fx;
            let mut dx: f32 = (*e).fdx;
            let mut xb: f32 = x0_0 + dx;
            let mut dy: f32 = (*e).fdy;

            assert!((*e).sy <= y_bottom && (*e).ey >= y_top);

            let mut x_top;
            let mut sy0;
            if (*e).sy > y_top {
                x_top = x0_0 + dx * ((*e).sy - y_top);
                sy0 = (*e).sy
            } else {
                x_top = x0_0;
                sy0 = y_top
            }

            let mut x_bottom;
            let mut sy1;
            if (*e).ey < y_bottom {
                x_bottom = x0_0 + dx * ((*e).ey - y_top);
                sy1 = (*e).ey
            } else {
                x_bottom = xb;
                sy1 = y_bottom
            }

            if x_top >= 0.0 && x_bottom >= 0.0 && x_top < len as f32 && x_bottom < len as f32 {
                if x_top as i32 == x_bottom as i32 {
                    let x: i32 = x_top as i32;
                    let height = sy1 - sy0;
                    assert!(x >= 0 && x < len);
                    *scanline.offset(x as isize) += (*e).direction
                        * (1.0 - (x_top - x as f32 + (x_bottom - x as f32)) / 2.0)
                        * height;
                    *scanline_fill.offset(x as isize) += (*e).direction * height
                } else {
                    if x_top > x_bottom {
                        sy0 = y_bottom - (sy0 - y_top);
                        sy1 = y_bottom - (sy1 - y_top);

                        std::mem::swap(&mut sy0, &mut sy1);
                        std::mem::swap(&mut x_top, &mut x_bottom);
                        std::mem::swap(&mut x0_0, &mut xb);

                        dx = -dx;
                        dy = -dy;
                    }
                    let x1 = x_top as i32;
                    let x2 = x_bottom as i32;
                    let mut y_crossing = ((x1 + 1) as f32 - x0_0) * dy + y_top;
                    let sign = (*e).direction;
                    let mut area = sign * (y_crossing - sy0);
                    *scanline.offset(x1 as isize) += area
                        * (1.0 - (x_top - x1 as f32 + (x1 + 1 - x1) as f32) / 2.0);
                    let step = sign * dy;
                    let mut x_0 = x1 + 1;
                    while x_0 < x2 {
                        *scanline.offset(x_0 as isize) += area + step / 2.0;
                        area += step;
                        x_0 += 1
                    }
                    y_crossing += dy * (x2 - (x1 + 1)) as f32;
                    assert!(area.abs() <= 1.01);
                    *scanline.offset(x2 as isize) += area
                        + sign
                            * (1.0 - ((x2 - x1) as f32 + (x_bottom - x2 as f32)) / 2.0)
                            * (sy1 - y_crossing);
                    *scanline_fill.offset(x2 as isize) += sign * (sy1 - sy0)
                }
            } else {
                let mut x_1 = 0;
                while x_1 < len {
                    let y0: f32 = y_top;
                    let x1_0: f32 = x_1 as f32;
                    let x2_0: f32 = (x_1 + 1) as f32;
                    let x3: f32 = xb;
                    let y3: f32 = y_bottom;
                    let y1 = (x_1 as f32 - x0_0) / dx + y_top;
                    let y2 = ((x_1 + 1) as f32 - x0_0) / dx + y_top;
                    if x0_0 < x1_0 && x3 > x2_0 {
                        handle_clipped_edge(scanline, x_1, e, x0_0, y0, x1_0, y1);
                        handle_clipped_edge(scanline, x_1, e, x1_0, y1, x2_0, y2);
                        handle_clipped_edge(scanline, x_1, e, x2_0, y2, x3, y3);
                    } else if x3 < x1_0 && x0_0 > x2_0 {
                        handle_clipped_edge(scanline, x_1, e, x0_0, y0, x2_0, y2);
                        handle_clipped_edge(scanline, x_1, e, x2_0, y2, x1_0, y1);
                        handle_clipped_edge(scanline, x_1, e, x1_0, y1, x3, y3);
                    } else if x0_0 < x1_0 && x3 > x1_0 {
                        handle_clipped_edge(scanline, x_1, e, x0_0, y0, x1_0, y1);
                        handle_clipped_edge(scanline, x_1, e, x1_0, y1, x3, y3);
                    } else if x3 < x1_0 && x0_0 > x1_0 {
                        handle_clipped_edge(scanline, x_1, e, x0_0, y0, x1_0, y1);
                        handle_clipped_edge(scanline, x_1, e, x1_0, y1, x3, y3);
                    } else if x0_0 < x2_0 && x3 > x2_0 {
                        handle_clipped_edge(scanline, x_1, e, x0_0, y0, x2_0, y2);
                        handle_clipped_edge(scanline, x_1, e, x2_0, y2, x3, y3);
                    } else if x3 < x2_0 && x0_0 > x2_0 {
                        handle_clipped_edge(scanline, x_1, e, x0_0, y0, x2_0, y2);
                        handle_clipped_edge(scanline, x_1, e, x2_0, y2, x3, y3);
                    } else {
                        handle_clipped_edge(scanline, x_1, e, x0_0, y0, x3, y3);
                    }
                    x_1 += 1
                }
            }
        }
        e = (*e).next
    }
}
// the edge passed in here does not cross the vertical line at x or the vertical line at x+1
// (i.e. it has already been clipped to those)
unsafe fn handle_clipped_edge(
    scanline: *mut f32,
    x: i32,
    e: *mut ActiveEdge,
    mut x0: f32, mut y0: f32,
    mut x1: f32, mut y1: f32,
) {
    if y0 == y1 {
        return;
    }
    assert!(y0 < y1);

    assert!((*e).sy <= (*e).ey);
    if y0 > (*e).ey {
        return;
    }
    if y1 < (*e).sy {
        return;
    }
    if y0 < (*e).sy {
        x0 += (x1 - x0) * ((*e).sy - y0) / (y1 - y0);
        y0 = (*e).sy
    }
    if y1 > (*e).ey {
        x1 += (x1 - x0) * ((*e).ey - y1) / (y1 - y0);
        y1 = (*e).ey
    }

    if x0 == x as f32 {
        assert!(x1 <= (x + 1) as f32);
    } else if x0 == (x + 1) as f32 {
        assert!(x1 >= x as f32);
    } else if x0 <= x as f32 {
        assert!(x1 <= x as f32);
    } else if x0 >= (x + 1) as f32 {
        assert!(x1 >= (x + 1) as f32);
    } else if x1 >= x as f32 && x1 <= (x + 1) as f32 {
    } else {
        panic!("wtf?");
    }
    if x0 <= x as f32 && x1 <= x as f32 {
        *scanline.offset(x as isize) += (*e).direction * (y1 - y0)
    } else if !(x0 >= (x + 1) as f32 && x1 >= (x + 1) as f32) {
        assert!(x0 >= x as f32 && x0 <= (x + 1) as f32 && x1 >= x as f32 && x1 <= (x + 1) as f32);
        *scanline.offset(x as isize) +=
            (*e).direction * (y1 - y0) * (1.0 - (x0 - x as f32 + (x1 - x as f32)) / 2.0)
    };
}

unsafe fn stbtt__new_active(
    hh: *mut Heap,
    e: *mut Edge,
    off_x: i32,
    start_point: f32,
    stash: &mut Stash,
) -> *mut ActiveEdge {
    let z = (*hh).alloc(size_of::<ActiveEdge>() as u64, stash) as *mut ActiveEdge;
    let dxdy: f32 = ((*e).x1 - (*e).x0) / ((*e).y1 - (*e).y0);
    assert!(!z.is_null());
    if z.is_null() {
        return z;
    }

    let z = &mut *z;
    z.fdx = dxdy;
    z.fdy = if dxdy != 0.0 { 1.0 / dxdy } else { 0.0 };
    z.fx = (*e).x0 + dxdy * (start_point - (*e).y0);
    z.fx -= off_x as f32;
    z.direction = if 0 != (*e).invert { 1.0 } else { -1.0 };
    z.sy = (*e).y0;
    z.ey = (*e).y1;
    z.next = null_mut();
    z
}

unsafe fn sort_edges(edges: *mut Edge, len: i32) {
    sort_edges_quicksort(edges, len);

    let edges = std::slice::from_raw_parts_mut(edges, len as usize);

    for i in 0..edges.len() {
        let first = edges[i];
        let mut j = i;
        while j > 0 {
            let second = &edges[j - 1];
            if first.y0 >= second.y0 {
                break;
            }
            edges[j] = *second;
            j -= 1;
        }
        if i != j {
            edges[j] = first;
        }
    }
}
unsafe fn sort_edges_quicksort(mut ptr: *mut Edge, len: i32) {
    let mut len = len as isize;
    while len > 12 {
        let m = len >> 1;
        let c01 = (*ptr.offset(0)).y0 < (*ptr.offset(m)).y0;
        let c12 = (*ptr.offset(m)).y0 < (*ptr.offset(len - 1)).y0;
        if c01 != c12 {
            let c = (*ptr.offset(0)).y0 < (*ptr.offset(len - 1)).y0;
            let z = if c == c12 { 0 } else { len - 1 };
            std::ptr::swap(ptr.offset(z), ptr.offset(m));
        }

        std::ptr::swap(ptr.offset(0), ptr.offset(m));

        let mut i = 1;
        let mut j = len - 1;
        loop {
            while (*ptr.offset(i)).y0 < (*ptr.offset(0)).y0 {
                i += 1
            }
            while (*ptr.offset(0)).y0 < (*ptr.offset(j)).y0 {
                j -= 1
            }
            /* make sure we haven't crossed */
            if i >= j {
                break;
            }

            std::ptr::swap(ptr.offset(i), ptr.offset(j));

            i += 1;
            j -= 1;
        }

        if j < len - i {
            sort_edges_quicksort(ptr, j as i32);
            ptr = ptr.offset(i);
            len -= i
        } else {
            sort_edges_quicksort(ptr.offset(i), (len - i) as i32);
            len = j
        }
    }
}

fn hashint(mut a: u32) -> u32 {
    a = a.wrapping_add(!(a << 15));
    a ^= a >> 10;
    a = a.wrapping_add(a << 3);
    a ^= a >> 6;
    a = a.wrapping_add(!(a << 11));
    a ^= a >> 16;
    a
}

impl Font {
    unsafe fn alloc_glyph(&mut self) -> *mut Glyph {
        self.glyphs.push(std::mem::zeroed());
        self.glyphs.last_mut().unwrap()
    }

    fn vert_align(&self, align: i32, isize: i16) -> f32 {
        if 0 != align & FONS_ALIGN_TOP as i32 {
            self.ascender * isize as f32 / 10.0
        } else if 0 != align & FONS_ALIGN_MIDDLE as i32 {
            (self.ascender + self.descender) / 2.0 * isize as f32 / 10.0
        } else if 0 != align & FONS_ALIGN_BASELINE as i32 {
            0.0
        } else if 0 != align & FONS_ALIGN_BOTTOM as i32 {
            self.descender * isize as f32 / 10.0
        } else {
            0.0
        }
    }
}


impl Stash {
    pub fn text_bounds(&mut self, mut x: f32, mut y: f32, mut str: *const u8, end: *const u8) -> (f32, Rect) {
        assert!(!end.is_null());
        unsafe {
            let state: *mut State = self.state_mut();
            let mut codepoint = 0;
            let mut utf8state = 0;
            let mut q: Quad = Default::default();
            let mut prevGlyphIndex = -1;
            let isize: i16 = ((*state).size * 10.0) as i16;
            let iblur: i16 = (*state).blur as i16;

            if (*state).font < 0 || (*state).font >= self.fonts.len() as i32 {
                return Default::default();
            }

            let font: *mut Font = {
                &mut self.fonts[(*state).font as usize]
            };

            let scale = (*font).font.pixel_height_scale(isize as f32 / 10.0);
            y += (*font).vert_align((*state).align, isize);

            let mut maxx = x;
            let mut minx = maxx;
            let mut maxy = y;
            let mut miny = maxy;
            let startx = x;
            while str != end {
                if 0 == decutf8(&mut utf8state, &mut codepoint, *(str as *const u8) as u32) {
                    let glyph = self.get_glyph(
                        font,
                        codepoint,
                        isize,
                        iblur,
                        FONS_GLYPH_BITMAP_OPTIONAL as i32,
                    );
                    if !glyph.is_null() {
                        self.get_quad(
                            font,
                            prevGlyphIndex,
                            &*glyph,
                            scale,
                            (*state).spacing,
                            &mut x,
                            &mut y,
                            &mut q,
                        );

                        minx = minx.min(q.x0);
                        maxx = maxx.max(q.x1);

                        miny = miny.min(q.y0);
                        maxy = maxy.max(q.y1);
                    }
                    prevGlyphIndex = if !glyph.is_null() { (*glyph).index } else { -1 }
                }
                str = str.offset(1)
            }
            let advance = x - startx;
            if 0 == (*state).align & FONS_ALIGN_LEFT as i32 {
                if 0 != (*state).align & FONS_ALIGN_RIGHT as i32 {
                    minx -= advance;
                    maxx -= advance
                } else if 0 != (*state).align & FONS_ALIGN_CENTER as i32 {
                    minx -= advance * 0.5;
                    maxx -= advance * 0.5;
                }
            }

            (advance, Rect { min: point2(minx, miny), max: point2(maxx, maxy) })
        }
    }

    pub fn line_bounds(&self, y: f32) -> (f32, f32) {
        let state = self.state();
        if state.font < 0 || state.font >= self.fonts.len() as i32 {
            return Default::default();
        }
        let align = state.align;
        let isize = (state.size * 10.0) as i16;

        let font = &self.fonts[state.font as usize];
        let y = y + font.vert_align(align, isize);
        let miny = y - font.ascender * isize as f32 / 10.0;
        let maxy = miny + font.lineh * isize as f32 / 10.0;
        (miny, maxy)
    }

    pub fn metrics(&self) -> Option<Metrics> {
        let state = self.state();
        if state.font < 0 || state.font >= self.fonts.len() as i32 {
            return None;
        }
        let font = &self.fonts[state.font as usize];
        let size = ((state.size * 10.0) as i16) as f32;
        Some(Metrics {
            ascender: font.ascender * size / 10.,
            descender: font.descender * size / 10.,
            line_gap: font.lineh * size / 10.,
        })
    }

    pub fn add_fallback_font(&mut self, base: i32, fallback: i32) -> i32 {
        let base = &mut self.fonts[base as usize];
        if base.nfallbacks < 20 {
            base.fallbacks[base.nfallbacks as usize] = fallback;
            base.nfallbacks += 1;
            1
        } else {
            0
        }
    }
}

// Text iterator

pub unsafe fn fonsTextIterInit(
    stash: *mut Stash,
    mut iter: &mut TextIter,
    mut x: f32,
    mut y: f32,
    str: *const u8,
    end: *const u8,
    bitmapOption: i32,
) -> i32 {
    assert!(!end.is_null());
    let state: *mut State = (*stash).state_mut();

    if stash.is_null() {
        return 0;
    }
    *iter = std::mem::zeroed();

    if (*state).font < 0 || (*state).font >= (*stash).fonts.len() as i32 {
        return 0;
    }
    iter.font = &mut(*stash).fonts[(*state).font as usize];
    iter.isize_0 = ((*state).size * 10.0) as i16;
    iter.iblur = (*state).blur as i16;
    iter.scale = (*iter.font)
        .font
        .pixel_height_scale(iter.isize_0 as f32 / 10.0);
    if 0 == (*state).align & FONS_ALIGN_LEFT as i32 {
        if 0 != (*state).align & FONS_ALIGN_RIGHT as i32 {
            let (width, _) = (*stash).text_bounds(x, y, str, end);
            x -= width;
        } else if 0 != (*state).align & FONS_ALIGN_CENTER as i32 {
            let (width, _) = (*stash).text_bounds(x, y, str, end);
            x -= width * 0.5;
        }
    }

    y += (*iter.font).vert_align((*state).align, iter.isize_0);

    iter.nextx = x;
    iter.nexty = y;

    iter.x = iter.nextx;
    iter.y = iter.nexty;

    iter.spacing = (*state).spacing;
    iter.str_0 = str;
    iter.next = str;
    iter.end = end;
    iter.codepoint = 0 as u32;
    iter.prev_glyph_index = -1;
    iter.bitmapOption = bitmapOption;
    1
}