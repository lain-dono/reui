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

#![feature(extern_types, label_break_value)]

pub struct Metrics {
    pub ascender: f32,
    pub descender: f32,
    pub line_height: f32,
}

use std::mem::transmute;
use std::ptr::null_mut;
unsafe fn read_i16(p: *const u8) -> i16 {
    (*p.offset(0) as i16).wrapping_mul(256) + (*p.offset(1) as i16)
}
unsafe fn read_u16(p: *const u8) -> u16 {
    (*p.offset(0) as u16) * 256 + (*p.offset(1) as u16)
}
unsafe fn read_u32(p: *const u8) -> u32 {
    (((*p.offset(0) as i32) << 24)
        + ((*p.offset(1) as i32) << 16)
        + ((*p.offset(2) as i32) << 8)
        + *p.offset(3) as i32) as u32
}

extern crate libc;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;

    fn malloc(_: u64) -> *mut libc::c_void;
    fn realloc(_: *mut libc::c_void, _: u64) -> *mut libc::c_void;
    fn free(__ptr: *mut libc::c_void);
    fn fclose(__stream: *mut FILE) -> i32;
    fn fopen(_: *const i8, _: *const i8) -> *mut FILE;
    fn fread(_: *mut libc::c_void, _: u64, _: u64, _: *mut FILE) -> u64;
    fn fseek(__stream: *mut FILE, __off: libc::c_long, __whence: i32) -> i32;
    fn ftell(__stream: *mut FILE) -> libc::c_long;

    fn sqrt(_: f64) -> f64;
    fn fabs(_: f64) -> f64;
    fn expf(_: f32) -> f32;

    fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: u64) -> *mut libc::c_void;
    fn memset(_: *mut libc::c_void, _: i32, _: u64) -> *mut libc::c_void;
    fn strncpy(_: *mut i8, _: *const i8, _: u64) -> *mut i8;
    fn strcmp(_: *const i8, _: *const i8) -> i32;
    fn strlen(_: *const i8) -> u64;

    fn __assert_fail(
        __assertion: *const i8,
        __file: *const i8,
        __line: u32,
        __function: *const i8,
    ) -> !;
}

pub type size_t = u64;
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: i32,
    pub _IO_read_ptr: *mut i8,
    pub _IO_read_end: *mut i8,
    pub _IO_read_base: *mut i8,
    pub _IO_write_base: *mut i8,
    pub _IO_write_ptr: *mut i8,
    pub _IO_write_end: *mut i8,
    pub _IO_buf_base: *mut i8,
    pub _IO_buf_end: *mut i8,
    pub _IO_save_base: *mut i8,
    pub _IO_backup_base: *mut i8,
    pub _IO_save_end: *mut i8,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: i32,
    pub _flags2: i32,
    pub _old_offset: __off_t,
    pub _cur_column: u16,
    pub _vtable_offset: libc::c_schar,
    pub _shortbuf: [i8; 1],
    pub _lock: *mut libc::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut libc::c_void,
    pub __pad5: size_t,
    pub _mode: i32,
    pub _unused2: [i8; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
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
pub struct FONStextIter {
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
    pub prevGlyphIndex: i32,
    pub str_0: *const i8,
    pub next: *const i8,
    pub end: *const i8,
    pub utf8state: u32,
    pub bitmapOption: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Font {
    pub font: FontInfo,
    pub name: [i8; 64],
    pub data: *mut u8,
    pub dataSize: i32,
    pub freeData: u8,
    pub ascender: f32,
    pub descender: f32,
    pub lineh: f32,
    pub glyphs: *mut Glyph,
    pub cglyphs: i32,
    pub nglyphs: i32,
    pub lut: [i32; 256],
    pub fallbacks: [i32; 20],
    pub nfallbacks: i32,
}

#[derive(Copy, Clone)]
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

// Each .ttf/.ttc file may have more than one font. Each font has a sequential
// index number starting from 0. Call this function to get the font offset for
// a given index; it returns -1 if the index is out of range. A regular .ttf
// file will only define one font and it always be at offset 0, so it will
// return '0' for index 0, and -1 for all other indices. You can just skip
// this step if you know it's that kind of font.
// The following structure is defined publically so you can declare one on
// the stack or as a global or etc, but you should treat it as opaque.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FontInfo {
    pub userdata: *mut libc::c_void,
    pub data: *mut u8,
    pub fontstart: i32,
    pub numGlyphs: i32,
    pub loca: i32,
    pub head: i32,
    pub glyf: i32,
    pub hhea: i32,
    pub hmtx: i32,
    pub kern: i32,
    pub index_map: i32,
    pub indexToLocFormat: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Stash {
    pub width: i32,
    pub height: i32,

    pub itw: f32,
    pub ith: f32,
    pub texData: *mut u8,
    pub dirty_rect: [i32; 4],

    pub atlas: *mut Atlas,

    pub fonts: *mut *mut Font,
    pub cfonts: usize,
    pub nfonts: usize,

    pub verts: [f32; 2048],
    pub tcoords: [f32; 2048],
    pub colors: [u32; 1024],

    pub nverts: i32,

    pub scratch: *mut u8,
    pub nscratch: i32,

    pub states: [State; 20],
    pub nstates: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct State {
    pub font: i32,
    pub align: i32,
    pub size: f32,
    pub color: u32,
    pub blur: f32,
    pub spacing: f32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Atlas {
    pub width: i32,
    pub height: i32,
    pub nodes: *mut AtlasNode,
    pub nnodes: i32,
    pub cnodes: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct AtlasNode {
    pub x: i16,
    pub y: i16,
    pub width: i16,
}

// you can predefine this to use different values
// (we share this with other code at RAD)
// can't use i16 because that's not visible in the header file
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbtt_vertex {
    pub x: i16,
    pub y: i16,
    pub cx: i16,
    pub cy: i16,
    pub type_0: u8,
    pub padding: u8,
}

pub const STBTT_vline: unnamed = 2;
pub const STBTT_vcurve: unnamed = 3;
pub const STBTT_vmove: unnamed = 1;

// @TODO: don't expose this structure
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbtt__bitmap {
    pub w: i32,
    pub h: i32,
    pub stride: i32,
    pub pixels: *mut u8,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbtt__point {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbtt__edge {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub invert: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbtt__hheap {
    pub head: *mut stbtt__hheap_chunk,
    pub first_free: *mut libc::c_void,
    pub num_remaining_in_head_chunk: i32,
}

// ////////////////////////////////////////////////////////////////////////////
//
//  Rasterizer
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbtt__hheap_chunk {
    pub next: *mut stbtt__hheap_chunk,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbtt__active_edge {
    pub next: *mut stbtt__active_edge,
    pub fx: f32,
    pub fdx: f32,
    pub fdy: f32,
    pub direction: f32,
    pub sy: f32,
    pub ey: f32,
}
// #define your own STBTT_ifloor/STBTT_iceil() to avoid math.h
// #define your own functions "STBTT_malloc" / "STBTT_free" to avoid malloc.h
// /////////////////////////////////////////////////////////////////////////////
// /////////////////////////////////////////////////////////////////////////////
// //
// //   INTERFACE
// //
// //
// ////////////////////////////////////////////////////////////////////////////
//
// TEXTURE BAKING API
//
// If you use this API, you only have to call two functions ever.
//
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbtt_bakedchar {
    pub x0: u16,
    pub y0: u16,
    pub x1: u16,
    pub y1: u16,
    pub xoff: f32,
    pub yoff: f32,
    pub xadvance: f32,
}
// height of font in pixels
// bitmap to be filled in
// characters to bake
// you allocate this, it's num_chars long
// if return is positive, the first unused row of the bitmap
// if return is negative, returns the negative of the number of characters that fit
// if return is 0, no characters fit and no rows were used
// This uses a very crappy packing.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbtt_aligned_quad {
    pub x0: f32,
    pub y0: f32,
    pub s0: f32,
    pub t0: f32,
    pub x1: f32,
    pub y1: f32,
    pub s1: f32,
    pub t1: f32,
}
// character to display
// pointers to current position in screen pixel space
// output: quad to draw
// true if opengl fill rule; false if DX9 or earlier
// Call GetBakedQuad with char_index = 'character - first_char', and it
// creates the quad you need to draw and advances the current position.
//
// The coordinate system used assumes y increases downwards.
//
// Characters will extend both above and below the current position;
// see discussion of "BASELINE" above.
//
// It's inefficient; you might want to c&p it and optimize it.
// ////////////////////////////////////////////////////////////////////////////
//
// NEW TEXTURE BAKING API
//
// This provides options for packing multiple fonts into one atlas, not
// perfectly but better than nothing.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbtt_packedchar {
    pub x0: u16,
    pub y0: u16,
    pub x1: u16,
    pub y1: u16,
    pub xoff: f32,
    pub yoff: f32,
    pub xadvance: f32,
    pub xoff2: f32,
    pub yoff2: f32,
}
// Calling these functions in sequence is roughly equivalent to calling
// stbtt_PackFontRanges(). If you more control over the packing of multiple
// fonts, or if you want to pack custom data into a font texture, take a look
// at the source to of stbtt_PackFontRanges() and create a custom version
// using these functions, e.g. call GatherRects multiple times,
// building up a single array of rects, then call PackRects once,
// then call RenderIntoRects repeatedly. This may result in a
// better packing than calling PackFontRanges multiple times
// (or it may not).
// this is an opaque structure that you shouldn't mess with which holds
// all the context needed from PackBegin to PackEnd.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbtt_pack_context {
    pub user_allocator_context: *mut libc::c_void,
    pub pack_info: *mut libc::c_void,
    pub width: i32,
    pub height: i32,
    pub stride_in_bytes: i32,
    pub padding: i32,
    pub h_oversample: u32,
    pub v_oversample: u32,
    pub pixels: *mut u8,
    pub nodes: *mut libc::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbrp_rect {
    pub x: stbrp_coord,
    pub y: stbrp_coord,
    pub id: i32,
    pub w: i32,
    pub h: i32,
    pub was_packed: i32,
}
// ////////////////////////////////////////////////////////////////////////////
//
// rectangle packing replacement routines if you don't have stb_rect_pack.h
//
pub type stbrp_coord = i32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbrp_node {
    pub x: u8,
}
// //////////////////////////////////////////////////////////////////////////////////
//                                                                                //
//                                                                                //
// COMPILER WARNING ?!?!?                                                         //
//                                                                                //
//                                                                                //
// if you get a compile warning due to these symbols being defined more than      //
// once, move #include "stb_rect_pack.h" before #include "stb_truetype.h"         //
//                                                                                //
// //////////////////////////////////////////////////////////////////////////////////
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbrp_context {
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
    pub bottom_y: i32,
}
// Creates character bitmaps from the font_index'th font found in fontdata (use
// font_index=0 if you don't know what that is). It creates num_chars_in_range
// bitmaps for characters with unicode values starting at first_unicode_char_in_range
// and increasing. Data for how to render them is stored in chardata_for_range;
// pass these to stbtt_GetPackedQuad to get back renderable quads.
//
// font_size is the full height of the character from ascender to descender,
// as computed by stbtt_ScaleForPixelHeight. To use a point size as computed
// by stbtt_ScaleForMappingEmToPixels, wrap the point size in STBTT_POINT_SIZE()
// and pass that result as 'font_size':
//       ...,                  20 , ... // font max minus min y is 20 pixels tall
//       ..., STBTT_POINT_SIZE(20), ... // 'M' is 20 pixels tall
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbtt_pack_range {
    pub font_size: f32,
    pub first_unicode_codepoint_in_range: i32,
    pub array_of_unicode_codepoints: *mut i32,
    pub num_chars: i32,
    pub chardata_for_range: *mut stbtt_packedchar,
    pub h_oversample: u8,
    pub v_oversample: u8,
}

pub type unnamed = u32;

pub unsafe fn fonsCreateInternal(width: i32, height: i32) -> *mut Stash {
    let stash = malloc(::std::mem::size_of::<Stash>() as u64) as *mut Stash;
    if !stash.is_null() {
        memset(
            stash as *mut libc::c_void,
            0,
            ::std::mem::size_of::<Stash>() as u64,
        );
        (*stash).width = width;
        (*stash).height = height;
        (*stash).scratch = malloc(96000) as *mut u8;
        if !(*stash).scratch.is_null() {
            // Initialize implementation library
            (*stash).atlas = fons__allocAtlas(width, height, 256);
            if !(*stash).atlas.is_null() {
                (*stash).fonts =
                    malloc((::std::mem::size_of::<*mut Font>() as u64).wrapping_mul(4i32 as u64))
                        as *mut *mut Font;
                if !(*stash).fonts.is_null() {
                    memset(
                        (*stash).fonts as *mut libc::c_void,
                        0,
                        (::std::mem::size_of::<*mut Font>() as u64).wrapping_mul(4i32 as u64),
                    );
                    (*stash).cfonts = 4;
                    (*stash).nfonts = 0;
                    (*stash).itw = 1.0f32 / width as f32;
                    (*stash).ith = 1.0f32 / height as f32;
                    (*stash).texData =
                        malloc((width * height) as u64) as *mut u8;
                    if !(*stash).texData.is_null() {
                        memset(
                            (*stash).texData as *mut libc::c_void,
                            0,
                            (width * height) as u64,
                        );
                        (*stash).dirty_rect = [width, height, 0, 0];
                        fons__addWhiteRect(stash, 2i32, 2i32);
                        fonsPushState(stash);
                        (*stash).clear_state();
                        return stash;
                    }
                }
            }
        }
    }
    fonsDeleteInternal(stash);
    null_mut()
}

pub unsafe fn fonsDeleteInternal(stash: *mut Stash) {
    if stash.is_null() {
        return;
    }
    for i in 0..(*stash).nfonts {
        fons__freeFont(*(*stash).fonts.add(i));
    }
    if !(*stash).atlas.is_null() {
        fons__deleteAtlas((*stash).atlas);
    }
    if !(*stash).fonts.is_null() {
        free((*stash).fonts as *mut libc::c_void);
    }
    if !(*stash).texData.is_null() {
        free((*stash).texData as *mut libc::c_void);
    }
    if !(*stash).scratch.is_null() {
        free((*stash).scratch as *mut libc::c_void);
    }
    free(stash as *mut libc::c_void);
}

// Atlas based on Skyline Bin Packer by Jukka Jylänki

pub unsafe fn fons__deleteAtlas(atlas: *mut Atlas) {
    if atlas.is_null() {
        return;
    }
    if !(*atlas).nodes.is_null() {
        free((*atlas).nodes as *mut libc::c_void);
    }
    free(atlas as *mut libc::c_void);
}

pub unsafe fn fons__freeFont(font: *mut Font) {
    if font.is_null() {
        return;
    }
    if !(*font).glyphs.is_null() {
        free((*font).glyphs as *mut libc::c_void);
    }
    if 0 != (*font).freeData && !(*font).data.is_null() {
        free((*font).data as *mut libc::c_void);
    }
    free(font as *mut libc::c_void);
}

// State setting

impl Stash {
    pub fn state(&self) -> &State {
        unsafe { &*fons__getState(self as *const Self as *mut Self) }
    }
    pub fn state_mut(&mut self) -> &mut State {
        unsafe { &mut *fons__getState(self) }
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

}

unsafe fn fons__getState(stash: *mut Stash) -> *mut State {
    &mut *(*stash)
        .states
        .as_mut_ptr()
        .offset(((*stash).nstates - 1) as isize) as *mut State
}

// State handling

unsafe fn fonsPushState(mut stash: *mut Stash) {
    if (*stash).nstates >= 20 {
        return;
    }
    if (*stash).nstates > 0 {
        memcpy(
            &mut *(*stash)
                .states
                .as_mut_ptr()
                .offset((*stash).nstates as isize) as *mut State as *mut libc::c_void,
            &mut *(*stash)
                .states
                .as_mut_ptr()
                .offset(((*stash).nstates - 1) as isize) as *mut State
                as *const libc::c_void,
            ::std::mem::size_of::<State>() as u64,
        );
    }
    (*stash).nstates += 1;
}

pub unsafe fn fons__addWhiteRect(mut stash: *mut Stash, w: i32, h: i32) {
    let mut gx: i32 = 0;
    let mut gy: i32 = 0;
    if fons__atlasAddRect((*stash).atlas, w, h, &mut gx, &mut gy) == 0 {
        return;
    }
    let mut dst = &mut *(*stash)
        .texData
        .offset((gx + gy * (*stash).width) as isize) as *mut u8;
    let mut y = 0;
    while y < h {
        let mut x = 0;
        while x < w {
            *dst.offset(x as isize) = 0xffi32 as u8;
            x += 1
        }
        dst = dst.offset((*stash).width as isize);
        y += 1
    }
    (*stash).dirty_rect = [
        mini((*stash).dirty_rect[0], gx),
        mini((*stash).dirty_rect[1], gy),
        maxi((*stash).dirty_rect[2], gx + w),
        maxi((*stash).dirty_rect[3], gy + h),
    ];
}

pub fn maxi(a: i32, b: i32) -> i32 {
    if a > b {
        a
    } else {
        b
    }
}

pub fn mini(a: i32, b: i32) -> i32 {
    if a < b {
        a
    } else {
        b
    }
}

pub unsafe fn fons__atlasAddRect(
    atlas: *mut Atlas,
    rw: i32,
    rh: i32,
    rx: *mut i32,
    ry: *mut i32,
) -> i32 {
    let mut besth: i32 = (*atlas).height;
    let mut bestw: i32 = (*atlas).width;
    let mut besti: i32 = -1;
    let mut bestx: i32 = -1;
    let mut besty: i32 = -1;
    for i in 0..(*atlas).nnodes {
        let y = fons__atlasRectFits(atlas, i, rw, rh);
        if y != -1 {
            let node = &*(*atlas).nodes.offset(i as isize);
            if y + rh < besth || y + rh == besth && (node.width as i32) < bestw {
                besti = i;
                bestw = node.width as i32;
                besth = y + rh;
                bestx = node.x as i32;
                besty = y
            }
        }
    }
    if besti == -1 {
        return 0;
    }
    if fons__atlasAddSkylineLevel(atlas, besti, bestx, besty, rw, rh) == 0 {
        return 0;
    }
    *rx = bestx;
    *ry = besty;
    1
}

pub unsafe fn fons__atlasAddSkylineLevel(
    atlas: *mut Atlas,
    idx: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> i32 {
    let mut i: i32 = 0;
    if fons__atlasInsertNode(atlas, idx, x, y + h, w) == 0 {
        return 0;
    }
    i = idx + 1;
    while i < (*atlas).nnodes {
        if !(((*(*atlas).nodes.offset(i as isize)).x as i32)
            < (*(*atlas).nodes.offset((i - 1) as isize)).x as i32
                + (*(*atlas).nodes.offset((i - 1) as isize)).width as i32)
        {
            break;
        }
        let mut shrink: i32 = (*(*atlas).nodes.offset((i - 1) as isize)).x as i32
            + (*(*atlas).nodes.offset((i - 1) as isize)).width as i32
            - (*(*atlas).nodes.offset(i as isize)).x as i32;
        let ref mut fresh0 = (*(*atlas).nodes.offset(i as isize)).x;
        *fresh0 = (*fresh0 as i32 + shrink as i16 as i32) as i16;
        let ref mut fresh1 = (*(*atlas).nodes.offset(i as isize)).width;
        *fresh1 = (*fresh1 as i32 - shrink as i16 as i32) as i16;
        if !((*(*atlas).nodes.offset(i as isize)).width as i32 <= 0) {
            break;
        }
        fons__atlasRemoveNode(atlas, i);
        i -= 1;
        i += 1
    }
    i = 0;
    while i < (*atlas).nnodes - 1 {
        if (*(*atlas).nodes.offset(i as isize)).y as i32
            == (*(*atlas).nodes.offset((i + 1) as isize)).y as i32
        {
            let ref mut fresh2 = (*(*atlas).nodes.offset(i as isize)).width;
            *fresh2 =
                (*fresh2 as i32 + (*(*atlas).nodes.offset((i + 1) as isize)).width as i32) as i16;
            fons__atlasRemoveNode(atlas, i + 1);
            i -= 1
        }
        i += 1
    }
    1
}

pub unsafe fn fons__atlasRemoveNode(mut atlas: *mut Atlas, idx: i32) {
    let mut i: i32 = 0;
    if (*atlas).nnodes == 0 {
        return;
    }
    i = idx;
    while i < (*atlas).nnodes - 1 {
        *(*atlas).nodes.offset(i as isize) = *(*atlas).nodes.offset((i + 1) as isize);
        i += 1
    }
    (*atlas).nnodes -= 1;
}

pub unsafe fn fons__atlasInsertNode(
    mut atlas: *mut Atlas,
    idx: i32,
    x: i32,
    y: i32,
    w: i32,
) -> i32 {
    let mut i: i32 = 0;
    if (*atlas).nnodes + 1 > (*atlas).cnodes {
        (*atlas).cnodes = if (*atlas).cnodes == 0 {
            8i32
        } else {
            (*atlas).cnodes * 2i32
        };
        (*atlas).nodes = realloc(
            (*atlas).nodes as *mut libc::c_void,
            (::std::mem::size_of::<AtlasNode>() as u64).wrapping_mul((*atlas).cnodes as u64),
        ) as *mut AtlasNode;
        if (*atlas).nodes.is_null() {
            return 0;
        }
    }
    i = (*atlas).nnodes;
    while i > idx {
        *(*atlas).nodes.offset(i as isize) = *(*atlas).nodes.offset((i - 1) as isize);
        i -= 1
    }
    (*(*atlas).nodes.offset(idx as isize)).x = x as i16;
    (*(*atlas).nodes.offset(idx as isize)).y = y as i16;
    (*(*atlas).nodes.offset(idx as isize)).width = w as i16;
    (*atlas).nnodes += 1;
    1
}

pub unsafe fn fons__atlasRectFits(atlas: *mut Atlas, mut i: i32, w: i32, h: i32) -> i32 {
    // Checks if there is enough space at the location of skyline span 'i',
    // and return the max height of all skyline spans under that at that location,
    // (think tetris block being dropped at that position). Or -1 if no space found.
    let mut x = (*(*atlas).nodes.offset(i as isize)).x as i32;
    let mut y = (*(*atlas).nodes.offset(i as isize)).y as i32;
    if x + w > (*atlas).width {
        return -1;
    }
    let mut spaceLeft = w;
    while spaceLeft > 0 {
        if i == (*atlas).nnodes {
            return -1;
        }
        y = maxi(y, (*(*atlas).nodes.offset(i as isize)).y as i32);
        if y + h > (*atlas).height {
            return -1;
        }
        spaceLeft -= (*(*atlas).nodes.offset(i as isize)).width as i32;
        i += 1
    }
    y
}

pub unsafe fn fons__allocAtlas(w: i32, h: i32, nnodes: i32) -> *mut Atlas {
    let atlas = malloc(::std::mem::size_of::<Atlas>() as u64) as *mut Atlas;
    if !atlas.is_null() {
        memset(
            atlas as *mut libc::c_void,
            0,
            ::std::mem::size_of::<Atlas>() as u64,
        );
        (*atlas).width = w;
        (*atlas).height = h;
        (*atlas).nodes =
            malloc((::std::mem::size_of::<AtlasNode>() as u64).wrapping_mul(nnodes as u64))
                as *mut AtlasNode;
        if !(*atlas).nodes.is_null() {
            memset(
                (*atlas).nodes as *mut libc::c_void,
                0,
                (::std::mem::size_of::<AtlasNode>() as u64).wrapping_mul(nnodes as u64),
            );
            (*atlas).nnodes = 0;
            (*atlas).cnodes = nnodes;
            (*(*atlas).nodes.offset(0)).x = 0;
            (*(*atlas).nodes.offset(0)).y = 0;
            (*(*atlas).nodes.offset(0)).width = w as i16;
            (*atlas).nnodes += 1;
            return atlas;
        }
    }
    if !atlas.is_null() {
        fons__deleteAtlas(atlas);
    }
    null_mut()
}

pub unsafe fn fons__flush(mut stash: *mut Stash) {
    if (*stash).dirty_rect[0] < (*stash).dirty_rect[2]
        && (*stash).dirty_rect[1] < (*stash).dirty_rect[3]
    {
        (*stash).dirty_rect = [(*stash).width, (*stash).height, 0, 0];
    }
    if (*stash).nverts > 0 {
        (*stash).nverts = 0
    };
}
// Resets the whole stash.

pub unsafe fn fonsResetAtlas(mut stash: *mut Stash, width: i32, height: i32) -> i32 {
    if stash.is_null() {
        return 0;
    }

    fons__flush(stash);
    (*(*stash).atlas).reset(width, height);
    (*stash).texData = realloc(
        (*stash).texData as *mut libc::c_void,
        (width * height) as u64,
    ) as *mut u8;
    if (*stash).texData.is_null() {
        return 0;
    }
    memset(
        (*stash).texData as *mut libc::c_void,
        0,
        (width * height) as u64,
    );
    (*stash).dirty_rect = [width, height, 0, 0];
    for i in 0..(*stash).nfonts {
        let mut font: *mut Font = *(*stash).fonts.add(i);
        (*font).nglyphs = 0;
        for j in 0..256 {
            (*font).lut[j] = -1;
        }
    }
    (*stash).width = width;
    (*stash).height = height;
    (*stash).itw = 1.0 / (*stash).width as f32;
    (*stash).ith = 1.0 / (*stash).height as f32;
    fons__addWhiteRect(stash, 2, 2);
    1
}

impl Atlas {
    pub fn reset(&mut self, w: i32, h: i32) {
        self.width = w;
        self.height = h;
        self.nnodes = 0;
        unsafe {
            (*self.nodes.offset(0)).x = 0;
            (*self.nodes.offset(0)).y = 0;
            (*self.nodes.offset(0)).width = w as i16;
        }
        self.nnodes += 1;
    }

}
// Add fonts

pub unsafe fn fonsAddFont(stash: *mut Stash, name: *const i8, path: *const i8) -> i32 {
    let mut data: *mut u8 = null_mut();
    let mut fp = fopen(path, b"rb\x00" as *const u8 as *const i8);
    if !fp.is_null() {
        fseek(fp, 0 as libc::c_long, 2i32);
        let size = ftell(fp) as i32;
        fseek(fp, 0 as libc::c_long, 0);
        data = malloc(size as u64) as *mut u8;
        if !data.is_null() {
            let readed = fread(data as *mut libc::c_void, 1 as u64, size as u64, fp);
            fclose(fp);
            fp = null_mut();
            if readed == size as u64 {
                return fonsAddFontMem(stash, name, data, size, 1);
            }
        }
    }
    if !data.is_null() {
        free(data as *mut libc::c_void);
    }
    if !fp.is_null() {
        fclose(fp);
    }
    -1
}

pub unsafe fn fonsAddFontMem(
    mut stash: *mut Stash,
    name: *const i8,
    data: *mut u8,
    dataSize: i32,
    freeData: i32,
) -> i32 {
    let idx: i32 = fons__allocFont(stash);
    if idx == -1 {
        return -1;
    }
    let font = *(*stash).fonts.offset(idx as isize);
    strncpy(
        (*font).name.as_mut_ptr(),
        name,
        ::std::mem::size_of::<[i8; 64]>() as u64,
    );
    (*font).name[(::std::mem::size_of::<[i8; 64]>() as u64).wrapping_sub(1 as u64) as usize] =
        '\u{0}' as i32 as i8;

    for i in 0..256 {
        (*font).lut[i] = -1;
    }

    (*font).dataSize = dataSize;
    (*font).data = data;
    (*font).freeData = freeData as u8;
    (*stash).nscratch = 0;
    if 0 == fons__tt_loadFont(stash, &mut (*font).font, data, dataSize) {
        fons__freeFont(font);
        (*stash).nfonts -= 1;
        -1
    } else {
        let info = &mut (*font).font;
        let hhea = info.hhea as isize;

        let ascent = read_i16((*info).data.offset(hhea).offset(4)) as i32;
        let descent = read_i16((*info).data.offset(hhea).offset(6)) as i32;
        let lineGap = read_i16((*info).data.offset(hhea).offset(8)) as i32;

        let fh = ascent - descent;
        (*font).ascender = ascent as f32 / fh as f32;
        (*font).descender = descent as f32 / fh as f32;
        (*font).lineh = (fh + lineGap) as f32 / fh as f32;
        idx
    }
}

pub unsafe fn fons__allocFont(mut stash: *mut Stash) -> i32 {
    if (*stash).nfonts + 1 > (*stash).cfonts {
        (*stash).cfonts = if (*stash).cfonts == 0 {
            8
        } else {
            (*stash).cfonts * 2
        };
        (*stash).fonts = realloc(
            (*stash).fonts as *mut libc::c_void,
            (::std::mem::size_of::<*mut Font>() as u64).wrapping_mul((*stash).cfonts as u64),
        ) as *mut *mut Font;
        if (*stash).fonts.is_null() {
            return -1;
        }
    }
    let font = malloc(::std::mem::size_of::<Font>() as u64) as *mut Font;
    if !font.is_null() {
        memset(
            font as *mut libc::c_void,
            0,
            ::std::mem::size_of::<Font>() as u64,
        );
        (*font).glyphs = malloc((::std::mem::size_of::<Glyph>() as u64).wrapping_mul(256i32 as u64))
            as *mut Glyph;
        if !(*font).glyphs.is_null() {
            (*font).cglyphs = 256i32;
            (*font).nglyphs = 0;
            let fresh3 = (*stash).nfonts;
            (*stash).nfonts += 1;
            let fresh4 = &mut (*(*stash).fonts.add(fresh3));
            *fresh4 = font;
            return (*stash).nfonts as i32 - 1;
        }
    }
    fons__freeFont(font);
    -1
}

// computes a scale factor to produce a font whose EM size is mapped to
// 'pixels' tall. This is probably what traditional APIs compute, but
// I'm not positive.


pub unsafe fn fons__tt_loadFont(
    context: *mut Stash,
    info: *mut FontInfo,
    data: *mut u8,
    dataSize: i32,
) -> i32 {
    (*info).userdata = context as *mut libc::c_void;
    let fontstart = 0;

    (*info).data = data;
    (*info).fontstart = fontstart;
    let cmap = stbtt__find_table(data, fontstart as u32, b"cmap");
    (*info).loca = stbtt__find_table(data, fontstart as u32, b"loca") as i32;
    (*info).head = stbtt__find_table(data, fontstart as u32, b"head") as i32;
    (*info).glyf = stbtt__find_table(data, fontstart as u32, b"glyf") as i32;
    (*info).hhea = stbtt__find_table(data, fontstart as u32, b"hhea") as i32;
    (*info).hmtx = stbtt__find_table(data, fontstart as u32, b"hmtx") as i32;
    (*info).kern = stbtt__find_table(data, fontstart as u32, b"kern") as i32;
    if 0 == cmap
        || 0 == (*info).loca
        || 0 == (*info).head
        || 0 == (*info).glyf
        || 0 == (*info).hhea
        || 0 == (*info).hmtx
    {
        return 0;
    }

    let t = stbtt__find_table(data, fontstart as u32, b"maxp");
    (*info).numGlyphs = if 0 != t {
        read_u16(data.offset(t as isize).offset(4)) as i32
    } else {
        0xffffi32
    };

    let num_tables = read_u16(data.offset(cmap as isize).offset(2)) as i32;
    (*info).index_map = 0;

    for i in 0..num_tables {
        let encoding_record: u32 = cmap.wrapping_add(4).wrapping_add((8 * i) as u32);
        match read_u16(data.offset(encoding_record as isize)) {
            3 => match read_u16(data.offset(encoding_record as isize).offset(2)) {
                1 | 10 => {
                    (*info).index_map = cmap.wrapping_add(read_u32(data.offset(encoding_record as isize).offset(4))) as i32
                }
                _ => {}
            },
            0 => {
                (*info).index_map = cmap.wrapping_add(read_u32(data.offset(encoding_record as isize).offset(4))) as i32
            }
            _ => {}
        }
    }

    if (*info).index_map == 0 {
        return 0;
    }
    (*info).indexToLocFormat = read_u16(data.offset((*info).head as isize).offset(50isize)) as i32;
    1
}
// @OPTIMIZE: binary search
unsafe fn stbtt__find_table(data: *mut u8, fontstart: u32, tag: &[u8]) -> u32 {
    let num_tables: i32 = read_u16(data.offset(fontstart as isize).offset(4)) as i32;
    let tabledir: u32 = fontstart.wrapping_add(12 as u32);
    for i in 0..num_tables {
        let loc = tabledir.wrapping_add((16 * i) as u32) as isize;
        let loc = data.offset(loc);
        if *loc.offset(0) == tag[0]
            && *loc.offset(1) == tag[1]
            && *loc.offset(2) == tag[2]
            && *loc.offset(3) == tag[3]
        {
            return read_u32(loc.offset(8));
        }
    }
    0
}

pub unsafe fn fonsGetFontByName(s: *mut Stash, name: *const i8) -> i32 {
    for i in 0..(*s).nfonts {
        if strcmp((**(*s).fonts.offset(i as isize)).name.as_mut_ptr(), name) == 0 {
            return i as i32;
        }
    }
    -1
}
// Draw text

pub unsafe fn fons__getQuad(
    mut stash: *mut Stash,
    mut font: *mut Font,
    mut prevGlyphIndex: i32,
    mut glyph: *mut Glyph,
    mut scale: f32,
    mut spacing: f32,
    mut x: *mut f32,
    mut y: *mut f32,
    mut q: *mut Quad,
) {
    let mut rx: f32 = 0.;
    let mut ry: f32 = 0.;
    let mut xoff: f32 = 0.;
    let mut yoff: f32 = 0.;
    let mut x0: f32 = 0.;
    let mut y0: f32 = 0.;
    let mut x1: f32 = 0.;
    let mut y1: f32 = 0.;
    if prevGlyphIndex != -1 {
        let mut adv: f32 =
            fons__tt_getGlyphKernAdvance(&mut (*font).font, prevGlyphIndex, (*glyph).index) as f32
                * scale;
        *x += (adv + spacing + 0.5f32) as i32 as f32
    }
    xoff = ((*glyph).xoff as i32 + 1) as i16 as f32;
    yoff = ((*glyph).yoff as i32 + 1) as i16 as f32;
    x0 = ((*glyph).x0 as i32 + 1) as f32;
    y0 = ((*glyph).y0 as i32 + 1) as f32;
    x1 = ((*glyph).x1 as i32 - 1) as f32;
    y1 = ((*glyph).y1 as i32 - 1) as f32;

    rx = (*x + xoff) as i32 as f32;
    ry = (*y + yoff) as i32 as f32;
    (*q).x0 = rx;
    (*q).y0 = ry;
    (*q).x1 = rx + x1 - x0;
    (*q).y1 = ry + y1 - y0;
    (*q).s0 = x0 * (*stash).itw;
    (*q).t0 = y0 * (*stash).ith;
    (*q).s1 = x1 * (*stash).itw;
    (*q).t1 = y1 * (*stash).ith;

    *x += ((*glyph).xadv as i32 as f32 / 10.0f32 + 0.5f32) as i32 as f32;
}

pub unsafe fn fons__tt_getGlyphKernAdvance(
    mut font: *mut FontInfo,
    mut glyph1: i32,
    mut glyph2: i32,
) -> i32 {
    stbtt_GetGlyphKernAdvance(font, glyph1, glyph2)
}

pub unsafe fn stbtt_GetGlyphKernAdvance(
    mut info: *const FontInfo,
    mut glyph1: i32,
    mut glyph2: i32,
) -> i32 {
    let mut data: *mut u8 = (*info).data.offset((*info).kern as isize);
    if 0 == (*info).kern {
        return 0;
    }
    if (read_u16(data.offset(2isize)) as i32) < 1 {
        return 0;
    }
    if read_u16(data.offset(8isize)) as i32 != 1 {
        return 0;
    }
    let mut l = 0;
    let mut r = read_u16(data.offset(10isize)) as i32 - 1;
    let needle = (glyph1 << 16i32 | glyph2) as u32;
    while l <= r {
        let m = l + r >> 1;
        let straw = read_u32(data.offset(18isize).offset((m * 6i32) as isize));
        if needle < straw {
            r = m - 1
        } else if needle > straw {
            l = m + 1
        } else {
            return read_i16(data.offset(22isize).offset((m * 6i32) as isize)) as i32;
        }
    }
    0
}
//	fons__blurrows(dst, w, h, dstStride, alpha);
//	fons__blurcols(dst, w, h, dstStride, alpha);

pub unsafe fn fons__getGlyph(
    mut stash: *mut Stash,
    mut font: *mut Font,
    mut codepoint: u32,
    mut isize: i16,
    mut iblur: i16,
    mut bitmapOption: i32,
) -> *mut Glyph {
    let mut i: i32 = 0;
    let mut g: i32 = 0;
    let mut advance: i32 = 0;
    let mut lsb: i32 = 0;
    let mut x0: i32 = 0;
    let mut y0: i32 = 0;
    let mut x1: i32 = 0;
    let mut y1: i32 = 0;
    let mut gw: i32 = 0;
    let mut gh: i32 = 0;
    let mut gx: i32 = 0;
    let mut gy: i32 = 0;
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut scale: f32 = 0.;
    let mut glyph: *mut Glyph = 0 as *mut Glyph;
    let mut h: u32 = 0;
    let mut size: f32 = isize as i32 as f32 / 10.0f32;
    let mut pad: i32 = 0;
    let mut added: i32 = 0;
    let mut bdst: *mut u8 = 0 as *mut u8;
    let mut dst: *mut u8 = 0 as *mut u8;
    let mut renderFont: *mut Font = font;
    if (isize as i32) < 2i32 {
        return 0 as *mut Glyph;
    }
    if iblur as i32 > 20 {
        iblur = 20 as i16
    }
    pad = iblur as i32 + 2i32;
    (*stash).nscratch = 0;
    h = fons__hashint(codepoint) & (256i32 - 1) as u32;
    i = (*font).lut[h as usize];
    while i != -1 {
        if (*(*font).glyphs.offset(i as isize)).codepoint == codepoint
            && (*(*font).glyphs.offset(i as isize)).size as i32 == isize as i32
            && (*(*font).glyphs.offset(i as isize)).blur as i32 == iblur as i32
        {
            glyph = &mut *(*font).glyphs.offset(i as isize) as *mut Glyph;
            if bitmapOption == FONS_GLYPH_BITMAP_OPTIONAL as i32
                || (*glyph).x0 as i32 >= 0 && (*glyph).y0 as i32 >= 0
            {
                return glyph;
            }
            // At this point, glyph exists but the bitmap data is not yet created.
            break;
        } else {
            i = (*(*font).glyphs.offset(i as isize)).next
        }
    }
    g = fons__tt_getGlyphIndex(&mut (*font).font, codepoint as i32);
    if g == 0 {
        i = 0;
        while i < (*font).nfallbacks {
            let mut fallbackFont: *mut Font = *(*stash)
                .fonts
                .offset((*font).fallbacks[i as usize] as isize);
            let mut fallbackIndex: i32 =
                fons__tt_getGlyphIndex(&mut (*fallbackFont).font, codepoint as i32);
            if fallbackIndex != 0 {
                g = fallbackIndex;
                renderFont = fallbackFont;
                break;
            } else {
                i += 1
            }
        }
    }
    scale = fons__tt_getPixelHeightScale(&mut (*renderFont).font, size);
    fons__tt_buildGlyphBitmap(
        &mut (*renderFont).font,
        g,
        size,
        scale,
        &mut advance,
        &mut lsb,
        &mut x0,
        &mut y0,
        &mut x1,
        &mut y1,
    );
    gw = x1 - x0 + pad * 2i32;
    gh = y1 - y0 + pad * 2i32;
    if bitmapOption == FONS_GLYPH_BITMAP_REQUIRED as i32 {
        added = fons__atlasAddRect((*stash).atlas, gw, gh, &mut gx, &mut gy);
        if added == 0 {
            return 0 as *mut Glyph;
        }
    } else {
        gx = -1;
        gy = -1
    }
    if glyph.is_null() {
        glyph = fons__allocGlyph(font);
        (*glyph).codepoint = codepoint;
        (*glyph).size = isize;
        (*glyph).blur = iblur;
        (*glyph).next = 0;
        (*glyph).next = (*font).lut[h as usize];
        (*font).lut[h as usize] = (*font).nglyphs - 1
    }
    (*glyph).index = g;
    (*glyph).x0 = gx as i16;
    (*glyph).y0 = gy as i16;
    (*glyph).x1 = ((*glyph).x0 as i32 + gw) as i16;
    (*glyph).y1 = ((*glyph).y0 as i32 + gh) as i16;
    (*glyph).xadv = (scale * advance as f32 * 10.0f32) as i16;
    (*glyph).xoff = (x0 - pad) as i16;
    (*glyph).yoff = (y0 - pad) as i16;
    if bitmapOption == FONS_GLYPH_BITMAP_OPTIONAL as i32 {
        return glyph;
    }
    dst = &mut *(*stash).texData.offset(
        ((*glyph).x0 as i32 + pad + ((*glyph).y0 as i32 + pad) * (*stash).width) as isize,
    ) as *mut u8;
    fons__tt_renderGlyphBitmap(
        &mut (*renderFont).font,
        dst,
        gw - pad * 2i32,
        gh - pad * 2i32,
        (*stash).width,
        scale,
        scale,
        g,
    );
    dst = &mut *(*stash)
        .texData
        .offset(((*glyph).x0 as i32 + (*glyph).y0 as i32 * (*stash).width) as isize)
        as *mut u8;
    y = 0;
    while y < gh {
        *dst.offset((y * (*stash).width) as isize) = 0 as u8;
        *dst.offset((gw - 1 + y * (*stash).width) as isize) = 0 as u8;
        y += 1
    }
    x = 0;
    while x < gw {
        *dst.offset(x as isize) = 0 as u8;
        *dst.offset((x + (gh - 1) * (*stash).width) as isize) = 0 as u8;
        x += 1
    }
    if iblur as i32 > 0 {
        (*stash).nscratch = 0;
        bdst = &mut *(*stash)
            .texData
            .offset(((*glyph).x0 as i32 + (*glyph).y0 as i32 * (*stash).width) as isize)
            as *mut u8;
        fons__blur(bdst, gw, gh, (*stash).width, iblur as i32);
    }
    (*stash).dirty_rect = [
        mini((*stash).dirty_rect[0], (*glyph).x0 as i32),
        mini((*stash).dirty_rect[1], (*glyph).y0 as i32),
        maxi((*stash).dirty_rect[2], (*glyph).x1 as i32),
        maxi((*stash).dirty_rect[3], (*glyph).y1 as i32),
    ];
    glyph
}

pub unsafe fn fons__blur(dst: *mut u8, w: i32, h: i32, dstStride: i32, blur: i32) {
    if blur < 1 {
        return;
    }
    let sigma = blur as f32 * 0.57735;
    let alpha = ((1 << 16) as f32 * (1.0 - expf(-2.3 / (sigma + 1.0)))) as i32;
    fons__blurRows(dst, w, h, dstStride, alpha);
    fons__blurCols(dst, w, h, dstStride, alpha);
    fons__blurRows(dst, w, h, dstStride, alpha);
    fons__blurCols(dst, w, h, dstStride, alpha);
}
// Based on Exponential blur, Jani Huhtanen, 2006

pub unsafe fn fons__blurCols(mut dst: *mut u8, w: i32, h: i32, dstStride: i32, alpha: i32) {
    let mut y = 0;
    while y < h {
        let mut z = 0;
        let mut x = 1;
        while x < w {
            z += alpha * (((*dst.offset(x as isize) as i32) << 7) - z) >> 16;
            *dst.offset(x as isize) = (z >> 7) as u8;
            x += 1
        }
        *dst.offset((w - 1) as isize) = 0 as u8;
        z = 0;
        x = w - 2i32;
        while x >= 0 {
            z += alpha * (((*dst.offset(x as isize) as i32) << 7) - z) >> 16;
            *dst.offset(x as isize) = (z >> 7) as u8;
            x -= 1
        }
        *dst.offset(0isize) = 0 as u8;
        dst = dst.offset(dstStride as isize);
        y += 1
    }
}

pub unsafe fn fons__blurRows(mut dst: *mut u8, w: i32, h: i32, dstStride: i32, alpha: i32) {
    let mut x = 0;
    while x < w {
        let mut z = 0;
        let mut y = dstStride;
        while y < h * dstStride {
            z += alpha * (((*dst.offset(y as isize) as i32) << 7) - z) >> 16;
            *dst.offset(y as isize) = (z >> 7) as u8;
            y += dstStride
        }
        *dst.offset(((h - 1) * dstStride) as isize) = 0 as u8;
        z = 0;
        y = (h - 2i32) * dstStride;
        while y >= 0 {
            z += alpha * (((*dst.offset(y as isize) as i32) << 7) - z) >> 16;
            *dst.offset(y as isize) = (z >> 7) as u8;
            y -= dstStride
        }
        *dst.offset(0isize) = 0 as u8;
        dst = dst.offset(1isize);
        x += 1
    }
}

pub unsafe fn fons__tt_renderGlyphBitmap(
    font: *mut FontInfo,
    output: *mut u8,
    outWidth: i32,
    outHeight: i32,
    outStride: i32,
    scaleX: f32,
    scaleY: f32,
    glyph: i32,
) {
    stbtt_MakeGlyphBitmap(
        font,
        output,
        outWidth,
        outHeight,
        outStride,
        scaleX,
        scaleY,
        glyph,
    );
}

pub unsafe fn stbtt_MakeGlyphBitmap(
    mut info: *const FontInfo,
    mut output: *mut u8,
    mut out_w: i32,
    mut out_h: i32,
    mut out_stride: i32,
    mut scale_x: f32,
    mut scale_y: f32,
    mut glyph: i32,
) {
    stbtt_MakeGlyphBitmapSubpixel(
        info, output, out_w, out_h, out_stride, scale_x, scale_y, 0.0f32, 0.0f32, glyph,
    );
}

pub unsafe fn stbtt_MakeGlyphBitmapSubpixel(
    mut info: *const FontInfo,
    mut output: *mut u8,
    mut out_w: i32,
    mut out_h: i32,
    mut out_stride: i32,
    mut scale_x: f32,
    mut scale_y: f32,
    mut shift_x: f32,
    mut shift_y: f32,
    mut glyph: i32,
) {
    let mut ix0: i32 = 0;
    let mut iy0: i32 = 0;
    let mut vertices: *mut stbtt_vertex = 0 as *mut stbtt_vertex;
    let mut num_verts: i32 = stbtt_GetGlyphShape(info, glyph, &mut vertices);
    let mut gbm: stbtt__bitmap = stbtt__bitmap {
        w: 0,
        h: 0,
        stride: 0,
        pixels: 0 as *mut u8,
    };
    stbtt_GetGlyphBitmapBoxSubpixel(
        info,
        glyph,
        scale_x,
        scale_y,
        shift_x,
        shift_y,
        &mut ix0,
        &mut iy0,
        0 as *mut i32,
        0 as *mut i32,
    );
    gbm.pixels = output;
    gbm.w = out_w;
    gbm.h = out_h;
    gbm.stride = out_stride;
    if 0 != gbm.w && 0 != gbm.h {
        stbtt_Rasterize(
            &mut gbm,
            0.35f32,
            vertices,
            num_verts,
            scale_x,
            scale_y,
            shift_x,
            shift_y,
            ix0,
            iy0,
            1,
            (*info).userdata,
        );
    }
}

pub unsafe fn stbtt_GetGlyphShape(
    mut info: *const FontInfo,
    mut glyph_index: i32,
    mut pvertices: *mut *mut stbtt_vertex,
) -> i32 {
    let mut numberOfContours: i16 = 0;
    let mut endPtsOfContours: *mut u8 = 0 as *mut u8;
    let mut data: *mut u8 = (*info).data;
    let mut vertices: *mut stbtt_vertex = 0 as *mut stbtt_vertex;
    let mut num_vertices: i32 = 0;
    let mut g: i32 = stbtt__GetGlyfOffset(info, glyph_index);
    *pvertices = 0 as *mut stbtt_vertex;
    if g < 0 {
        return 0;
    }
    numberOfContours = read_i16(data.offset(g as isize));
    if numberOfContours as i32 > 0 {
        let mut flags: u8 = 0 as u8;
        let mut flagcount: u8 = 0;
        let mut ins: i32 = 0;
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        let mut m: i32 = 0;
        let mut n: i32 = 0;
        let mut next_move: i32 = 0;
        let mut was_off: i32 = 0;
        let mut off: i32 = 0;
        let mut start_off: i32 = 0;
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut cx: i32 = 0;
        let mut cy: i32 = 0;
        let mut sx: i32 = 0;
        let mut sy: i32 = 0;
        let mut scx: i32 = 0;
        let mut scy: i32 = 0;
        let mut points: *mut u8 = 0 as *mut u8;
        endPtsOfContours = data.offset(g as isize).offset(10isize);
        ins = read_u16(
            data.offset(g as isize)
                .offset(10isize)
                .offset((numberOfContours as i32 * 2i32) as isize),
        ) as i32;
        points = data
            .offset(g as isize)
            .offset(10isize)
            .offset((numberOfContours as i32 * 2i32) as isize)
            .offset(2isize)
            .offset(ins as isize);
        n = 1 + read_u16(
            endPtsOfContours
                .offset((numberOfContours as i32 * 2i32) as isize)
                .offset(-2isize),
        ) as i32;
        m = n + 2i32 * numberOfContours as i32;
        vertices = fons__tmpalloc(
            (m as u64).wrapping_mul(::std::mem::size_of::<stbtt_vertex>() as u64),
            (*info).userdata,
        ) as *mut stbtt_vertex;
        if vertices.is_null() {
            return 0;
        }
        next_move = 0;
        flagcount = 0 as u8;
        off = m - n;
        i = 0;
        while i < n {
            if flagcount as i32 == 0 {
                let fresh5 = points;
                points = points.offset(1);
                flags = *fresh5;
                if 0 != flags as i32 & 8i32 {
                    let fresh6 = points;
                    points = points.offset(1);
                    flagcount = *fresh6
                }
            } else {
                flagcount = flagcount.wrapping_sub(1)
            }
            (*vertices.offset((off + i) as isize)).type_0 = flags;
            i += 1
        }
        x = 0;
        i = 0;
        while i < n {
            flags = (*vertices.offset((off + i) as isize)).type_0;
            if 0 != flags as i32 & 2i32 {
                let fresh7 = points;
                points = points.offset(1);
                let mut dx: i16 = *fresh7 as i16;
                x += if 0 != flags as i32 & 16i32 {
                    dx as i32
                } else {
                    -(dx as i32)
                }
            } else if 0 == flags as i32 & 16i32 {
                x = x
                    + (*points.offset(0isize) as i32 * 256i32 + *points.offset(1isize) as i32)
                        as i16 as i32;
                points = points.offset(2isize)
            }
            (*vertices.offset((off + i) as isize)).x = x as i16;
            i += 1
        }
        y = 0;
        i = 0;
        while i < n {
            flags = (*vertices.offset((off + i) as isize)).type_0;
            if 0 != flags as i32 & 4i32 {
                let fresh8 = points;
                points = points.offset(1);
                let mut dy: i16 = *fresh8 as i16;
                y += if 0 != flags as i32 & 32i32 {
                    dy as i32
                } else {
                    -(dy as i32)
                }
            } else if 0 == flags as i32 & 32i32 {
                y = y
                    + (*points.offset(0isize) as i32 * 256i32 + *points.offset(1isize) as i32)
                        as i16 as i32;
                points = points.offset(2isize)
            }
            (*vertices.offset((off + i) as isize)).y = y as i16;
            i += 1
        }
        num_vertices = 0;
        scy = 0;
        scx = scy;
        cy = scx;
        cx = cy;
        sy = cx;
        sx = sy;
        i = 0;
        while i < n {
            flags = (*vertices.offset((off + i) as isize)).type_0;
            x = (*vertices.offset((off + i) as isize)).x as i16 as i32;
            y = (*vertices.offset((off + i) as isize)).y as i16 as i32;
            if next_move == i {
                if i != 0 {
                    num_vertices = stbtt__close_shape(
                        vertices,
                        num_vertices,
                        was_off,
                        start_off,
                        sx,
                        sy,
                        scx,
                        scy,
                        cx,
                        cy,
                    )
                }
                start_off = (0 == flags as i32 & 1) as i32;
                if 0 != start_off {
                    scx = x;
                    scy = y;
                    if 0 == (*vertices.offset((off + i + 1) as isize)).type_0 as i32 & 1 {
                        sx = x + (*vertices.offset((off + i + 1) as isize)).x as i32 >> 1;
                        sy = y + (*vertices.offset((off + i + 1) as isize)).y as i32 >> 1
                    } else {
                        sx = (*vertices.offset((off + i + 1) as isize)).x as i32;
                        sy = (*vertices.offset((off + i + 1) as isize)).y as i32;
                        i += 1
                    }
                } else {
                    sx = x;
                    sy = y
                }
                let fresh9 = num_vertices;
                num_vertices = num_vertices + 1;
                stbtt_setvertex(
                    &mut *vertices.offset(fresh9 as isize),
                    STBTT_vmove as i32 as u8,
                    sx,
                    sy,
                    0,
                    0,
                );
                was_off = 0;
                next_move = 1 + read_u16(endPtsOfContours.offset((j * 2i32) as isize)) as i32;
                j += 1
            } else if 0 == flags as i32 & 1 {
                if 0 != was_off {
                    let fresh10 = num_vertices;
                    num_vertices = num_vertices + 1;
                    stbtt_setvertex(
                        &mut *vertices.offset(fresh10 as isize),
                        STBTT_vcurve as i32 as u8,
                        cx + x >> 1,
                        cy + y >> 1,
                        cx,
                        cy,
                    );
                }
                cx = x;
                cy = y;
                was_off = 1
            } else {
                if 0 != was_off {
                    let fresh11 = num_vertices;
                    num_vertices = num_vertices + 1;
                    stbtt_setvertex(
                        &mut *vertices.offset(fresh11 as isize),
                        STBTT_vcurve as i32 as u8,
                        x,
                        y,
                        cx,
                        cy,
                    );
                } else {
                    let fresh12 = num_vertices;
                    num_vertices = num_vertices + 1;
                    stbtt_setvertex(
                        &mut *vertices.offset(fresh12 as isize),
                        STBTT_vline as i32 as u8,
                        x,
                        y,
                        0,
                        0,
                    );
                }
                was_off = 0
            }
            i += 1
        }
        num_vertices = stbtt__close_shape(
            vertices,
            num_vertices,
            was_off,
            start_off,
            sx,
            sy,
            scx,
            scy,
            cx,
            cy,
        )
    } else if numberOfContours as i32 == -1 {
        let mut more: i32 = 1;
        let mut comp: *mut u8 = data.offset(g as isize).offset(10isize);
        num_vertices = 0;
        vertices = 0 as *mut stbtt_vertex;
        while 0 != more {
            let mut flags_0: u16 = 0;
            let mut gidx: u16 = 0;
            let mut comp_num_verts: i32 = 0;
            let mut i_0: i32 = 0;
            let mut comp_verts: *mut stbtt_vertex = 0 as *mut stbtt_vertex;
            let mut tmp: *mut stbtt_vertex = 0 as *mut stbtt_vertex;
            let mut mtx: [f32; 6] = [1 as f32, 0 as f32, 0 as f32, 1 as f32, 0 as f32, 0 as f32];
            let mut m_0: f32 = 0.;
            let mut n_0: f32 = 0.;
            flags_0 = read_i16(comp) as u16;
            comp = comp.offset(2isize);
            gidx = read_i16(comp) as u16;
            comp = comp.offset(2isize);
            if 0 != flags_0 as i32 & 2i32 {
                if 0 != flags_0 as i32 & 1 {
                    mtx[4usize] = read_i16(comp) as f32;
                    comp = comp.offset(2isize);
                    mtx[5usize] = read_i16(comp) as f32;
                    comp = comp.offset(2isize)
                } else {
                    mtx[4usize] = *(comp as *mut i8) as f32;
                    comp = comp.offset(1isize);
                    mtx[5usize] = *(comp as *mut i8) as f32;
                    comp = comp.offset(1isize)
                }
            } else {
                __assert_fail(b"0\x00" as *const u8 as *const i8,
                              b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                                  as *const u8 as *const i8,
                              1407i32 as u32,
                              (*::std::mem::transmute::<&[u8; 70],
                                                        &[i8; 70]>(b"int stbtt_GetGlyphShape(const stbtt_fontinfo *, int, stbtt_vertex **)\x00")).as_ptr());
            }
            if 0 != flags_0 as i32 & 1 << 3i32 {
                mtx[3usize] = read_i16(comp) as i32 as f32 / 16384.0f32;
                mtx[0usize] = mtx[3usize];
                comp = comp.offset(2isize);
                mtx[2usize] = 0 as f32;
                mtx[1usize] = mtx[2usize]
            } else if 0 != flags_0 as i32 & 1 << 6i32 {
                mtx[0usize] = read_i16(comp) as i32 as f32 / 16384.0f32;
                comp = comp.offset(2isize);
                mtx[2usize] = 0 as f32;
                mtx[1usize] = mtx[2usize];
                mtx[3usize] = read_i16(comp) as i32 as f32 / 16384.0f32;
                comp = comp.offset(2isize)
            } else if 0 != flags_0 as i32 & 1 << 7i32 {
                mtx[0usize] = read_i16(comp) as i32 as f32 / 16384.0f32;
                comp = comp.offset(2isize);
                mtx[1usize] = read_i16(comp) as i32 as f32 / 16384.0f32;
                comp = comp.offset(2isize);
                mtx[2usize] = read_i16(comp) as i32 as f32 / 16384.0f32;
                comp = comp.offset(2isize);
                mtx[3usize] = read_i16(comp) as i32 as f32 / 16384.0f32;
                comp = comp.offset(2isize)
            }
            m_0 = sqrt((mtx[0usize] * mtx[0usize] + mtx[1usize] * mtx[1usize]) as f64) as f32;
            n_0 = sqrt((mtx[2usize] * mtx[2usize] + mtx[3usize] * mtx[3usize]) as f64) as f32;
            comp_num_verts = stbtt_GetGlyphShape(info, gidx as i32, &mut comp_verts);
            if comp_num_verts > 0 {
                i_0 = 0;
                while i_0 < comp_num_verts {
                    let mut v: *mut stbtt_vertex =
                        &mut *comp_verts.offset(i_0 as isize) as *mut stbtt_vertex;
                    let mut x_0: i16 = 0;
                    let mut y_0: i16 = 0;
                    x_0 = (*v).x;
                    y_0 = (*v).y;
                    (*v).x = (m_0
                        * (mtx[0usize] * x_0 as i32 as f32
                            + mtx[2usize] * y_0 as i32 as f32
                            + mtx[4usize])) as i16;
                    (*v).y = (n_0
                        * (mtx[1usize] * x_0 as i32 as f32
                            + mtx[3usize] * y_0 as i32 as f32
                            + mtx[5usize])) as i16;
                    x_0 = (*v).cx;
                    y_0 = (*v).cy;
                    (*v).cx = (m_0
                        * (mtx[0usize] * x_0 as i32 as f32
                            + mtx[2usize] * y_0 as i32 as f32
                            + mtx[4usize])) as i16;
                    (*v).cy = (n_0
                        * (mtx[1usize] * x_0 as i32 as f32
                            + mtx[3usize] * y_0 as i32 as f32
                            + mtx[5usize])) as i16;
                    i_0 += 1
                }
                tmp = fons__tmpalloc(
                    ((num_vertices + comp_num_verts) as u64)
                        .wrapping_mul(::std::mem::size_of::<stbtt_vertex>() as u64),
                    (*info).userdata,
                ) as *mut stbtt_vertex;
                if tmp.is_null() {
                    return 0;
                }
                if num_vertices > 0 {
                    memcpy(
                        tmp as *mut libc::c_void,
                        vertices as *const libc::c_void,
                        (num_vertices as u64)
                            .wrapping_mul(::std::mem::size_of::<stbtt_vertex>() as u64),
                    );
                }
                memcpy(
                    tmp.offset(num_vertices as isize) as *mut libc::c_void,
                    comp_verts as *const libc::c_void,
                    (comp_num_verts as u64)
                        .wrapping_mul(::std::mem::size_of::<stbtt_vertex>() as u64),
                );
                vertices = tmp;
                num_vertices += comp_num_verts
            }
            more = flags_0 as i32 & 1 << 5i32
        }
    } else if (numberOfContours as i32) < 0 {
        __assert_fail(
            b"0\x00" as *const u8 as *const i8,
            b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00" as *const u8 as *const i8,
            1460 as u32,
            (*::std::mem::transmute::<&[u8; 70], &[i8; 70]>(
                b"int stbtt_GetGlyphShape(const stbtt_fontinfo *, int, stbtt_vertex **)\x00",
            ))
            .as_ptr(),
        );
    }
    *pvertices = vertices;
    return num_vertices;
}
// FONTSTASH_H

pub unsafe fn fons__tmpalloc(mut size: size_t, mut up: *mut libc::c_void) -> *mut libc::c_void {
    let mut ptr: *mut u8 = 0 as *mut u8;
    let mut stash: *mut Stash = up as *mut Stash;
    size = size.wrapping_add(0xfi32 as u64) & !0xfi32 as u64;
    if (*stash).nscratch + size as i32 > 96000 {
        return 0 as *mut libc::c_void;
    }
    ptr = (*stash).scratch.offset((*stash).nscratch as isize);
    (*stash).nscratch += size as i32;
    return ptr as *mut libc::c_void;
}
unsafe fn stbtt__GetGlyfOffset(mut info: *const FontInfo, mut glyph_index: i32) -> i32 {
    let mut g1: i32 = 0;
    let mut g2: i32 = 0;
    if glyph_index >= (*info).numGlyphs {
        return -1;
    }
    if (*info).indexToLocFormat >= 2i32 {
        return -1;
    }
    if (*info).indexToLocFormat == 0 {
        g1 = (*info).glyf
            + read_u16(
                (*info)
                    .data
                    .offset((*info).loca as isize)
                    .offset((glyph_index * 2i32) as isize),
            ) as i32
                * 2i32;
        g2 = (*info).glyf
            + read_u16(
                (*info)
                    .data
                    .offset((*info).loca as isize)
                    .offset((glyph_index * 2i32) as isize)
                    .offset(2isize),
            ) as i32
                * 2i32
    } else {
        g1 = ((*info).glyf as u32).wrapping_add(read_u32(
            (*info)
                .data
                .offset((*info).loca as isize)
                .offset((glyph_index * 4i32) as isize),
        )) as i32;
        g2 = ((*info).glyf as u32).wrapping_add(read_u32(
            (*info)
                .data
                .offset((*info).loca as isize)
                .offset((glyph_index * 4i32) as isize)
                .offset(4isize),
        )) as i32
    }
    return if g1 == g2 { -1 } else { g1 };
}
unsafe fn stbtt__close_shape(
    mut vertices: *mut stbtt_vertex,
    mut num_vertices: i32,
    mut was_off: i32,
    mut start_off: i32,
    mut sx: i32,
    mut sy: i32,
    mut scx: i32,
    mut scy: i32,
    mut cx: i32,
    mut cy: i32,
) -> i32 {
    if 0 != start_off {
        if 0 != was_off {
            let fresh13 = num_vertices;
            num_vertices = num_vertices + 1;
            stbtt_setvertex(
                &mut *vertices.offset(fresh13 as isize),
                STBTT_vcurve as i32 as u8,
                cx + scx >> 1,
                cy + scy >> 1,
                cx,
                cy,
            );
        }
        let fresh14 = num_vertices;
        num_vertices = num_vertices + 1;
        stbtt_setvertex(
            &mut *vertices.offset(fresh14 as isize),
            STBTT_vcurve as i32 as u8,
            sx,
            sy,
            scx,
            scy,
        );
    } else if 0 != was_off {
        let fresh15 = num_vertices;
        num_vertices = num_vertices + 1;
        stbtt_setvertex(
            &mut *vertices.offset(fresh15 as isize),
            STBTT_vcurve as i32 as u8,
            sx,
            sy,
            cx,
            cy,
        );
    } else {
        let fresh16 = num_vertices;
        num_vertices = num_vertices + 1;
        stbtt_setvertex(
            &mut *vertices.offset(fresh16 as isize),
            STBTT_vline as i32 as u8,
            sx,
            sy,
            0,
            0,
        );
    }
    return num_vertices;
}
unsafe fn stbtt_setvertex(
    mut v: *mut stbtt_vertex,
    mut type_0: u8,
    mut x: i32,
    mut y: i32,
    mut cx: i32,
    mut cy: i32,
) {
    (*v).type_0 = type_0;
    (*v).x = x as i16;
    (*v).y = y as i16;
    (*v).cx = cx as i16;
    (*v).cy = cy as i16;
}
// rasterize a shape with quadratic beziers into a bitmap
// 1-channel bitmap to draw into

pub unsafe fn stbtt_Rasterize(
    mut result: *mut stbtt__bitmap,
    mut flatness_in_pixels: f32,
    mut vertices: *mut stbtt_vertex,
    mut num_verts: i32,
    mut scale_x: f32,
    mut scale_y: f32,
    mut shift_x: f32,
    mut shift_y: f32,
    mut x_off: i32,
    mut y_off: i32,
    mut invert: i32,
    mut userdata: *mut libc::c_void,
) {
    let mut scale: f32 = if scale_x > scale_y { scale_y } else { scale_x };
    let mut winding_count: i32 = 0;
    let mut winding_lengths: *mut i32 = 0 as *mut i32;
    let mut windings: *mut stbtt__point = stbtt_FlattenCurves(
        vertices,
        num_verts,
        flatness_in_pixels / scale,
        &mut winding_lengths,
        &mut winding_count,
        userdata,
    );
    if !windings.is_null() {
        stbtt__rasterize(
            result,
            windings,
            winding_lengths,
            winding_count,
            scale_x,
            scale_y,
            shift_x,
            shift_y,
            x_off,
            y_off,
            invert,
            userdata,
        );
    };
}
// returns number of contours
unsafe fn stbtt_FlattenCurves(
    mut vertices: *mut stbtt_vertex,
    mut num_verts: i32,
    mut objspace_flatness: f32,
    mut contour_lengths: *mut *mut i32,
    mut num_contours: *mut i32,
    mut userdata: *mut libc::c_void,
) -> *mut stbtt__point {
    let mut current_block: u64;
    let mut points: *mut stbtt__point = 0 as *mut stbtt__point;
    let mut num_points: i32 = 0;
    let mut objspace_flatness_squared: f32 = objspace_flatness * objspace_flatness;
    let mut i: i32 = 0;
    let mut n: i32 = 0;
    let mut start: i32 = 0;
    let mut pass: i32 = 0;
    i = 0;
    while i < num_verts {
        if (*vertices.offset(i as isize)).type_0 as i32 == STBTT_vmove as i32 {
            n += 1
        }
        i += 1
    }
    *num_contours = n;
    if n == 0 {
        return 0 as *mut stbtt__point;
    }
    *contour_lengths = fons__tmpalloc(
        (::std::mem::size_of::<i32>() as u64).wrapping_mul(n as u64),
        userdata,
    ) as *mut i32;
    if (*contour_lengths).is_null() {
        *num_contours = 0;
        return 0 as *mut stbtt__point;
    }
    // make two passes through the points so we don't need to realloc
    pass = 0;
    loop {
        if !(pass < 2i32) {
            current_block = 8845338526596852646;
            break;
        }
        let mut x: f32 = 0 as f32;
        let mut y: f32 = 0 as f32;
        if pass == 1 {
            points = fons__tmpalloc(
                (num_points as u64).wrapping_mul(::std::mem::size_of::<stbtt__point>() as u64),
                userdata,
            ) as *mut stbtt__point;
            if points.is_null() {
                current_block = 9535040653783544971;
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
                    let fresh17 = num_points;
                    num_points = num_points + 1;
                    stbtt__add_point(points, fresh17, x, y);
                }
                2 => {
                    x = (*vertices.offset(i as isize)).x as f32;
                    y = (*vertices.offset(i as isize)).y as f32;
                    let fresh18 = num_points;
                    num_points = num_points + 1;
                    stbtt__add_point(points, fresh18, x, y);
                }
                3 => {
                    stbtt__tesselate_curve(
                        points,
                        &mut num_points,
                        x,
                        y,
                        (*vertices.offset(i as isize)).cx as f32,
                        (*vertices.offset(i as isize)).cy as f32,
                        (*vertices.offset(i as isize)).x as f32,
                        (*vertices.offset(i as isize)).y as f32,
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
        8845338526596852646 => return points,
        _ => {
            *contour_lengths = 0 as *mut i32;
            *num_contours = 0;
            return 0 as *mut stbtt__point;
        }
    };
}
// tesselate until threshhold p is happy... @TODO warped to compensate for non-linear stretching
unsafe fn stbtt__tesselate_curve(
    mut points: *mut stbtt__point,
    mut num_points: *mut i32,
    mut x0: f32,
    mut y0: f32,
    mut x1: f32,
    mut y1: f32,
    mut x2: f32,
    mut y2: f32,
    mut objspace_flatness_squared: f32,
    mut n: i32,
) -> i32 {
    // midpoint
    let mut mx: f32 = (x0 + 2i32 as f32 * x1 + x2) / 4i32 as f32;
    let mut my: f32 = (y0 + 2i32 as f32 * y1 + y2) / 4i32 as f32;
    // versus directly drawn line
    let mut dx: f32 = (x0 + x2) / 2i32 as f32 - mx;
    let mut dy: f32 = (y0 + y2) / 2i32 as f32 - my;
    if n > 16i32 {
        return 1;
    }
    if dx * dx + dy * dy > objspace_flatness_squared {
        stbtt__tesselate_curve(
            points,
            num_points,
            x0,
            y0,
            (x0 + x1) / 2.0f32,
            (y0 + y1) / 2.0f32,
            mx,
            my,
            objspace_flatness_squared,
            n + 1,
        );
        stbtt__tesselate_curve(
            points,
            num_points,
            mx,
            my,
            (x1 + x2) / 2.0f32,
            (y1 + y2) / 2.0f32,
            x2,
            y2,
            objspace_flatness_squared,
            n + 1,
        );
    } else {
        stbtt__add_point(points, *num_points, x2, y2);
        *num_points = *num_points + 1
    }
    return 1;
}
unsafe fn stbtt__add_point(mut points: *mut stbtt__point, mut n: i32, mut x: f32, mut y: f32) {
    if points.is_null() {
        return;
    }
    (*points.offset(n as isize)).x = x;
    (*points.offset(n as isize)).y = y;
}
unsafe fn stbtt__rasterize(
    mut result: *mut stbtt__bitmap,
    mut pts: *mut stbtt__point,
    mut wcount: *mut i32,
    mut windings: i32,
    mut scale_x: f32,
    mut scale_y: f32,
    mut shift_x: f32,
    mut shift_y: f32,
    mut off_x: i32,
    mut off_y: i32,
    mut invert: i32,
    mut userdata: *mut libc::c_void,
) {
    let mut y_scale_inv: f32 = if 0 != invert { -scale_y } else { scale_y };
    let mut e: *mut stbtt__edge = 0 as *mut stbtt__edge;
    let mut n: i32 = 0;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut k: i32 = 0;
    let mut m: i32 = 0;
    let mut vsubsample: i32 = 1;
    n = 0;
    i = 0;
    while i < windings {
        n += *wcount.offset(i as isize);
        i += 1
    }
    e = fons__tmpalloc(
        (::std::mem::size_of::<stbtt__edge>() as u64).wrapping_mul((n + 1) as u64),
        userdata,
    ) as *mut stbtt__edge;
    if e.is_null() {
        return;
    }
    n = 0;
    m = 0;
    i = 0;
    while i < windings {
        let mut p: *mut stbtt__point = pts.offset(m as isize);
        m += *wcount.offset(i as isize);
        j = *wcount.offset(i as isize) - 1;
        k = 0;
        while k < *wcount.offset(i as isize) {
            let mut a: i32 = k;
            let mut b: i32 = j;
            // skip the edge if horizontal
            if !((*p.offset(j as isize)).y == (*p.offset(k as isize)).y) {
                (*e.offset(n as isize)).invert = 0;
                if 0 != if 0 != invert {
                    ((*p.offset(j as isize)).y > (*p.offset(k as isize)).y) as i32
                } else {
                    ((*p.offset(j as isize)).y < (*p.offset(k as isize)).y) as i32
                } {
                    (*e.offset(n as isize)).invert = 1;
                    a = j;
                    b = k
                }
                (*e.offset(n as isize)).x0 = (*p.offset(a as isize)).x * scale_x + shift_x;
                (*e.offset(n as isize)).y0 =
                    ((*p.offset(a as isize)).y * y_scale_inv + shift_y) * vsubsample as f32;
                (*e.offset(n as isize)).x1 = (*p.offset(b as isize)).x * scale_x + shift_x;
                (*e.offset(n as isize)).y1 =
                    ((*p.offset(b as isize)).y * y_scale_inv + shift_y) * vsubsample as f32;
                n += 1
            }
            let fresh19 = k;
            k = k + 1;
            j = fresh19
        }
        i += 1
    }
    stbtt__sort_edges(e, n);
    stbtt__rasterize_sorted_edges(result, e, n, vsubsample, off_x, off_y, userdata);
}
// directly AA rasterize edges w/o supersampling
unsafe fn stbtt__rasterize_sorted_edges(
    mut result: *mut stbtt__bitmap,
    mut e: *mut stbtt__edge,
    mut n: i32,
    mut vsubsample: i32,
    mut off_x: i32,
    mut off_y: i32,
    mut userdata: *mut libc::c_void,
) {
    let mut hh: stbtt__hheap = stbtt__hheap {
        head: 0 as *mut stbtt__hheap_chunk,
        first_free: 0 as *mut libc::c_void,
        num_remaining_in_head_chunk: 0,
    };
    let mut active: *mut stbtt__active_edge = 0 as *mut stbtt__active_edge;
    let mut y: i32 = 0;
    let mut j: i32 = 0;
    let mut i: i32 = 0;
    let mut scanline_data: [f32; 129] = [0.; 129];
    let mut scanline: *mut f32 = 0 as *mut f32;
    let mut scanline2: *mut f32 = 0 as *mut f32;
    if (*result).w > 64i32 {
        scanline = fons__tmpalloc(
            (((*result).w * 2i32 + 1) as u64).wrapping_mul(::std::mem::size_of::<f32>() as u64),
            userdata,
        ) as *mut f32
    } else {
        scanline = scanline_data.as_mut_ptr()
    }
    scanline2 = scanline.offset((*result).w as isize);
    y = off_y;
    (*e.offset(n as isize)).y0 = (off_y + (*result).h) as f32 + 1 as f32;
    while j < (*result).h {
        let mut scan_y_top: f32 = y as f32 + 0.0f32;
        let mut scan_y_bottom: f32 = y as f32 + 1.0f32;
        let mut step: *mut *mut stbtt__active_edge = &mut active;
        memset(
            scanline as *mut libc::c_void,
            0,
            ((*result).w as u64).wrapping_mul(::std::mem::size_of::<f32>() as u64),
        );
        memset(
            scanline2 as *mut libc::c_void,
            0,
            (((*result).w + 1) as u64).wrapping_mul(::std::mem::size_of::<f32>() as u64),
        );
        while !(*step).is_null() {
            let mut z: *mut stbtt__active_edge = *step;
            if (*z).ey <= scan_y_top {
                *step = (*z).next;
                if 0. != (*z).direction {
                } else {
                    __assert_fail(b"z->direction\x00" as *const u8 as
                                      *const i8,
                                  b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                                      as *const u8 as *const i8,
                                  2099i32 as u32,
                                  (*::std::mem::transmute::<&[u8; 95],
                                                            &[i8; 95]>(b"void stbtt__rasterize_sorted_edges(stbtt__bitmap *, stbtt__edge *, int, int, int, int, void *)\x00")).as_ptr());
                }
                (*z).direction = 0 as f32;
                stbtt__hheap_free(&mut hh, z as *mut libc::c_void);
            } else {
                step = &mut (**step).next
            }
        }
        while (*e).y0 <= scan_y_bottom {
            if (*e).y0 != (*e).y1 {
                let mut z_0: *mut stbtt__active_edge =
                    stbtt__new_active(&mut hh, e, off_x, scan_y_top, userdata);
                if !z_0.is_null() {
                    if (*z_0).ey >= scan_y_top {
                    } else {
                        __assert_fail(b"z->ey >= scan_y_top\x00" as *const u8
                                          as *const i8,
                                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                                          as *const u8 as *const i8,
                                      2112i32 as u32,
                                      (*::std::mem::transmute::<&[u8; 95],
                                                                &[i8; 95]>(b"void stbtt__rasterize_sorted_edges(stbtt__bitmap *, stbtt__edge *, int, int, int, int, void *)\x00")).as_ptr());
                    }
                    (*z_0).next = active;
                    active = z_0
                }
            }
            e = e.offset(1isize)
        }
        if !active.is_null() {
            stbtt__fill_active_edges_new(
                scanline,
                scanline2.offset(1isize),
                (*result).w,
                active,
                scan_y_top,
            );
        }
        let mut sum: f32 = 0 as f32;
        i = 0;
        while i < (*result).w {
            let mut k: f32 = 0.;
            let mut m: i32 = 0;
            sum += *scanline2.offset(i as isize);
            k = *scanline.offset(i as isize) + sum;
            k = fabs(k as f64) as f32 * 255i32 as f32 + 0.5f32;
            m = k as i32;
            if m > 255i32 {
                m = 255i32
            }
            *(*result).pixels.offset((j * (*result).stride + i) as isize) = m as u8;
            i += 1
        }
        step = &mut active;
        while !(*step).is_null() {
            let mut z_1: *mut stbtt__active_edge = *step;
            (*z_1).fx += (*z_1).fdx;
            step = &mut (**step).next
        }
        y += 1;
        j += 1
    }
    stbtt__hheap_cleanup(&mut hh, userdata);
}
unsafe fn stbtt__hheap_cleanup(mut hh: *mut stbtt__hheap, mut userdata: *mut libc::c_void) {
    let mut c: *mut stbtt__hheap_chunk = (*hh).head;
    while !c.is_null() {
        let mut n: *mut stbtt__hheap_chunk = (*c).next;
        c = n
    }
}
unsafe fn stbtt__fill_active_edges_new(
    mut scanline: *mut f32,
    mut scanline_fill: *mut f32,
    mut len: i32,
    mut e: *mut stbtt__active_edge,
    mut y_top: f32,
) {
    let mut y_bottom: f32 = y_top + 1 as f32;
    while !e.is_null() {
        if (*e).ey >= y_top {
        } else {
            __assert_fail(b"e->ey >= y_top\x00" as *const u8 as
                              *const i8,
                          b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                              as *const u8 as *const i8,
                          1912i32 as u32,
                          (*::std::mem::transmute::<&[u8; 86],
                                                    &[i8; 86]>(b"void stbtt__fill_active_edges_new(float *, float *, int, stbtt__active_edge *, float)\x00")).as_ptr());
        }
        if (*e).fdx == 0 as f32 {
            let mut x0: f32 = (*e).fx;
            if x0 < len as f32 {
                if x0 >= 0 as f32 {
                    stbtt__handle_clipped_edge(scanline, x0 as i32, e, x0, y_top, x0, y_bottom);
                    stbtt__handle_clipped_edge(
                        scanline_fill.offset(-1isize),
                        x0 as i32 + 1,
                        e,
                        x0,
                        y_top,
                        x0,
                        y_bottom,
                    );
                } else {
                    stbtt__handle_clipped_edge(
                        scanline_fill.offset(-1isize),
                        0,
                        e,
                        x0,
                        y_top,
                        x0,
                        y_bottom,
                    );
                }
            }
        } else {
            let mut x0_0: f32 = (*e).fx;
            let mut dx: f32 = (*e).fdx;
            let mut xb: f32 = x0_0 + dx;
            let mut x_top: f32 = 0.;
            let mut x_bottom: f32 = 0.;
            let mut sy0: f32 = 0.;
            let mut sy1: f32 = 0.;
            let mut dy: f32 = (*e).fdy;
            if (*e).sy <= y_bottom && (*e).ey >= y_top {
            } else {
                __assert_fail(b"e->sy <= y_bottom && e->ey >= y_top\x00" as
                                  *const u8 as *const i8,
                              b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                                  as *const u8 as *const i8,
                              1931 as u32,
                              (*::std::mem::transmute::<&[u8; 86],
                                                        &[i8; 86]>(b"void stbtt__fill_active_edges_new(float *, float *, int, stbtt__active_edge *, float)\x00")).as_ptr());
            }
            if (*e).sy > y_top {
                x_top = x0_0 + dx * ((*e).sy - y_top);
                sy0 = (*e).sy
            } else {
                x_top = x0_0;
                sy0 = y_top
            }
            if (*e).ey < y_bottom {
                x_bottom = x0_0 + dx * ((*e).ey - y_top);
                sy1 = (*e).ey
            } else {
                x_bottom = xb;
                sy1 = y_bottom
            }
            if x_top >= 0 as f32
                && x_bottom >= 0 as f32
                && x_top < len as f32
                && x_bottom < len as f32
            {
                if x_top as i32 == x_bottom as i32 {
                    let mut height: f32 = 0.;
                    let mut x: i32 = x_top as i32;
                    height = sy1 - sy0;
                    if x >= 0 && x < len {
                    } else {
                        __assert_fail(b"x >= 0 && x < len\x00" as *const u8 as
                                          *const i8,
                                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                                          as *const u8 as *const i8,
                                      1959i32 as u32,
                                      (*::std::mem::transmute::<&[u8; 86],
                                                                &[i8; 86]>(b"void stbtt__fill_active_edges_new(float *, float *, int, stbtt__active_edge *, float)\x00")).as_ptr());
                    }
                    *scanline.offset(x as isize) += (*e).direction
                        * (1 as f32 - (x_top - x as f32 + (x_bottom - x as f32)) / 2i32 as f32)
                        * height;
                    *scanline_fill.offset(x as isize) += (*e).direction * height
                } else {
                    let mut x_0: i32 = 0;
                    let mut x1: i32 = 0;
                    let mut x2: i32 = 0;
                    let mut y_crossing: f32 = 0.;
                    let mut step: f32 = 0.;
                    let mut sign: f32 = 0.;
                    let mut area: f32 = 0.;
                    if x_top > x_bottom {
                        let mut t: f32 = 0.;
                        sy0 = y_bottom - (sy0 - y_top);
                        sy1 = y_bottom - (sy1 - y_top);
                        t = sy0;
                        sy0 = sy1;
                        sy1 = t;
                        t = x_bottom;
                        x_bottom = x_top;
                        x_top = t;
                        dx = -dx;
                        dy = -dy;
                        t = x0_0;
                        x0_0 = xb;
                        xb = t
                    }
                    x1 = x_top as i32;
                    x2 = x_bottom as i32;
                    y_crossing = ((x1 + 1) as f32 - x0_0) * dy + y_top;
                    sign = (*e).direction;
                    area = sign * (y_crossing - sy0);
                    *scanline.offset(x1 as isize) += area
                        * (1 as f32 - (x_top - x1 as f32 + (x1 + 1 - x1) as f32) / 2i32 as f32);
                    step = sign * dy;
                    x_0 = x1 + 1;
                    while x_0 < x2 {
                        *scanline.offset(x_0 as isize) += area + step / 2i32 as f32;
                        area += step;
                        x_0 += 1
                    }
                    y_crossing += dy * (x2 - (x1 + 1)) as f32;
                    if fabs(area as f64) <= 1.01f32 as f64 {
                    } else {
                        __assert_fail(b"fabs(area) <= 1.01f\x00" as *const u8
                                          as *const i8,
                                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                                          as *const u8 as *const i8,
                                      1996i32 as u32,
                                      (*::std::mem::transmute::<&[u8; 86],
                                                                &[i8; 86]>(b"void stbtt__fill_active_edges_new(float *, float *, int, stbtt__active_edge *, float)\x00")).as_ptr());
                    }
                    *scanline.offset(x2 as isize) += area
                        + sign
                            * (1 as f32
                                - ((x2 - x2) as f32 + (x_bottom - x2 as f32)) / 2i32 as f32)
                            * (sy1 - y_crossing);
                    *scanline_fill.offset(x2 as isize) += sign * (sy1 - sy0)
                }
            } else {
                let mut x_1: i32 = 0;
                x_1 = 0;
                while x_1 < len {
                    let mut y0: f32 = y_top;
                    let mut x1_0: f32 = x_1 as f32;
                    let mut x2_0: f32 = (x_1 + 1) as f32;
                    let mut x3: f32 = xb;
                    let mut y3: f32 = y_bottom;
                    let mut y1: f32 = 0.;
                    let mut y2: f32 = 0.;
                    y1 = (x_1 as f32 - x0_0) / dx + y_top;
                    y2 = ((x_1 + 1) as f32 - x0_0) / dx + y_top;
                    if x0_0 < x1_0 && x3 > x2_0 {
                        stbtt__handle_clipped_edge(scanline, x_1, e, x0_0, y0, x1_0, y1);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x1_0, y1, x2_0, y2);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x2_0, y2, x3, y3);
                    } else if x3 < x1_0 && x0_0 > x2_0 {
                        stbtt__handle_clipped_edge(scanline, x_1, e, x0_0, y0, x2_0, y2);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x2_0, y2, x1_0, y1);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x1_0, y1, x3, y3);
                    } else if x0_0 < x1_0 && x3 > x1_0 {
                        stbtt__handle_clipped_edge(scanline, x_1, e, x0_0, y0, x1_0, y1);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x1_0, y1, x3, y3);
                    } else if x3 < x1_0 && x0_0 > x1_0 {
                        stbtt__handle_clipped_edge(scanline, x_1, e, x0_0, y0, x1_0, y1);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x1_0, y1, x3, y3);
                    } else if x0_0 < x2_0 && x3 > x2_0 {
                        stbtt__handle_clipped_edge(scanline, x_1, e, x0_0, y0, x2_0, y2);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x2_0, y2, x3, y3);
                    } else if x3 < x2_0 && x0_0 > x2_0 {
                        stbtt__handle_clipped_edge(scanline, x_1, e, x0_0, y0, x2_0, y2);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x2_0, y2, x3, y3);
                    } else {
                        stbtt__handle_clipped_edge(scanline, x_1, e, x0_0, y0, x3, y3);
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
unsafe fn stbtt__handle_clipped_edge(
    mut scanline: *mut f32,
    mut x: i32,
    mut e: *mut stbtt__active_edge,
    mut x0: f32,
    mut y0: f32,
    mut x1: f32,
    mut y1: f32,
) {
    if y0 == y1 {
        return;
    }
    if y0 < y1 {
    } else {
        __assert_fail(b"y0 < y1\x00" as *const u8 as *const i8,
                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00" as
                          *const u8 as *const i8,
                      1870 as u32,
                      (*::std::mem::transmute::<&[u8; 96],
                                                &[i8; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
    }
    if (*e).sy <= (*e).ey {
    } else {
        __assert_fail(b"e->sy <= e->ey\x00" as *const u8 as
                          *const i8,
                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00" as
                          *const u8 as *const i8,
                      1871 as u32,
                      (*::std::mem::transmute::<&[u8; 96],
                                                &[i8; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
    }
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
        if x1 <= (x + 1) as f32 {
        } else {
            __assert_fail(b"x1 <= x+1\x00" as *const u8 as
                              *const i8,
                          b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                              as *const u8 as *const i8,
                          1884i32 as u32,
                          (*::std::mem::transmute::<&[u8; 96],
                                                    &[i8; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
        }
    } else if x0 == (x + 1) as f32 {
        if x1 >= x as f32 {
        } else {
            __assert_fail(b"x1 >= x\x00" as *const u8 as *const i8,
                          b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                              as *const u8 as *const i8,
                          1886i32 as u32,
                          (*::std::mem::transmute::<&[u8; 96],
                                                    &[i8; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
        }
    } else if x0 <= x as f32 {
        if x1 <= x as f32 {
        } else {
            __assert_fail(b"x1 <= x\x00" as *const u8 as *const i8,
                          b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                              as *const u8 as *const i8,
                          1888i32 as u32,
                          (*::std::mem::transmute::<&[u8; 96],
                                                    &[i8; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
        }
    } else if x0 >= (x + 1) as f32 {
        if x1 >= (x + 1) as f32 {
        } else {
            __assert_fail(b"x1 >= x+1\x00" as *const u8 as
                              *const i8,
                          b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                              as *const u8 as *const i8,
                          1890 as u32,
                          (*::std::mem::transmute::<&[u8; 96],
                                                    &[i8; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
        }
    } else if x1 >= x as f32 && x1 <= (x + 1) as f32 {
    } else {
        __assert_fail(b"x1 >= x && x1 <= x+1\x00" as *const u8 as
                          *const i8,
                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00" as
                          *const u8 as *const i8,
                      1892i32 as u32,
                      (*::std::mem::transmute::<&[u8; 96],
                                                &[i8; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
    }
    if x0 <= x as f32 && x1 <= x as f32 {
        *scanline.offset(x as isize) += (*e).direction * (y1 - y0)
    } else if !(x0 >= (x + 1) as f32 && x1 >= (x + 1) as f32) {
        if x0 >= x as f32 && x0 <= (x + 1) as f32 && x1 >= x as f32 && x1 <= (x + 1) as f32 {
        } else {
            __assert_fail(b"x0 >= x && x0 <= x+1 && x1 >= x && x1 <= x+1\x00"
                              as *const u8 as *const i8,
                          b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                              as *const u8 as *const i8,
                          1899i32 as u32,
                          (*::std::mem::transmute::<&[u8; 96],
                                                    &[i8; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
        }
        *scanline.offset(x as isize) += (*e).direction
            * (y1 - y0)
            * (1 as f32 - (x0 - x as f32 + (x1 - x as f32)) / 2i32 as f32)
    };
}
unsafe fn stbtt__new_active(
    hh: *mut stbtt__hheap,
    e: *mut stbtt__edge,
    off_x: i32,
    start_point: f32,
    userdata: *mut libc::c_void,
) -> *mut stbtt__active_edge {
    let mut z: *mut stbtt__active_edge = stbtt__hheap_alloc(
        hh,
        ::std::mem::size_of::<stbtt__active_edge>() as u64,
        userdata,
    ) as *mut stbtt__active_edge;
    let dxdy: f32 = ((*e).x1 - (*e).x0) / ((*e).y1 - (*e).y0);
    if !z.is_null() {
    } else {
        __assert_fail(b"z != ((void*)0)\x00" as *const u8 as
                          *const i8,
                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00" as
                          *const u8 as *const i8,
                      1700 as u32,
                      (*::std::mem::transmute::<&[u8; 89],
                                                &[i8; 89]>(b"stbtt__active_edge *stbtt__new_active(stbtt__hheap *, stbtt__edge *, int, float, void *)\x00")).as_ptr());
    }
    if z.is_null() {
        return z;
    }
    (*z).fdx = dxdy;
    (*z).fdy = if dxdy != 0.0f32 {
        1.0f32 / dxdy
    } else {
        0.0f32
    };
    (*z).fx = (*e).x0 + dxdy * (start_point - (*e).y0);
    (*z).fx -= off_x as f32;
    (*z).direction = if 0 != (*e).invert { 1.0f32 } else { -1.0f32 };
    (*z).sy = (*e).y0;
    (*z).ey = (*e).y1;
    (*z).next = 0 as *mut stbtt__active_edge;
    z
}
unsafe fn stbtt__hheap_alloc(
    mut hh: *mut stbtt__hheap,
    mut size: size_t,
    mut userdata: *mut libc::c_void,
) -> *mut libc::c_void {
    if !(*hh).first_free.is_null() {
        let mut p: *mut libc::c_void = (*hh).first_free;
        (*hh).first_free = *(p as *mut *mut libc::c_void);
        p
    } else {
        if (*hh).num_remaining_in_head_chunk == 0 {
            let mut count: i32 = if size < 32i32 as u64 {
                2000
            } else if size < 128i32 as u64 {
                800
            } else {
                100
            };
            let mut c: *mut stbtt__hheap_chunk = fons__tmpalloc(
                (::std::mem::size_of::<stbtt__hheap_chunk>() as u64)
                    .wrapping_add(size.wrapping_mul(count as u64)),
                userdata,
            ) as *mut stbtt__hheap_chunk;
            if c.is_null() {
                return 0 as *mut libc::c_void;
            }
            (*c).next = (*hh).head;
            (*hh).head = c;
            (*hh).num_remaining_in_head_chunk = count
        }
        (*hh).num_remaining_in_head_chunk -= 1;
        ((*hh).head as *mut i8)
            .offset(size.wrapping_mul((*hh).num_remaining_in_head_chunk as u64) as isize)
            as *mut libc::c_void
    }
}
unsafe fn stbtt__hheap_free(mut hh: *mut stbtt__hheap, p: *mut libc::c_void) {
    let ref mut fresh20 = *(p as *mut *mut libc::c_void);
    *fresh20 = (*hh).first_free;
    (*hh).first_free = p;
}
unsafe fn stbtt__sort_edges(p: *mut stbtt__edge, n: i32) {
    stbtt__sort_edges_quicksort(p, n);
    stbtt__sort_edges_ins_sort(p, n);
}
unsafe fn stbtt__sort_edges_ins_sort(p: *mut stbtt__edge, n: i32) {
    let mut i = 1;
    while i < n {
        let mut t: stbtt__edge = *p.offset(i as isize);
        let a: *mut stbtt__edge = &mut t;
        let mut j = i;
        while j > 0 {
            let b: *mut stbtt__edge = &mut *p.offset((j - 1) as isize) as *mut stbtt__edge;
            let c: i32 = ((*a).y0 < (*b).y0) as i32;
            if 0 == c {
                break;
            }
            *p.offset(j as isize) = *p.offset((j - 1) as isize);
            j -= 1
        }
        if i != j {
            *p.offset(j as isize) = t
        }
        i += 1
    }
}
unsafe fn stbtt__sort_edges_quicksort(mut p: *mut stbtt__edge, mut n: i32) {
    while n > 12i32 {
        let mut t: stbtt__edge = stbtt__edge {
            x0: 0.,
            y0: 0.,
            x1: 0.,
            y1: 0.,
            invert: 0,
        };
        let mut c01: i32 = 0;
        let mut c12: i32 = 0;
        let mut c: i32 = 0;
        let mut m: i32 = 0;
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        m = n >> 1;
        c01 = ((*p.offset(0isize)).y0 < (*p.offset(m as isize)).y0) as i32;
        c12 = ((*p.offset(m as isize)).y0 < (*p.offset((n - 1) as isize)).y0) as i32;
        if c01 != c12 {
            let mut z: i32 = 0;
            c = ((*p.offset(0isize)).y0 < (*p.offset((n - 1) as isize)).y0) as i32;
            z = if c == c12 { 0 } else { n - 1 };
            t = *p.offset(z as isize);
            *p.offset(z as isize) = *p.offset(m as isize);
            *p.offset(m as isize) = t
        }
        t = *p.offset(0isize);
        *p.offset(0isize) = *p.offset(m as isize);
        *p.offset(m as isize) = t;
        i = 1;
        j = n - 1;
        loop {
            while (*p.offset(i as isize)).y0 < (*p.offset(0isize)).y0 {
                i += 1
            }
            while (*p.offset(0isize)).y0 < (*p.offset(j as isize)).y0 {
                j -= 1
            }
            /* make sure we haven't crossed */
            if i >= j {
                break;
            }
            t = *p.offset(i as isize);
            *p.offset(i as isize) = *p.offset(j as isize);
            *p.offset(j as isize) = t;
            i += 1;
            j -= 1
        }
        if j < n - i {
            stbtt__sort_edges_quicksort(p, j);
            p = p.offset(i as isize);
            n = n - i
        } else {
            stbtt__sort_edges_quicksort(p.offset(i as isize), n - i);
            n = j
        }
    }
}

pub unsafe fn stbtt_GetGlyphBitmapBoxSubpixel(
    font: *const FontInfo,
    glyph: i32,
    scale_x: f32,
    scale_y: f32,
    shift_x: f32,
    shift_y: f32,
    ix0: *mut i32,
    iy0: *mut i32,
    ix1: *mut i32,
    iy1: *mut i32,
) {
    let mut x0: i32 = 0;
    let mut y0: i32 = 0;
    let mut x1: i32 = 0;
    let mut y1: i32 = 0;
    if 0 == stbtt_GetGlyphBox(font, glyph, &mut x0, &mut y0, &mut x1, &mut y1) {
        if !ix0.is_null() {
            *ix0 = 0
        }
        if !iy0.is_null() {
            *iy0 = 0
        }
        if !ix1.is_null() {
            *ix1 = 0
        }
        if !iy1.is_null() {
            *iy1 = 0
        }
    } else {
        if !ix0.is_null() {
            *ix0 = (x0 as f32 * scale_x + shift_x).floor() as i32
        }
        if !iy0.is_null() {
            *iy0 = (-y1 as f32 * scale_y + shift_y).floor() as i32
        }
        if !ix1.is_null() {
            *ix1 = (x1 as f32 * scale_x + shift_x).ceil() as i32
        }
        if !iy1.is_null() {
            *iy1 = (-y0 as f32 * scale_y + shift_y).ceil() as i32
        }
    };
}

pub unsafe fn stbtt_GetGlyphBox(
    info: *const FontInfo,
    glyph_index: i32,
    x0: *mut i32,
    y0: *mut i32,
    x1: *mut i32,
    y1: *mut i32,
) -> i32 {
    let g: i32 = stbtt__GetGlyfOffset(info, glyph_index);
    if g < 0 {
        return 0;
    }
    if !x0.is_null() {
        *x0 = read_i16((*info).data.offset(g as isize).offset(2)) as i32
    }
    if !y0.is_null() {
        *y0 = read_i16((*info).data.offset(g as isize).offset(4)) as i32
    }
    if !x1.is_null() {
        *x1 = read_i16((*info).data.offset(g as isize).offset(6)) as i32
    }
    if !y1.is_null() {
        *y1 = read_i16((*info).data.offset(g as isize).offset(8)) as i32
    }
    1
}

pub unsafe fn fons__allocGlyph(mut font: *mut Font) -> *mut Glyph {
    if (*font).nglyphs + 1 > (*font).cglyphs {
        (*font).cglyphs = if (*font).cglyphs == 0 {
            8
        } else {
            (*font).cglyphs * 2
        };
        (*font).glyphs = realloc(
            (*font).glyphs as *mut libc::c_void,
            (::std::mem::size_of::<Glyph>() as u64).wrapping_mul((*font).cglyphs as u64),
        ) as *mut Glyph;
        if (*font).glyphs.is_null() {
            return 0 as *mut Glyph;
        }
    }
    (*font).nglyphs += 1;
    &mut *(*font).glyphs.offset(((*font).nglyphs - 1) as isize) as *mut Glyph
}

pub unsafe fn fons__tt_buildGlyphBitmap(
    font: *mut FontInfo,
    glyph: i32,
    _size: f32,
    scale: f32,
    advance: *mut i32,
    lsb: *mut i32,
    x0: *mut i32,
    y0: *mut i32,
    x1: *mut i32,
    y1: *mut i32,
) -> i32 {
    stbtt_GetGlyphHMetrics(font, glyph, advance, lsb);
    stbtt_GetGlyphBitmapBox(font, glyph, scale, scale, x0, y0, x1, y1);
    1
}

pub unsafe fn stbtt_GetGlyphBitmapBox(
    font: *const FontInfo,
    glyph: i32,
    scale_x: f32,
    scale_y: f32,
    ix0: *mut i32,
    iy0: *mut i32,
    ix1: *mut i32,
    iy1: *mut i32,
) {
    stbtt_GetGlyphBitmapBoxSubpixel(
        font, glyph, scale_x, scale_y, 0.0f32, 0.0f32, ix0, iy0, ix1, iy1,
    );
}
// Gets the bounding box of the visible part of the glyph, in unscaled coordinates

pub unsafe fn stbtt_GetGlyphHMetrics(
    info: *const FontInfo,
    glyph_index: i32,
    advanceWidth: *mut i32,
    leftSideBearing: *mut i32,
) {
    let numOfLongHorMetrics = read_u16((*info).data.offset((*info).hhea as isize).offset(34isize));
    if glyph_index < numOfLongHorMetrics as i32 {
        if !advanceWidth.is_null() {
            *advanceWidth = read_i16(
                (*info)
                    .data
                    .offset((*info).hmtx as isize)
                    .offset((4i32 * glyph_index) as isize),
            ) as i32
        }
        if !leftSideBearing.is_null() {
            *leftSideBearing = read_i16(
                (*info)
                    .data
                    .offset((*info).hmtx as isize)
                    .offset((4i32 * glyph_index) as isize)
                    .offset(2isize),
            ) as i32
        }
    } else {
        if !advanceWidth.is_null() {
            *advanceWidth = read_i16(
                (*info)
                    .data
                    .offset((*info).hmtx as isize)
                    .offset((4i32 * (numOfLongHorMetrics as i32 - 1)) as isize),
            ) as i32
        }
        if !leftSideBearing.is_null() {
            *leftSideBearing = read_i16(
                (*info)
                    .data
                    .offset((*info).hmtx as isize)
                    .offset((4i32 * numOfLongHorMetrics as i32) as isize)
                    .offset((2i32 * (glyph_index - numOfLongHorMetrics as i32)) as isize),
            ) as i32
        }
    };
}

pub unsafe fn fons__tt_getPixelHeightScale(mut font: *mut FontInfo, mut size: f32) -> f32 {
    return stbtt_ScaleForPixelHeight(font, size);
}
// If you're going to perform multiple operations on the same character
// and you want a speed-up, call this function with the character you're
// going to process, then use glyph-based functions instead of the
// codepoint-based functions.
// ////////////////////////////////////////////////////////////////////////////
//
// CHARACTER PROPERTIES
//

pub unsafe fn stbtt_ScaleForPixelHeight(mut info: *const FontInfo, mut height: f32) -> f32 {
    let mut fheight: i32 = read_i16((*info).data.offset((*info).hhea as isize).offset(4isize))
        as i32
        - read_i16((*info).data.offset((*info).hhea as isize).offset(6isize)) as i32;
    return height as f32 / fheight as f32;
}

pub unsafe fn fons__tt_getGlyphIndex(font: *mut FontInfo, codepoint: i32) -> i32 {
    stbtt_FindGlyphIndex(font, codepoint)
}
// Given an offset into the file that defines a font, this function builds
// the necessary cached info for the rest of the system. You must allocate
// the stbtt_fontinfo yourself, and stbtt_InitFont will fill it out. You don't
// need to do anything special to free it, because the contents are pure
// value data with no additional data structures. Returns 0 on failure.
// ////////////////////////////////////////////////////////////////////////////
//
// CHARACTER TO GLYPH-INDEX CONVERSIOn

pub unsafe fn stbtt_FindGlyphIndex(info: *const FontInfo, unicode_codepoint: i32) -> i32 {
    let data: *mut u8 = (*info).data;
    let index_map: u32 = (*info).index_map as u32;
    let format: u16 = read_u16(data.offset(index_map as isize).offset(0isize));
    if format as i32 == 0 {
        let bytes: i32 = read_u16(data.offset(index_map as isize).offset(2isize)) as i32;
        if unicode_codepoint < bytes - 6i32 {
            return *(data
                .offset(index_map as isize)
                .offset(6isize)
                .offset(unicode_codepoint as isize) as *mut u8) as i32;
        }
        return 0;
    } else {
        if format as i32 == 6i32 {
            let first: u32 = read_u16(data.offset(index_map as isize).offset(6isize)) as u32;
            let count: u32 = read_u16(data.offset(index_map as isize).offset(8isize)) as u32;
            if unicode_codepoint as u32 >= first
                && (unicode_codepoint as u32) < first.wrapping_add(count)
            {
                return read_u16(
                    data.offset(index_map as isize).offset(10isize).offset(
                        (unicode_codepoint as u32)
                            .wrapping_sub(first)
                            .wrapping_mul(2i32 as u32) as isize,
                    ),
                ) as i32;
            }
            return 0;
        } else {
            if format as i32 == 2i32 {
                __assert_fail(
                    b"0\x00" as *const u8 as *const i8,
                    b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00" as *const u8 as *const i8,
                    1094i32 as u32,
                    (*::std::mem::transmute::<&[u8; 54], &[i8; 54]>(
                        b"int stbtt_FindGlyphIndex(const stbtt_fontinfo *, int)\x00",
                    ))
                    .as_ptr(),
                );
                return 0;
            } else {
                if format as i32 == 4i32 {
                    let segcount: u16 = (read_u16(data.offset(index_map as isize).offset(6isize)) as i32 >> 1) as u16;
                    let mut searchRange: u16 = (read_u16(data.offset(index_map as isize).offset(8isize)) as i32 >> 1) as u16;
                    let mut entrySelector: u16 = read_u16(data.offset(index_map as isize).offset(10isize));
                    let rangeShift: u16 = (read_u16(data.offset(index_map as isize).offset(12isize)) as i32 >> 1) as u16;
                    let endCount: u32 = index_map.wrapping_add(14i32 as u32);
                    let mut search: u32 = endCount;
                    if unicode_codepoint > 0xffffi32 {
                        return 0;
                    }
                    if unicode_codepoint
                        >= read_u16(data.offset(search as isize).offset((rangeShift as i32 * 2i32) as isize)) as i32
                    {
                        search = (search as u32).wrapping_add((rangeShift as i32 * 2i32) as u32)
                            as u32 as u32
                    }
                    search = (search as u32).wrapping_sub(2i32 as u32) as u32 as u32;
                    while 0 != entrySelector {
                        searchRange = (searchRange as i32 >> 1) as u16;
                        let end = read_u16(
                            data.offset(search as isize)
                                .offset((searchRange as i32 * 2i32) as isize),
                        );
                        if unicode_codepoint > end as i32 {
                            search = (search as u32)
                                .wrapping_add((searchRange as i32 * 2i32) as u32)
                                as u32 as u32
                        }
                        entrySelector = entrySelector.wrapping_sub(1)
                    }
                    search = (search as u32).wrapping_add(2i32 as u32) as u32 as u32;
                    let offset;
                    let start;
                    let item: u16 = (search.wrapping_sub(endCount) >> 1) as u16;
                    if unicode_codepoint
                        <= read_u16(
                            data.offset(endCount as isize)
                                .offset((2i32 * item as i32) as isize),
                        ) as i32
                    {
                    } else {
                        __assert_fail(
                            b"unicode_codepoint <= ttUSHORT(data + endCount + 2*item)\x00"
                                as *const u8 as *const i8,
                            b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00" as *const u8
                                as *const i8,
                            1130 as u32,
                            (*::std::mem::transmute::<&[u8; 54], &[i8; 54]>(
                                b"int stbtt_FindGlyphIndex(const stbtt_fontinfo *, int)\x00",
                            ))
                            .as_ptr(),
                        );
                    }
                    start = read_u16(
                        data.offset(index_map as isize)
                            .offset(14isize)
                            .offset((segcount as i32 * 2i32) as isize)
                            .offset(2isize)
                            .offset((2i32 * item as i32) as isize),
                    );
                    if unicode_codepoint < start as i32 {
                        return 0;
                    }
                    offset = read_u16(
                        data.offset(index_map as isize)
                            .offset(14isize)
                            .offset((segcount as i32 * 6i32) as isize)
                            .offset(2isize)
                            .offset((2i32 * item as i32) as isize),
                    );
                    if offset as i32 == 0 {
                        return (unicode_codepoint
                            + read_i16(
                                data.offset(index_map as isize)
                                    .offset(14isize)
                                    .offset((segcount as i32 * 4i32) as isize)
                                    .offset(2isize)
                                    .offset((2i32 * item as i32) as isize),
                            ) as i32) as u16 as i32;
                    }
                    return read_u16(
                        data.offset(offset as i32 as isize)
                            .offset(((unicode_codepoint - start as i32) * 2i32) as isize)
                            .offset(index_map as isize)
                            .offset(14isize)
                            .offset((segcount as i32 * 6i32) as isize)
                            .offset(2isize)
                            .offset((2i32 * item as i32) as isize),
                    ) as i32;
                } else {
                    if format as i32 == 12i32 || format as i32 == 13i32 {
                        let ngroups: u32 =
                            read_u32(data.offset(index_map as isize).offset(12isize));
                        let mut low = 0;
                        let mut high = ngroups as i32;
                        while low < high {
                            let mid: i32 = low + (high - low >> 1);
                            let start_char: u32 = read_u32(
                                data.offset(index_map as isize)
                                    .offset(16isize)
                                    .offset((mid * 12i32) as isize),
                            );
                            let end_char: u32 = read_u32(
                                data.offset(index_map as isize)
                                    .offset(16isize)
                                    .offset((mid * 12i32) as isize)
                                    .offset(4isize),
                            );
                            if (unicode_codepoint as u32) < start_char {
                                high = mid
                            } else if unicode_codepoint as u32 > end_char {
                                low = mid + 1
                            } else {
                                let start_glyph: u32 = read_u32(
                                    data.offset(index_map as isize)
                                        .offset(16isize)
                                        .offset((mid * 12i32) as isize)
                                        .offset(8isize),
                                );
                                if format as i32 == 12i32 {
                                    return start_glyph
                                        .wrapping_add(unicode_codepoint as u32)
                                        .wrapping_sub(start_char)
                                        as i32;
                                } else {
                                    return start_glyph as i32;
                                }
                            }
                        }
                        return 0;
                    }
                }
            }
        }
    }
    __assert_fail(
        b"0\x00" as *const u8 as *const i8,
        b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00" as *const u8 as *const i8,
        1165i32 as u32,
        (*::std::mem::transmute::<&[u8; 54], &[i8; 54]>(
            b"int stbtt_FindGlyphIndex(const stbtt_fontinfo *, int)\x00",
        ))
        .as_ptr(),
    );
    0
}

pub fn fons__hashint(mut a: u32) -> u32 {
    a = a.wrapping_add(!(a << 15));
    a ^= a >> 10;
    a = a.wrapping_add(a << 3);
    a ^= a >> 6;
    a = a.wrapping_add(!(a << 11));
    a ^= a >> 16;
    a
}
// empty
// STB_TRUETYPE_IMPLEMENTATION
// Copyright (c) 2008-2010 Bjoern Hoehrmann <bjoern@hoehrmann.de>
// See http://bjoern.hoehrmann.de/utf-8/decoder/dfa/ for details.

pub unsafe fn fons__decutf8(state: *mut u32, codep: *mut u32, byte: u32) -> u32 {
    static mut utf8d: [u8; 364] = [
        // The first part of the table maps bytes to character classes that
        // to reduce the size of the transition table and create bitmasks.
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 9, 9, 9, 9, 9, 9, 9,
        9, 9, 9, 9, 9, 9, 9, 9, 9, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
        7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 10, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 3, 3, 11,
        6, 6, 6, 5, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
        // The second part is a transition table that maps a combination
        // of a state of the automaton and a character class to a state.
        0, 12, 24, 36, 60, 96, 84, 12, 12, 12, 48, 72, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
        12, 12, 0, 12, 12, 12, 12, 12, 0, 12, 0, 12, 12, 12, 24, 12, 12, 12, 12, 12, 24, 12, 24,
        12, 12, 12, 12, 12, 12, 12, 12, 12, 24, 12, 12, 12, 12, 12, 24, 12, 12, 12, 12, 12, 12, 12,
        24, 12, 12, 12, 12, 12, 12, 12, 12, 12, 36, 12, 36, 12, 12, 12, 36, 12, 12, 12, 12, 12, 36,
        12, 36, 12, 12, 12, 36, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
    ];
    let type_0: u32 = utf8d[byte as usize] as u32;
    *codep = if *state != 0 {
        byte & 0x3f | *codep << 6
    } else {
        (0xff >> type_0) as u32 & byte
    };
    *state = utf8d[256u32.wrapping_add(*state).wrapping_add(type_0) as usize] as u32;
    *state
}

pub unsafe fn fons__getVertAlign(
    _stash: *mut Stash,
    font: *mut Font,
    align: i32,
    isize: i16,
) -> f32 {
    let font: &mut Font = transmute(font);

    if 0 != align & FONS_ALIGN_TOP as i32 {
        font.ascender * isize as f32 / 10.0
    } else if 0 != align & FONS_ALIGN_MIDDLE as i32 {
        (font.ascender + font.descender) / 2.0 * isize as f32 / 10.0
    } else if 0 != align & FONS_ALIGN_BASELINE as i32 {
        0.0
    } else if 0 != align & FONS_ALIGN_BOTTOM as i32 {
        font.descender * isize as f32 / 10.0
    } else {
        0.0
    }
}
// Measure text

pub unsafe fn fonsTextBounds(
    stash: *mut Stash,
    mut x: f32,
    mut y: f32,
    mut str: *const i8,
    mut end: *const i8,
    bounds: *mut f32,
) -> f32 {
    let state: *mut State = (*stash).state_mut();
    let mut codepoint: u32 = 0;
    let mut utf8state: u32 = 0 as u32;
    let mut q: Quad = Default::default();
    let mut prevGlyphIndex: i32 = -1;
    let isize: i16 = ((*state).size * 10.0f32) as i16;
    let iblur: i16 = (*state).blur as i16;

    if stash.is_null() {
        return 0 as f32;
    }
    if (*state).font < 0 || (*state).font >= (*stash).nfonts as i32 {
        return 0 as f32;
    }

    let font = *(*stash).fonts.offset((*state).font as isize);
    if (*font).data.is_null() {
        return 0 as f32;
    }

    let scale = fons__tt_getPixelHeightScale(&mut (*font).font, isize as f32 / 10.0f32);
    y += fons__getVertAlign(stash, font, (*state).align, isize);

    let mut maxx = x;
    let mut minx = maxx;
    let mut maxy = y;
    let mut miny = maxy;
    let startx = x;
    if end.is_null() {
        end = str.offset(strlen(str) as isize)
    }
    while str != end {
        if !(0 != fons__decutf8(&mut utf8state, &mut codepoint, *(str as *const u8) as u32)) {
            let glyph = fons__getGlyph(
                stash,
                font,
                codepoint,
                isize,
                iblur,
                FONS_GLYPH_BITMAP_OPTIONAL as i32,
            );
            if !glyph.is_null() {
                fons__getQuad(
                    stash,
                    font,
                    prevGlyphIndex,
                    glyph,
                    scale,
                    (*state).spacing,
                    &mut x,
                    &mut y,
                    &mut q,
                );
                if q.x0 < minx {
                    minx = q.x0
                }
                if q.x1 > maxx {
                    maxx = q.x1
                }

                if q.y0 < miny {
                    miny = q.y0
                }
                if q.y1 > maxy {
                    maxy = q.y1
                }
            }
            prevGlyphIndex = if !glyph.is_null() { (*glyph).index } else { -1 }
        }
        str = str.offset(1isize)
    }
    let advance = x - startx;
    if !(0 != (*state).align & FONS_ALIGN_LEFT as i32) {
        if 0 != (*state).align & FONS_ALIGN_RIGHT as i32 {
            minx -= advance;
            maxx -= advance
        } else if 0 != (*state).align & FONS_ALIGN_CENTER as i32 {
            minx -= advance * 0.5f32;
            maxx -= advance * 0.5f32
        }
    }
    if !bounds.is_null() {
        *bounds.offset(0isize) = minx;
        *bounds.offset(1isize) = miny;
        *bounds.offset(2isize) = maxx;
        *bounds.offset(3isize) = maxy
    }
    advance
}

pub unsafe fn fonsLineBounds(
    stash: *mut Stash,
    mut y: f32,
    miny: *mut f32,
    maxy: *mut f32,
) {
    let state: *mut State = (*stash).state_mut();
    if stash.is_null() {
        return;
    }
    if (*state).font < 0 || (*state).font >= (*stash).nfonts as i32 {
        return;
    }
    let font = *(*stash).fonts.offset((*state).font as isize);
    let isize = ((*state).size * 10.0f32) as i16;
    if (*font).data.is_null() {
        return;
    }
    y += fons__getVertAlign(stash, font, (*state).align, isize);
    *miny = y - (*font).ascender * isize as f32 / 10.0f32;
    *maxy = *miny + (*font).lineh * isize as i32 as f32 / 10.0f32
}

pub unsafe fn fonsVertMetrics(stash: &mut Stash) -> Option<Metrics> {
    let nfonts = stash.nfonts as i32;
    let state = stash.state();
    if state.font < 0 || state.font >= nfonts {
        return None;
    }
    let font = &*(*stash.fonts.offset(state.font as isize));
    let size = (state.size * 10.0) as i16;
    if font.data.is_null() {
        return None;
    }
    Some(Metrics {
        ascender: font.ascender * size as f32 / 10.,
        descender: font.descender * size as f32 / 10.,
        line_height: font.lineh * size as f32 / 10.,
    })
}

// Text iterator

pub unsafe fn fonsTextIterInit(
    stash: *mut Stash,
    mut iter: *mut FONStextIter,
    mut x: f32,
    mut y: f32,
    str: *const i8,
    mut end: *const i8,
    bitmapOption: i32,
) -> i32 {
    let state: *mut State = (*stash).state_mut();
    memset(
        iter as *mut libc::c_void,
        0,
        ::std::mem::size_of::<FONStextIter>() as u64,
    );
    if stash.is_null() {
        return 0;
    }
    if (*state).font < 0 || (*state).font >= (*stash).nfonts as i32 {
        return 0;
    }
    (*iter).font = *(*stash).fonts.offset((*state).font as isize);
    if (*(*iter).font).data.is_null() {
        return 0;
    }
    (*iter).isize_0 = ((*state).size * 10.0f32) as i16;
    (*iter).iblur = (*state).blur as i16;
    (*iter).scale =
        fons__tt_getPixelHeightScale(&mut (*(*iter).font).font, (*iter).isize_0 as f32 / 10.0f32);
    if !(0 != (*state).align & FONS_ALIGN_LEFT as i32) {
        if 0 != (*state).align & FONS_ALIGN_RIGHT as i32 {
            let width = fonsTextBounds(stash, x, y, str, end, 0 as *mut f32);
            x -= width
        } else if 0 != (*state).align & FONS_ALIGN_CENTER as i32 {
            let width = fonsTextBounds(stash, x, y, str, end, 0 as *mut f32);
            x -= width * 0.5f32
        }
    }
    y += fons__getVertAlign(stash, (*iter).font, (*state).align, (*iter).isize_0);
    if end.is_null() {
        end = str.offset(strlen(str) as isize)
    }
    (*iter).nextx = x;
    (*iter).x = (*iter).nextx;
    (*iter).nexty = y;
    (*iter).y = (*iter).nexty;
    (*iter).spacing = (*state).spacing;
    (*iter).str_0 = str;
    (*iter).next = str;
    (*iter).end = end;
    (*iter).codepoint = 0 as u32;
    (*iter).prevGlyphIndex = -1;
    (*iter).bitmapOption = bitmapOption;
    1
}

pub unsafe fn fonsTextIterNext(
    stash: *mut Stash,
    mut iter: *mut FONStextIter,
    quad: *mut Quad,
) -> i32 {
    let mut str: *const i8 = (*iter).next;
    (*iter).str_0 = (*iter).next;
    if str == (*iter).end {
        return 0;
    }
    while str != (*iter).end {
        if 0 != fons__decutf8(
            &mut (*iter).utf8state,
            &mut (*iter).codepoint,
            *(str as *const u8) as u32,
        ) {
            str = str.offset(1isize)
        } else {
            str = str.offset(1isize);
            (*iter).x = (*iter).nextx;
            (*iter).y = (*iter).nexty;
            let glyph = fons__getGlyph(
                stash,
                (*iter).font,
                (*iter).codepoint,
                (*iter).isize_0,
                (*iter).iblur,
                (*iter).bitmapOption,
            );
            if !glyph.is_null() {
                fons__getQuad(
                    stash,
                    (*iter).font,
                    (*iter).prevGlyphIndex,
                    glyph,
                    (*iter).scale,
                    (*iter).spacing,
                    &mut (*iter).nextx,
                    &mut (*iter).nexty,
                    quad,
                );
            }
            (*iter).prevGlyphIndex = if !glyph.is_null() { (*glyph).index } else { -1 };
            break;
        }
    }
    (*iter).next = str;
    1
}

pub unsafe fn fonsAddFallbackFont(stash: *mut Stash, base: i32, fallback: i32) -> i32 {
    let mut baseFont: *mut Font = *(*stash).fonts.offset(base as isize);
    if (*baseFont).nfallbacks < 20 {
        let fresh34 = (*baseFont).nfallbacks;
        (*baseFont).nfallbacks = (*baseFont).nfallbacks + 1;
        (*baseFont).fallbacks[fresh34 as usize] = fallback;
        return 1;
    }
    0
}

// Pull texture changes

pub unsafe fn fonsGetTextureData(stash: *mut Stash, width: *mut i32, height: *mut i32) -> *const u8 {
    if !width.is_null() {
        *width = (*stash).width
    }
    if !height.is_null() {
        *height = (*stash).height
    }
    (*stash).texData
}