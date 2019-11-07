#![allow(clippy::cast_lossless)]
#![warn(dead_code)]

use slotmap::Key;
use std::ptr::null;

mod atlas;
mod fons;
mod font_info;
mod stash;
mod utils;

use self::fons::FONS_INVALID;
pub use self::stash::{Stash, Metrics};

use crate::{
    vg::State,
    context::{
        Context,
        MAX_FONTIMAGE_SIZE, MAX_FONTIMAGES,
    },
    backend::TEXTURE_ALPHA,
    cache::Vertex,
    vg::utils::{
        average_scale,
        str_start_end,
    },
};

use std::{
    str::from_utf8_unchecked,
    slice::from_raw_parts,
};

unsafe fn raw_str<'a>(start: *const u8, end: *const u8) -> &'a str {
    from_utf8_unchecked(raw_slice(start, end))
}

unsafe fn raw_slice<'a>(start: *const u8, end: *const u8) -> &'a [u8] {
    let len = if end.is_null() {
        libc::strlen(start as *const i8)
    } else {
        end.offset_from(start) as usize
    };
    from_raw_parts(start, len)
}

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Codepoint {
    Space,
    Newline,
    Char,
    CjkChar,
}

fn quantize(a: f32, d: f32) -> f32 {
    (a / d + 0.5).floor() * d
}

fn font_scale(state: &State) -> f32 {
    quantize(average_scale(&state.xform), 0.01).min(4.0)
}

impl Context {
    pub fn create_font(&mut self, name: &str, path: &str) -> i32 {
        self.fs.add_font(name, path).unwrap_or(-1)
    }

    pub fn load_font(&mut self, name: &str, path: &str) -> std::io::Result<i32> {
        self.fs.add_font(name, path)
    }

    pub fn add_fallback_font_id(&mut self, base: i32, fallback: i32) -> bool {
        if base == -1 || fallback == -1 {
            false
        } else {
            self.fs.add_fallback_font(base, fallback) != 0
        }
    }

    pub fn font_face(&mut self, name: &str) {
        self.states.last_mut().font_id = self.fs.font_by_name(name);
    }

    fn flush_text_texture(&mut self) {
        if let Some(dirty) = self.fs.validate_texture() {
            let image = self.font_images[self.font_image_idx as usize];
            // Update texture
            if !image.is_null() {
                let (_, _, data) = self.fs.texture_data();
                let x = dirty[0];
                let y = dirty[1];
                let w = (dirty[2] - dirty[0]) as u32;
                let h = (dirty[3] - dirty[1]) as u32;
                self.params.update_texture(image, x,y, w,h, data);
            }
        }
    }

    fn alloc_text_atlas(&mut self) -> bool {
        self.flush_text_texture();
        if self.font_image_idx >= (MAX_FONTIMAGES-1) as i32 {
            return false;
        }
        // if next fontImage already have a texture
        let (iw, ih) = if !self.font_images[(self.font_image_idx+1) as usize].is_null() {
            self.image_size(self.font_images[(self.font_image_idx+1) as usize])
                .expect("font image texture")
        } else { // calculate the new font image size and create it.
            let (mut iw, mut ih) = self.image_size(self.font_images[self.font_image_idx as usize])
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
            self.font_images[(self.font_image_idx+1) as usize] =
                self.params.create_texture(TEXTURE_ALPHA, iw, ih, Default::default(), null());
            (iw, ih)
        };
        self.font_image_idx += 1;
        self.fs.reset_atlas(iw, ih);
        true
    }

    fn render_text(&mut self, verts: &mut [Vertex]) {
        let state = self.states.last();
        let mut paint = state.fill;

        // Render triangles.
        paint.image = self.font_images[self.font_image_idx as usize];

        // Apply global alpha
        paint.inner_color.a *= state.alpha;
        paint.outer_color.a *= state.alpha;

        self.params.draw_triangles(
            &paint,
            &state.scissor,
            &verts,
        );
    }

    unsafe fn text_raw(&mut self, x: f32, y: f32, start: *const u8, end: *const u8) -> f32 {
        self.text_slice(x, y, raw_slice(start, end))
    }

    fn text_slice(&mut self, x: f32, y: f32, text: &[u8]) -> f32 {
        let text = unsafe { std::str::from_utf8_unchecked(text) };
        self.text(x, y, text)
    }

    pub fn text(&mut self, x: f32, y: f32, text: &str) -> f32 {
        let (start, end) = str_start_end(text);

        let state = self.states.last();
        let scale = font_scale(state) * self.device_px_ratio;
        let invscale = 1.0 / scale;

        if state.font_id == FONS_INVALID {
            return x;
        }

        self.fs.sync_state(state, scale);

        let xform = state.xform;

        let cverts = 6 * text.len().max(2); // conservative estimate.
        let verts = self.cache.temp_verts(cverts);
        let verts = unsafe { std::slice::from_raw_parts_mut(verts.as_mut_ptr(), verts.len()) };

        let mut nverts = 0;

        let mut iter = self.fs.text_iter_required(x*scale, y*scale, start, end);
        let mut prev_iter = iter;

        while let Some(mut q) = iter.next() {
            if iter.prev_glyph_index == -1 { // can not retrieve glyph?
                if nverts != 0 {
                    self.render_text(&mut verts[..nverts]);
                    nverts = 0;
                }
                if !self.alloc_text_atlas() {
                    break; // no memory :(
                }
                iter = prev_iter;
                q = iter.next().unwrap(); // try again
                if iter.prev_glyph_index == -1 { // still can not find glyph?
                    break;
                }
            }
            prev_iter = iter;

            // Create triangles
            if nverts+6 <= cverts {
                // Transform corners.
                let pts = (
                    xform.apply([q.x0*invscale, q.y0*invscale]),
                    xform.apply([q.x1*invscale, q.y0*invscale]),
                    xform.apply([q.x1*invscale, q.y1*invscale]),
                    xform.apply([q.x0*invscale, q.y1*invscale]),
                );

                verts[nverts    ].set(pts.0, [q.s0, q.t0]); // 01
                verts[nverts + 1].set(pts.2, [q.s1, q.t1]); // 45
                verts[nverts + 2].set(pts.1, [q.s1, q.t0]); // 23
                verts[nverts + 3].set(pts.0, [q.s0, q.t0]); // 01
                verts[nverts + 4].set(pts.3, [q.s0, q.t1]); // 67
                verts[nverts + 5].set(pts.2, [q.s1, q.t1]); // 45
                nverts += 6;
            }
        }

        // TODO: add back-end bit to do this just once per frame.
        self.flush_text_texture();
        self.render_text(&mut verts[..nverts]);
        iter.nextx / scale
    }

    pub fn text_bounds(&mut self, x: f32, y: f32, text: &str) -> (f32, [f32; 4]) {
        let state = self.states.last();
        let scale = font_scale(state) * self.device_px_ratio;
        let invscale = 1.0 / scale;

        if state.font_id == FONS_INVALID {
            return (0.0, [0.0; 4]);
        }

        self.fs.sync_state(state, scale);

        let (start, end) = str_start_end(text);
        let (width, mut bounds) = self.fs.text_bounds(x*scale, y*scale, start, end);

        // Use line bounds for height.
        let (rminy, rmaxy) = self.fs.line_bounds(y*scale);

        bounds[0] *= invscale;
        bounds[1] = rminy * invscale;
        bounds[2] *= invscale;
        bounds[3] = rmaxy * invscale;

        (width * invscale, bounds)
    }

    pub fn text_box(&mut self, x: f32, mut y: f32, break_row_width: f32, text: &str) {
        let old_align;
        let (haling, valign);

        let lineh;
        let line_height = {
            let state = self.states.last();
            if state.font_id == FONS_INVALID {
                return;
            }

            old_align = state.text_align;
            haling = state.text_align & (Align::LEFT | Align::CENTER | Align::RIGHT);
            valign = state.text_align & (Align::TOP  | Align::MIDDLE | Align::BOTTOM | Align::BASELINE);

            lineh = self.text_metrics().unwrap().line_gap;

            self.states.last_mut().text_align = Align::LEFT | valign;
            self.states.last().line_height
        };

        let (mut start, end) = str_start_end(text);
        let mut rows: [TextRow; 2] = unsafe { std::mem::zeroed() };
        loop {
            let text = unsafe { raw_str(start, end) };
            let rows = self.text_break_lines(text, break_row_width, &mut rows);
            if rows.is_empty() { break }
            for row in rows {
                unsafe {
                    if haling.contains(Align::LEFT) {
                        self.text_raw(x, y, row.start, row.end);
                    } else if haling.contains(Align::CENTER) {
                        self.text_raw(x + break_row_width*0.5 - row.width*0.5, y, row.start, row.end);
                    } else if haling.contains(Align::RIGHT) {
                        self.text_raw(x + break_row_width - row.width, y, row.start, row.end);
                    }
                }
                y += lineh * line_height;
            }
            start = rows[rows.len()-1].next;
        }

        self.states.last_mut().text_align = old_align;
    }

    pub fn text_glyph_positions<'a>(
        &mut self, x: f32, y: f32, text: &str,
        positions: &'a mut [GlyphPosition],
    ) -> &'a [GlyphPosition] {
        let state = self.states.last();
        let scale = font_scale(state) * self.device_px_ratio;
        let invscale = 1.0 / scale;

        let mut npos = 0;

        if text.is_empty() || state.font_id == FONS_INVALID {
            return &[];
        }

        let (start, end) = str_start_end(text);
        self.fs.sync_state(state, scale);

        let mut iter = self.fs.text_iter_optional(x*scale, y*scale, start, end);
        let mut prev_iter = iter;

        while let Some(mut q) = iter.next() {
            if iter.prev_glyph_index < 0 && self.alloc_text_atlas() { // can not retrieve glyph?
                iter = prev_iter;
                q = iter.next().unwrap(); // try again
            }
            prev_iter = iter;
            positions[npos].s = iter.str_0;
            positions[npos].x = iter.x * invscale;
            positions[npos].minx = q.x0.min(iter.x) * invscale;
            positions[npos].maxx = q.x1.max(iter.nextx) * invscale;
            npos += 1;
            if npos >= positions.len() {
                break;
            }
        }

        &positions[..npos]
    }

    pub fn text_box_bounds(
        &mut self, x: f32, mut y: f32, break_row_width: f32, text: &str,
    ) -> [f32; 4] {
        let (halign, valign);

        let (invscale, old_align) = {
            let state = self.states.last();
            let scale = font_scale(state) * self.device_px_ratio;
            let invscale = 1.0 / scale;
            let old_align = state.text_align;
            halign = state.text_align & (Align::LEFT | Align::CENTER | Align::RIGHT);
            valign = state.text_align & (Align::TOP  | Align::MIDDLE | Align::BOTTOM | Align::BASELINE);

            if state.font_id == FONS_INVALID {
                return [0.0; 4];
            }

            self.fs.sync_state(state, scale);

            (invscale, old_align)
        };

        let (mut rminy, mut rmaxy) = self.fs.line_bounds(0.0);
        rmaxy *= invscale;
        rminy *= invscale;

        let lineh = self.text_metrics().unwrap().line_gap;
        self.states.last_mut().text_align = Align::LEFT | valign;

        let (mut minx, mut maxx) = (x, x);
        let (mut miny, mut maxy) = (y, y);

        let (mut start, end) = str_start_end(text);
        let mut rows: [TextRow; 2] = unsafe { std::mem::zeroed() };
        loop {
            let text = unsafe { raw_str(start, end) };
            let rows = self.text_break_lines(text, break_row_width, &mut rows);
            if rows.is_empty() { break }
            for row in rows {
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
                minx = minx.min(rminx);
                maxx = maxx.max(rmaxx);
                // Vertical bounds.
                miny = miny.min(y + rminy);
                maxy = maxy.max(y + rmaxy);

                y += lineh * self.states.last().line_height;
            }
            start = rows[rows.len()-1].next;
        }

        self.states.last_mut().text_align = old_align;

        [minx, miny, maxx, maxy]
    }

    pub fn text_metrics(&mut self) -> Option<Metrics> {
        let state = self.states.last();
        let scale = font_scale(state) * self.device_px_ratio;
        let invscale = 1.0 / scale;

        if state.font_id == FONS_INVALID {
            return None;
        }

        self.fs.sync_state(state, scale);

        self.fs.metrics().map(|mut m| {
            m.ascender *= invscale;
            m.descender *= invscale;
            m.line_gap *= invscale;
            m
        })
    }

    pub fn text_break_lines<'a>(
        &mut self, text: &str,
        mut break_row_width: f32, rows: &'a mut [TextRow],
    ) -> &'a [TextRow] {
        let state = self.states.last();
        let scale = font_scale(state) * self.device_px_ratio;
        let invscale = 1.0 / scale;

        if rows.is_empty() || text.is_empty() || state.font_id == FONS_INVALID {
            return &[];
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
        let mut ptype = Codepoint::Space;
        let mut pcodepoint = 0u32;

        let (start, end) = str_start_end(text);

        self.fs.sync_state(state, scale);

        break_row_width *= scale;

        let mut iter = self.fs.text_iter_optional(0.0, 0.0, start, end);
        let mut prev_iter = iter;

        while let Some(mut q) = iter.next() {
            if iter.prev_glyph_index < 0 && self.alloc_text_atlas() { // can not retrieve glyph?
                iter = prev_iter;
                q = iter.next().unwrap(); // try again
            }
            prev_iter = iter;

            let cp = iter.codepoint;
            let _type = if [9, 11, 12, 32, 0x00a0].contains(&cp) {
                // [\t \v \f space nbsp]
                Codepoint::Space
            } else if cp == 10 { // \n
                if pcodepoint == 13 { Codepoint::Space } else { Codepoint::Newline }
            } else if cp == 13 { // \r
                if pcodepoint == 10 { Codepoint::Space } else { Codepoint::Newline }
            } else if cp == 0x0085 { // NEL
                Codepoint::Newline
            } else if  (cp >= 0x4E00 && cp <= 0x9FFF) ||
                (cp >= 0x3000 && cp <= 0x30FF) ||
                (cp >= 0xFF00 && cp <= 0xFFEF) ||
                (cp >= 0x1100 && cp <= 0x11FF) ||
                (cp >= 0x3130 && cp <= 0x318F) ||
                (cp >= 0xAC00 && cp <= 0xD7AF)
            {
                Codepoint::CjkChar
            } else {
                Codepoint::Char
            };

            if _type == Codepoint::Newline {
                // Always handle new lines.
                rows[nrows] = TextRow {
                    start: if !row_start.is_null() { row_start } else { iter.str_0 },
                    end: if !row_end.is_null() { row_end } else { iter.str_0 },
                    width: row_width * invscale,
                    minx: row_minx * invscale,
                    maxx: row_maxx * invscale,
                    next: iter.next,
                };
                nrows += 1;
                if nrows >= rows.len() {
                    return &rows[..nrows];
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
            } else if row_start.is_null() {
            // Skip white space until the beginning of the line
            if _type == Codepoint::Char || _type == Codepoint::CjkChar {
                // The current char is the row so far
                row_startx = iter.x;
                row_start = iter.str_0;
                row_end = iter.next;
                row_width = iter.nextx - row_startx; // q.x1 - rowStartX;
                row_minx = q.x0 - row_startx;
                row_maxx = q.x1 - row_startx;

                word_start = iter.str_0;
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
            if _type == Codepoint::Char || _type == Codepoint::CjkChar {
                row_end = iter.next;
                row_width = iter.nextx - row_startx;
                row_maxx = q.x1 - row_startx;
            }
            // track last end of a word
            if ((ptype == Codepoint::Char || ptype == Codepoint::CjkChar) && _type == Codepoint::Space)
                || _type == Codepoint::CjkChar {
                break_end = iter.str_0;
                break_width = row_width;
                break_maxx = row_maxx;
            }
            // track last beginning of a word
            if ptype == Codepoint::Space && _type == Codepoint::Char || _type == Codepoint::CjkChar {
                word_start = iter.str_0;
                word_startx = iter.x;
                word_minx = q.x0 - row_startx;
            }

            // Break to new line when a character is beyond break width.
            if (_type == Codepoint::Char || _type == Codepoint::CjkChar) && next_width > break_row_width {
                // The run length is too long, need to break to new line.
                if break_end == row_start {
                    // The current word is longer than the row length, just break it from here.
                    rows[nrows] = TextRow {
                        start: row_start,
                        end: iter.str_0,
                        width: row_width * invscale,
                        minx:  row_minx * invscale,
                        maxx:  row_maxx * invscale,
                        next: iter.str_0,
                    };
                    nrows += 1;
                    if nrows >= rows.len() {
                        return &rows[..nrows];
                    }
                    row_startx = iter.x;
                    row_start = iter.str_0;
                    row_end = iter.next;
                    row_width = iter.nextx - row_startx;
                    row_minx = q.x0 - row_startx;
                    row_maxx = q.x1 - row_startx;

                    word_start = iter.str_0;
                    word_startx = iter.x;
                    word_minx = q.x0 - row_startx;
                } else {
                    // Break the line from the end of the last word,
                    // and start new line from the beginning of the new.
                    rows[nrows] = TextRow {
                        start: row_start,
                        end: break_end,
                        width: break_width * invscale,
                        minx: row_minx * invscale,
                        maxx: break_maxx * invscale,
                        next: word_start,
                    };
                    nrows += 1;
                    if nrows >= rows.len() {
                        return &rows[..nrows];
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

            pcodepoint = iter.codepoint;
            ptype = _type;
        }

        // Break the line from the end of the last word, and start new line from the beginning of the new.
        if !row_start.is_null() {
            rows[nrows] = TextRow {
                start: row_start,
                end: row_end,
                width: row_width * invscale,
                minx: row_minx * invscale,
                maxx: row_maxx * invscale,
                next: end,
            };
            nrows += 1;
        }

        &rows[..nrows]
    }
}