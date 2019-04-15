#![allow(unused_unsafe)]
#![allow(dead_code)] 

use std::ptr::{null, null_mut};
use std::ffi::{CStr};
use std::os::raw::{c_char};
use crate::context::{
    Context, Align, State,
    TextRow,
    GlyphPosition,
};
use crate::cache::Vertex;
use crate::fons::*;
use crate::context::{MAX_FONTIMAGE_SIZE, MAX_FONTIMAGES, TEXTURE_ALPHA};
use crate::transform::transform_point;
use crate::vg::utils::{
    min, max,
    average_scale,
};
use slotmap::Key;

extern "C" {
    fn fonsTextIterInit(
        s: *mut FONScontext, iter: *mut FONStextIter,
        x: f32, y: f32, str: *const u8, end: *const u8, bitmap_option: i32) -> i32;
    fn fonsTextIterNext(s: *mut FONScontext, iter: *mut FONStextIter, quad: *mut FONSquad) -> bool;

    fn fonsTextBounds(s: *mut FONScontext, x: f32, y: f32, start: *const u8, end: *const u8, bounds: *mut f32) -> f32;
    fn fonsLineBounds(s: *mut FONScontext, y: f32, miny: *mut f32, maxy: *mut f32);
    fn fonsVertMetrics(s: *mut FONScontext, ascender: *mut f32, descender: *mut f32, lineh: *mut f32);
}

impl Context {
    pub fn create_font(&mut self, name: &str, path: &str) -> i32 {
        self.fs.add_font(name, path)
    }
}

// Add fonts
#[no_mangle] extern "C"
fn nvgCreateFont(ctx: &mut Context, name: *const c_char, path: *const c_char) -> i32 {
    let name = unsafe { CStr::from_ptr(name).to_string_lossy() };
    let path = unsafe { CStr::from_ptr(path).to_string_lossy() };
    ctx.create_font(&name, &path)
}

#[no_mangle] extern "C"
fn nvgCreateFontMem(ctx: &mut Context, name: *const c_char, data: *mut u8, ndata: i32, free_data: i32) -> i32 {
    let name = unsafe { CStr::from_ptr(name).to_string_lossy() };
    ctx.fs.add_font_mem(&name, data, ndata, free_data)
}

#[no_mangle] extern "C"
fn nvgFindFont(ctx: &mut Context, name: *const u8) -> i32 {
    if name.is_null() {
        -1
    } else {
        ctx.fs.font_by_name(name)
    }
}

#[no_mangle] extern "C"
fn nvgAddFallbackFontId(ctx: &mut Context, base: i32, fallback: i32) -> i32 {
    if base == -1 || fallback == -1 {
        0
    } else {
        ctx.fs.add_fallback_font(base, fallback)
    }
}

#[no_mangle] extern "C"
fn nvgAddFallbackFont(ctx: &mut Context, base: *const u8, fallback: *const u8) -> i32 {
    let base = nvgFindFont(ctx, base);
    let fallback = nvgFindFont(ctx, fallback);
    nvgAddFallbackFontId(ctx, base, fallback)
}

// State setting
#[no_mangle] extern "C"
fn nvgFontSize(ctx: &mut Context, size: f32) {
    ctx.states.last_mut().font_size = size;
}

#[no_mangle] extern "C"
fn nvgFontBlur(ctx: &mut Context, blur: f32) {
    ctx.states.last_mut().font_blur = blur;
}

#[no_mangle] extern "C"
fn nvgTextLetterSpacing(ctx: &mut Context, spacing: f32) {
    ctx.states.last_mut().letter_spacing = spacing;
}

#[no_mangle] extern "C"
fn nvgTextLineHeight(ctx: &mut Context, line_height: f32) {
    ctx.states.last_mut().line_height = line_height;
}

#[no_mangle] extern "C"
fn nvgTextAlign(ctx: &mut Context, align: Align) {
    ctx.states.last_mut().text_align = align;
}

#[no_mangle] extern "C"
fn nvgFontFaceId(ctx: &mut Context, font_id: i32) {
    ctx.states.last_mut().font_id = font_id;
}

#[no_mangle] extern "C"
fn nvgFontFace(ctx: &mut Context, font: *const u8) {
    ctx.states.last_mut().font_id = ctx.fs.font_by_name(font);
}



fn quantize(a: f32, d: f32) -> f32 {
    (a / d + 0.5).floor() * d
}

fn font_scale(state: &State) -> f32 {
    min(quantize(average_scale(&state.xform), 0.01), 4.0)
}

unsafe fn flush_text_texture(ctx: &mut Context) {
    let mut dirty = [0i32; 4];

    if ctx.fs.validate_texture(&mut dirty) {
        let image = ctx.font_images[ctx.font_image_idx as usize];
        // Update texture
        if !image.is_null() {
            let (_, _, data) = ctx.fs.texture_data();
            let x = dirty[0];
            let y = dirty[1];
            let w = (dirty[2] - dirty[0]) as u32;
            let h = (dirty[3] - dirty[1]) as u32;
            ctx.params.update_texture(image, x,y, w,h, data);
        }
    }
}

unsafe fn alloc_text_atlas(ctx: &mut Context) -> bool {
    flush_text_texture(ctx);
    if ctx.font_image_idx >= (MAX_FONTIMAGES-1) as i32 {
        return false;
    }
    // if next fontImage already have a texture
    let (iw, ih) = if !ctx.font_images[(ctx.font_image_idx+1) as usize].is_null() {
        ctx.image_size(ctx.font_images[(ctx.font_image_idx+1) as usize])
            .expect("font image texture")
    } else { // calculate the new font image size and create it.
        let (mut iw, mut ih) = ctx.image_size(ctx.font_images[ctx.font_image_idx as usize])
            .expect("font image texture");
        if iw > ih {
            ih *= 2;
        } else {
            iw *= 2;
        }
        if iw > MAX_FONTIMAGE_SIZE || ih > MAX_FONTIMAGE_SIZE {
            iw = MAX_FONTIMAGE_SIZE as u32;
            ih = MAX_FONTIMAGE_SIZE as u32;
        }
        ctx.font_images[(ctx.font_image_idx+1) as usize] =
            ctx.params.create_texture(TEXTURE_ALPHA, iw, ih, Default::default(), null());
        (iw, ih)
    };
    ctx.font_image_idx += 1;
    ctx.fs.reset_atlas(iw, ih);
    true
}

unsafe fn render_text(ctx: &mut Context, verts: &mut [Vertex]) {
    let state = ctx.states.last();
    let mut paint = state.fill;

    // Render triangles.
    paint.image = ctx.font_images[ctx.font_image_idx as usize];

    // Apply global alpha
    paint.inner_color.a *= state.alpha;
    paint.outer_color.a *= state.alpha;

    ctx.params.draw_triangles(
        &paint,
        state.composite,
        &state.scissor,
        &verts,
    );

    ctx.counters.text_call(verts.len() / 3);
}

#[no_mangle] unsafe extern "C"
fn nvgText(ctx: &mut Context, x: f32, y: f32, start: *const u8, mut end: *const u8) -> f32 {
    let state = ctx.states.last();
    let scale = font_scale(state) * ctx.device_px_ratio;
    let invscale = 1.0 / scale;

    if end.is_null() {
        end = start.add(libc::strlen(start as *const i8));
    }

    if state.font_id == FONS_INVALID {
        return x;
    }

    ctx.fs.sync_state(state, scale);

    let xform = state.xform;

    let cverts = 6 * max(2, end.offset_from(start) as usize); // conservative estimate.
    let verts = ctx.cache.temp_verts(cverts);
    let verts = unsafe { std::slice::from_raw_parts_mut(verts.as_mut_ptr(), verts.len()) };

    let mut nverts = 0;

    let mut q = FONSquad::default();
    let mut iter: FONStextIter = unsafe { std::mem::zeroed() };
    fonsTextIterInit(ctx.fs.as_mut(), &mut iter, x*scale, y*scale, start, end, GLYPH_BITMAP_REQUIRED);
    let mut prev_iter = iter;
    while fonsTextIterNext(ctx.fs.as_mut(), &mut iter, &mut q) {
        if iter.prev_glyph_index == -1 { // can not retrieve glyph?
            if nverts != 0 {
                render_text(ctx, &mut verts[..nverts]);
                nverts = 0;
            }
            if !alloc_text_atlas(ctx) {
                break; // no memory :(
            }
            iter = prev_iter;
            fonsTextIterNext(ctx.fs.as_mut(), &mut iter, &mut q); // try again
            if iter.prev_glyph_index == -1 { // still can not find glyph?
                break;
            }
        }
        prev_iter = iter;

        // Transform corners.
        let a = transform_point(&xform, q.x0*invscale, q.y0*invscale);
        let b = transform_point(&xform, q.x1*invscale, q.y0*invscale);
        let c = transform_point(&xform, q.x1*invscale, q.y1*invscale);
        let d = transform_point(&xform, q.x0*invscale, q.y1*invscale);

        // Create triangles
        if nverts+6 <= cverts {
            verts[nverts + 0].set(a, [q.s0, q.t0]); // 01
            verts[nverts + 1].set(c, [q.s1, q.t1]); // 45
            verts[nverts + 2].set(b, [q.s1, q.t0]); // 23
            verts[nverts + 3].set(a, [q.s0, q.t0]); // 01
            verts[nverts + 4].set(d, [q.s0, q.t1]); // 67
            verts[nverts + 5].set(c, [q.s1, q.t1]); // 45
            nverts += 6;
        }
    }

    // TODO: add back-end bit to do this just once per frame.
    flush_text_texture(ctx);

    render_text(ctx, &mut verts[..nverts]);

    iter.nextx / scale
}

#[no_mangle] unsafe extern "C"
fn nvgTextBox(ctx: &mut Context, x: f32, mut y: f32, break_row_width: f32, mut start: *const u8, end: *const u8) {
    let mut lineh = 0.0;
    let old_align;
    let (haling, valign);

    let line_height = {
        let state = ctx.states.last();
        if state.font_id == FONS_INVALID {
            return;
        }

        old_align = state.text_align;
        haling = state.text_align & (Align::LEFT | Align::CENTER | Align::RIGHT);
        valign = state.text_align & (Align::TOP  | Align::MIDDLE | Align::BOTTOM | Align::BASELINE);

        nvgTextMetrics(ctx, null_mut(), null_mut(), &mut lineh);

        ctx.states.last_mut().text_align = Align::LEFT | valign;
        ctx.states.last().line_height
    };

    let mut rows: [TextRow; 2] = unsafe { std::mem::zeroed() };
    loop {
        let nrows = nvgTextBreakLines(ctx, start, end, break_row_width, &mut rows[0], 2);
        if nrows == 0 { break }
        for i in 0..nrows {
            let row = &mut rows[i];
            if haling.contains(Align::LEFT) {
                nvgText(ctx, x, y, row.start, row.end);
            } else if haling.contains(Align::CENTER) {
                nvgText(ctx, x + break_row_width*0.5 - row.width*0.5, y, row.start, row.end);
            } else if haling.contains(Align::RIGHT) {
                nvgText(ctx, x + break_row_width - row.width, y, row.start, row.end);
            }
            y += lineh * line_height;
        }
        start = rows[nrows-1].next;
    }

    ctx.states.last_mut().text_align = old_align;
}

#[no_mangle] unsafe extern "C"
fn nvgTextGlyphPositions(
    ctx: &mut Context, x: f32, y: f32,
    start: *const u8, mut end: *const u8,
    positions: *mut GlyphPosition,
    max_positions: i32,
) -> usize {
    let state = ctx.states.last();
    let scale = font_scale(state) * ctx.device_px_ratio;
    let invscale = 1.0 / scale;

    let mut npos = 0;

    let positions = unsafe { std::slice::from_raw_parts_mut(positions, max_positions as usize) };

    if state.font_id == FONS_INVALID {
        return 0;
    }

    if end.is_null() {
        end = start.add(libc::strlen(start as *const i8));
    }

    if start == end {
        return 0;
    }

    ctx.fs.sync_state(state, scale);

    let mut q = FONSquad::default();
    let mut iter: FONStextIter = unsafe { std::mem::zeroed() };
    fonsTextIterInit(ctx.fs.as_mut(), &mut iter, x*scale, y*scale, start, end, GLYPH_BITMAP_OPTIONAL);
    let mut prev_iter = iter;

    while fonsTextIterNext(ctx.fs.as_mut(), &mut iter, &mut q) {
        if iter.prev_glyph_index < 0 && alloc_text_atlas(ctx) { // can not retrieve glyph?
            iter = prev_iter;
            fonsTextIterNext(ctx.fs.as_mut(), &mut iter, &mut q); // try again
        }
        prev_iter = iter;
        positions[npos].s = iter.str;
        positions[npos].x = iter.x * invscale;
        positions[npos].minx = min(iter.x, q.x0) * invscale;
        positions[npos].maxx = max(iter.nextx, q.x1) * invscale;
        npos += 1;
        if npos >= max_positions as usize {
            break;
        }
    }

    npos
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Codepoint {
    SPACE,
    NEWLINE,
    CHAR,
    CJK_CHAR,
}

#[no_mangle] unsafe extern "C"
fn nvgTextBreakLines(
    ctx: &mut Context, start: *const u8, mut end: *const u8,
    mut break_row_width: f32, rows: *mut TextRow, max_rows: usize,
) -> usize {
    if max_rows == 0 { return 0; }

    let state = ctx.states.last();
    let scale = font_scale(state) * ctx.device_px_ratio;
    let invscale = 1.0 / scale;

    if state.font_id == FONS_INVALID {
        return 0;
    }

    let mut nrows = 0;

    let mut row_startx = 0.0;
    let mut row_width = 0.0;
    let mut row_minx = 0.0;
    let mut row_maxx = 0.0;
    let mut row_start: *const u8 = null();
    let mut row_end: *const u8 = null();

    let mut word_start: *const u8 = null();
    let mut word_startx = 0.0;
    let mut word_minx = 0.0;

    let mut break_end: *const u8 = null();
    let mut break_width = 0.0;
    let mut break_maxx = 0.0;

    //let mut _type = Codepoint::SPACE;
    let mut ptype = Codepoint::SPACE;
    let mut pcodepoint = 0u32;

    if end.is_null() {
        end = start.add(libc::strlen(start as *const i8));
    }

    if start == end {
        return 0;
    }

    ctx.fs.sync_state(state, scale);

    break_row_width *= scale;

    let mut q = FONSquad::default();
    let mut iter: FONStextIter = unsafe { std::mem::zeroed() };
    fonsTextIterInit(ctx.fs.as_mut(), &mut iter, 0.0, 0.0, start, end, GLYPH_BITMAP_OPTIONAL);
    let mut prev_iter = iter;

    while fonsTextIterNext(ctx.fs.as_mut(), &mut iter, &mut q) {
        if iter.prev_glyph_index < 0 && alloc_text_atlas(ctx) { // can not retrieve glyph?
            iter = prev_iter;
            fonsTextIterNext(ctx.fs.as_mut(), &mut iter, &mut q); // try again
        }
        prev_iter = iter;


        let cp = iter.codepoint;
        let _type = if [9, 11, 12, 32, 0x00a0].contains(&cp) {
            // [\t \v \f space nbsp]
            Codepoint::SPACE
        } else if cp == 10 { // \n
            if pcodepoint == 13 { Codepoint::SPACE } else { Codepoint::NEWLINE }
        } else if cp == 13 { // \r
            if pcodepoint == 10 { Codepoint::SPACE } else { Codepoint::NEWLINE }
        } else if cp == 0x0085 { // NEL
            Codepoint::NEWLINE
        } else {
            if  (cp >= 0x4E00 && cp <= 0x9FFF) ||
                (cp >= 0x3000 && cp <= 0x30FF) ||
                (cp >= 0xFF00 && cp <= 0xFFEF) ||
                (cp >= 0x1100 && cp <= 0x11FF) ||
                (cp >= 0x3130 && cp <= 0x318F) ||
                (cp >= 0xAC00 && cp <= 0xD7AF)
            {
                Codepoint::CJK_CHAR
            } else {
                Codepoint::CHAR
            }
        };


        if _type == Codepoint::NEWLINE {
            // Always handle new lines.
            rows.add(nrows).write(TextRow {
                start: if !row_start.is_null() { row_start } else { iter.str },
                end: if !row_end.is_null() { row_end } else { iter.str },
                width: row_width * invscale,
                minx: row_minx * invscale,
                maxx: row_maxx * invscale,
                next: iter.next,
            });
            nrows += 1;
            if nrows >= max_rows {
                return nrows;
            }
            // Set null break point
            break_end = row_start;
            break_width = 0.0;
            break_maxx = 0.0;
            // Indicate to skip the white space at the beginning of the row.
            row_start = null();
            row_end = null();
            row_width = 0.0;
            row_minx = 0.0;
            row_maxx = 0.0;
        } else {
            if row_start.is_null() {
                // Skip white space until the beginning of the line
                if _type == Codepoint::CHAR || _type == Codepoint::CJK_CHAR {
                    // The current char is the row so far
                    row_startx = iter.x;
                    row_start = iter.str;
                    row_end = iter.next;
                    row_width = iter.nextx - row_startx; // q.x1 - rowStartX;
                    row_minx = q.x0 - row_startx;
                    row_maxx = q.x1 - row_startx;

                    word_start = iter.str;
                    word_startx = iter.x;
                    word_minx = q.x0 - row_startx;
                    // Set null break point
                    break_end = row_start;
                    break_width = 0.0;
                    break_maxx = 0.0;
                }
            } else {
                let next_width = iter.nextx - row_startx;

                // track last non-white space character
                if _type == Codepoint::CHAR || _type == Codepoint::CJK_CHAR {
                    row_end = iter.next;
                    row_width = iter.nextx - row_startx;
                    row_maxx = q.x1 - row_startx;
                }
                // track last end of a word
                if ((ptype == Codepoint::CHAR || ptype == Codepoint::CJK_CHAR) && _type == Codepoint::SPACE)
                    || _type == Codepoint::CJK_CHAR {
                    break_end = iter.str;
                    break_width = row_width;
                    break_maxx = row_maxx;
                }
                // track last beginning of a word
                if (ptype == Codepoint::SPACE && (_type == Codepoint::CHAR || _type == Codepoint::CJK_CHAR))
                    || _type == Codepoint::CJK_CHAR {
                    word_start = iter.str;
                    word_startx = iter.x;
                    word_minx = q.x0 - row_startx;
                }

                // Break to new line when a character is beyond break width.
                if (_type == Codepoint::CHAR || _type == Codepoint::CJK_CHAR) && next_width > break_row_width {
                    // The run length is too long, need to break to new line.
                    if break_end == row_start {
                        // The current word is longer than the row length, just break it from here.
                        rows.add(nrows).write(TextRow {
                            start: row_start,
                            end: iter.str,
                            width: row_width * invscale,
                            minx:  row_minx * invscale,
                            maxx:  row_maxx * invscale,
                            next: iter.str,
                        });
                        nrows += 1;
                        if nrows >= max_rows {
                            return nrows;
                        }
                        row_startx = iter.x;
                        row_start = iter.str;
                        row_end = iter.next;
                        row_width = iter.nextx - row_startx;
                        row_minx = q.x0 - row_startx;
                        row_maxx = q.x1 - row_startx;

                        word_start = iter.str;
                        word_startx = iter.x;
                        word_minx = q.x0 - row_startx;
                    } else {
                        // Break the line from the end of the last word,
                        // and start new line from the beginning of the new.
                        rows.add(nrows).write(TextRow {
                            start: row_start,
                            end: break_end,
                            width: break_width * invscale,
                            minx: row_minx * invscale,
                            maxx: break_maxx * invscale,
                            next: word_start,
                        });
                        nrows += 1;
                        if nrows >= max_rows {
                            return nrows;
                        }
                        row_startx = word_startx;
                        row_start = word_start;
                        row_end = iter.next;
                        row_width = iter.nextx - row_startx;
                        row_minx = word_minx;
                        row_maxx = q.x1 - row_startx;
                        // No change to the word start
                    }
                    // Set null break point
                    break_end = row_start;
                    break_width = 0.0;
                    break_maxx = 0.0;
                }
            }
        }

        pcodepoint = iter.codepoint;
        ptype = _type;
    }

    // Break the line from the end of the last word, and start new line from the beginning of the new.
    if !row_start.is_null() {
        rows.add(nrows).write(TextRow {
            start: row_start,
            end: row_end,
            width: row_width * invscale,
            minx: row_minx * invscale,
            maxx: row_maxx * invscale,
            next: end,
        });
        nrows += 1;
    }

    nrows
}

#[no_mangle] unsafe extern "C"
fn nvgTextBounds(ctx: &mut Context, x: f32, y: f32, start: *const u8, end: *const u8, bounds: *mut f32) -> f32 {
    let state = ctx.states.last();
    let scale = font_scale(state) * ctx.device_px_ratio;
    let invscale = 1.0 / scale;

    if state.font_id == FONS_INVALID {
        return 0.0;
    }

    ctx.fs.sync_state(state, scale);

    let width = fonsTextBounds(ctx.fs.as_mut(), x*scale, y*scale, start, end, bounds);
    if !bounds.is_null() {
        // Use line bounds for height.
        fonsLineBounds(ctx.fs.as_mut(), y*scale, bounds.add(1), bounds.add(3));
        *bounds.add(0) *= invscale;
        *bounds.add(1) *= invscale;
        *bounds.add(2) *= invscale;
        *bounds.add(3) *= invscale;
    }
    width * invscale
}

#[no_mangle] unsafe extern "C"
fn nvgTextBoxBounds(
    ctx: &mut Context, x: f32, mut y: f32, break_row_width: f32,
    mut start: *const u8, end: *const u8, bounds: *mut f32,
) {
    let (halign, valign);

    let (invscale, old_align) = {
        let state = ctx.states.last();
        let scale = font_scale(state) * ctx.device_px_ratio;
        let invscale = 1.0 / scale;
        let old_align = state.text_align;
        halign = state.text_align & (Align::LEFT | Align::CENTER | Align::RIGHT);
        valign = state.text_align & (Align::TOP  | Align::MIDDLE | Align::BOTTOM | Align::BASELINE);

        if state.font_id == FONS_INVALID {
            if !bounds.is_null() {
                bounds.add(0).write(0.0);
                bounds.add(1).write(0.0);
                bounds.add(2).write(0.0);
                bounds.add(3).write(0.0);
            }
            return;
        }

        ctx.fs.sync_state(state, scale);

        (invscale, old_align)
    };

    let (mut rminy, mut rmaxy) = (0.0, 0.0);
    fonsLineBounds(ctx.fs.as_mut(), 0.0, &mut rminy, &mut rmaxy);
    rmaxy *= invscale;
    rminy *= invscale;

    let mut lineh = 0.0;
    nvgTextMetrics(ctx, null_mut(), null_mut(), &mut lineh);
    ctx.states.last_mut().text_align = Align::LEFT | valign;

    let (mut minx, mut maxx) = (x, x);
    let (mut miny, mut maxy) = (y, y);

    let mut rows: [TextRow; 2] = unsafe { std::mem::zeroed() };
    loop {
        let nrows = nvgTextBreakLines(ctx, start, end, break_row_width, rows.as_mut_ptr(), 2);
        if nrows == 0 { break }
        for i in 0..nrows {
            let row = &rows[i];
            // Horizontal bounds
            let dx = if halign.contains(Align::LEFT) {
                0.0
            } else if halign.contains(Align::CENTER) { 
                break_row_width*0.5 - row.width*0.5
            } else if halign.contains(Align::RIGHT) {
                break_row_width - row.width
            } else {
                0.0
            };
            let rminx = x + row.minx + dx;
            let rmaxx = x + row.maxx + dx;
            // Horizontal bounds.
            minx = min(minx, rminx);
            maxx = max(maxx, rmaxx);
            // Vertical bounds.
            miny = min(miny, y + rminy);
            maxy = max(maxy, y + rmaxy);

            y += lineh * ctx.states.last().line_height;
        }
        start = rows[nrows-1].next;
    }

    ctx.states.last_mut().text_align = old_align;

    if !bounds.is_null() {
        bounds.add(0).write(minx);
        bounds.add(1).write(miny);
        bounds.add(2).write(maxx);
        bounds.add(3).write(maxy);
    }
}

#[no_mangle] unsafe extern "C"
fn nvgTextMetrics(ctx: &mut Context, ascender: *mut f32, descender: *mut f32, lineh: *mut f32) {
    let state = ctx.states.last();
    let scale = font_scale(state) * ctx.device_px_ratio;
    let invscale = 1.0 / scale;

    if state.font_id == FONS_INVALID {
        return;
    }

    ctx.fs.sync_state(state, scale);

    fonsVertMetrics(ctx.fs.as_mut(), ascender, descender, lineh);
    if !ascender.is_null() {
        *ascender *= invscale;
    }
    if !descender.is_null() {
        *descender *= invscale;
    }
    if !lineh.is_null() {
        *lineh *= invscale;
    }
}
