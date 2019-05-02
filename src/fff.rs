#![allow(/*dead_code,*/
         mutable_transmutes,
         non_camel_case_types,
         non_snake_case,
         non_upper_case_globals,
         unused_assignments,
         unused_mut)]

#![deny(dead_code)]

#![feature(extern_types, label_break_value)]
extern crate libc;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    
    fn free(__ptr: *mut libc::c_void);
    
    fn fclose(__stream: *mut FILE) -> libc::c_int;
    
    fn fopen(_: *const libc::c_char, _: *const libc::c_char) -> *mut FILE;
    
    fn fread(_: *mut libc::c_void, _: libc::c_ulong, _: libc::c_ulong,
             _: *mut FILE) -> libc::c_ulong;
    
    fn fseek(__stream: *mut FILE, __off: libc::c_long, __whence: libc::c_int)
     -> libc::c_int;
    
    fn ftell(__stream: *mut FILE) -> libc::c_long;
    
    fn sqrt(_: libc::c_double) -> libc::c_double;
    
    fn ceil(_: libc::c_double) -> libc::c_double;
    
    fn fabs(_: libc::c_double) -> libc::c_double;
    
    fn floor(_: libc::c_double) -> libc::c_double;
    
    fn expf(_: libc::c_float) -> libc::c_float;
    
    fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong)
     -> *mut libc::c_void;
    
    fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong)
     -> *mut libc::c_void;
    
    fn strncpy(_: *mut libc::c_char, _: *const libc::c_char, _: libc::c_ulong)
     -> *mut libc::c_char;
    
    fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> libc::c_int;
    
    fn strlen(_: *const libc::c_char) -> libc::c_ulong;
    
    fn __assert_fail(__assertion: *const libc::c_char,
                     __file: *const libc::c_char, __line: libc::c_uint,
                     __function: *const libc::c_char) -> !;
}
pub type size_t = libc::c_ulong;
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: libc::c_int,
    pub _IO_read_ptr: *mut libc::c_char,
    pub _IO_read_end: *mut libc::c_char,
    pub _IO_read_base: *mut libc::c_char,
    pub _IO_write_base: *mut libc::c_char,
    pub _IO_write_ptr: *mut libc::c_char,
    pub _IO_write_end: *mut libc::c_char,
    pub _IO_buf_base: *mut libc::c_char,
    pub _IO_buf_end: *mut libc::c_char,
    pub _IO_save_base: *mut libc::c_char,
    pub _IO_backup_base: *mut libc::c_char,
    pub _IO_save_end: *mut libc::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: libc::c_int,
    pub _flags2: libc::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: libc::c_ushort,
    pub _vtable_offset: libc::c_schar,
    pub _shortbuf: [libc::c_char; 1],
    pub _lock: *mut libc::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut libc::c_void,
    pub __pad5: size_t,
    pub _mode: libc::c_int,
    pub _unused2: [libc::c_char; 20],
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
pub type FONSflags = libc::c_uint;
//pub const FONS_ZERO_BOTTOMLEFT: FONSflags = 2;
pub const FONS_ZERO_TOPLEFT: FONSflags = 1;
pub type FONSalign = libc::c_uint;
// Default
pub const FONS_ALIGN_BASELINE: FONSalign = 64;
pub const FONS_ALIGN_BOTTOM: FONSalign = 32;
pub const FONS_ALIGN_MIDDLE: FONSalign = 16;
// Vertical align
pub const FONS_ALIGN_TOP: FONSalign = 8;
pub const FONS_ALIGN_RIGHT: FONSalign = 4;
pub const FONS_ALIGN_CENTER: FONSalign = 2;
// Horizontal align
// Default
pub const FONS_ALIGN_LEFT: FONSalign = 1;
pub type FONSglyphBitmap = libc::c_uint;
pub const FONS_GLYPH_BITMAP_REQUIRED: FONSglyphBitmap = 2;
pub const FONS_GLYPH_BITMAP_OPTIONAL: FONSglyphBitmap = 1;
pub type FONSerrorCode = libc::c_uint;
// Trying to pop too many states fonsPopState().
//pub const FONS_STATES_UNDERFLOW: FONSerrorCode = 4;
// Calls to fonsPushState has created too large stack, if you need deep state stack bump up FONS_MAX_STATES.
pub const FONS_STATES_OVERFLOW: FONSerrorCode = 3;
// Scratch memory used to render glyphs is full, requested size reported in 'val', you may need to bump up FONS_SCRATCH_BUF_SIZE.
pub const FONS_SCRATCH_FULL: FONSerrorCode = 2;
// Font atlas is full.
pub const FONS_ATLAS_FULL: FONSerrorCode = 1;
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct FONSparams {
    pub width: libc::c_int,
    pub height: libc::c_int,
    pub flags: libc::c_uchar,
    pub userPtr: *mut libc::c_void,
    pub renderCreate: Option<unsafe extern "C" fn(_: *mut libc::c_void,
                                                  _: libc::c_int,
                                                  _: libc::c_int)
                                 -> libc::c_int>,
    pub renderResize: Option<unsafe extern "C" fn(_: *mut libc::c_void,
                                                  _: libc::c_int,
                                                  _: libc::c_int)
                                 -> libc::c_int>,
    pub renderUpdate: Option<unsafe extern "C" fn(_: *mut libc::c_void,
                                                  _: *mut libc::c_int,
                                                  _: *const libc::c_uchar)
                                 -> ()>,
    pub renderDraw: Option<unsafe extern "C" fn(_: *mut libc::c_void,
                                                _: *const libc::c_float,
                                                _: *const libc::c_float,
                                                _: *const libc::c_uint,
                                                _: libc::c_int) -> ()>,
    pub renderDelete: Option<unsafe extern "C" fn(_: *mut libc::c_void)
                                 -> ()>,
}
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct FONSquad {
    pub x0: libc::c_float,
    pub y0: libc::c_float,
    pub s0: libc::c_float,
    pub t0: libc::c_float,
    pub x1: libc::c_float,
    pub y1: libc::c_float,
    pub s1: libc::c_float,
    pub t1: libc::c_float,
}
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct FONStextIter {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub nextx: libc::c_float,
    pub nexty: libc::c_float,
    pub scale: libc::c_float,
    pub spacing: libc::c_float,
    pub codepoint: libc::c_uint,
    pub isize_0: libc::c_short,
    pub iblur: libc::c_short,
    pub font: *mut FONSfont,
    pub prevGlyphIndex: libc::c_int,
    pub str_0: *const libc::c_char,
    pub next: *const libc::c_char,
    pub end: *const libc::c_char,
    pub utf8state: libc::c_uint,
    pub bitmapOption: libc::c_int,
}
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct FONSfont {
    pub font: FONSttFontImpl,
    pub name: [libc::c_char; 64],
    pub data: *mut libc::c_uchar,
    pub dataSize: libc::c_int,
    pub freeData: libc::c_uchar,
    pub ascender: libc::c_float,
    pub descender: libc::c_float,
    pub lineh: libc::c_float,
    pub glyphs: *mut FONSglyph,
    pub cglyphs: libc::c_int,
    pub nglyphs: libc::c_int,
    pub lut: [libc::c_int; 256],
    pub fallbacks: [libc::c_int; 20],
    pub nfallbacks: libc::c_int,
}
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct FONSglyph {
    pub codepoint: libc::c_uint,
    pub index: libc::c_int,
    pub next: libc::c_int,
    pub size: libc::c_short,
    pub blur: libc::c_short,
    pub x0: libc::c_short,
    pub y0: libc::c_short,
    pub x1: libc::c_short,
    pub y1: libc::c_short,
    pub xadv: libc::c_short,
    pub xoff: libc::c_short,
    pub yoff: libc::c_short,
}
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct FONSttFontImpl {
    pub font: stbtt_fontinfo,
}
// Each .ttf/.ttc file may have more than one font. Each font has a sequential
// index number starting from 0. Call this function to get the font offset for
// a given index; it returns -1 if the index is out of range. A regular .ttf
// file will only define one font and it always be at offset 0, so it will
// return '0' for index 0, and -1 for all other indices. You can just skip
// this step if you know it's that kind of font.
// The following structure is defined publically so you can declare one on
// the stack or as a global or etc, but you should treat it as opaque.
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbtt_fontinfo {
    pub userdata: *mut libc::c_void,
    pub data: *mut libc::c_uchar,
    pub fontstart: libc::c_int,
    pub numGlyphs: libc::c_int,
    pub loca: libc::c_int,
    pub head: libc::c_int,
    pub glyf: libc::c_int,
    pub hhea: libc::c_int,
    pub hmtx: libc::c_int,
    pub kern: libc::c_int,
    pub index_map: libc::c_int,
    pub indexToLocFormat: libc::c_int,
}
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct FONScontext {
    pub params: FONSparams,
    pub itw: libc::c_float,
    pub ith: libc::c_float,
    pub texData: *mut libc::c_uchar,
    pub dirtyRect: [libc::c_int; 4],
    pub fonts: *mut *mut FONSfont,
    pub atlas: *mut FONSatlas,
    pub cfonts: libc::c_int,
    pub nfonts: libc::c_int,
    pub verts: [libc::c_float; 2048],
    pub tcoords: [libc::c_float; 2048],
    pub colors: [libc::c_uint; 1024],
    pub nverts: libc::c_int,
    pub scratch: *mut libc::c_uchar,
    pub nscratch: libc::c_int,
    pub states: [FONSstate; 20],
    pub nstates: libc::c_int,
    pub handleError: Option<unsafe extern "C" fn(_: *mut libc::c_void,
                                                 _: libc::c_int,
                                                 _: libc::c_int) -> ()>,
    pub errorUptr: *mut libc::c_void,
}
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct FONSstate {
    pub font: libc::c_int,
    pub align: libc::c_int,
    pub size: libc::c_float,
    pub color: libc::c_uint,
    pub blur: libc::c_float,
    pub spacing: libc::c_float,
}
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct FONSatlas {
    pub width: libc::c_int,
    pub height: libc::c_int,
    pub nodes: *mut FONSatlasNode,
    pub nnodes: libc::c_int,
    pub cnodes: libc::c_int,
}
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct FONSatlasNode {
    pub x: libc::c_short,
    pub y: libc::c_short,
    pub width: libc::c_short,
}
pub type stbtt_int16 = libc::c_short;

pub type stbtt_uint8 = libc::c_uchar;
pub type stbtt_uint16 = libc::c_ushort;
pub type stbtt_uint32 = libc::c_uint;
pub type stbtt_int32 = libc::c_int;
// you can predefine this to use different values
// (we share this with other code at RAD)
// can't use stbtt_int16 because that's not visible in the header file
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbtt_vertex {
    pub x: libc::c_short,
    pub y: libc::c_short,
    pub cx: libc::c_short,
    pub cy: libc::c_short,
    pub type_0: libc::c_uchar,
    pub padding: libc::c_uchar,
}
pub type stbtt_int8 = libc::c_schar;
pub const STBTT_vline: unnamed = 2;
pub const STBTT_vcurve: unnamed = 3;
pub const STBTT_vmove: unnamed = 1;
// @TODO: don't expose this structure
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbtt__bitmap {
    pub w: libc::c_int,
    pub h: libc::c_int,
    pub stride: libc::c_int,
    pub pixels: *mut libc::c_uchar,
}
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbtt__point {
    pub x: libc::c_float,
    pub y: libc::c_float,
}
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbtt__edge {
    pub x0: libc::c_float,
    pub y0: libc::c_float,
    pub x1: libc::c_float,
    pub y1: libc::c_float,
    pub invert: libc::c_int,
}
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbtt__hheap {
    pub head: *mut stbtt__hheap_chunk,
    pub first_free: *mut libc::c_void,
    pub num_remaining_in_head_chunk: libc::c_int,
}
// ////////////////////////////////////////////////////////////////////////////
//
//  Rasterizer
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbtt__hheap_chunk {
    pub next: *mut stbtt__hheap_chunk,
}
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbtt__active_edge {
    pub next: *mut stbtt__active_edge,
    pub fx: libc::c_float,
    pub fdx: libc::c_float,
    pub fdy: libc::c_float,
    pub direction: libc::c_float,
    pub sy: libc::c_float,
    pub ey: libc::c_float,
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
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbtt_bakedchar {
    pub x0: libc::c_ushort,
    pub y0: libc::c_ushort,
    pub x1: libc::c_ushort,
    pub y1: libc::c_ushort,
    pub xoff: libc::c_float,
    pub yoff: libc::c_float,
    pub xadvance: libc::c_float,
}
// height of font in pixels
// bitmap to be filled in
// characters to bake
// you allocate this, it's num_chars long
// if return is positive, the first unused row of the bitmap
// if return is negative, returns the negative of the number of characters that fit
// if return is 0, no characters fit and no rows were used
// This uses a very crappy packing.
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbtt_aligned_quad {
    pub x0: libc::c_float,
    pub y0: libc::c_float,
    pub s0: libc::c_float,
    pub t0: libc::c_float,
    pub x1: libc::c_float,
    pub y1: libc::c_float,
    pub s1: libc::c_float,
    pub t1: libc::c_float,
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
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbtt_packedchar {
    pub x0: libc::c_ushort,
    pub y0: libc::c_ushort,
    pub x1: libc::c_ushort,
    pub y1: libc::c_ushort,
    pub xoff: libc::c_float,
    pub yoff: libc::c_float,
    pub xadvance: libc::c_float,
    pub xoff2: libc::c_float,
    pub yoff2: libc::c_float,
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
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbtt_pack_context {
    pub user_allocator_context: *mut libc::c_void,
    pub pack_info: *mut libc::c_void,
    pub width: libc::c_int,
    pub height: libc::c_int,
    pub stride_in_bytes: libc::c_int,
    pub padding: libc::c_int,
    pub h_oversample: libc::c_uint,
    pub v_oversample: libc::c_uint,
    pub pixels: *mut libc::c_uchar,
    pub nodes: *mut libc::c_void,
}
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbrp_rect {
    pub x: stbrp_coord,
    pub y: stbrp_coord,
    pub id: libc::c_int,
    pub w: libc::c_int,
    pub h: libc::c_int,
    pub was_packed: libc::c_int,
}
// ////////////////////////////////////////////////////////////////////////////
//
// rectangle packing replacement routines if you don't have stb_rect_pack.h
//
pub type stbrp_coord = libc::c_int;
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbrp_node {
    pub x: libc::c_uchar,
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
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbrp_context {
    pub width: libc::c_int,
    pub height: libc::c_int,
    pub x: libc::c_int,
    pub y: libc::c_int,
    pub bottom_y: libc::c_int,
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
#[derive ( Copy , Clone )]
#[repr(C)]
pub struct stbtt_pack_range {
    pub font_size: libc::c_float,
    pub first_unicode_codepoint_in_range: libc::c_int,
    pub array_of_unicode_codepoints: *mut libc::c_int,
    pub num_chars: libc::c_int,
    pub chardata_for_range: *mut stbtt_packedchar,
    pub h_oversample: libc::c_uchar,
    pub v_oversample: libc::c_uchar,
}

pub type unnamed = libc::c_uint;

pub unsafe extern "C" fn fonsCreateInternal(mut params: *mut FONSparams)
 -> *mut FONScontext {
    let mut current_block: u64;
    let mut stash: *mut FONScontext = 0 as *mut FONScontext;
    stash =
        malloc(::std::mem::size_of::<FONScontext>() as libc::c_ulong) as
            *mut FONScontext;
    if !stash.is_null() {
        memset(stash as *mut libc::c_void, 0i32,
               ::std::mem::size_of::<FONScontext>() as libc::c_ulong);
        (*stash).params = *params;
        (*stash).scratch =
            malloc(96000i32 as libc::c_ulong) as *mut libc::c_uchar;
        if !(*stash).scratch.is_null() {
            // Initialize implementation library
            if !(0 == fons__tt_init(stash)) {
                if (*stash).params.renderCreate.is_some() {
                    if (*stash).params.renderCreate.expect("non-null function pointer")((*stash).params.userPtr,
                                                                                        (*stash).params.width,
                                                                                        (*stash).params.height)
                           == 0i32 {
                        current_block = 2144346072430065838;
                    } else { current_block = 11812396948646013369; }
                } else { current_block = 11812396948646013369; }
                match current_block {
                    2144346072430065838 => { }
                    _ => {
                        (*stash).atlas =
                            fons__allocAtlas((*stash).params.width,
                                             (*stash).params.height, 256i32);
                        if !(*stash).atlas.is_null() {
                            (*stash).fonts =
                                malloc((::std::mem::size_of::<*mut FONSfont>()
                                            as
                                            libc::c_ulong).wrapping_mul(4i32
                                                                            as
                                                                            libc::c_ulong))
                                    as *mut *mut FONSfont;
                            if !(*stash).fonts.is_null() {
                                memset((*stash).fonts as *mut libc::c_void,
                                       0i32,
                                       (::std::mem::size_of::<*mut FONSfont>()
                                            as
                                            libc::c_ulong).wrapping_mul(4i32
                                                                            as
                                                                            libc::c_ulong));
                                (*stash).cfonts = 4i32;
                                (*stash).nfonts = 0i32;
                                (*stash).itw =
                                    1.0f32 /
                                        (*stash).params.width as
                                            libc::c_float;
                                (*stash).ith =
                                    1.0f32 /
                                        (*stash).params.height as
                                            libc::c_float;
                                (*stash).texData =
                                    malloc(((*stash).params.width *
                                                (*stash).params.height) as
                                               libc::c_ulong) as
                                        *mut libc::c_uchar;
                                if !(*stash).texData.is_null() {
                                    memset((*stash).texData as
                                               *mut libc::c_void, 0i32,
                                           ((*stash).params.width *
                                                (*stash).params.height) as
                                               libc::c_ulong);
                                    (*stash).dirtyRect[0usize] =
                                        (*stash).params.width;
                                    (*stash).dirtyRect[1usize] =
                                        (*stash).params.height;
                                    (*stash).dirtyRect[2usize] = 0i32;
                                    (*stash).dirtyRect[3usize] = 0i32;
                                    fons__addWhiteRect(stash, 2i32, 2i32);
                                    fonsPushState(stash);
                                    fonsClearState(stash);
                                    return stash
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    fonsDeleteInternal(stash);
    return 0 as *mut FONScontext;
}

pub unsafe extern "C" fn fonsDeleteInternal(mut stash: *mut FONScontext) {
    let mut i: libc::c_int = 0;
    if stash.is_null() { return }
    if (*stash).params.renderDelete.is_some() {
        (*stash).params.renderDelete.expect("non-null function pointer")((*stash).params.userPtr);
    }
    i = 0i32;
    while i < (*stash).nfonts {
        fons__freeFont(*(*stash).fonts.offset(i as isize));
        i += 1
    }
    if !(*stash).atlas.is_null() { fons__deleteAtlas((*stash).atlas); }
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
    fons__tt_done(stash);
}

pub unsafe extern "C" fn fons__tt_done(mut context: *mut FONScontext)
 -> libc::c_int {
    return 1i32;
}
// Atlas based on Skyline Bin Packer by Jukka Jylänki

pub unsafe extern "C" fn fons__deleteAtlas(mut atlas: *mut FONSatlas) {
    if atlas.is_null() { return }
    if !(*atlas).nodes.is_null() {
        free((*atlas).nodes as *mut libc::c_void);
    }
    free(atlas as *mut libc::c_void);
}

pub unsafe extern "C" fn fons__freeFont(mut font: *mut FONSfont) {
    if font.is_null() { return }
    if !(*font).glyphs.is_null() {
        free((*font).glyphs as *mut libc::c_void);
    }
    if 0 != (*font).freeData as libc::c_int && !(*font).data.is_null() {
        free((*font).data as *mut libc::c_void);
    }
    free(font as *mut libc::c_void);
}

pub unsafe extern "C" fn fonsClearState(mut stash: *mut FONScontext) {
    let mut state: *mut FONSstate = fons__getState(stash);
    (*state).size = 12.0f32;
    (*state).color = 0xffffffffu32;
    (*state).font = 0i32;
    (*state).blur = 0i32 as libc::c_float;
    (*state).spacing = 0i32 as libc::c_float;
    (*state).align =
        FONS_ALIGN_LEFT as libc::c_int | FONS_ALIGN_BASELINE as libc::c_int;
}

pub unsafe extern "C" fn fons__getState(mut stash: *mut FONScontext)
 -> *mut FONSstate {
    return &mut *(*stash).states.as_mut_ptr().offset(((*stash).nstates - 1i32)
                                                         as isize) as
               *mut FONSstate;
}
// State handling

pub unsafe extern "C" fn fonsPushState(mut stash: *mut FONScontext) {
    if (*stash).nstates >= 20i32 {
        if (*stash).handleError.is_some() {
            (*stash).handleError.expect("non-null function pointer")((*stash).errorUptr,
                                                                     FONS_STATES_OVERFLOW
                                                                         as
                                                                         libc::c_int,
                                                                     0i32);
        }
        return
    }
    if (*stash).nstates > 0i32 {
        memcpy(&mut *(*stash).states.as_mut_ptr().offset((*stash).nstates as
                                                             isize) as
                   *mut FONSstate as *mut libc::c_void,
               &mut *(*stash).states.as_mut_ptr().offset(((*stash).nstates -
                                                              1i32) as isize)
                   as *mut FONSstate as *const libc::c_void,
               ::std::mem::size_of::<FONSstate>() as libc::c_ulong);
    }
    (*stash).nstates += 1;
}

pub unsafe extern "C" fn fons__addWhiteRect(mut stash: *mut FONScontext,
                                            mut w: libc::c_int,
                                            mut h: libc::c_int) {
    let mut x: libc::c_int = 0;
    let mut y: libc::c_int = 0;
    let mut gx: libc::c_int = 0;
    let mut gy: libc::c_int = 0;
    let mut dst: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    if fons__atlasAddRect((*stash).atlas, w, h, &mut gx, &mut gy) == 0i32 {
        return
    }
    dst =
        &mut *(*stash).texData.offset((gx + gy * (*stash).params.width) as
                                          isize) as *mut libc::c_uchar;
    y = 0i32;
    while y < h {
        x = 0i32;
        while x < w {
            *dst.offset(x as isize) = 0xffi32 as libc::c_uchar;
            x += 1
        }
        dst = dst.offset((*stash).params.width as isize);
        y += 1
    }
    (*stash).dirtyRect[0usize] = fons__mini((*stash).dirtyRect[0usize], gx);
    (*stash).dirtyRect[1usize] = fons__mini((*stash).dirtyRect[1usize], gy);
    (*stash).dirtyRect[2usize] =
        fons__maxi((*stash).dirtyRect[2usize], gx + w);
    (*stash).dirtyRect[3usize] =
        fons__maxi((*stash).dirtyRect[3usize], gy + h);
}

pub unsafe extern "C" fn fons__maxi(mut a: libc::c_int, mut b: libc::c_int)
 -> libc::c_int {
    return if a > b { a } else { b };
}

pub unsafe extern "C" fn fons__mini(mut a: libc::c_int, mut b: libc::c_int)
 -> libc::c_int {
    return if a < b { a } else { b };
}

pub unsafe extern "C" fn fons__atlasAddRect(mut atlas: *mut FONSatlas,
                                            mut rw: libc::c_int,
                                            mut rh: libc::c_int,
                                            mut rx: *mut libc::c_int,
                                            mut ry: *mut libc::c_int)
 -> libc::c_int {
    let mut besth: libc::c_int = (*atlas).height;
    let mut bestw: libc::c_int = (*atlas).width;
    let mut besti: libc::c_int = -1i32;
    let mut bestx: libc::c_int = -1i32;
    let mut besty: libc::c_int = -1i32;
    let mut i: libc::c_int = 0;
    i = 0i32;
    while i < (*atlas).nnodes {
        let mut y: libc::c_int = fons__atlasRectFits(atlas, i, rw, rh);
        if y != -1i32 {
            if y + rh < besth ||
                   y + rh == besth &&
                       ((*(*atlas).nodes.offset(i as isize)).width as
                            libc::c_int) < bestw {
                besti = i;
                bestw =
                    (*(*atlas).nodes.offset(i as isize)).width as libc::c_int;
                besth = y + rh;
                bestx = (*(*atlas).nodes.offset(i as isize)).x as libc::c_int;
                besty = y
            }
        }
        i += 1
    }
    if besti == -1i32 { return 0i32 }
    if fons__atlasAddSkylineLevel(atlas, besti, bestx, besty, rw, rh) == 0i32
       {
        return 0i32
    }
    *rx = bestx;
    *ry = besty;
    return 1i32;
}

pub unsafe extern "C" fn fons__atlasAddSkylineLevel(mut atlas: *mut FONSatlas,
                                                    mut idx: libc::c_int,
                                                    mut x: libc::c_int,
                                                    mut y: libc::c_int,
                                                    mut w: libc::c_int,
                                                    mut h: libc::c_int)
 -> libc::c_int {
    let mut i: libc::c_int = 0;
    if fons__atlasInsertNode(atlas, idx, x, y + h, w) == 0i32 { return 0i32 }
    i = idx + 1i32;
    while i < (*atlas).nnodes {
        if !(((*(*atlas).nodes.offset(i as isize)).x as libc::c_int) <
                 (*(*atlas).nodes.offset((i - 1i32) as isize)).x as
                     libc::c_int +
                     (*(*atlas).nodes.offset((i - 1i32) as isize)).width as
                         libc::c_int) {
            break ;
        }
        let mut shrink: libc::c_int =
            (*(*atlas).nodes.offset((i - 1i32) as isize)).x as libc::c_int +
                (*(*atlas).nodes.offset((i - 1i32) as isize)).width as
                    libc::c_int -
                (*(*atlas).nodes.offset(i as isize)).x as libc::c_int;
        let ref mut fresh0 = (*(*atlas).nodes.offset(i as isize)).x;
        *fresh0 =
            (*fresh0 as libc::c_int + shrink as libc::c_short as libc::c_int)
                as libc::c_short;
        let ref mut fresh1 = (*(*atlas).nodes.offset(i as isize)).width;
        *fresh1 =
            (*fresh1 as libc::c_int - shrink as libc::c_short as libc::c_int)
                as libc::c_short;
        if !((*(*atlas).nodes.offset(i as isize)).width as libc::c_int <=
                 0i32) {
            break ;
        }
        fons__atlasRemoveNode(atlas, i);
        i -= 1;
        i += 1
    }
    i = 0i32;
    while i < (*atlas).nnodes - 1i32 {
        if (*(*atlas).nodes.offset(i as isize)).y as libc::c_int ==
               (*(*atlas).nodes.offset((i + 1i32) as isize)).y as libc::c_int
           {
            let ref mut fresh2 = (*(*atlas).nodes.offset(i as isize)).width;
            *fresh2 =
                (*fresh2 as libc::c_int +
                     (*(*atlas).nodes.offset((i + 1i32) as isize)).width as
                         libc::c_int) as libc::c_short;
            fons__atlasRemoveNode(atlas, i + 1i32);
            i -= 1
        }
        i += 1
    }
    return 1i32;
}

pub unsafe extern "C" fn fons__atlasRemoveNode(mut atlas: *mut FONSatlas,
                                               mut idx: libc::c_int) {
    let mut i: libc::c_int = 0;
    if (*atlas).nnodes == 0i32 { return }
    i = idx;
    while i < (*atlas).nnodes - 1i32 {
        *(*atlas).nodes.offset(i as isize) =
            *(*atlas).nodes.offset((i + 1i32) as isize);
        i += 1
    }
    (*atlas).nnodes -= 1;
}

pub unsafe extern "C" fn fons__atlasInsertNode(mut atlas: *mut FONSatlas,
                                               mut idx: libc::c_int,
                                               mut x: libc::c_int,
                                               mut y: libc::c_int,
                                               mut w: libc::c_int)
 -> libc::c_int {
    let mut i: libc::c_int = 0;
    if (*atlas).nnodes + 1i32 > (*atlas).cnodes {
        (*atlas).cnodes =
            if (*atlas).cnodes == 0i32 {
                8i32
            } else { (*atlas).cnodes * 2i32 };
        (*atlas).nodes =
            realloc((*atlas).nodes as *mut libc::c_void,
                    (::std::mem::size_of::<FONSatlasNode>() as
                         libc::c_ulong).wrapping_mul((*atlas).cnodes as
                                                         libc::c_ulong)) as
                *mut FONSatlasNode;
        if (*atlas).nodes.is_null() { return 0i32 }
    }
    i = (*atlas).nnodes;
    while i > idx {
        *(*atlas).nodes.offset(i as isize) =
            *(*atlas).nodes.offset((i - 1i32) as isize);
        i -= 1
    }
    (*(*atlas).nodes.offset(idx as isize)).x = x as libc::c_short;
    (*(*atlas).nodes.offset(idx as isize)).y = y as libc::c_short;
    (*(*atlas).nodes.offset(idx as isize)).width = w as libc::c_short;
    (*atlas).nnodes += 1;
    return 1i32;
}

pub unsafe extern "C" fn fons__atlasRectFits(mut atlas: *mut FONSatlas,
                                             mut i: libc::c_int,
                                             mut w: libc::c_int,
                                             mut h: libc::c_int)
 -> libc::c_int {
    // Checks if there is enough space at the location of skyline span 'i',
	// and return the max height of all skyline spans under that at that location,
	// (think tetris block being dropped at that position). Or -1 if no space found.
    let mut x: libc::c_int =
        (*(*atlas).nodes.offset(i as isize)).x as libc::c_int;
    let mut y: libc::c_int =
        (*(*atlas).nodes.offset(i as isize)).y as libc::c_int;
    let mut spaceLeft: libc::c_int = 0;
    if x + w > (*atlas).width { return -1i32 }
    spaceLeft = w;
    while spaceLeft > 0i32 {
        if i == (*atlas).nnodes { return -1i32 }
        y =
            fons__maxi(y,
                       (*(*atlas).nodes.offset(i as isize)).y as libc::c_int);
        if y + h > (*atlas).height { return -1i32 }
        spaceLeft -=
            (*(*atlas).nodes.offset(i as isize)).width as libc::c_int;
        i += 1
    }
    return y;
}

pub unsafe extern "C" fn fons__allocAtlas(mut w: libc::c_int,
                                          mut h: libc::c_int,
                                          mut nnodes: libc::c_int)
 -> *mut FONSatlas {
    let mut atlas: *mut FONSatlas = 0 as *mut FONSatlas;
    atlas =
        malloc(::std::mem::size_of::<FONSatlas>() as libc::c_ulong) as
            *mut FONSatlas;
    if !atlas.is_null() {
        memset(atlas as *mut libc::c_void, 0i32,
               ::std::mem::size_of::<FONSatlas>() as libc::c_ulong);
        (*atlas).width = w;
        (*atlas).height = h;
        (*atlas).nodes =
            malloc((::std::mem::size_of::<FONSatlasNode>() as
                        libc::c_ulong).wrapping_mul(nnodes as libc::c_ulong))
                as *mut FONSatlasNode;
        if !(*atlas).nodes.is_null() {
            memset((*atlas).nodes as *mut libc::c_void, 0i32,
                   (::std::mem::size_of::<FONSatlasNode>() as
                        libc::c_ulong).wrapping_mul(nnodes as libc::c_ulong));
            (*atlas).nnodes = 0i32;
            (*atlas).cnodes = nnodes;
            (*(*atlas).nodes.offset(0isize)).x = 0i32 as libc::c_short;
            (*(*atlas).nodes.offset(0isize)).y = 0i32 as libc::c_short;
            (*(*atlas).nodes.offset(0isize)).width = w as libc::c_short;
            (*atlas).nnodes += 1;
            return atlas
        }
    }
    if !atlas.is_null() { fons__deleteAtlas(atlas); }
    return 0 as *mut FONSatlas;
}

pub unsafe extern "C" fn fons__tt_init(mut context: *mut FONScontext)
 -> libc::c_int {
    return 1i32;
}


pub unsafe extern "C" fn fons__flush(mut stash: *mut FONScontext) {
    if (*stash).dirtyRect[0usize] < (*stash).dirtyRect[2usize] &&
           (*stash).dirtyRect[1usize] < (*stash).dirtyRect[3usize] {
        if (*stash).params.renderUpdate.is_some() {
            (*stash).params.renderUpdate.expect("non-null function pointer")((*stash).params.userPtr,
                                                                             (*stash).dirtyRect.as_mut_ptr(),
                                                                             (*stash).texData);
        }
        (*stash).dirtyRect[0usize] = (*stash).params.width;
        (*stash).dirtyRect[1usize] = (*stash).params.height;
        (*stash).dirtyRect[2usize] = 0i32;
        (*stash).dirtyRect[3usize] = 0i32
    }
    if (*stash).nverts > 0i32 {
        if (*stash).params.renderDraw.is_some() {
            (*stash).params.renderDraw.expect("non-null function pointer")((*stash).params.userPtr,
                                                                           (*stash).verts.as_mut_ptr(),
                                                                           (*stash).tcoords.as_mut_ptr(),
                                                                           (*stash).colors.as_mut_ptr(),
                                                                           (*stash).nverts);
        }
        (*stash).nverts = 0i32
    };
}
// Resets the whole stash.

pub unsafe extern "C" fn fonsResetAtlas(mut stash: *mut FONScontext,
                                        mut width: libc::c_int,
                                        mut height: libc::c_int)
 -> libc::c_int {
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    if stash.is_null() { return 0i32 }
    fons__flush(stash);
    if (*stash).params.renderResize.is_some() {
        if (*stash).params.renderResize.expect("non-null function pointer")((*stash).params.userPtr,
                                                                            width,
                                                                            height)
               == 0i32 {
            return 0i32
        }
    }
    fons__atlasReset((*stash).atlas, width, height);
    (*stash).texData =
        realloc((*stash).texData as *mut libc::c_void,
                (width * height) as libc::c_ulong) as *mut libc::c_uchar;
    if (*stash).texData.is_null() { return 0i32 }
    memset((*stash).texData as *mut libc::c_void, 0i32,
           (width * height) as libc::c_ulong);
    (*stash).dirtyRect[0usize] = width;
    (*stash).dirtyRect[1usize] = height;
    (*stash).dirtyRect[2usize] = 0i32;
    (*stash).dirtyRect[3usize] = 0i32;
    i = 0i32;
    while i < (*stash).nfonts {
        let mut font: *mut FONSfont = *(*stash).fonts.offset(i as isize);
        (*font).nglyphs = 0i32;
        j = 0i32;
        while j < 256i32 { (*font).lut[j as usize] = -1i32; j += 1 }
        i += 1
    }
    (*stash).params.width = width;
    (*stash).params.height = height;
    (*stash).itw = 1.0f32 / (*stash).params.width as libc::c_float;
    (*stash).ith = 1.0f32 / (*stash).params.height as libc::c_float;
    fons__addWhiteRect(stash, 2i32, 2i32);
    return 1i32;
}

pub unsafe extern "C" fn fons__atlasReset(mut atlas: *mut FONSatlas,
                                          mut w: libc::c_int,
                                          mut h: libc::c_int) {
    (*atlas).width = w;
    (*atlas).height = h;
    (*atlas).nnodes = 0i32;
    (*(*atlas).nodes.offset(0isize)).x = 0i32 as libc::c_short;
    (*(*atlas).nodes.offset(0isize)).y = 0i32 as libc::c_short;
    (*(*atlas).nodes.offset(0isize)).width = w as libc::c_short;
    (*atlas).nnodes += 1;
}
// Add fonts

pub unsafe extern "C" fn fonsAddFont(mut stash: *mut FONScontext,
                                     mut name: *const libc::c_char,
                                     mut path: *const libc::c_char)
 -> libc::c_int {
    let mut fp: *mut FILE = 0 as *mut FILE;
    let mut dataSize: libc::c_int = 0i32;
    let mut readed: size_t = 0;
    let mut data: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    fp = fopen(path, b"rb\x00" as *const u8 as *const libc::c_char);
    if !fp.is_null() {
        fseek(fp, 0i32 as libc::c_long, 2i32);
        dataSize = ftell(fp) as libc::c_int;
        fseek(fp, 0i32 as libc::c_long, 0i32);
        data = malloc(dataSize as libc::c_ulong) as *mut libc::c_uchar;
        if !data.is_null() {
            readed =
                fread(data as *mut libc::c_void, 1i32 as libc::c_ulong,
                      dataSize as libc::c_ulong, fp);
            fclose(fp);
            fp = 0 as *mut FILE;
            if !(readed != dataSize as libc::c_ulong) {
                return fonsAddFontMem(stash, name, data, dataSize, 1i32)
            }
        }
    }
    if !data.is_null() { free(data as *mut libc::c_void); }
    if !fp.is_null() { fclose(fp); }
    return -1i32;
}

pub unsafe extern "C" fn fonsAddFontMem(mut stash: *mut FONScontext,
                                        mut name: *const libc::c_char,
                                        mut data: *mut libc::c_uchar,
                                        mut dataSize: libc::c_int,
                                        mut freeData: libc::c_int)
 -> libc::c_int {
    let mut i: libc::c_int = 0;
    let mut ascent: libc::c_int = 0;
    let mut descent: libc::c_int = 0;
    let mut fh: libc::c_int = 0;
    let mut lineGap: libc::c_int = 0;
    let mut font: *mut FONSfont = 0 as *mut FONSfont;
    let mut idx: libc::c_int = fons__allocFont(stash);
    if idx == -1i32 { return -1i32 }
    font = *(*stash).fonts.offset(idx as isize);
    strncpy((*font).name.as_mut_ptr(), name,
            ::std::mem::size_of::<[libc::c_char; 64]>() as libc::c_ulong);
    (*font).name[(::std::mem::size_of::<[libc::c_char; 64]>() as
                      libc::c_ulong).wrapping_sub(1i32 as libc::c_ulong) as
                     usize] = '\u{0}' as i32 as libc::c_char;
    i = 0i32;
    while i < 256i32 { (*font).lut[i as usize] = -1i32; i += 1 }
    (*font).dataSize = dataSize;
    (*font).data = data;
    (*font).freeData = freeData as libc::c_uchar;
    (*stash).nscratch = 0i32;
    if 0 == fons__tt_loadFont(stash, &mut (*font).font, data, dataSize) {
        fons__freeFont(font);
        (*stash).nfonts -= 1;
        return -1i32
    } else {
        fons__tt_getFontVMetrics(&mut (*font).font, &mut ascent, &mut descent,
                                 &mut lineGap);
        fh = ascent - descent;
        (*font).ascender = ascent as libc::c_float / fh as libc::c_float;
        (*font).descender = descent as libc::c_float / fh as libc::c_float;
        (*font).lineh = (fh + lineGap) as libc::c_float / fh as libc::c_float;
        return idx
    };
}

pub unsafe extern "C" fn fons__allocFont(mut stash: *mut FONScontext)
 -> libc::c_int {
    let mut font: *mut FONSfont = 0 as *mut FONSfont;
    if (*stash).nfonts + 1i32 > (*stash).cfonts {
        (*stash).cfonts =
            if (*stash).cfonts == 0i32 {
                8i32
            } else { (*stash).cfonts * 2i32 };
        (*stash).fonts =
            realloc((*stash).fonts as *mut libc::c_void,
                    (::std::mem::size_of::<*mut FONSfont>() as
                         libc::c_ulong).wrapping_mul((*stash).cfonts as
                                                         libc::c_ulong)) as
                *mut *mut FONSfont;
        if (*stash).fonts.is_null() { return -1i32 }
    }
    font =
        malloc(::std::mem::size_of::<FONSfont>() as libc::c_ulong) as
            *mut FONSfont;
    if !font.is_null() {
        memset(font as *mut libc::c_void, 0i32,
               ::std::mem::size_of::<FONSfont>() as libc::c_ulong);
        (*font).glyphs =
            malloc((::std::mem::size_of::<FONSglyph>() as
                        libc::c_ulong).wrapping_mul(256i32 as libc::c_ulong))
                as *mut FONSglyph;
        if !(*font).glyphs.is_null() {
            (*font).cglyphs = 256i32;
            (*font).nglyphs = 0i32;
            let fresh3 = (*stash).nfonts;
            (*stash).nfonts = (*stash).nfonts + 1;
            let ref mut fresh4 = *(*stash).fonts.offset(fresh3 as isize);
            *fresh4 = font;
            return (*stash).nfonts - 1i32
        }
    }
    fons__freeFont(font);
    return -1i32;
}

pub unsafe extern "C" fn fons__tt_getFontVMetrics(mut font:
                                                      *mut FONSttFontImpl,
                                                  mut ascent:
                                                      *mut libc::c_int,
                                                  mut descent:
                                                      *mut libc::c_int,
                                                  mut lineGap:
                                                      *mut libc::c_int) {
    stbtt_GetFontVMetrics(&mut (*font).font, ascent, descent, lineGap);
}
// computes a scale factor to produce a font whose EM size is mapped to
// 'pixels' tall. This is probably what traditional APIs compute, but
// I'm not positive.

pub unsafe extern "C" fn stbtt_GetFontVMetrics(mut info:
                                                   *const stbtt_fontinfo,
                                               mut ascent: *mut libc::c_int,
                                               mut descent: *mut libc::c_int,
                                               mut lineGap:
                                                   *mut libc::c_int) {
    if !ascent.is_null() {
        *ascent =
            ttSHORT((*info).data.offset((*info).hhea as isize).offset(4isize))
                as libc::c_int
    }
    if !descent.is_null() {
        *descent =
            ttSHORT((*info).data.offset((*info).hhea as isize).offset(6isize))
                as libc::c_int
    }
    if !lineGap.is_null() {
        *lineGap =
            ttSHORT((*info).data.offset((*info).hhea as isize).offset(8isize))
                as libc::c_int
    };
}
unsafe extern "C" fn ttSHORT(mut p: *const stbtt_uint8) -> stbtt_int16 {
    return (*p.offset(0isize) as libc::c_int * 256i32 +
                *p.offset(1isize) as libc::c_int) as stbtt_int16;
}

pub unsafe extern "C" fn fons__tt_loadFont(mut context: *mut FONScontext,
                                           mut font: *mut FONSttFontImpl,
                                           mut data: *mut libc::c_uchar,
                                           mut dataSize: libc::c_int)
 -> libc::c_int {
    let mut stbError: libc::c_int = 0;
    (*font).font.userdata = context as *mut libc::c_void;
    stbError = stbtt_InitFont(&mut (*font).font, data, 0i32);
    return stbError;
}

pub unsafe extern "C" fn stbtt_InitFont(mut info: *mut stbtt_fontinfo,
                                        mut data2: *const libc::c_uchar,
                                        mut fontstart: libc::c_int)
 -> libc::c_int {
    let mut data: *mut stbtt_uint8 = data2 as *mut stbtt_uint8;
    let mut cmap: stbtt_uint32 = 0;
    let mut t: stbtt_uint32 = 0;
    let mut i: stbtt_int32 = 0;
    let mut numTables: stbtt_int32 = 0;
    (*info).data = data;
    (*info).fontstart = fontstart;
    cmap =
        stbtt__find_table(data, fontstart as stbtt_uint32,
                          b"cmap\x00" as *const u8 as *const libc::c_char);
    (*info).loca =
        stbtt__find_table(data, fontstart as stbtt_uint32,
                          b"loca\x00" as *const u8 as *const libc::c_char) as
            libc::c_int;
    (*info).head =
        stbtt__find_table(data, fontstart as stbtt_uint32,
                          b"head\x00" as *const u8 as *const libc::c_char) as
            libc::c_int;
    (*info).glyf =
        stbtt__find_table(data, fontstart as stbtt_uint32,
                          b"glyf\x00" as *const u8 as *const libc::c_char) as
            libc::c_int;
    (*info).hhea =
        stbtt__find_table(data, fontstart as stbtt_uint32,
                          b"hhea\x00" as *const u8 as *const libc::c_char) as
            libc::c_int;
    (*info).hmtx =
        stbtt__find_table(data, fontstart as stbtt_uint32,
                          b"hmtx\x00" as *const u8 as *const libc::c_char) as
            libc::c_int;
    (*info).kern =
        stbtt__find_table(data, fontstart as stbtt_uint32,
                          b"kern\x00" as *const u8 as *const libc::c_char) as
            libc::c_int;
    if 0 == cmap || 0 == (*info).loca || 0 == (*info).head ||
           0 == (*info).glyf || 0 == (*info).hhea || 0 == (*info).hmtx {
        return 0i32
    }
    t =
        stbtt__find_table(data, fontstart as stbtt_uint32,
                          b"maxp\x00" as *const u8 as *const libc::c_char);
    if 0 != t {
        (*info).numGlyphs =
            ttUSHORT(data.offset(t as isize).offset(4isize)) as libc::c_int
    } else { (*info).numGlyphs = 0xffffi32 }
    numTables =
        ttUSHORT(data.offset(cmap as isize).offset(2isize)) as stbtt_int32;
    (*info).index_map = 0i32;
    i = 0i32;
    while i < numTables {
        let mut encoding_record: stbtt_uint32 =
            cmap.wrapping_add(4i32 as
                                  libc::c_uint).wrapping_add((8i32 * i) as
                                                                 libc::c_uint);
        match ttUSHORT(data.offset(encoding_record as isize)) as libc::c_int {
            3 => {
                match ttUSHORT(data.offset(encoding_record as
                                               isize).offset(2isize)) as
                          libc::c_int {
                    1 | 10 => {
                        (*info).index_map =
                            cmap.wrapping_add(ttULONG(data.offset(encoding_record
                                                                      as
                                                                      isize).offset(4isize)))
                                as libc::c_int
                    }
                    _ => { }
                }
            }
            0 => {
                (*info).index_map =
                    cmap.wrapping_add(ttULONG(data.offset(encoding_record as
                                                              isize).offset(4isize)))
                        as libc::c_int
            }
            _ => { }
        }
        i += 1
    }
    if (*info).index_map == 0i32 { return 0i32 }
    (*info).indexToLocFormat =
        ttUSHORT(data.offset((*info).head as isize).offset(50isize)) as
            libc::c_int;
    return 1i32;
}
// ////////////////////////////////////////////////////////////////////////
//
// accessors to parse data from file
//
// on platforms that don't allow misaligned reads, if we want to allow
// truetype fonts that aren't padded to alignment, define ALLOW_UNALIGNED_TRUETYPE
unsafe extern "C" fn ttUSHORT(mut p: *const stbtt_uint8) -> stbtt_uint16 {
    return (*p.offset(0isize) as libc::c_int * 256i32 +
                *p.offset(1isize) as libc::c_int) as stbtt_uint16;
}
unsafe extern "C" fn ttULONG(mut p: *const stbtt_uint8) -> stbtt_uint32 {
    return (((*p.offset(0isize) as libc::c_int) << 24i32) +
                ((*p.offset(1isize) as libc::c_int) << 16i32) +
                ((*p.offset(2isize) as libc::c_int) << 8i32) +
                *p.offset(3isize) as libc::c_int) as stbtt_uint32;
}
// @OPTIMIZE: binary search
unsafe extern "C" fn stbtt__find_table(mut data: *mut stbtt_uint8,
                                       mut fontstart: stbtt_uint32,
                                       mut tag: *const libc::c_char)
 -> stbtt_uint32 {
    let mut num_tables: stbtt_int32 =
        ttUSHORT(data.offset(fontstart as isize).offset(4isize)) as
            stbtt_int32;
    let mut tabledir: stbtt_uint32 =
        fontstart.wrapping_add(12i32 as libc::c_uint);
    let mut i: stbtt_int32 = 0;
    i = 0i32;
    while i < num_tables {
        let mut loc: stbtt_uint32 =
            tabledir.wrapping_add((16i32 * i) as libc::c_uint);
        if *data.offset(loc as isize).offset(0isize).offset(0isize) as
               libc::c_int == *tag.offset(0isize) as libc::c_int &&
               *data.offset(loc as isize).offset(0isize).offset(1isize) as
                   libc::c_int == *tag.offset(1isize) as libc::c_int &&
               *data.offset(loc as isize).offset(0isize).offset(2isize) as
                   libc::c_int == *tag.offset(2isize) as libc::c_int &&
               *data.offset(loc as isize).offset(0isize).offset(3isize) as
                   libc::c_int == *tag.offset(3isize) as libc::c_int {
            return ttULONG(data.offset(loc as isize).offset(8isize))
        }
        i += 1
    }
    return 0i32 as stbtt_uint32;
}

pub unsafe extern "C" fn fonsGetFontByName(mut s: *mut FONScontext,
                                           mut name: *const libc::c_char)
 -> libc::c_int {
    let mut i: libc::c_int = 0;
    i = 0i32;
    while i < (*s).nfonts {
        if strcmp((**(*s).fonts.offset(i as isize)).name.as_mut_ptr(), name)
               == 0i32 {
            return i
        }
        i += 1
    }
    return -1i32;
}

// State setting

pub unsafe extern "C" fn fonsSetSize(mut stash: *mut FONScontext,
                                     mut size: libc::c_float) {
    (*fons__getState(stash)).size = size;
}
pub unsafe extern "C" fn fonsSetSpacing(mut stash: *mut FONScontext,
                                        mut spacing: libc::c_float) {
    (*fons__getState(stash)).spacing = spacing;
}

pub unsafe extern "C" fn fonsSetBlur(mut stash: *mut FONScontext,
                                     mut blur: libc::c_float) {
    (*fons__getState(stash)).blur = blur;
}

pub unsafe extern "C" fn fonsSetAlign(mut stash: *mut FONScontext,
                                      mut align: libc::c_int) {
    (*fons__getState(stash)).align = align;
}

pub unsafe extern "C" fn fonsSetFont(mut stash: *mut FONScontext,
                                     mut font: libc::c_int) {
    (*fons__getState(stash)).font = font;
}
// Draw text


pub unsafe extern "C" fn fons__getQuad(mut stash: *mut FONScontext,
                                       mut font: *mut FONSfont,
                                       mut prevGlyphIndex: libc::c_int,
                                       mut glyph: *mut FONSglyph,
                                       mut scale: libc::c_float,
                                       mut spacing: libc::c_float,
                                       mut x: *mut libc::c_float,
                                       mut y: *mut libc::c_float,
                                       mut q: *mut FONSquad) {
    let mut rx: libc::c_float = 0.;
    let mut ry: libc::c_float = 0.;
    let mut xoff: libc::c_float = 0.;
    let mut yoff: libc::c_float = 0.;
    let mut x0: libc::c_float = 0.;
    let mut y0: libc::c_float = 0.;
    let mut x1: libc::c_float = 0.;
    let mut y1: libc::c_float = 0.;
    if prevGlyphIndex != -1i32 {
        let mut adv: libc::c_float =
            fons__tt_getGlyphKernAdvance(&mut (*font).font, prevGlyphIndex,
                                         (*glyph).index) as libc::c_float *
                scale;
        *x += (adv + spacing + 0.5f32) as libc::c_int as libc::c_float
    }
    xoff =
        ((*glyph).xoff as libc::c_int + 1i32) as libc::c_short as
            libc::c_float;
    yoff =
        ((*glyph).yoff as libc::c_int + 1i32) as libc::c_short as
            libc::c_float;
    x0 = ((*glyph).x0 as libc::c_int + 1i32) as libc::c_float;
    y0 = ((*glyph).y0 as libc::c_int + 1i32) as libc::c_float;
    x1 = ((*glyph).x1 as libc::c_int - 1i32) as libc::c_float;
    y1 = ((*glyph).y1 as libc::c_int - 1i32) as libc::c_float;
    if 0 !=
           (*stash).params.flags as libc::c_int &
               FONS_ZERO_TOPLEFT as libc::c_int {
        rx = (*x + xoff) as libc::c_int as libc::c_float;
        ry = (*y + yoff) as libc::c_int as libc::c_float;
        (*q).x0 = rx;
        (*q).y0 = ry;
        (*q).x1 = rx + x1 - x0;
        (*q).y1 = ry + y1 - y0;
        (*q).s0 = x0 * (*stash).itw;
        (*q).t0 = y0 * (*stash).ith;
        (*q).s1 = x1 * (*stash).itw;
        (*q).t1 = y1 * (*stash).ith
    } else {
        rx = (*x + xoff) as libc::c_int as libc::c_float;
        ry = (*y - yoff) as libc::c_int as libc::c_float;
        (*q).x0 = rx;
        (*q).y0 = ry;
        (*q).x1 = rx + x1 - x0;
        (*q).y1 = ry - y1 + y0;
        (*q).s0 = x0 * (*stash).itw;
        (*q).t0 = y0 * (*stash).ith;
        (*q).s1 = x1 * (*stash).itw;
        (*q).t1 = y1 * (*stash).ith
    }
    *x +=
        ((*glyph).xadv as libc::c_int as libc::c_float / 10.0f32 + 0.5f32) as
            libc::c_int as libc::c_float;
}

pub unsafe extern "C" fn fons__tt_getGlyphKernAdvance(mut font:
                                                          *mut FONSttFontImpl,
                                                      mut glyph1: libc::c_int,
                                                      mut glyph2: libc::c_int)
 -> libc::c_int {
    return stbtt_GetGlyphKernAdvance(&mut (*font).font, glyph1, glyph2);
}

pub unsafe extern "C" fn stbtt_GetGlyphKernAdvance(mut info:
                                                       *const stbtt_fontinfo,
                                                   mut glyph1: libc::c_int,
                                                   mut glyph2: libc::c_int)
 -> libc::c_int {
    let mut data: *mut stbtt_uint8 =
        (*info).data.offset((*info).kern as isize);
    let mut needle: stbtt_uint32 = 0;
    let mut straw: stbtt_uint32 = 0;
    let mut l: libc::c_int = 0;
    let mut r: libc::c_int = 0;
    let mut m: libc::c_int = 0;
    if 0 == (*info).kern { return 0i32 }
    if (ttUSHORT(data.offset(2isize)) as libc::c_int) < 1i32 { return 0i32 }
    if ttUSHORT(data.offset(8isize)) as libc::c_int != 1i32 { return 0i32 }
    l = 0i32;
    r = ttUSHORT(data.offset(10isize)) as libc::c_int - 1i32;
    needle = (glyph1 << 16i32 | glyph2) as stbtt_uint32;
    while l <= r {
        m = l + r >> 1i32;
        straw = ttULONG(data.offset(18isize).offset((m * 6i32) as isize));
        if needle < straw {
            r = m - 1i32
        } else if needle > straw {
            l = m + 1i32
        } else {
            return ttSHORT(data.offset(22isize).offset((m * 6i32) as isize))
                       as libc::c_int
        }
    }
    return 0i32;
}
//	fons__blurrows(dst, w, h, dstStride, alpha);
//	fons__blurcols(dst, w, h, dstStride, alpha);

pub unsafe extern "C" fn fons__getGlyph(mut stash: *mut FONScontext,
                                        mut font: *mut FONSfont,
                                        mut codepoint: libc::c_uint,
                                        mut isize: libc::c_short,
                                        mut iblur: libc::c_short,
                                        mut bitmapOption: libc::c_int)
 -> *mut FONSglyph {
    let mut i: libc::c_int = 0;
    let mut g: libc::c_int = 0;
    let mut advance: libc::c_int = 0;
    let mut lsb: libc::c_int = 0;
    let mut x0: libc::c_int = 0;
    let mut y0: libc::c_int = 0;
    let mut x1: libc::c_int = 0;
    let mut y1: libc::c_int = 0;
    let mut gw: libc::c_int = 0;
    let mut gh: libc::c_int = 0;
    let mut gx: libc::c_int = 0;
    let mut gy: libc::c_int = 0;
    let mut x: libc::c_int = 0;
    let mut y: libc::c_int = 0;
    let mut scale: libc::c_float = 0.;
    let mut glyph: *mut FONSglyph = 0 as *mut FONSglyph;
    let mut h: libc::c_uint = 0;
    let mut size: libc::c_float =
        isize as libc::c_int as libc::c_float / 10.0f32;
    let mut pad: libc::c_int = 0;
    let mut added: libc::c_int = 0;
    let mut bdst: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    let mut dst: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    let mut renderFont: *mut FONSfont = font;
    if (isize as libc::c_int) < 2i32 { return 0 as *mut FONSglyph }
    if iblur as libc::c_int > 20i32 { iblur = 20i32 as libc::c_short }
    pad = iblur as libc::c_int + 2i32;
    (*stash).nscratch = 0i32;
    h = fons__hashint(codepoint) & (256i32 - 1i32) as libc::c_uint;
    i = (*font).lut[h as usize];
    while i != -1i32 {
        if (*(*font).glyphs.offset(i as isize)).codepoint == codepoint &&
               (*(*font).glyphs.offset(i as isize)).size as libc::c_int ==
                   isize as libc::c_int &&
               (*(*font).glyphs.offset(i as isize)).blur as libc::c_int ==
                   iblur as libc::c_int {
            glyph = &mut *(*font).glyphs.offset(i as isize) as *mut FONSglyph;
            if bitmapOption == FONS_GLYPH_BITMAP_OPTIONAL as libc::c_int ||
                   (*glyph).x0 as libc::c_int >= 0i32 &&
                       (*glyph).y0 as libc::c_int >= 0i32 {
                return glyph
            }
            // At this point, glyph exists but the bitmap data is not yet created.
            break ;
        } else { i = (*(*font).glyphs.offset(i as isize)).next }
    }
    g = fons__tt_getGlyphIndex(&mut (*font).font, codepoint as libc::c_int);
    if g == 0i32 {
        i = 0i32;
        while i < (*font).nfallbacks {
            let mut fallbackFont: *mut FONSfont =
                *(*stash).fonts.offset((*font).fallbacks[i as usize] as
                                           isize);
            let mut fallbackIndex: libc::c_int =
                fons__tt_getGlyphIndex(&mut (*fallbackFont).font,
                                       codepoint as libc::c_int);
            if fallbackIndex != 0i32 {
                g = fallbackIndex;
                renderFont = fallbackFont;
                break ;
            } else { i += 1 }
        }
    }
    scale = fons__tt_getPixelHeightScale(&mut (*renderFont).font, size);
    fons__tt_buildGlyphBitmap(&mut (*renderFont).font, g, size, scale,
                              &mut advance, &mut lsb, &mut x0, &mut y0,
                              &mut x1, &mut y1);
    gw = x1 - x0 + pad * 2i32;
    gh = y1 - y0 + pad * 2i32;
    if bitmapOption == FONS_GLYPH_BITMAP_REQUIRED as libc::c_int {
        added = fons__atlasAddRect((*stash).atlas, gw, gh, &mut gx, &mut gy);
        if added == 0i32 && (*stash).handleError.is_some() {
            (*stash).handleError.expect("non-null function pointer")((*stash).errorUptr,
                                                                     FONS_ATLAS_FULL
                                                                         as
                                                                         libc::c_int,
                                                                     0i32);
            added =
                fons__atlasAddRect((*stash).atlas, gw, gh, &mut gx, &mut gy)
        }
        if added == 0i32 { return 0 as *mut FONSglyph }
    } else { gx = -1i32; gy = -1i32 }
    if glyph.is_null() {
        glyph = fons__allocGlyph(font);
        (*glyph).codepoint = codepoint;
        (*glyph).size = isize;
        (*glyph).blur = iblur;
        (*glyph).next = 0i32;
        (*glyph).next = (*font).lut[h as usize];
        (*font).lut[h as usize] = (*font).nglyphs - 1i32
    }
    (*glyph).index = g;
    (*glyph).x0 = gx as libc::c_short;
    (*glyph).y0 = gy as libc::c_short;
    (*glyph).x1 = ((*glyph).x0 as libc::c_int + gw) as libc::c_short;
    (*glyph).y1 = ((*glyph).y0 as libc::c_int + gh) as libc::c_short;
    (*glyph).xadv =
        (scale * advance as libc::c_float * 10.0f32) as libc::c_short;
    (*glyph).xoff = (x0 - pad) as libc::c_short;
    (*glyph).yoff = (y0 - pad) as libc::c_short;
    if bitmapOption == FONS_GLYPH_BITMAP_OPTIONAL as libc::c_int {
        return glyph
    }
    dst =
        &mut *(*stash).texData.offset(((*glyph).x0 as libc::c_int + pad +
                                           ((*glyph).y0 as libc::c_int + pad)
                                               * (*stash).params.width) as
                                          isize) as *mut libc::c_uchar;
    fons__tt_renderGlyphBitmap(&mut (*renderFont).font, dst, gw - pad * 2i32,
                               gh - pad * 2i32, (*stash).params.width, scale,
                               scale, g);
    dst =
        &mut *(*stash).texData.offset(((*glyph).x0 as libc::c_int +
                                           (*glyph).y0 as libc::c_int *
                                               (*stash).params.width) as
                                          isize) as *mut libc::c_uchar;
    y = 0i32;
    while y < gh {
        *dst.offset((y * (*stash).params.width) as isize) =
            0i32 as libc::c_uchar;
        *dst.offset((gw - 1i32 + y * (*stash).params.width) as isize) =
            0i32 as libc::c_uchar;
        y += 1
    }
    x = 0i32;
    while x < gw {
        *dst.offset(x as isize) = 0i32 as libc::c_uchar;
        *dst.offset((x + (gh - 1i32) * (*stash).params.width) as isize) =
            0i32 as libc::c_uchar;
        x += 1
    }
    if iblur as libc::c_int > 0i32 {
        (*stash).nscratch = 0i32;
        bdst =
            &mut *(*stash).texData.offset(((*glyph).x0 as libc::c_int +
                                               (*glyph).y0 as libc::c_int *
                                                   (*stash).params.width) as
                                              isize) as *mut libc::c_uchar;
        fons__blur(stash, bdst, gw, gh, (*stash).params.width,
                   iblur as libc::c_int);
    }
    (*stash).dirtyRect[0usize] =
        fons__mini((*stash).dirtyRect[0usize], (*glyph).x0 as libc::c_int);
    (*stash).dirtyRect[1usize] =
        fons__mini((*stash).dirtyRect[1usize], (*glyph).y0 as libc::c_int);
    (*stash).dirtyRect[2usize] =
        fons__maxi((*stash).dirtyRect[2usize], (*glyph).x1 as libc::c_int);
    (*stash).dirtyRect[3usize] =
        fons__maxi((*stash).dirtyRect[3usize], (*glyph).y1 as libc::c_int);
    return glyph;
}

pub unsafe extern "C" fn fons__blur(mut stash: *mut FONScontext,
                                    mut dst: *mut libc::c_uchar,
                                    mut w: libc::c_int, mut h: libc::c_int,
                                    mut dstStride: libc::c_int,
                                    mut blur: libc::c_int) {
    let mut alpha: libc::c_int = 0;
    let mut sigma: libc::c_float = 0.;
    if blur < 1i32 { return }
    sigma = blur as libc::c_float * 0.57735f32;
    alpha =
        ((1i32 << 16i32) as libc::c_float *
             (1.0f32 - expf(-2.3f32 / (sigma + 1.0f32)))) as libc::c_int;
    fons__blurRows(dst, w, h, dstStride, alpha);
    fons__blurCols(dst, w, h, dstStride, alpha);
    fons__blurRows(dst, w, h, dstStride, alpha);
    fons__blurCols(dst, w, h, dstStride, alpha);
}
// Based on Exponential blur, Jani Huhtanen, 2006

pub unsafe extern "C" fn fons__blurCols(mut dst: *mut libc::c_uchar,
                                        mut w: libc::c_int,
                                        mut h: libc::c_int,
                                        mut dstStride: libc::c_int,
                                        mut alpha: libc::c_int) {
    let mut x: libc::c_int = 0;
    let mut y: libc::c_int = 0;
    y = 0i32;
    while y < h {
        let mut z: libc::c_int = 0i32;
        x = 1i32;
        while x < w {
            z +=
                alpha *
                    (((*dst.offset(x as isize) as libc::c_int) << 7i32) - z)
                    >> 16i32;
            *dst.offset(x as isize) = (z >> 7i32) as libc::c_uchar;
            x += 1
        }
        *dst.offset((w - 1i32) as isize) = 0i32 as libc::c_uchar;
        z = 0i32;
        x = w - 2i32;
        while x >= 0i32 {
            z +=
                alpha *
                    (((*dst.offset(x as isize) as libc::c_int) << 7i32) - z)
                    >> 16i32;
            *dst.offset(x as isize) = (z >> 7i32) as libc::c_uchar;
            x -= 1
        }
        *dst.offset(0isize) = 0i32 as libc::c_uchar;
        dst = dst.offset(dstStride as isize);
        y += 1
    };
}

pub unsafe extern "C" fn fons__blurRows(mut dst: *mut libc::c_uchar,
                                        mut w: libc::c_int,
                                        mut h: libc::c_int,
                                        mut dstStride: libc::c_int,
                                        mut alpha: libc::c_int) {
    let mut x: libc::c_int = 0;
    let mut y: libc::c_int = 0;
    x = 0i32;
    while x < w {
        let mut z: libc::c_int = 0i32;
        y = dstStride;
        while y < h * dstStride {
            z +=
                alpha *
                    (((*dst.offset(y as isize) as libc::c_int) << 7i32) - z)
                    >> 16i32;
            *dst.offset(y as isize) = (z >> 7i32) as libc::c_uchar;
            y += dstStride
        }
        *dst.offset(((h - 1i32) * dstStride) as isize) =
            0i32 as libc::c_uchar;
        z = 0i32;
        y = (h - 2i32) * dstStride;
        while y >= 0i32 {
            z +=
                alpha *
                    (((*dst.offset(y as isize) as libc::c_int) << 7i32) - z)
                    >> 16i32;
            *dst.offset(y as isize) = (z >> 7i32) as libc::c_uchar;
            y -= dstStride
        }
        *dst.offset(0isize) = 0i32 as libc::c_uchar;
        dst = dst.offset(1isize);
        x += 1
    };
}

pub unsafe extern "C" fn fons__tt_renderGlyphBitmap(mut font:
                                                        *mut FONSttFontImpl,
                                                    mut output:
                                                        *mut libc::c_uchar,
                                                    mut outWidth: libc::c_int,
                                                    mut outHeight:
                                                        libc::c_int,
                                                    mut outStride:
                                                        libc::c_int,
                                                    mut scaleX: libc::c_float,
                                                    mut scaleY: libc::c_float,
                                                    mut glyph: libc::c_int) {
    stbtt_MakeGlyphBitmap(&mut (*font).font, output, outWidth, outHeight,
                          outStride, scaleX, scaleY, glyph);
}

pub unsafe extern "C" fn stbtt_MakeGlyphBitmap(mut info:
                                                   *const stbtt_fontinfo,
                                               mut output: *mut libc::c_uchar,
                                               mut out_w: libc::c_int,
                                               mut out_h: libc::c_int,
                                               mut out_stride: libc::c_int,
                                               mut scale_x: libc::c_float,
                                               mut scale_y: libc::c_float,
                                               mut glyph: libc::c_int) {
    stbtt_MakeGlyphBitmapSubpixel(info, output, out_w, out_h, out_stride,
                                  scale_x, scale_y, 0.0f32, 0.0f32, glyph);
}

pub unsafe extern "C" fn stbtt_MakeGlyphBitmapSubpixel(mut info:
                                                           *const stbtt_fontinfo,
                                                       mut output:
                                                           *mut libc::c_uchar,
                                                       mut out_w: libc::c_int,
                                                       mut out_h: libc::c_int,
                                                       mut out_stride:
                                                           libc::c_int,
                                                       mut scale_x:
                                                           libc::c_float,
                                                       mut scale_y:
                                                           libc::c_float,
                                                       mut shift_x:
                                                           libc::c_float,
                                                       mut shift_y:
                                                           libc::c_float,
                                                       mut glyph:
                                                           libc::c_int) {
    let mut ix0: libc::c_int = 0;
    let mut iy0: libc::c_int = 0;
    let mut vertices: *mut stbtt_vertex = 0 as *mut stbtt_vertex;
    let mut num_verts: libc::c_int =
        stbtt_GetGlyphShape(info, glyph, &mut vertices);
    let mut gbm: stbtt__bitmap =
        stbtt__bitmap{w: 0,
                      h: 0,
                      stride: 0,
                      pixels: 0 as *mut libc::c_uchar,};
    stbtt_GetGlyphBitmapBoxSubpixel(info, glyph, scale_x, scale_y, shift_x,
                                    shift_y, &mut ix0, &mut iy0,
                                    0 as *mut libc::c_int,
                                    0 as *mut libc::c_int);
    gbm.pixels = output;
    gbm.w = out_w;
    gbm.h = out_h;
    gbm.stride = out_stride;
    if 0 != gbm.w && 0 != gbm.h {
        stbtt_Rasterize(&mut gbm, 0.35f32, vertices, num_verts, scale_x,
                        scale_y, shift_x, shift_y, ix0, iy0, 1i32,
                        (*info).userdata);
    }
    fons__tmpfree(vertices as *mut libc::c_void, (*info).userdata);
}

pub unsafe extern "C" fn fons__tmpfree(mut ptr: *mut libc::c_void,
                                       mut up: *mut libc::c_void) {
}

pub unsafe extern "C" fn stbtt_GetGlyphShape(mut info: *const stbtt_fontinfo,
                                             mut glyph_index: libc::c_int,
                                             mut pvertices:
                                                 *mut *mut stbtt_vertex)
 -> libc::c_int {
    let mut numberOfContours: stbtt_int16 = 0;
    let mut endPtsOfContours: *mut stbtt_uint8 = 0 as *mut stbtt_uint8;
    let mut data: *mut stbtt_uint8 = (*info).data;
    let mut vertices: *mut stbtt_vertex = 0 as *mut stbtt_vertex;
    let mut num_vertices: libc::c_int = 0i32;
    let mut g: libc::c_int = stbtt__GetGlyfOffset(info, glyph_index);
    *pvertices = 0 as *mut stbtt_vertex;
    if g < 0i32 { return 0i32 }
    numberOfContours = ttSHORT(data.offset(g as isize));
    if numberOfContours as libc::c_int > 0i32 {
        let mut flags: stbtt_uint8 = 0i32 as stbtt_uint8;
        let mut flagcount: stbtt_uint8 = 0;
        let mut ins: stbtt_int32 = 0;
        let mut i: stbtt_int32 = 0;
        let mut j: stbtt_int32 = 0i32;
        let mut m: stbtt_int32 = 0;
        let mut n: stbtt_int32 = 0;
        let mut next_move: stbtt_int32 = 0;
        let mut was_off: stbtt_int32 = 0i32;
        let mut off: stbtt_int32 = 0;
        let mut start_off: stbtt_int32 = 0i32;
        let mut x: stbtt_int32 = 0;
        let mut y: stbtt_int32 = 0;
        let mut cx: stbtt_int32 = 0;
        let mut cy: stbtt_int32 = 0;
        let mut sx: stbtt_int32 = 0;
        let mut sy: stbtt_int32 = 0;
        let mut scx: stbtt_int32 = 0;
        let mut scy: stbtt_int32 = 0;
        let mut points: *mut stbtt_uint8 = 0 as *mut stbtt_uint8;
        endPtsOfContours = data.offset(g as isize).offset(10isize);
        ins =
            ttUSHORT(data.offset(g as
                                     isize).offset(10isize).offset((numberOfContours
                                                                        as
                                                                        libc::c_int
                                                                        *
                                                                        2i32)
                                                                       as
                                                                       isize))
                as stbtt_int32;
        points =
            data.offset(g as
                            isize).offset(10isize).offset((numberOfContours as
                                                               libc::c_int *
                                                               2i32) as
                                                              isize).offset(2isize).offset(ins
                                                                                               as
                                                                                               isize);
        n =
            1i32 +
                ttUSHORT(endPtsOfContours.offset((numberOfContours as
                                                      libc::c_int * 2i32) as
                                                     isize).offset(-2isize))
                    as libc::c_int;
        m = n + 2i32 * numberOfContours as libc::c_int;
        vertices =
            fons__tmpalloc((m as
                                libc::c_ulong).wrapping_mul(::std::mem::size_of::<stbtt_vertex>()
                                                                as
                                                                libc::c_ulong),
                           (*info).userdata) as *mut stbtt_vertex;
        if vertices.is_null() { return 0i32 }
        next_move = 0i32;
        flagcount = 0i32 as stbtt_uint8;
        off = m - n;
        i = 0i32;
        while i < n {
            if flagcount as libc::c_int == 0i32 {
                let fresh5 = points;
                points = points.offset(1);
                flags = *fresh5;
                if 0 != flags as libc::c_int & 8i32 {
                    let fresh6 = points;
                    points = points.offset(1);
                    flagcount = *fresh6
                }
            } else { flagcount = flagcount.wrapping_sub(1) }
            (*vertices.offset((off + i) as isize)).type_0 = flags;
            i += 1
        }
        x = 0i32;
        i = 0i32;
        while i < n {
            flags = (*vertices.offset((off + i) as isize)).type_0;
            if 0 != flags as libc::c_int & 2i32 {
                let fresh7 = points;
                points = points.offset(1);
                let mut dx: stbtt_int16 = *fresh7 as stbtt_int16;
                x +=
                    if 0 != flags as libc::c_int & 16i32 {
                        dx as libc::c_int
                    } else { -(dx as libc::c_int) }
            } else if 0 == flags as libc::c_int & 16i32 {
                x =
                    x +
                        (*points.offset(0isize) as libc::c_int * 256i32 +
                             *points.offset(1isize) as libc::c_int) as
                            stbtt_int16 as libc::c_int;
                points = points.offset(2isize)
            }
            (*vertices.offset((off + i) as isize)).x = x as stbtt_int16;
            i += 1
        }
        y = 0i32;
        i = 0i32;
        while i < n {
            flags = (*vertices.offset((off + i) as isize)).type_0;
            if 0 != flags as libc::c_int & 4i32 {
                let fresh8 = points;
                points = points.offset(1);
                let mut dy: stbtt_int16 = *fresh8 as stbtt_int16;
                y +=
                    if 0 != flags as libc::c_int & 32i32 {
                        dy as libc::c_int
                    } else { -(dy as libc::c_int) }
            } else if 0 == flags as libc::c_int & 32i32 {
                y =
                    y +
                        (*points.offset(0isize) as libc::c_int * 256i32 +
                             *points.offset(1isize) as libc::c_int) as
                            stbtt_int16 as libc::c_int;
                points = points.offset(2isize)
            }
            (*vertices.offset((off + i) as isize)).y = y as stbtt_int16;
            i += 1
        }
        num_vertices = 0i32;
        scy = 0i32;
        scx = scy;
        cy = scx;
        cx = cy;
        sy = cx;
        sx = sy;
        i = 0i32;
        while i < n {
            flags = (*vertices.offset((off + i) as isize)).type_0;
            x =
                (*vertices.offset((off + i) as isize)).x as stbtt_int16 as
                    stbtt_int32;
            y =
                (*vertices.offset((off + i) as isize)).y as stbtt_int16 as
                    stbtt_int32;
            if next_move == i {
                if i != 0i32 {
                    num_vertices =
                        stbtt__close_shape(vertices, num_vertices, was_off,
                                           start_off, sx, sy, scx, scy, cx,
                                           cy)
                }
                start_off = (0 == flags as libc::c_int & 1i32) as libc::c_int;
                if 0 != start_off {
                    scx = x;
                    scy = y;
                    if 0 ==
                           (*vertices.offset((off + i + 1i32) as
                                                 isize)).type_0 as libc::c_int
                               & 1i32 {
                        sx =
                            x +
                                (*vertices.offset((off + i + 1i32) as
                                                      isize)).x as stbtt_int32
                                >> 1i32;
                        sy =
                            y +
                                (*vertices.offset((off + i + 1i32) as
                                                      isize)).y as stbtt_int32
                                >> 1i32
                    } else {
                        sx =
                            (*vertices.offset((off + i + 1i32) as isize)).x as
                                stbtt_int32;
                        sy =
                            (*vertices.offset((off + i + 1i32) as isize)).y as
                                stbtt_int32;
                        i += 1
                    }
                } else { sx = x; sy = y }
                let fresh9 = num_vertices;
                num_vertices = num_vertices + 1;
                stbtt_setvertex(&mut *vertices.offset(fresh9 as isize),
                                STBTT_vmove as libc::c_int as stbtt_uint8, sx,
                                sy, 0i32, 0i32);
                was_off = 0i32;
                next_move =
                    1i32 +
                        ttUSHORT(endPtsOfContours.offset((j * 2i32) as isize))
                            as libc::c_int;
                j += 1
            } else if 0 == flags as libc::c_int & 1i32 {
                if 0 != was_off {
                    let fresh10 = num_vertices;
                    num_vertices = num_vertices + 1;
                    stbtt_setvertex(&mut *vertices.offset(fresh10 as isize),
                                    STBTT_vcurve as libc::c_int as
                                        stbtt_uint8, cx + x >> 1i32,
                                    cy + y >> 1i32, cx, cy);
                }
                cx = x;
                cy = y;
                was_off = 1i32
            } else {
                if 0 != was_off {
                    let fresh11 = num_vertices;
                    num_vertices = num_vertices + 1;
                    stbtt_setvertex(&mut *vertices.offset(fresh11 as isize),
                                    STBTT_vcurve as libc::c_int as
                                        stbtt_uint8, x, y, cx, cy);
                } else {
                    let fresh12 = num_vertices;
                    num_vertices = num_vertices + 1;
                    stbtt_setvertex(&mut *vertices.offset(fresh12 as isize),
                                    STBTT_vline as libc::c_int as stbtt_uint8,
                                    x, y, 0i32, 0i32);
                }
                was_off = 0i32
            }
            i += 1
        }
        num_vertices =
            stbtt__close_shape(vertices, num_vertices, was_off, start_off, sx,
                               sy, scx, scy, cx, cy)
    } else if numberOfContours as libc::c_int == -1i32 {
        let mut more: libc::c_int = 1i32;
        let mut comp: *mut stbtt_uint8 =
            data.offset(g as isize).offset(10isize);
        num_vertices = 0i32;
        vertices = 0 as *mut stbtt_vertex;
        while 0 != more {
            let mut flags_0: stbtt_uint16 = 0;
            let mut gidx: stbtt_uint16 = 0;
            let mut comp_num_verts: libc::c_int = 0i32;
            let mut i_0: libc::c_int = 0;
            let mut comp_verts: *mut stbtt_vertex = 0 as *mut stbtt_vertex;
            let mut tmp: *mut stbtt_vertex = 0 as *mut stbtt_vertex;
            let mut mtx: [libc::c_float; 6] =
                [1i32 as libc::c_float, 0i32 as libc::c_float,
                 0i32 as libc::c_float, 1i32 as libc::c_float,
                 0i32 as libc::c_float, 0i32 as libc::c_float];
            let mut m_0: libc::c_float = 0.;
            let mut n_0: libc::c_float = 0.;
            flags_0 = ttSHORT(comp) as stbtt_uint16;
            comp = comp.offset(2isize);
            gidx = ttSHORT(comp) as stbtt_uint16;
            comp = comp.offset(2isize);
            if 0 != flags_0 as libc::c_int & 2i32 {
                if 0 != flags_0 as libc::c_int & 1i32 {
                    mtx[4usize] = ttSHORT(comp) as libc::c_float;
                    comp = comp.offset(2isize);
                    mtx[5usize] = ttSHORT(comp) as libc::c_float;
                    comp = comp.offset(2isize)
                } else {
                    mtx[4usize] = *(comp as *mut stbtt_int8) as libc::c_float;
                    comp = comp.offset(1isize);
                    mtx[5usize] = *(comp as *mut stbtt_int8) as libc::c_float;
                    comp = comp.offset(1isize)
                }
            } else {
                __assert_fail(b"0\x00" as *const u8 as *const libc::c_char,
                              b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                                  as *const u8 as *const libc::c_char,
                              1407i32 as libc::c_uint,
                              (*::std::mem::transmute::<&[u8; 70],
                                                        &[libc::c_char; 70]>(b"int stbtt_GetGlyphShape(const stbtt_fontinfo *, int, stbtt_vertex **)\x00")).as_ptr());
            }
            if 0 != flags_0 as libc::c_int & 1i32 << 3i32 {
                mtx[3usize] =
                    ttSHORT(comp) as libc::c_int as libc::c_float /
                        16384.0f32;
                mtx[0usize] = mtx[3usize];
                comp = comp.offset(2isize);
                mtx[2usize] = 0i32 as libc::c_float;
                mtx[1usize] = mtx[2usize]
            } else if 0 != flags_0 as libc::c_int & 1i32 << 6i32 {
                mtx[0usize] =
                    ttSHORT(comp) as libc::c_int as libc::c_float /
                        16384.0f32;
                comp = comp.offset(2isize);
                mtx[2usize] = 0i32 as libc::c_float;
                mtx[1usize] = mtx[2usize];
                mtx[3usize] =
                    ttSHORT(comp) as libc::c_int as libc::c_float /
                        16384.0f32;
                comp = comp.offset(2isize)
            } else if 0 != flags_0 as libc::c_int & 1i32 << 7i32 {
                mtx[0usize] =
                    ttSHORT(comp) as libc::c_int as libc::c_float /
                        16384.0f32;
                comp = comp.offset(2isize);
                mtx[1usize] =
                    ttSHORT(comp) as libc::c_int as libc::c_float /
                        16384.0f32;
                comp = comp.offset(2isize);
                mtx[2usize] =
                    ttSHORT(comp) as libc::c_int as libc::c_float /
                        16384.0f32;
                comp = comp.offset(2isize);
                mtx[3usize] =
                    ttSHORT(comp) as libc::c_int as libc::c_float /
                        16384.0f32;
                comp = comp.offset(2isize)
            }
            m_0 =
                sqrt((mtx[0usize] * mtx[0usize] + mtx[1usize] * mtx[1usize])
                         as libc::c_double) as libc::c_float;
            n_0 =
                sqrt((mtx[2usize] * mtx[2usize] + mtx[3usize] * mtx[3usize])
                         as libc::c_double) as libc::c_float;
            comp_num_verts =
                stbtt_GetGlyphShape(info, gidx as libc::c_int,
                                    &mut comp_verts);
            if comp_num_verts > 0i32 {
                i_0 = 0i32;
                while i_0 < comp_num_verts {
                    let mut v: *mut stbtt_vertex =
                        &mut *comp_verts.offset(i_0 as isize) as
                            *mut stbtt_vertex;
                    let mut x_0: libc::c_short = 0;
                    let mut y_0: libc::c_short = 0;
                    x_0 = (*v).x;
                    y_0 = (*v).y;
                    (*v).x =
                        (m_0 *
                             (mtx[0usize] *
                                  x_0 as libc::c_int as libc::c_float +
                                  mtx[2usize] *
                                      y_0 as libc::c_int as libc::c_float +
                                  mtx[4usize])) as libc::c_short;
                    (*v).y =
                        (n_0 *
                             (mtx[1usize] *
                                  x_0 as libc::c_int as libc::c_float +
                                  mtx[3usize] *
                                      y_0 as libc::c_int as libc::c_float +
                                  mtx[5usize])) as libc::c_short;
                    x_0 = (*v).cx;
                    y_0 = (*v).cy;
                    (*v).cx =
                        (m_0 *
                             (mtx[0usize] *
                                  x_0 as libc::c_int as libc::c_float +
                                  mtx[2usize] *
                                      y_0 as libc::c_int as libc::c_float +
                                  mtx[4usize])) as libc::c_short;
                    (*v).cy =
                        (n_0 *
                             (mtx[1usize] *
                                  x_0 as libc::c_int as libc::c_float +
                                  mtx[3usize] *
                                      y_0 as libc::c_int as libc::c_float +
                                  mtx[5usize])) as libc::c_short;
                    i_0 += 1
                }
                tmp =
                    fons__tmpalloc(((num_vertices + comp_num_verts) as
                                        libc::c_ulong).wrapping_mul(::std::mem::size_of::<stbtt_vertex>()
                                                                        as
                                                                        libc::c_ulong),
                                   (*info).userdata) as *mut stbtt_vertex;
                if tmp.is_null() {
                    if !vertices.is_null() {
                        fons__tmpfree(vertices as *mut libc::c_void,
                                      (*info).userdata);
                    }
                    if !comp_verts.is_null() {
                        fons__tmpfree(comp_verts as *mut libc::c_void,
                                      (*info).userdata);
                    }
                    return 0i32
                }
                if num_vertices > 0i32 {
                    memcpy(tmp as *mut libc::c_void,
                           vertices as *const libc::c_void,
                           (num_vertices as
                                libc::c_ulong).wrapping_mul(::std::mem::size_of::<stbtt_vertex>()
                                                                as
                                                                libc::c_ulong));
                }
                memcpy(tmp.offset(num_vertices as isize) as *mut libc::c_void,
                       comp_verts as *const libc::c_void,
                       (comp_num_verts as
                            libc::c_ulong).wrapping_mul(::std::mem::size_of::<stbtt_vertex>()
                                                            as
                                                            libc::c_ulong));
                if !vertices.is_null() {
                    fons__tmpfree(vertices as *mut libc::c_void,
                                  (*info).userdata);
                }
                vertices = tmp;
                fons__tmpfree(comp_verts as *mut libc::c_void,
                              (*info).userdata);
                num_vertices += comp_num_verts
            }
            more = flags_0 as libc::c_int & 1i32 << 5i32
        }
    } else if (numberOfContours as libc::c_int) < 0i32 {
        __assert_fail(b"0\x00" as *const u8 as *const libc::c_char,
                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00" as
                          *const u8 as *const libc::c_char,
                      1460i32 as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 70],
                                                &[libc::c_char; 70]>(b"int stbtt_GetGlyphShape(const stbtt_fontinfo *, int, stbtt_vertex **)\x00")).as_ptr());
    }
    *pvertices = vertices;
    return num_vertices;
}
// FONTSTASH_H

pub unsafe extern "C" fn fons__tmpalloc(mut size: size_t,
                                        mut up: *mut libc::c_void)
 -> *mut libc::c_void {
    let mut ptr: *mut libc::c_uchar = 0 as *mut libc::c_uchar;
    let mut stash: *mut FONScontext = up as *mut FONScontext;
    size =
        size.wrapping_add(0xfi32 as libc::c_ulong) & !0xfi32 as libc::c_ulong;
    if (*stash).nscratch + size as libc::c_int > 96000i32 {
        if (*stash).handleError.is_some() {
            (*stash).handleError.expect("non-null function pointer")((*stash).errorUptr,
                                                                     FONS_SCRATCH_FULL
                                                                         as
                                                                         libc::c_int,
                                                                     (*stash).nscratch
                                                                         +
                                                                         size
                                                                             as
                                                                             libc::c_int);
        }
        return 0 as *mut libc::c_void
    }
    ptr = (*stash).scratch.offset((*stash).nscratch as isize);
    (*stash).nscratch += size as libc::c_int;
    return ptr as *mut libc::c_void;
}
unsafe extern "C" fn stbtt__GetGlyfOffset(mut info: *const stbtt_fontinfo,
                                          mut glyph_index: libc::c_int)
 -> libc::c_int {
    let mut g1: libc::c_int = 0;
    let mut g2: libc::c_int = 0;
    if glyph_index >= (*info).numGlyphs { return -1i32 }
    if (*info).indexToLocFormat >= 2i32 { return -1i32 }
    if (*info).indexToLocFormat == 0i32 {
        g1 =
            (*info).glyf +
                ttUSHORT((*info).data.offset((*info).loca as
                                                 isize).offset((glyph_index *
                                                                    2i32) as
                                                                   isize)) as
                    libc::c_int * 2i32;
        g2 =
            (*info).glyf +
                ttUSHORT((*info).data.offset((*info).loca as
                                                 isize).offset((glyph_index *
                                                                    2i32) as
                                                                   isize).offset(2isize))
                    as libc::c_int * 2i32
    } else {
        g1 =
            ((*info).glyf as
                 libc::c_uint).wrapping_add(ttULONG((*info).data.offset((*info).loca
                                                                            as
                                                                            isize).offset((glyph_index
                                                                                               *
                                                                                               4i32)
                                                                                              as
                                                                                              isize)))
                as libc::c_int;
        g2 =
            ((*info).glyf as
                 libc::c_uint).wrapping_add(ttULONG((*info).data.offset((*info).loca
                                                                            as
                                                                            isize).offset((glyph_index
                                                                                               *
                                                                                               4i32)
                                                                                              as
                                                                                              isize).offset(4isize)))
                as libc::c_int
    }
    return if g1 == g2 { -1i32 } else { g1 };
}
unsafe extern "C" fn stbtt__close_shape(mut vertices: *mut stbtt_vertex,
                                        mut num_vertices: libc::c_int,
                                        mut was_off: libc::c_int,
                                        mut start_off: libc::c_int,
                                        mut sx: stbtt_int32,
                                        mut sy: stbtt_int32,
                                        mut scx: stbtt_int32,
                                        mut scy: stbtt_int32,
                                        mut cx: stbtt_int32,
                                        mut cy: stbtt_int32) -> libc::c_int {
    if 0 != start_off {
        if 0 != was_off {
            let fresh13 = num_vertices;
            num_vertices = num_vertices + 1;
            stbtt_setvertex(&mut *vertices.offset(fresh13 as isize),
                            STBTT_vcurve as libc::c_int as stbtt_uint8,
                            cx + scx >> 1i32, cy + scy >> 1i32, cx, cy);
        }
        let fresh14 = num_vertices;
        num_vertices = num_vertices + 1;
        stbtt_setvertex(&mut *vertices.offset(fresh14 as isize),
                        STBTT_vcurve as libc::c_int as stbtt_uint8, sx, sy,
                        scx, scy);
    } else if 0 != was_off {
        let fresh15 = num_vertices;
        num_vertices = num_vertices + 1;
        stbtt_setvertex(&mut *vertices.offset(fresh15 as isize),
                        STBTT_vcurve as libc::c_int as stbtt_uint8, sx, sy,
                        cx, cy);
    } else {
        let fresh16 = num_vertices;
        num_vertices = num_vertices + 1;
        stbtt_setvertex(&mut *vertices.offset(fresh16 as isize),
                        STBTT_vline as libc::c_int as stbtt_uint8, sx, sy,
                        0i32, 0i32);
    }
    return num_vertices;
}
unsafe extern "C" fn stbtt_setvertex(mut v: *mut stbtt_vertex,
                                     mut type_0: stbtt_uint8,
                                     mut x: stbtt_int32, mut y: stbtt_int32,
                                     mut cx: stbtt_int32,
                                     mut cy: stbtt_int32) {
    (*v).type_0 = type_0;
    (*v).x = x as stbtt_int16;
    (*v).y = y as stbtt_int16;
    (*v).cx = cx as stbtt_int16;
    (*v).cy = cy as stbtt_int16;
}
// rasterize a shape with quadratic beziers into a bitmap
// 1-channel bitmap to draw into

pub unsafe extern "C" fn stbtt_Rasterize(mut result: *mut stbtt__bitmap,
                                         mut flatness_in_pixels:
                                             libc::c_float,
                                         mut vertices: *mut stbtt_vertex,
                                         mut num_verts: libc::c_int,
                                         mut scale_x: libc::c_float,
                                         mut scale_y: libc::c_float,
                                         mut shift_x: libc::c_float,
                                         mut shift_y: libc::c_float,
                                         mut x_off: libc::c_int,
                                         mut y_off: libc::c_int,
                                         mut invert: libc::c_int,
                                         mut userdata: *mut libc::c_void) {
    let mut scale: libc::c_float =
        if scale_x > scale_y { scale_y } else { scale_x };
    let mut winding_count: libc::c_int = 0;
    let mut winding_lengths: *mut libc::c_int = 0 as *mut libc::c_int;
    let mut windings: *mut stbtt__point =
        stbtt_FlattenCurves(vertices, num_verts, flatness_in_pixels / scale,
                            &mut winding_lengths, &mut winding_count,
                            userdata);
    if !windings.is_null() {
        stbtt__rasterize(result, windings, winding_lengths, winding_count,
                         scale_x, scale_y, shift_x, shift_y, x_off, y_off,
                         invert, userdata);
        fons__tmpfree(winding_lengths as *mut libc::c_void, userdata);
        fons__tmpfree(windings as *mut libc::c_void, userdata);
    };
}
// returns number of contours
unsafe extern "C" fn stbtt_FlattenCurves(mut vertices: *mut stbtt_vertex,
                                         mut num_verts: libc::c_int,
                                         mut objspace_flatness: libc::c_float,
                                         mut contour_lengths:
                                             *mut *mut libc::c_int,
                                         mut num_contours: *mut libc::c_int,
                                         mut userdata: *mut libc::c_void)
 -> *mut stbtt__point {
    let mut current_block: u64;
    let mut points: *mut stbtt__point = 0 as *mut stbtt__point;
    let mut num_points: libc::c_int = 0i32;
    let mut objspace_flatness_squared: libc::c_float =
        objspace_flatness * objspace_flatness;
    let mut i: libc::c_int = 0;
    let mut n: libc::c_int = 0i32;
    let mut start: libc::c_int = 0i32;
    let mut pass: libc::c_int = 0;
    i = 0i32;
    while i < num_verts {
        if (*vertices.offset(i as isize)).type_0 as libc::c_int ==
               STBTT_vmove as libc::c_int {
            n += 1
        }
        i += 1
    }
    *num_contours = n;
    if n == 0i32 { return 0 as *mut stbtt__point }
    *contour_lengths =
        fons__tmpalloc((::std::mem::size_of::<libc::c_int>() as
                            libc::c_ulong).wrapping_mul(n as libc::c_ulong),
                       userdata) as *mut libc::c_int;
    if (*contour_lengths).is_null() {
        *num_contours = 0i32;
        return 0 as *mut stbtt__point
    }
    // make two passes through the points so we don't need to realloc
    pass = 0i32;
    loop  {
        if !(pass < 2i32) { current_block = 8845338526596852646; break ; }
        let mut x: libc::c_float = 0i32 as libc::c_float;
        let mut y: libc::c_float = 0i32 as libc::c_float;
        if pass == 1i32 {
            points =
                fons__tmpalloc((num_points as
                                    libc::c_ulong).wrapping_mul(::std::mem::size_of::<stbtt__point>()
                                                                    as
                                                                    libc::c_ulong),
                               userdata) as *mut stbtt__point;
            if points.is_null() {
                current_block = 9535040653783544971;
                break ;
            }
        }
        num_points = 0i32;
        n = -1i32;
        i = 0i32;
        while i < num_verts {
            match (*vertices.offset(i as isize)).type_0 as libc::c_int {
                1 => {
                    if n >= 0i32 {
                        *(*contour_lengths).offset(n as isize) =
                            num_points - start
                    }
                    n += 1;
                    start = num_points;
                    x = (*vertices.offset(i as isize)).x as libc::c_float;
                    y = (*vertices.offset(i as isize)).y as libc::c_float;
                    let fresh17 = num_points;
                    num_points = num_points + 1;
                    stbtt__add_point(points, fresh17, x, y);
                }
                2 => {
                    x = (*vertices.offset(i as isize)).x as libc::c_float;
                    y = (*vertices.offset(i as isize)).y as libc::c_float;
                    let fresh18 = num_points;
                    num_points = num_points + 1;
                    stbtt__add_point(points, fresh18, x, y);
                }
                3 => {
                    stbtt__tesselate_curve(points, &mut num_points, x, y,
                                           (*vertices.offset(i as isize)).cx
                                               as libc::c_float,
                                           (*vertices.offset(i as isize)).cy
                                               as libc::c_float,
                                           (*vertices.offset(i as isize)).x as
                                               libc::c_float,
                                           (*vertices.offset(i as isize)).y as
                                               libc::c_float,
                                           objspace_flatness_squared, 0i32);
                    x = (*vertices.offset(i as isize)).x as libc::c_float;
                    y = (*vertices.offset(i as isize)).y as libc::c_float
                }
                _ => { }
            }
            i += 1
        }
        *(*contour_lengths).offset(n as isize) = num_points - start;
        pass += 1
    }
    match current_block {
        8845338526596852646 => { return points }
        _ => {
            fons__tmpfree(points as *mut libc::c_void, userdata);
            fons__tmpfree(*contour_lengths as *mut libc::c_void, userdata);
            *contour_lengths = 0 as *mut libc::c_int;
            *num_contours = 0i32;
            return 0 as *mut stbtt__point
        }
    };
}
// tesselate until threshhold p is happy... @TODO warped to compensate for non-linear stretching
unsafe extern "C" fn stbtt__tesselate_curve(mut points: *mut stbtt__point,
                                            mut num_points: *mut libc::c_int,
                                            mut x0: libc::c_float,
                                            mut y0: libc::c_float,
                                            mut x1: libc::c_float,
                                            mut y1: libc::c_float,
                                            mut x2: libc::c_float,
                                            mut y2: libc::c_float,
                                            mut objspace_flatness_squared:
                                                libc::c_float,
                                            mut n: libc::c_int)
 -> libc::c_int {
    // midpoint
    let mut mx: libc::c_float =
        (x0 + 2i32 as libc::c_float * x1 + x2) / 4i32 as libc::c_float;
    let mut my: libc::c_float =
        (y0 + 2i32 as libc::c_float * y1 + y2) / 4i32 as libc::c_float;
    // versus directly drawn line
    let mut dx: libc::c_float = (x0 + x2) / 2i32 as libc::c_float - mx;
    let mut dy: libc::c_float = (y0 + y2) / 2i32 as libc::c_float - my;
    if n > 16i32 { return 1i32 }
    if dx * dx + dy * dy > objspace_flatness_squared {
        stbtt__tesselate_curve(points, num_points, x0, y0, (x0 + x1) / 2.0f32,
                               (y0 + y1) / 2.0f32, mx, my,
                               objspace_flatness_squared, n + 1i32);
        stbtt__tesselate_curve(points, num_points, mx, my, (x1 + x2) / 2.0f32,
                               (y1 + y2) / 2.0f32, x2, y2,
                               objspace_flatness_squared, n + 1i32);
    } else {
        stbtt__add_point(points, *num_points, x2, y2);
        *num_points = *num_points + 1i32
    }
    return 1i32;
}
unsafe extern "C" fn stbtt__add_point(mut points: *mut stbtt__point,
                                      mut n: libc::c_int,
                                      mut x: libc::c_float,
                                      mut y: libc::c_float) {
    if points.is_null() { return }
    (*points.offset(n as isize)).x = x;
    (*points.offset(n as isize)).y = y;
}
unsafe extern "C" fn stbtt__rasterize(mut result: *mut stbtt__bitmap,
                                      mut pts: *mut stbtt__point,
                                      mut wcount: *mut libc::c_int,
                                      mut windings: libc::c_int,
                                      mut scale_x: libc::c_float,
                                      mut scale_y: libc::c_float,
                                      mut shift_x: libc::c_float,
                                      mut shift_y: libc::c_float,
                                      mut off_x: libc::c_int,
                                      mut off_y: libc::c_int,
                                      mut invert: libc::c_int,
                                      mut userdata: *mut libc::c_void) {
    let mut y_scale_inv: libc::c_float =
        if 0 != invert { -scale_y } else { scale_y };
    let mut e: *mut stbtt__edge = 0 as *mut stbtt__edge;
    let mut n: libc::c_int = 0;
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    let mut k: libc::c_int = 0;
    let mut m: libc::c_int = 0;
    let mut vsubsample: libc::c_int = 1i32;
    n = 0i32;
    i = 0i32;
    while i < windings { n += *wcount.offset(i as isize); i += 1 }
    e =
        fons__tmpalloc((::std::mem::size_of::<stbtt__edge>() as
                            libc::c_ulong).wrapping_mul((n + 1i32) as
                                                            libc::c_ulong),
                       userdata) as *mut stbtt__edge;
    if e.is_null() { return }
    n = 0i32;
    m = 0i32;
    i = 0i32;
    while i < windings {
        let mut p: *mut stbtt__point = pts.offset(m as isize);
        m += *wcount.offset(i as isize);
        j = *wcount.offset(i as isize) - 1i32;
        k = 0i32;
        while k < *wcount.offset(i as isize) {
            let mut a: libc::c_int = k;
            let mut b: libc::c_int = j;
            // skip the edge if horizontal
            if !((*p.offset(j as isize)).y == (*p.offset(k as isize)).y) {
                (*e.offset(n as isize)).invert = 0i32;
                if 0 !=
                       if 0 != invert {
                           ((*p.offset(j as isize)).y >
                                (*p.offset(k as isize)).y) as libc::c_int
                       } else {
                           ((*p.offset(j as isize)).y <
                                (*p.offset(k as isize)).y) as libc::c_int
                       } {
                    (*e.offset(n as isize)).invert = 1i32;
                    a = j;
                    b = k
                }
                (*e.offset(n as isize)).x0 =
                    (*p.offset(a as isize)).x * scale_x + shift_x;
                (*e.offset(n as isize)).y0 =
                    ((*p.offset(a as isize)).y * y_scale_inv + shift_y) *
                        vsubsample as libc::c_float;
                (*e.offset(n as isize)).x1 =
                    (*p.offset(b as isize)).x * scale_x + shift_x;
                (*e.offset(n as isize)).y1 =
                    ((*p.offset(b as isize)).y * y_scale_inv + shift_y) *
                        vsubsample as libc::c_float;
                n += 1
            }
            let fresh19 = k;
            k = k + 1;
            j = fresh19
        }
        i += 1
    }
    stbtt__sort_edges(e, n);
    stbtt__rasterize_sorted_edges(result, e, n, vsubsample, off_x, off_y,
                                  userdata);
    fons__tmpfree(e as *mut libc::c_void, userdata);
}
// directly AA rasterize edges w/o supersampling
unsafe extern "C" fn stbtt__rasterize_sorted_edges(mut result:
                                                       *mut stbtt__bitmap,
                                                   mut e: *mut stbtt__edge,
                                                   mut n: libc::c_int,
                                                   mut vsubsample:
                                                       libc::c_int,
                                                   mut off_x: libc::c_int,
                                                   mut off_y: libc::c_int,
                                                   mut userdata:
                                                       *mut libc::c_void) {
    let mut hh: stbtt__hheap =
        stbtt__hheap{head: 0 as *mut stbtt__hheap_chunk,
                     first_free: 0 as *mut libc::c_void,
                     num_remaining_in_head_chunk: 0i32,};
    let mut active: *mut stbtt__active_edge = 0 as *mut stbtt__active_edge;
    let mut y: libc::c_int = 0;
    let mut j: libc::c_int = 0i32;
    let mut i: libc::c_int = 0;
    let mut scanline_data: [libc::c_float; 129] = [0.; 129];
    let mut scanline: *mut libc::c_float = 0 as *mut libc::c_float;
    let mut scanline2: *mut libc::c_float = 0 as *mut libc::c_float;
    if (*result).w > 64i32 {
        scanline =
            fons__tmpalloc((((*result).w * 2i32 + 1i32) as
                                libc::c_ulong).wrapping_mul(::std::mem::size_of::<libc::c_float>()
                                                                as
                                                                libc::c_ulong),
                           userdata) as *mut libc::c_float
    } else { scanline = scanline_data.as_mut_ptr() }
    scanline2 = scanline.offset((*result).w as isize);
    y = off_y;
    (*e.offset(n as isize)).y0 =
        (off_y + (*result).h) as libc::c_float + 1i32 as libc::c_float;
    while j < (*result).h {
        let mut scan_y_top: libc::c_float = y as libc::c_float + 0.0f32;
        let mut scan_y_bottom: libc::c_float = y as libc::c_float + 1.0f32;
        let mut step: *mut *mut stbtt__active_edge = &mut active;
        memset(scanline as *mut libc::c_void, 0i32,
               ((*result).w as
                    libc::c_ulong).wrapping_mul(::std::mem::size_of::<libc::c_float>()
                                                    as libc::c_ulong));
        memset(scanline2 as *mut libc::c_void, 0i32,
               (((*result).w + 1i32) as
                    libc::c_ulong).wrapping_mul(::std::mem::size_of::<libc::c_float>()
                                                    as libc::c_ulong));
        while !(*step).is_null() {
            let mut z: *mut stbtt__active_edge = *step;
            if (*z).ey <= scan_y_top {
                *step = (*z).next;
                if 0. != (*z).direction {
                } else {
                    __assert_fail(b"z->direction\x00" as *const u8 as
                                      *const libc::c_char,
                                  b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                                      as *const u8 as *const libc::c_char,
                                  2099i32 as libc::c_uint,
                                  (*::std::mem::transmute::<&[u8; 95],
                                                            &[libc::c_char; 95]>(b"void stbtt__rasterize_sorted_edges(stbtt__bitmap *, stbtt__edge *, int, int, int, int, void *)\x00")).as_ptr());
                }
                (*z).direction = 0i32 as libc::c_float;
                stbtt__hheap_free(&mut hh, z as *mut libc::c_void);
            } else { step = &mut (**step).next }
        }
        while (*e).y0 <= scan_y_bottom {
            if (*e).y0 != (*e).y1 {
                let mut z_0: *mut stbtt__active_edge =
                    stbtt__new_active(&mut hh, e, off_x, scan_y_top,
                                      userdata);
                if !z_0.is_null() {
                    if (*z_0).ey >= scan_y_top {
                    } else {
                        __assert_fail(b"z->ey >= scan_y_top\x00" as *const u8
                                          as *const libc::c_char,
                                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                                          as *const u8 as *const libc::c_char,
                                      2112i32 as libc::c_uint,
                                      (*::std::mem::transmute::<&[u8; 95],
                                                                &[libc::c_char; 95]>(b"void stbtt__rasterize_sorted_edges(stbtt__bitmap *, stbtt__edge *, int, int, int, int, void *)\x00")).as_ptr());
                    }
                    (*z_0).next = active;
                    active = z_0
                }
            }
            e = e.offset(1isize)
        }
        if !active.is_null() {
            stbtt__fill_active_edges_new(scanline, scanline2.offset(1isize),
                                         (*result).w, active, scan_y_top);
        }
        let mut sum: libc::c_float = 0i32 as libc::c_float;
        i = 0i32;
        while i < (*result).w {
            let mut k: libc::c_float = 0.;
            let mut m: libc::c_int = 0;
            sum += *scanline2.offset(i as isize);
            k = *scanline.offset(i as isize) + sum;
            k =
                fabs(k as libc::c_double) as libc::c_float *
                    255i32 as libc::c_float + 0.5f32;
            m = k as libc::c_int;
            if m > 255i32 { m = 255i32 }
            *(*result).pixels.offset((j * (*result).stride + i) as isize) =
                m as libc::c_uchar;
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
    if scanline != scanline_data.as_mut_ptr() {
        fons__tmpfree(scanline as *mut libc::c_void, userdata);
    };
}
unsafe extern "C" fn stbtt__hheap_cleanup(mut hh: *mut stbtt__hheap,
                                          mut userdata: *mut libc::c_void) {
    let mut c: *mut stbtt__hheap_chunk = (*hh).head;
    while !c.is_null() {
        let mut n: *mut stbtt__hheap_chunk = (*c).next;
        fons__tmpfree(c as *mut libc::c_void, userdata);
        c = n
    };
}
unsafe extern "C" fn stbtt__fill_active_edges_new(mut scanline:
                                                      *mut libc::c_float,
                                                  mut scanline_fill:
                                                      *mut libc::c_float,
                                                  mut len: libc::c_int,
                                                  mut e:
                                                      *mut stbtt__active_edge,
                                                  mut y_top: libc::c_float) {
    let mut y_bottom: libc::c_float = y_top + 1i32 as libc::c_float;
    while !e.is_null() {
        if (*e).ey >= y_top {
        } else {
            __assert_fail(b"e->ey >= y_top\x00" as *const u8 as
                              *const libc::c_char,
                          b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                              as *const u8 as *const libc::c_char,
                          1912i32 as libc::c_uint,
                          (*::std::mem::transmute::<&[u8; 86],
                                                    &[libc::c_char; 86]>(b"void stbtt__fill_active_edges_new(float *, float *, int, stbtt__active_edge *, float)\x00")).as_ptr());
        }
        if (*e).fdx == 0i32 as libc::c_float {
            let mut x0: libc::c_float = (*e).fx;
            if x0 < len as libc::c_float {
                if x0 >= 0i32 as libc::c_float {
                    stbtt__handle_clipped_edge(scanline, x0 as libc::c_int, e,
                                               x0, y_top, x0, y_bottom);
                    stbtt__handle_clipped_edge(scanline_fill.offset(-1isize),
                                               x0 as libc::c_int + 1i32, e,
                                               x0, y_top, x0, y_bottom);
                } else {
                    stbtt__handle_clipped_edge(scanline_fill.offset(-1isize),
                                               0i32, e, x0, y_top, x0,
                                               y_bottom);
                }
            }
        } else {
            let mut x0_0: libc::c_float = (*e).fx;
            let mut dx: libc::c_float = (*e).fdx;
            let mut xb: libc::c_float = x0_0 + dx;
            let mut x_top: libc::c_float = 0.;
            let mut x_bottom: libc::c_float = 0.;
            let mut sy0: libc::c_float = 0.;
            let mut sy1: libc::c_float = 0.;
            let mut dy: libc::c_float = (*e).fdy;
            if (*e).sy <= y_bottom && (*e).ey >= y_top {
            } else {
                __assert_fail(b"e->sy <= y_bottom && e->ey >= y_top\x00" as
                                  *const u8 as *const libc::c_char,
                              b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                                  as *const u8 as *const libc::c_char,
                              1931i32 as libc::c_uint,
                              (*::std::mem::transmute::<&[u8; 86],
                                                        &[libc::c_char; 86]>(b"void stbtt__fill_active_edges_new(float *, float *, int, stbtt__active_edge *, float)\x00")).as_ptr());
            }
            if (*e).sy > y_top {
                x_top = x0_0 + dx * ((*e).sy - y_top);
                sy0 = (*e).sy
            } else { x_top = x0_0; sy0 = y_top }
            if (*e).ey < y_bottom {
                x_bottom = x0_0 + dx * ((*e).ey - y_top);
                sy1 = (*e).ey
            } else { x_bottom = xb; sy1 = y_bottom }
            if x_top >= 0i32 as libc::c_float &&
                   x_bottom >= 0i32 as libc::c_float &&
                   x_top < len as libc::c_float &&
                   x_bottom < len as libc::c_float {
                if x_top as libc::c_int == x_bottom as libc::c_int {
                    let mut height: libc::c_float = 0.;
                    let mut x: libc::c_int = x_top as libc::c_int;
                    height = sy1 - sy0;
                    if x >= 0i32 && x < len {
                    } else {
                        __assert_fail(b"x >= 0 && x < len\x00" as *const u8 as
                                          *const libc::c_char,
                                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                                          as *const u8 as *const libc::c_char,
                                      1959i32 as libc::c_uint,
                                      (*::std::mem::transmute::<&[u8; 86],
                                                                &[libc::c_char; 86]>(b"void stbtt__fill_active_edges_new(float *, float *, int, stbtt__active_edge *, float)\x00")).as_ptr());
                    }
                    *scanline.offset(x as isize) +=
                        (*e).direction *
                            (1i32 as libc::c_float -
                                 (x_top - x as libc::c_float +
                                      (x_bottom - x as libc::c_float)) /
                                     2i32 as libc::c_float) * height;
                    *scanline_fill.offset(x as isize) +=
                        (*e).direction * height
                } else {
                    let mut x_0: libc::c_int = 0;
                    let mut x1: libc::c_int = 0;
                    let mut x2: libc::c_int = 0;
                    let mut y_crossing: libc::c_float = 0.;
                    let mut step: libc::c_float = 0.;
                    let mut sign: libc::c_float = 0.;
                    let mut area: libc::c_float = 0.;
                    if x_top > x_bottom {
                        let mut t: libc::c_float = 0.;
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
                    x1 = x_top as libc::c_int;
                    x2 = x_bottom as libc::c_int;
                    y_crossing =
                        ((x1 + 1i32) as libc::c_float - x0_0) * dy + y_top;
                    sign = (*e).direction;
                    area = sign * (y_crossing - sy0);
                    *scanline.offset(x1 as isize) +=
                        area *
                            (1i32 as libc::c_float -
                                 (x_top - x1 as libc::c_float +
                                      (x1 + 1i32 - x1) as libc::c_float) /
                                     2i32 as libc::c_float);
                    step = sign * dy;
                    x_0 = x1 + 1i32;
                    while x_0 < x2 {
                        *scanline.offset(x_0 as isize) +=
                            area + step / 2i32 as libc::c_float;
                        area += step;
                        x_0 += 1
                    }
                    y_crossing += dy * (x2 - (x1 + 1i32)) as libc::c_float;
                    if fabs(area as libc::c_double) <=
                           1.01f32 as libc::c_double {
                    } else {
                        __assert_fail(b"fabs(area) <= 1.01f\x00" as *const u8
                                          as *const libc::c_char,
                                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                                          as *const u8 as *const libc::c_char,
                                      1996i32 as libc::c_uint,
                                      (*::std::mem::transmute::<&[u8; 86],
                                                                &[libc::c_char; 86]>(b"void stbtt__fill_active_edges_new(float *, float *, int, stbtt__active_edge *, float)\x00")).as_ptr());
                    }
                    *scanline.offset(x2 as isize) +=
                        area +
                            sign *
                                (1i32 as libc::c_float -
                                     ((x2 - x2) as libc::c_float +
                                          (x_bottom - x2 as libc::c_float)) /
                                         2i32 as libc::c_float) *
                                (sy1 - y_crossing);
                    *scanline_fill.offset(x2 as isize) += sign * (sy1 - sy0)
                }
            } else {
                let mut x_1: libc::c_int = 0;
                x_1 = 0i32;
                while x_1 < len {
                    let mut y0: libc::c_float = y_top;
                    let mut x1_0: libc::c_float = x_1 as libc::c_float;
                    let mut x2_0: libc::c_float =
                        (x_1 + 1i32) as libc::c_float;
                    let mut x3: libc::c_float = xb;
                    let mut y3: libc::c_float = y_bottom;
                    let mut y1: libc::c_float = 0.;
                    let mut y2: libc::c_float = 0.;
                    y1 = (x_1 as libc::c_float - x0_0) / dx + y_top;
                    y2 = ((x_1 + 1i32) as libc::c_float - x0_0) / dx + y_top;
                    if x0_0 < x1_0 && x3 > x2_0 {
                        stbtt__handle_clipped_edge(scanline, x_1, e, x0_0, y0,
                                                   x1_0, y1);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x1_0, y1,
                                                   x2_0, y2);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x2_0, y2,
                                                   x3, y3);
                    } else if x3 < x1_0 && x0_0 > x2_0 {
                        stbtt__handle_clipped_edge(scanline, x_1, e, x0_0, y0,
                                                   x2_0, y2);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x2_0, y2,
                                                   x1_0, y1);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x1_0, y1,
                                                   x3, y3);
                    } else if x0_0 < x1_0 && x3 > x1_0 {
                        stbtt__handle_clipped_edge(scanline, x_1, e, x0_0, y0,
                                                   x1_0, y1);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x1_0, y1,
                                                   x3, y3);
                    } else if x3 < x1_0 && x0_0 > x1_0 {
                        stbtt__handle_clipped_edge(scanline, x_1, e, x0_0, y0,
                                                   x1_0, y1);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x1_0, y1,
                                                   x3, y3);
                    } else if x0_0 < x2_0 && x3 > x2_0 {
                        stbtt__handle_clipped_edge(scanline, x_1, e, x0_0, y0,
                                                   x2_0, y2);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x2_0, y2,
                                                   x3, y3);
                    } else if x3 < x2_0 && x0_0 > x2_0 {
                        stbtt__handle_clipped_edge(scanline, x_1, e, x0_0, y0,
                                                   x2_0, y2);
                        stbtt__handle_clipped_edge(scanline, x_1, e, x2_0, y2,
                                                   x3, y3);
                    } else {
                        stbtt__handle_clipped_edge(scanline, x_1, e, x0_0, y0,
                                                   x3, y3);
                    }
                    x_1 += 1
                }
            }
        }
        e = (*e).next
    };
}
// the edge passed in here does not cross the vertical line at x or the vertical line at x+1
// (i.e. it has already been clipped to those)
unsafe extern "C" fn stbtt__handle_clipped_edge(mut scanline:
                                                    *mut libc::c_float,
                                                mut x: libc::c_int,
                                                mut e:
                                                    *mut stbtt__active_edge,
                                                mut x0: libc::c_float,
                                                mut y0: libc::c_float,
                                                mut x1: libc::c_float,
                                                mut y1: libc::c_float) {
    if y0 == y1 { return }
    if y0 < y1 {
    } else {
        __assert_fail(b"y0 < y1\x00" as *const u8 as *const libc::c_char,
                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00" as
                          *const u8 as *const libc::c_char,
                      1870i32 as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 96],
                                                &[libc::c_char; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
    }
    if (*e).sy <= (*e).ey {
    } else {
        __assert_fail(b"e->sy <= e->ey\x00" as *const u8 as
                          *const libc::c_char,
                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00" as
                          *const u8 as *const libc::c_char,
                      1871i32 as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 96],
                                                &[libc::c_char; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
    }
    if y0 > (*e).ey { return }
    if y1 < (*e).sy { return }
    if y0 < (*e).sy {
        x0 += (x1 - x0) * ((*e).sy - y0) / (y1 - y0);
        y0 = (*e).sy
    }
    if y1 > (*e).ey {
        x1 += (x1 - x0) * ((*e).ey - y1) / (y1 - y0);
        y1 = (*e).ey
    }
    if x0 == x as libc::c_float {
        if x1 <= (x + 1i32) as libc::c_float {
        } else {
            __assert_fail(b"x1 <= x+1\x00" as *const u8 as
                              *const libc::c_char,
                          b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                              as *const u8 as *const libc::c_char,
                          1884i32 as libc::c_uint,
                          (*::std::mem::transmute::<&[u8; 96],
                                                    &[libc::c_char; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
        }
    } else if x0 == (x + 1i32) as libc::c_float {
        if x1 >= x as libc::c_float {
        } else {
            __assert_fail(b"x1 >= x\x00" as *const u8 as *const libc::c_char,
                          b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                              as *const u8 as *const libc::c_char,
                          1886i32 as libc::c_uint,
                          (*::std::mem::transmute::<&[u8; 96],
                                                    &[libc::c_char; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
        }
    } else if x0 <= x as libc::c_float {
        if x1 <= x as libc::c_float {
        } else {
            __assert_fail(b"x1 <= x\x00" as *const u8 as *const libc::c_char,
                          b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                              as *const u8 as *const libc::c_char,
                          1888i32 as libc::c_uint,
                          (*::std::mem::transmute::<&[u8; 96],
                                                    &[libc::c_char; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
        }
    } else if x0 >= (x + 1i32) as libc::c_float {
        if x1 >= (x + 1i32) as libc::c_float {
        } else {
            __assert_fail(b"x1 >= x+1\x00" as *const u8 as
                              *const libc::c_char,
                          b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                              as *const u8 as *const libc::c_char,
                          1890i32 as libc::c_uint,
                          (*::std::mem::transmute::<&[u8; 96],
                                                    &[libc::c_char; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
        }
    } else if x1 >= x as libc::c_float && x1 <= (x + 1i32) as libc::c_float {
    } else {
        __assert_fail(b"x1 >= x && x1 <= x+1\x00" as *const u8 as
                          *const libc::c_char,
                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00" as
                          *const u8 as *const libc::c_char,
                      1892i32 as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 96],
                                                &[libc::c_char; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
    }
    if x0 <= x as libc::c_float && x1 <= x as libc::c_float {
        *scanline.offset(x as isize) += (*e).direction * (y1 - y0)
    } else if !(x0 >= (x + 1i32) as libc::c_float &&
                    x1 >= (x + 1i32) as libc::c_float) {
        if x0 >= x as libc::c_float && x0 <= (x + 1i32) as libc::c_float &&
               x1 >= x as libc::c_float && x1 <= (x + 1i32) as libc::c_float {
        } else {
            __assert_fail(b"x0 >= x && x0 <= x+1 && x1 >= x && x1 <= x+1\x00"
                              as *const u8 as *const libc::c_char,
                          b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                              as *const u8 as *const libc::c_char,
                          1899i32 as libc::c_uint,
                          (*::std::mem::transmute::<&[u8; 96],
                                                    &[libc::c_char; 96]>(b"void stbtt__handle_clipped_edge(float *, int, stbtt__active_edge *, float, float, float, float)\x00")).as_ptr());
        }
        *scanline.offset(x as isize) +=
            (*e).direction * (y1 - y0) *
                (1i32 as libc::c_float -
                     (x0 - x as libc::c_float + (x1 - x as libc::c_float)) /
                         2i32 as libc::c_float)
    };
}
unsafe extern "C" fn stbtt__new_active(mut hh: *mut stbtt__hheap,
                                       mut e: *mut stbtt__edge,
                                       mut off_x: libc::c_int,
                                       mut start_point: libc::c_float,
                                       mut userdata: *mut libc::c_void)
 -> *mut stbtt__active_edge {
    let mut z: *mut stbtt__active_edge =
        stbtt__hheap_alloc(hh,
                           ::std::mem::size_of::<stbtt__active_edge>() as
                               libc::c_ulong, userdata) as
            *mut stbtt__active_edge;
    let mut dxdy: libc::c_float = ((*e).x1 - (*e).x0) / ((*e).y1 - (*e).y0);
    if !z.is_null() {
    } else {
        __assert_fail(b"z != ((void*)0)\x00" as *const u8 as
                          *const libc::c_char,
                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00" as
                          *const u8 as *const libc::c_char,
                      1700i32 as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 89],
                                                &[libc::c_char; 89]>(b"stbtt__active_edge *stbtt__new_active(stbtt__hheap *, stbtt__edge *, int, float, void *)\x00")).as_ptr());
    }
    if z.is_null() { return z }
    (*z).fdx = dxdy;
    (*z).fdy = if dxdy != 0.0f32 { 1.0f32 / dxdy } else { 0.0f32 };
    (*z).fx = (*e).x0 + dxdy * (start_point - (*e).y0);
    (*z).fx -= off_x as libc::c_float;
    (*z).direction = if 0 != (*e).invert { 1.0f32 } else { -1.0f32 };
    (*z).sy = (*e).y0;
    (*z).ey = (*e).y1;
    (*z).next = 0 as *mut stbtt__active_edge;
    return z;
}
unsafe extern "C" fn stbtt__hheap_alloc(mut hh: *mut stbtt__hheap,
                                        mut size: size_t,
                                        mut userdata: *mut libc::c_void)
 -> *mut libc::c_void {
    if !(*hh).first_free.is_null() {
        let mut p: *mut libc::c_void = (*hh).first_free;
        (*hh).first_free = *(p as *mut *mut libc::c_void);
        return p
    } else {
        if (*hh).num_remaining_in_head_chunk == 0i32 {
            let mut count: libc::c_int =
                if size < 32i32 as libc::c_ulong {
                    2000i32
                } else if size < 128i32 as libc::c_ulong {
                    800i32
                } else { 100i32 };
            let mut c: *mut stbtt__hheap_chunk =
                fons__tmpalloc((::std::mem::size_of::<stbtt__hheap_chunk>() as
                                    libc::c_ulong).wrapping_add(size.wrapping_mul(count
                                                                                      as
                                                                                      libc::c_ulong)),
                               userdata) as *mut stbtt__hheap_chunk;
            if c.is_null() { return 0 as *mut libc::c_void }
            (*c).next = (*hh).head;
            (*hh).head = c;
            (*hh).num_remaining_in_head_chunk = count
        }
        (*hh).num_remaining_in_head_chunk -= 1;
        return ((*hh).head as
                    *mut libc::c_char).offset(size.wrapping_mul((*hh).num_remaining_in_head_chunk
                                                                    as
                                                                    libc::c_ulong)
                                                  as isize) as
                   *mut libc::c_void
    };
}
unsafe extern "C" fn stbtt__hheap_free(mut hh: *mut stbtt__hheap,
                                       mut p: *mut libc::c_void) {
    let ref mut fresh20 = *(p as *mut *mut libc::c_void);
    *fresh20 = (*hh).first_free;
    (*hh).first_free = p;
}
unsafe extern "C" fn stbtt__sort_edges(mut p: *mut stbtt__edge,
                                       mut n: libc::c_int) {
    stbtt__sort_edges_quicksort(p, n);
    stbtt__sort_edges_ins_sort(p, n);
}
unsafe extern "C" fn stbtt__sort_edges_ins_sort(mut p: *mut stbtt__edge,
                                                mut n: libc::c_int) {
    let mut i: libc::c_int = 0;
    let mut j: libc::c_int = 0;
    i = 1i32;
    while i < n {
        let mut t: stbtt__edge = *p.offset(i as isize);
        let mut a: *mut stbtt__edge = &mut t;
        j = i;
        while j > 0i32 {
            let mut b: *mut stbtt__edge =
                &mut *p.offset((j - 1i32) as isize) as *mut stbtt__edge;
            let mut c: libc::c_int = ((*a).y0 < (*b).y0) as libc::c_int;
            if 0 == c { break ; }
            *p.offset(j as isize) = *p.offset((j - 1i32) as isize);
            j -= 1
        }
        if i != j { *p.offset(j as isize) = t }
        i += 1
    };
}
unsafe extern "C" fn stbtt__sort_edges_quicksort(mut p: *mut stbtt__edge,
                                                 mut n: libc::c_int) {
    while n > 12i32 {
        let mut t: stbtt__edge =
            stbtt__edge{x0: 0., y0: 0., x1: 0., y1: 0., invert: 0,};
        let mut c01: libc::c_int = 0;
        let mut c12: libc::c_int = 0;
        let mut c: libc::c_int = 0;
        let mut m: libc::c_int = 0;
        let mut i: libc::c_int = 0;
        let mut j: libc::c_int = 0;
        m = n >> 1i32;
        c01 =
            ((*p.offset(0isize)).y0 < (*p.offset(m as isize)).y0) as
                libc::c_int;
        c12 =
            ((*p.offset(m as isize)).y0 < (*p.offset((n - 1i32) as isize)).y0)
                as libc::c_int;
        if c01 != c12 {
            let mut z: libc::c_int = 0;
            c =
                ((*p.offset(0isize)).y0 < (*p.offset((n - 1i32) as isize)).y0)
                    as libc::c_int;
            z = if c == c12 { 0i32 } else { n - 1i32 };
            t = *p.offset(z as isize);
            *p.offset(z as isize) = *p.offset(m as isize);
            *p.offset(m as isize) = t
        }
        t = *p.offset(0isize);
        *p.offset(0isize) = *p.offset(m as isize);
        *p.offset(m as isize) = t;
        i = 1i32;
        j = n - 1i32;
        loop  {
            while (*p.offset(i as isize)).y0 < (*p.offset(0isize)).y0 {
                i += 1
            }
            while (*p.offset(0isize)).y0 < (*p.offset(j as isize)).y0 {
                j -= 1
            }
            /* make sure we haven't crossed */
            if i >= j { break ; }
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
    };
}

pub unsafe extern "C" fn stbtt_GetGlyphBitmapBoxSubpixel(mut font:
                                                             *const stbtt_fontinfo,
                                                         mut glyph:
                                                             libc::c_int,
                                                         mut scale_x:
                                                             libc::c_float,
                                                         mut scale_y:
                                                             libc::c_float,
                                                         mut shift_x:
                                                             libc::c_float,
                                                         mut shift_y:
                                                             libc::c_float,
                                                         mut ix0:
                                                             *mut libc::c_int,
                                                         mut iy0:
                                                             *mut libc::c_int,
                                                         mut ix1:
                                                             *mut libc::c_int,
                                                         mut iy1:
                                                             *mut libc::c_int) {
    // =0 suppresses compiler warning
    let mut x0: libc::c_int = 0i32;
    let mut y0: libc::c_int = 0i32;
    let mut x1: libc::c_int = 0;
    let mut y1: libc::c_int = 0;
    if 0 == stbtt_GetGlyphBox(font, glyph, &mut x0, &mut y0, &mut x1, &mut y1)
       {
        if !ix0.is_null() { *ix0 = 0i32 }
        if !iy0.is_null() { *iy0 = 0i32 }
        if !ix1.is_null() { *ix1 = 0i32 }
        if !iy1.is_null() { *iy1 = 0i32 }
    } else {
        if !ix0.is_null() {
            *ix0 =
                floor((x0 as libc::c_float * scale_x + shift_x) as
                          libc::c_double) as libc::c_int
        }
        if !iy0.is_null() {
            *iy0 =
                floor((-y1 as libc::c_float * scale_y + shift_y) as
                          libc::c_double) as libc::c_int
        }
        if !ix1.is_null() {
            *ix1 =
                ceil((x1 as libc::c_float * scale_x + shift_x) as
                         libc::c_double) as libc::c_int
        }
        if !iy1.is_null() {
            *iy1 =
                ceil((-y0 as libc::c_float * scale_y + shift_y) as
                         libc::c_double) as libc::c_int
        }
    };
}

pub unsafe extern "C" fn stbtt_GetGlyphBox(mut info: *const stbtt_fontinfo,
                                           mut glyph_index: libc::c_int,
                                           mut x0: *mut libc::c_int,
                                           mut y0: *mut libc::c_int,
                                           mut x1: *mut libc::c_int,
                                           mut y1: *mut libc::c_int)
 -> libc::c_int {
    let mut g: libc::c_int = stbtt__GetGlyfOffset(info, glyph_index);
    if g < 0i32 { return 0i32 }
    if !x0.is_null() {
        *x0 =
            ttSHORT((*info).data.offset(g as isize).offset(2isize)) as
                libc::c_int
    }
    if !y0.is_null() {
        *y0 =
            ttSHORT((*info).data.offset(g as isize).offset(4isize)) as
                libc::c_int
    }
    if !x1.is_null() {
        *x1 =
            ttSHORT((*info).data.offset(g as isize).offset(6isize)) as
                libc::c_int
    }
    if !y1.is_null() {
        *y1 =
            ttSHORT((*info).data.offset(g as isize).offset(8isize)) as
                libc::c_int
    }
    return 1i32;
}

pub unsafe extern "C" fn fons__allocGlyph(mut font: *mut FONSfont)
 -> *mut FONSglyph {
    if (*font).nglyphs + 1i32 > (*font).cglyphs {
        (*font).cglyphs =
            if (*font).cglyphs == 0i32 {
                8i32
            } else { (*font).cglyphs * 2i32 };
        (*font).glyphs =
            realloc((*font).glyphs as *mut libc::c_void,
                    (::std::mem::size_of::<FONSglyph>() as
                         libc::c_ulong).wrapping_mul((*font).cglyphs as
                                                         libc::c_ulong)) as
                *mut FONSglyph;
        if (*font).glyphs.is_null() { return 0 as *mut FONSglyph }
    }
    (*font).nglyphs += 1;
    return &mut *(*font).glyphs.offset(((*font).nglyphs - 1i32) as isize) as
               *mut FONSglyph;
}

pub unsafe extern "C" fn fons__tt_buildGlyphBitmap(mut font:
                                                       *mut FONSttFontImpl,
                                                   mut glyph: libc::c_int,
                                                   mut size: libc::c_float,
                                                   mut scale: libc::c_float,
                                                   mut advance:
                                                       *mut libc::c_int,
                                                   mut lsb: *mut libc::c_int,
                                                   mut x0: *mut libc::c_int,
                                                   mut y0: *mut libc::c_int,
                                                   mut x1: *mut libc::c_int,
                                                   mut y1: *mut libc::c_int)
 -> libc::c_int {
    stbtt_GetGlyphHMetrics(&mut (*font).font, glyph, advance, lsb);
    stbtt_GetGlyphBitmapBox(&mut (*font).font, glyph, scale, scale, x0, y0,
                            x1, y1);
    return 1i32;
}

pub unsafe extern "C" fn stbtt_GetGlyphBitmapBox(mut font:
                                                     *const stbtt_fontinfo,
                                                 mut glyph: libc::c_int,
                                                 mut scale_x: libc::c_float,
                                                 mut scale_y: libc::c_float,
                                                 mut ix0: *mut libc::c_int,
                                                 mut iy0: *mut libc::c_int,
                                                 mut ix1: *mut libc::c_int,
                                                 mut iy1: *mut libc::c_int) {
    stbtt_GetGlyphBitmapBoxSubpixel(font, glyph, scale_x, scale_y, 0.0f32,
                                    0.0f32, ix0, iy0, ix1, iy1);
}
// Gets the bounding box of the visible part of the glyph, in unscaled coordinates

pub unsafe extern "C" fn stbtt_GetGlyphHMetrics(mut info:
                                                    *const stbtt_fontinfo,
                                                mut glyph_index: libc::c_int,
                                                mut advanceWidth:
                                                    *mut libc::c_int,
                                                mut leftSideBearing:
                                                    *mut libc::c_int) {
    let mut numOfLongHorMetrics: stbtt_uint16 =
        ttUSHORT((*info).data.offset((*info).hhea as isize).offset(34isize));
    if glyph_index < numOfLongHorMetrics as libc::c_int {
        if !advanceWidth.is_null() {
            *advanceWidth =
                ttSHORT((*info).data.offset((*info).hmtx as
                                                isize).offset((4i32 *
                                                                   glyph_index)
                                                                  as isize))
                    as libc::c_int
        }
        if !leftSideBearing.is_null() {
            *leftSideBearing =
                ttSHORT((*info).data.offset((*info).hmtx as
                                                isize).offset((4i32 *
                                                                   glyph_index)
                                                                  as
                                                                  isize).offset(2isize))
                    as libc::c_int
        }
    } else {
        if !advanceWidth.is_null() {
            *advanceWidth =
                ttSHORT((*info).data.offset((*info).hmtx as
                                                isize).offset((4i32 *
                                                                   (numOfLongHorMetrics
                                                                        as
                                                                        libc::c_int
                                                                        -
                                                                        1i32))
                                                                  as isize))
                    as libc::c_int
        }
        if !leftSideBearing.is_null() {
            *leftSideBearing =
                ttSHORT((*info).data.offset((*info).hmtx as
                                                isize).offset((4i32 *
                                                                   numOfLongHorMetrics
                                                                       as
                                                                       libc::c_int)
                                                                  as
                                                                  isize).offset((2i32
                                                                                     *
                                                                                     (glyph_index
                                                                                          -
                                                                                          numOfLongHorMetrics
                                                                                              as
                                                                                              libc::c_int))
                                                                                    as
                                                                                    isize))
                    as libc::c_int
        }
    };
}

pub unsafe extern "C" fn fons__tt_getPixelHeightScale(mut font:
                                                          *mut FONSttFontImpl,
                                                      mut size: libc::c_float)
 -> libc::c_float {
    return stbtt_ScaleForPixelHeight(&mut (*font).font, size);
}
// If you're going to perform multiple operations on the same character
// and you want a speed-up, call this function with the character you're
// going to process, then use glyph-based functions instead of the
// codepoint-based functions.
// ////////////////////////////////////////////////////////////////////////////
//
// CHARACTER PROPERTIES
//

pub unsafe extern "C" fn stbtt_ScaleForPixelHeight(mut info:
                                                       *const stbtt_fontinfo,
                                                   mut height: libc::c_float)
 -> libc::c_float {
    let mut fheight: libc::c_int =
        ttSHORT((*info).data.offset((*info).hhea as isize).offset(4isize)) as
            libc::c_int -
            ttSHORT((*info).data.offset((*info).hhea as isize).offset(6isize))
                as libc::c_int;
    return height as libc::c_float / fheight as libc::c_float;
}

pub unsafe extern "C" fn fons__tt_getGlyphIndex(mut font: *mut FONSttFontImpl,
                                                mut codepoint: libc::c_int)
 -> libc::c_int {
    return stbtt_FindGlyphIndex(&mut (*font).font, codepoint);
}
// Given an offset into the file that defines a font, this function builds
// the necessary cached info for the rest of the system. You must allocate
// the stbtt_fontinfo yourself, and stbtt_InitFont will fill it out. You don't
// need to do anything special to free it, because the contents are pure
// value data with no additional data structures. Returns 0 on failure.
// ////////////////////////////////////////////////////////////////////////////
//
// CHARACTER TO GLYPH-INDEX CONVERSIOn

pub unsafe extern "C" fn stbtt_FindGlyphIndex(mut info: *const stbtt_fontinfo,
                                              mut unicode_codepoint:
                                                  libc::c_int)
 -> libc::c_int {
    let mut data: *mut stbtt_uint8 = (*info).data;
    let mut index_map: stbtt_uint32 = (*info).index_map as stbtt_uint32;
    let mut format: stbtt_uint16 =
        ttUSHORT(data.offset(index_map as isize).offset(0isize));
    if format as libc::c_int == 0i32 {
        let mut bytes: stbtt_int32 =
            ttUSHORT(data.offset(index_map as isize).offset(2isize)) as
                stbtt_int32;
        if unicode_codepoint < bytes - 6i32 {
            return *(data.offset(index_map as
                                     isize).offset(6isize).offset(unicode_codepoint
                                                                      as
                                                                      isize)
                         as *mut stbtt_uint8) as libc::c_int
        }
        return 0i32
    } else {
        if format as libc::c_int == 6i32 {
            let mut first: stbtt_uint32 =
                ttUSHORT(data.offset(index_map as isize).offset(6isize)) as
                    stbtt_uint32;
            let mut count: stbtt_uint32 =
                ttUSHORT(data.offset(index_map as isize).offset(8isize)) as
                    stbtt_uint32;
            if unicode_codepoint as stbtt_uint32 >= first &&
                   (unicode_codepoint as stbtt_uint32) <
                       first.wrapping_add(count) {
                return ttUSHORT(data.offset(index_map as
                                                isize).offset(10isize).offset((unicode_codepoint
                                                                                   as
                                                                                   libc::c_uint).wrapping_sub(first).wrapping_mul(2i32
                                                                                                                                      as
                                                                                                                                      libc::c_uint)
                                                                                  as
                                                                                  isize))
                           as libc::c_int
            }
            return 0i32
        } else {
            if format as libc::c_int == 2i32 {
                __assert_fail(b"0\x00" as *const u8 as *const libc::c_char,
                              b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                                  as *const u8 as *const libc::c_char,
                              1094i32 as libc::c_uint,
                              (*::std::mem::transmute::<&[u8; 54],
                                                        &[libc::c_char; 54]>(b"int stbtt_FindGlyphIndex(const stbtt_fontinfo *, int)\x00")).as_ptr());
                return 0i32
            } else {
                if format as libc::c_int == 4i32 {
                    let mut segcount: stbtt_uint16 =
                        (ttUSHORT(data.offset(index_map as
                                                  isize).offset(6isize)) as
                             libc::c_int >> 1i32) as stbtt_uint16;
                    let mut searchRange: stbtt_uint16 =
                        (ttUSHORT(data.offset(index_map as
                                                  isize).offset(8isize)) as
                             libc::c_int >> 1i32) as stbtt_uint16;
                    let mut entrySelector: stbtt_uint16 =
                        ttUSHORT(data.offset(index_map as
                                                 isize).offset(10isize));
                    let mut rangeShift: stbtt_uint16 =
                        (ttUSHORT(data.offset(index_map as
                                                  isize).offset(12isize)) as
                             libc::c_int >> 1i32) as stbtt_uint16;
                    let mut endCount: stbtt_uint32 =
                        index_map.wrapping_add(14i32 as libc::c_uint);
                    let mut search: stbtt_uint32 = endCount;
                    if unicode_codepoint > 0xffffi32 { return 0i32 }
                    if unicode_codepoint >=
                           ttUSHORT(data.offset(search as
                                                    isize).offset((rangeShift
                                                                       as
                                                                       libc::c_int
                                                                       * 2i32)
                                                                      as
                                                                      isize))
                               as libc::c_int {
                        search =
                            (search as
                                 libc::c_uint).wrapping_add((rangeShift as
                                                                 libc::c_int *
                                                                 2i32) as
                                                                libc::c_uint)
                                as stbtt_uint32 as stbtt_uint32
                    }
                    search =
                        (search as
                             libc::c_uint).wrapping_sub(2i32 as libc::c_uint)
                            as stbtt_uint32 as stbtt_uint32;
                    while 0 != entrySelector {
                        let mut end: stbtt_uint16 = 0;
                        searchRange =
                            (searchRange as libc::c_int >> 1i32) as
                                stbtt_uint16;
                        end =
                            ttUSHORT(data.offset(search as
                                                     isize).offset((searchRange
                                                                        as
                                                                        libc::c_int
                                                                        *
                                                                        2i32)
                                                                       as
                                                                       isize));
                        if unicode_codepoint > end as libc::c_int {
                            search =
                                (search as
                                     libc::c_uint).wrapping_add((searchRange
                                                                     as
                                                                     libc::c_int
                                                                     * 2i32)
                                                                    as
                                                                    libc::c_uint)
                                    as stbtt_uint32 as stbtt_uint32
                        }
                        entrySelector = entrySelector.wrapping_sub(1)
                    }
                    search =
                        (search as
                             libc::c_uint).wrapping_add(2i32 as libc::c_uint)
                            as stbtt_uint32 as stbtt_uint32;
                    let mut offset: stbtt_uint16 = 0;
                    let mut start: stbtt_uint16 = 0;
                    let mut item: stbtt_uint16 =
                        (search.wrapping_sub(endCount) >> 1i32) as
                            stbtt_uint16;
                    if unicode_codepoint <=
                           ttUSHORT(data.offset(endCount as
                                                    isize).offset((2i32 *
                                                                       item as
                                                                           libc::c_int)
                                                                      as
                                                                      isize))
                               as libc::c_int {
                    } else {
                        __assert_fail(b"unicode_codepoint <= ttUSHORT(data + endCount + 2*item)\x00"
                                          as *const u8 as *const libc::c_char,
                                      b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00"
                                          as *const u8 as *const libc::c_char,
                                      1130i32 as libc::c_uint,
                                      (*::std::mem::transmute::<&[u8; 54],
                                                                &[libc::c_char; 54]>(b"int stbtt_FindGlyphIndex(const stbtt_fontinfo *, int)\x00")).as_ptr());
                    }
                    start =
                        ttUSHORT(data.offset(index_map as
                                                 isize).offset(14isize).offset((segcount
                                                                                    as
                                                                                    libc::c_int
                                                                                    *
                                                                                    2i32)
                                                                                   as
                                                                                   isize).offset(2isize).offset((2i32
                                                                                                                     *
                                                                                                                     item
                                                                                                                         as
                                                                                                                         libc::c_int)
                                                                                                                    as
                                                                                                                    isize));
                    if unicode_codepoint < start as libc::c_int {
                        return 0i32
                    }
                    offset =
                        ttUSHORT(data.offset(index_map as
                                                 isize).offset(14isize).offset((segcount
                                                                                    as
                                                                                    libc::c_int
                                                                                    *
                                                                                    6i32)
                                                                                   as
                                                                                   isize).offset(2isize).offset((2i32
                                                                                                                     *
                                                                                                                     item
                                                                                                                         as
                                                                                                                         libc::c_int)
                                                                                                                    as
                                                                                                                    isize));
                    if offset as libc::c_int == 0i32 {
                        return (unicode_codepoint +
                                    ttSHORT(data.offset(index_map as
                                                            isize).offset(14isize).offset((segcount
                                                                                               as
                                                                                               libc::c_int
                                                                                               *
                                                                                               4i32)
                                                                                              as
                                                                                              isize).offset(2isize).offset((2i32
                                                                                                                                *
                                                                                                                                item
                                                                                                                                    as
                                                                                                                                    libc::c_int)
                                                                                                                               as
                                                                                                                               isize))
                                        as libc::c_int) as stbtt_uint16 as
                                   libc::c_int
                    }
                    return ttUSHORT(data.offset(offset as libc::c_int as
                                                    isize).offset(((unicode_codepoint
                                                                        -
                                                                        start
                                                                            as
                                                                            libc::c_int)
                                                                       * 2i32)
                                                                      as
                                                                      isize).offset(index_map
                                                                                        as
                                                                                        isize).offset(14isize).offset((segcount
                                                                                                                           as
                                                                                                                           libc::c_int
                                                                                                                           *
                                                                                                                           6i32)
                                                                                                                          as
                                                                                                                          isize).offset(2isize).offset((2i32
                                                                                                                                                            *
                                                                                                                                                            item
                                                                                                                                                                as
                                                                                                                                                                libc::c_int)
                                                                                                                                                           as
                                                                                                                                                           isize))
                               as libc::c_int
                } else {
                    if format as libc::c_int == 12i32 ||
                           format as libc::c_int == 13i32 {
                        let mut ngroups: stbtt_uint32 =
                            ttULONG(data.offset(index_map as
                                                    isize).offset(12isize));
                        let mut low: stbtt_int32 = 0;
                        let mut high: stbtt_int32 = 0;
                        low = 0i32;
                        high = ngroups as stbtt_int32;
                        while low < high {
                            let mut mid: stbtt_int32 =
                                low + (high - low >> 1i32);
                            let mut start_char: stbtt_uint32 =
                                ttULONG(data.offset(index_map as
                                                        isize).offset(16isize).offset((mid
                                                                                           *
                                                                                           12i32)
                                                                                          as
                                                                                          isize));
                            let mut end_char: stbtt_uint32 =
                                ttULONG(data.offset(index_map as
                                                        isize).offset(16isize).offset((mid
                                                                                           *
                                                                                           12i32)
                                                                                          as
                                                                                          isize).offset(4isize));
                            if (unicode_codepoint as stbtt_uint32) <
                                   start_char {
                                high = mid
                            } else if unicode_codepoint as stbtt_uint32 >
                                          end_char {
                                low = mid + 1i32
                            } else {
                                let mut start_glyph: stbtt_uint32 =
                                    ttULONG(data.offset(index_map as
                                                            isize).offset(16isize).offset((mid
                                                                                               *
                                                                                               12i32)
                                                                                              as
                                                                                              isize).offset(8isize));
                                if format as libc::c_int == 12i32 {
                                    return start_glyph.wrapping_add(unicode_codepoint
                                                                        as
                                                                        libc::c_uint).wrapping_sub(start_char)
                                               as libc::c_int
                                } else { return start_glyph as libc::c_int }
                            }
                        }
                        return 0i32
                    }
                }
            }
        }
    }
    __assert_fail(b"0\x00" as *const u8 as *const libc::c_char,
                  b"/home/lain/WORK/oni2d/nvg/src/stb_truetype.h\x00" as
                      *const u8 as *const libc::c_char,
                  1165i32 as libc::c_uint,
                  (*::std::mem::transmute::<&[u8; 54],
                                            &[libc::c_char; 54]>(b"int stbtt_FindGlyphIndex(const stbtt_fontinfo *, int)\x00")).as_ptr());
    return 0i32;
}

pub unsafe extern "C" fn fons__hashint(mut a: libc::c_uint) -> libc::c_uint {
    a = a.wrapping_add(!(a << 15i32));
    a ^= a >> 10i32;
    a = a.wrapping_add(a << 3i32);
    a ^= a >> 6i32;
    a = a.wrapping_add(!(a << 11i32));
    a ^= a >> 16i32;
    return a;
}
// empty
// STB_TRUETYPE_IMPLEMENTATION
// Copyright (c) 2008-2010 Bjoern Hoehrmann <bjoern@hoehrmann.de>
// See http://bjoern.hoehrmann.de/utf-8/decoder/dfa/ for details.

pub unsafe extern "C" fn fons__decutf8(mut state: *mut libc::c_uint,
                                       mut codep: *mut libc::c_uint,
                                       mut byte: libc::c_uint)
 -> libc::c_uint {
    static mut utf8d: [libc::c_uchar; 364] =
        [0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         0i32 as libc::c_uchar, 0i32 as libc::c_uchar, 1i32 as libc::c_uchar,
         1i32 as libc::c_uchar, 1i32 as libc::c_uchar, 1i32 as libc::c_uchar,
         1i32 as libc::c_uchar, 1i32 as libc::c_uchar, 1i32 as libc::c_uchar,
         1i32 as libc::c_uchar, 1i32 as libc::c_uchar, 1i32 as libc::c_uchar,
         1i32 as libc::c_uchar, 1i32 as libc::c_uchar, 1i32 as libc::c_uchar,
         1i32 as libc::c_uchar, 1i32 as libc::c_uchar, 1i32 as libc::c_uchar,
         9i32 as libc::c_uchar, 9i32 as libc::c_uchar, 9i32 as libc::c_uchar,
         9i32 as libc::c_uchar, 9i32 as libc::c_uchar, 9i32 as libc::c_uchar,
         9i32 as libc::c_uchar, 9i32 as libc::c_uchar, 9i32 as libc::c_uchar,
         9i32 as libc::c_uchar, 9i32 as libc::c_uchar, 9i32 as libc::c_uchar,
         9i32 as libc::c_uchar, 9i32 as libc::c_uchar, 9i32 as libc::c_uchar,
         9i32 as libc::c_uchar, 7i32 as libc::c_uchar, 7i32 as libc::c_uchar,
         7i32 as libc::c_uchar, 7i32 as libc::c_uchar, 7i32 as libc::c_uchar,
         7i32 as libc::c_uchar, 7i32 as libc::c_uchar, 7i32 as libc::c_uchar,
         7i32 as libc::c_uchar, 7i32 as libc::c_uchar, 7i32 as libc::c_uchar,
         7i32 as libc::c_uchar, 7i32 as libc::c_uchar, 7i32 as libc::c_uchar,
         7i32 as libc::c_uchar, 7i32 as libc::c_uchar, 7i32 as libc::c_uchar,
         7i32 as libc::c_uchar, 7i32 as libc::c_uchar, 7i32 as libc::c_uchar,
         7i32 as libc::c_uchar, 7i32 as libc::c_uchar, 7i32 as libc::c_uchar,
         7i32 as libc::c_uchar, 7i32 as libc::c_uchar, 7i32 as libc::c_uchar,
         7i32 as libc::c_uchar, 7i32 as libc::c_uchar, 7i32 as libc::c_uchar,
         7i32 as libc::c_uchar, 7i32 as libc::c_uchar, 7i32 as libc::c_uchar,
         8i32 as libc::c_uchar, 8i32 as libc::c_uchar, 2i32 as libc::c_uchar,
         2i32 as libc::c_uchar, 2i32 as libc::c_uchar, 2i32 as libc::c_uchar,
         2i32 as libc::c_uchar, 2i32 as libc::c_uchar, 2i32 as libc::c_uchar,
         2i32 as libc::c_uchar, 2i32 as libc::c_uchar, 2i32 as libc::c_uchar,
         2i32 as libc::c_uchar, 2i32 as libc::c_uchar, 2i32 as libc::c_uchar,
         2i32 as libc::c_uchar, 2i32 as libc::c_uchar, 2i32 as libc::c_uchar,
         2i32 as libc::c_uchar, 2i32 as libc::c_uchar, 2i32 as libc::c_uchar,
         2i32 as libc::c_uchar, 2i32 as libc::c_uchar, 2i32 as libc::c_uchar,
         2i32 as libc::c_uchar, 2i32 as libc::c_uchar, 2i32 as libc::c_uchar,
         2i32 as libc::c_uchar, 2i32 as libc::c_uchar, 2i32 as libc::c_uchar,
         2i32 as libc::c_uchar, 2i32 as libc::c_uchar, 10i32 as libc::c_uchar,
         3i32 as libc::c_uchar, 3i32 as libc::c_uchar, 3i32 as libc::c_uchar,
         3i32 as libc::c_uchar, 3i32 as libc::c_uchar, 3i32 as libc::c_uchar,
         3i32 as libc::c_uchar, 3i32 as libc::c_uchar, 3i32 as libc::c_uchar,
         3i32 as libc::c_uchar, 3i32 as libc::c_uchar, 3i32 as libc::c_uchar,
         4i32 as libc::c_uchar, 3i32 as libc::c_uchar, 3i32 as libc::c_uchar,
         11i32 as libc::c_uchar, 6i32 as libc::c_uchar, 6i32 as libc::c_uchar,
         6i32 as libc::c_uchar, 5i32 as libc::c_uchar, 8i32 as libc::c_uchar,
         8i32 as libc::c_uchar, 8i32 as libc::c_uchar, 8i32 as libc::c_uchar,
         8i32 as libc::c_uchar, 8i32 as libc::c_uchar, 8i32 as libc::c_uchar,
         8i32 as libc::c_uchar, 8i32 as libc::c_uchar, 8i32 as libc::c_uchar,
         8i32 as libc::c_uchar, 0i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         24i32 as libc::c_uchar, 36i32 as libc::c_uchar,
         60i32 as libc::c_uchar, 96i32 as libc::c_uchar,
         84i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         48i32 as libc::c_uchar, 72i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 0i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 24i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 24i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 24i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 24i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 24i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 24i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 36i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 36i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 36i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 36i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 36i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 36i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar,
         12i32 as libc::c_uchar, 12i32 as libc::c_uchar];
    // The first part of the table maps bytes to character classes that
		// to reduce the size of the transition table and create bitmasks.
    // The second part is a transition table that maps a combination
		// of a state of the automaton and a character class to a state.
    let mut type_0: libc::c_uint = utf8d[byte as usize] as libc::c_uint;
    *codep =
        if *state != 0i32 as libc::c_uint {
            byte & 0x3fu32 | *codep << 6i32
        } else { (0xffi32 >> type_0) as libc::c_uint & byte };
    *state =
        utf8d[(256i32 as
                   libc::c_uint).wrapping_add(*state).wrapping_add(type_0) as
                  usize] as libc::c_uint;
    return *state;
}

pub unsafe extern "C" fn fons__getVertAlign(mut stash: *mut FONScontext,
                                            mut font: *mut FONSfont,
                                            mut align: libc::c_int,
                                            mut isize: libc::c_short)
 -> libc::c_float {
    if 0 !=
           (*stash).params.flags as libc::c_int &
               FONS_ZERO_TOPLEFT as libc::c_int {
        if 0 != align & FONS_ALIGN_TOP as libc::c_int {
            return (*font).ascender * isize as libc::c_float / 10.0f32
        } else {
            if 0 != align & FONS_ALIGN_MIDDLE as libc::c_int {
                return ((*font).ascender + (*font).descender) / 2.0f32 *
                           isize as libc::c_float / 10.0f32
            } else {
                if 0 != align & FONS_ALIGN_BASELINE as libc::c_int {
                    return 0.0f32
                } else {
                    if 0 != align & FONS_ALIGN_BOTTOM as libc::c_int {
                        return (*font).descender * isize as libc::c_float /
                                   10.0f32
                    }
                }
            }
        }
    } else if 0 != align & FONS_ALIGN_TOP as libc::c_int {
        return -(*font).ascender * isize as libc::c_float / 10.0f32
    } else {
        if 0 != align & FONS_ALIGN_MIDDLE as libc::c_int {
            return -((*font).ascender + (*font).descender) / 2.0f32 *
                       isize as libc::c_float / 10.0f32
        } else {
            if 0 != align & FONS_ALIGN_BASELINE as libc::c_int {
                return 0.0f32
            } else {
                if 0 != align & FONS_ALIGN_BOTTOM as libc::c_int {
                    return -(*font).descender * isize as libc::c_float /
                               10.0f32
                }
            }
        }
    }
    return 0.0f64 as libc::c_float;
}
// Measure text

pub unsafe extern "C" fn fonsTextBounds(mut stash: *mut FONScontext,
                                        mut x: libc::c_float,
                                        mut y: libc::c_float,
                                        mut str: *const libc::c_char,
                                        mut end: *const libc::c_char,
                                        mut bounds: *mut libc::c_float)
 -> libc::c_float {
    let mut state: *mut FONSstate = fons__getState(stash);
    let mut codepoint: libc::c_uint = 0;
    let mut utf8state: libc::c_uint = 0i32 as libc::c_uint;
    let mut q: FONSquad =
        FONSquad{x0: 0.,
                 y0: 0.,
                 s0: 0.,
                 t0: 0.,
                 x1: 0.,
                 y1: 0.,
                 s1: 0.,
                 t1: 0.,};
    let mut glyph: *mut FONSglyph = 0 as *mut FONSglyph;
    let mut prevGlyphIndex: libc::c_int = -1i32;
    let mut isize: libc::c_short = ((*state).size * 10.0f32) as libc::c_short;
    let mut iblur: libc::c_short = (*state).blur as libc::c_short;
    let mut scale: libc::c_float = 0.;
    let mut font: *mut FONSfont = 0 as *mut FONSfont;
    let mut startx: libc::c_float = 0.;
    let mut advance: libc::c_float = 0.;
    let mut minx: libc::c_float = 0.;
    let mut miny: libc::c_float = 0.;
    let mut maxx: libc::c_float = 0.;
    let mut maxy: libc::c_float = 0.;
    if stash.is_null() { return 0i32 as libc::c_float }
    if (*state).font < 0i32 || (*state).font >= (*stash).nfonts {
        return 0i32 as libc::c_float
    }
    font = *(*stash).fonts.offset((*state).font as isize);
    if (*font).data.is_null() { return 0i32 as libc::c_float }
    scale =
        fons__tt_getPixelHeightScale(&mut (*font).font,
                                     isize as libc::c_float / 10.0f32);
    y += fons__getVertAlign(stash, font, (*state).align, isize);
    maxx = x;
    minx = maxx;
    maxy = y;
    miny = maxy;
    startx = x;
    if end.is_null() { end = str.offset(strlen(str) as isize) }
    while str != end {
        if !(0 !=
                 fons__decutf8(&mut utf8state, &mut codepoint,
                               *(str as *const libc::c_uchar) as
                                   libc::c_uint)) {
            glyph =
                fons__getGlyph(stash, font, codepoint, isize, iblur,
                               FONS_GLYPH_BITMAP_OPTIONAL as libc::c_int);
            if !glyph.is_null() {
                fons__getQuad(stash, font, prevGlyphIndex, glyph, scale,
                              (*state).spacing, &mut x, &mut y, &mut q);
                if q.x0 < minx { minx = q.x0 }
                if q.x1 > maxx { maxx = q.x1 }
                if 0 !=
                       (*stash).params.flags as libc::c_int &
                           FONS_ZERO_TOPLEFT as libc::c_int {
                    if q.y0 < miny { miny = q.y0 }
                    if q.y1 > maxy { maxy = q.y1 }
                } else {
                    if q.y1 < miny { miny = q.y1 }
                    if q.y0 > maxy { maxy = q.y0 }
                }
            }
            prevGlyphIndex =
                if !glyph.is_null() { (*glyph).index } else { -1i32 }
        }
        str = str.offset(1isize)
    }
    advance = x - startx;
    if !(0 != (*state).align & FONS_ALIGN_LEFT as libc::c_int) {
        if 0 != (*state).align & FONS_ALIGN_RIGHT as libc::c_int {
            minx -= advance;
            maxx -= advance
        } else if 0 != (*state).align & FONS_ALIGN_CENTER as libc::c_int {
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
    return advance;
}

pub unsafe extern "C" fn fonsLineBounds(mut stash: *mut FONScontext,
                                        mut y: libc::c_float,
                                        mut miny: *mut libc::c_float,
                                        mut maxy: *mut libc::c_float) {
    let mut font: *mut FONSfont = 0 as *mut FONSfont;
    let mut state: *mut FONSstate = fons__getState(stash);
    let mut isize: libc::c_short = 0;
    if stash.is_null() { return }
    if (*state).font < 0i32 || (*state).font >= (*stash).nfonts { return }
    font = *(*stash).fonts.offset((*state).font as isize);
    isize = ((*state).size * 10.0f32) as libc::c_short;
    if (*font).data.is_null() { return }
    y += fons__getVertAlign(stash, font, (*state).align, isize);
    if 0 !=
           (*stash).params.flags as libc::c_int &
               FONS_ZERO_TOPLEFT as libc::c_int {
        *miny = y - (*font).ascender * isize as libc::c_float / 10.0f32;
        *maxy =
            *miny +
                (*font).lineh * isize as libc::c_int as libc::c_float /
                    10.0f32
    } else {
        *maxy = y + (*font).descender * isize as libc::c_float / 10.0f32;
        *miny =
            *maxy -
                (*font).lineh * isize as libc::c_int as libc::c_float /
                    10.0f32
    };
}

pub unsafe extern "C" fn fonsVertMetrics(mut stash: *mut FONScontext,
                                         mut ascender: *mut libc::c_float,
                                         mut descender: *mut libc::c_float,
                                         mut lineh: *mut libc::c_float) {
    let mut font: *mut FONSfont = 0 as *mut FONSfont;
    let mut state: *mut FONSstate = fons__getState(stash);
    let mut isize: libc::c_short = 0;
    if stash.is_null() { return }
    if (*state).font < 0i32 || (*state).font >= (*stash).nfonts { return }
    font = *(*stash).fonts.offset((*state).font as isize);
    isize = ((*state).size * 10.0f32) as libc::c_short;
    if (*font).data.is_null() { return }
    if !ascender.is_null() {
        *ascender =
            (*font).ascender * isize as libc::c_int as libc::c_float / 10.0f32
    }
    if !descender.is_null() {
        *descender =
            (*font).descender * isize as libc::c_int as libc::c_float /
                10.0f32
    }
    if !lineh.is_null() {
        *lineh =
            (*font).lineh * isize as libc::c_int as libc::c_float / 10.0f32
    };
}
// Text iterator

pub unsafe extern "C" fn fonsTextIterInit(mut stash: *mut FONScontext,
                                          mut iter: *mut FONStextIter,
                                          mut x: libc::c_float,
                                          mut y: libc::c_float,
                                          mut str: *const libc::c_char,
                                          mut end: *const libc::c_char,
                                          mut bitmapOption: libc::c_int)
 -> libc::c_int {
    let mut state: *mut FONSstate = fons__getState(stash);
    let mut width: libc::c_float = 0.;
    memset(iter as *mut libc::c_void, 0i32,
           ::std::mem::size_of::<FONStextIter>() as libc::c_ulong);
    if stash.is_null() { return 0i32 }
    if (*state).font < 0i32 || (*state).font >= (*stash).nfonts {
        return 0i32
    }
    (*iter).font = *(*stash).fonts.offset((*state).font as isize);
    if (*(*iter).font).data.is_null() { return 0i32 }
    (*iter).isize_0 = ((*state).size * 10.0f32) as libc::c_short;
    (*iter).iblur = (*state).blur as libc::c_short;
    (*iter).scale =
        fons__tt_getPixelHeightScale(&mut (*(*iter).font).font,
                                     (*iter).isize_0 as libc::c_float /
                                         10.0f32);
    if !(0 != (*state).align & FONS_ALIGN_LEFT as libc::c_int) {
        if 0 != (*state).align & FONS_ALIGN_RIGHT as libc::c_int {
            width =
                fonsTextBounds(stash, x, y, str, end,
                               0 as *mut libc::c_float);
            x -= width
        } else if 0 != (*state).align & FONS_ALIGN_CENTER as libc::c_int {
            width =
                fonsTextBounds(stash, x, y, str, end,
                               0 as *mut libc::c_float);
            x -= width * 0.5f32
        }
    }
    y +=
        fons__getVertAlign(stash, (*iter).font, (*state).align,
                           (*iter).isize_0);
    if end.is_null() { end = str.offset(strlen(str) as isize) }
    (*iter).nextx = x;
    (*iter).x = (*iter).nextx;
    (*iter).nexty = y;
    (*iter).y = (*iter).nexty;
    (*iter).spacing = (*state).spacing;
    (*iter).str_0 = str;
    (*iter).next = str;
    (*iter).end = end;
    (*iter).codepoint = 0i32 as libc::c_uint;
    (*iter).prevGlyphIndex = -1i32;
    (*iter).bitmapOption = bitmapOption;
    return 1i32;
}

pub unsafe extern "C" fn fonsTextIterNext(mut stash: *mut FONScontext,
                                          mut iter: *mut FONStextIter,
                                          mut quad: *mut FONSquad)
 -> libc::c_int {
    let mut glyph: *mut FONSglyph = 0 as *mut FONSglyph;
    let mut str: *const libc::c_char = (*iter).next;
    (*iter).str_0 = (*iter).next;
    if str == (*iter).end { return 0i32 }
    while str != (*iter).end {
        if 0 !=
               fons__decutf8(&mut (*iter).utf8state, &mut (*iter).codepoint,
                             *(str as *const libc::c_uchar) as libc::c_uint) {
            str = str.offset(1isize)
        } else {
            str = str.offset(1isize);
            (*iter).x = (*iter).nextx;
            (*iter).y = (*iter).nexty;
            glyph =
                fons__getGlyph(stash, (*iter).font, (*iter).codepoint,
                               (*iter).isize_0, (*iter).iblur,
                               (*iter).bitmapOption);
            if !glyph.is_null() {
                fons__getQuad(stash, (*iter).font, (*iter).prevGlyphIndex,
                              glyph, (*iter).scale, (*iter).spacing,
                              &mut (*iter).nextx, &mut (*iter).nexty, quad);
            }
            (*iter).prevGlyphIndex =
                if !glyph.is_null() { (*glyph).index } else { -1i32 };
            break ;
        }
    }
    (*iter).next = str;
    return 1i32;
}

pub unsafe extern "C" fn fonsAddFallbackFont(mut stash: *mut FONScontext,
                                             mut base: libc::c_int,
                                             mut fallback: libc::c_int)
 -> libc::c_int {
    let mut baseFont: *mut FONSfont = *(*stash).fonts.offset(base as isize);
    if (*baseFont).nfallbacks < 20i32 {
        let fresh34 = (*baseFont).nfallbacks;
        (*baseFont).nfallbacks = (*baseFont).nfallbacks + 1;
        (*baseFont).fallbacks[fresh34 as usize] = fallback;
        return 1i32
    }
    return 0i32;
}

// Pull texture changes

pub unsafe extern "C" fn fonsGetTextureData(mut stash: *mut FONScontext,
                                            mut width: *mut libc::c_int,
                                            mut height: *mut libc::c_int)
 -> *const libc::c_uchar {
    if !width.is_null() { *width = (*stash).params.width }
    if !height.is_null() { *height = (*stash).params.height }
    return (*stash).texData;
}