use super::utils::*;
use super::stash::*;

// Each .ttf/.ttc file may have more than one font. Each font has a sequential
// index number starting from 0. Call this function to get the font offset for
// a given index; it returns -1 if the index is out of range. A regular .ttf
// file will only define one font and it always be at offset 0, so it will
// return '0' for index 0, and -1 for all other indices. You can just skip
// this step if you know it's that kind of font.
// The following structure is defined publically so you can declare one on
// the stack or as a global or etc, but you should treat it as opaque.
#[derive(Clone)]
#[repr(C)]
pub struct FontInfo {
    pub userdata: *mut Stash,
    pub data: *mut u8,
    pub fontstart: i32,
    pub num_glyphs: i32,
    pub loca: i32,
    pub head: i32,
    pub glyf: i32,
    pub hhea: i32,
    pub hmtx: i32,
    pub kern: i32,
    pub index_map: i32,
    pub index2loc_format: i32,
}

impl FontInfo {
    pub unsafe fn pixel_height_scale(&self, height: f32) -> f32 {
        let fheight: i32 = read_i16(self.data.offset(self.hhea as isize).offset(4)) as i32
            - read_i16(self.data.offset(self.hhea as isize).offset(6)) as i32;
        height as f32 / fheight as f32
    }

    pub unsafe fn glyph_kern_advance(&self, glyph1: i32, glyph2: i32) -> i32 {
        let data: *mut u8 = self.data.offset(self.kern as isize);
        if 0 == self.kern {
            return 0;
        }
        if (read_u16(data.offset(2)) as i32) < 1 {
            return 0;
        }
        if read_u16(data.offset(8)) != 1 {
            return 0;
        }
        let mut l = 0;
        let mut r = read_u16(data.offset(10)) as i32 - 1;
        let needle = (glyph1 << 16 | glyph2) as u32;
        while l <= r {
            let m = (l + r) >> 1;
            let straw = read_u32(data.offset(18).offset((m * 6) as isize));
            if needle < straw {
                r = m - 1
            } else if needle > straw {
                l = m + 1
            } else {
                return read_i16(data.offset(22).offset((m * 6) as isize)) as i32;
            }
        }
        0
    }

    pub unsafe fn glyf_offset(&self, glyph_index: i32) -> Option<i32> {
        if glyph_index >= self.num_glyphs {
            return None;
        }
        if self.index2loc_format >= 2 {
            return None;
        }
        let (g1, g2);
        if self.index2loc_format == 0 {
            let p = self.data.offset(self.loca as isize + (glyph_index * 2) as isize);
            g1 = self.glyf + read_u16(p) as i32 * 2;
            let p = self.data.offset(self.loca as isize + (glyph_index * 2) as isize + 2);
            g2 = self.glyf + read_u16(p) as i32 * 2
        } else {
            let p = self .data.offset(self.loca as isize + (glyph_index * 4) as isize);
            g1 = self.glyf.wrapping_add(read_u32(p) as i32);
            let p = self.data.offset(self.loca as isize + (glyph_index * 4) as isize + 4);
            g2 = self.glyf.wrapping_add(read_u32(p) as i32);
        }
        if g1 == g2 { None } else { Some(g1) }
    }

    pub unsafe fn glyph_bitmap_box_subpixel(
        &self,
        glyph: i32,
        scale_x: f32,
        scale_y: f32,
        shift_x: f32,
        shift_y: f32,
    ) -> [i32; 4] {
        self.glyf_offset(glyph).map(|g| {
            let g = g as isize;
            let x0 = read_i16(self.data.offset(g + 2));
            let y0 = read_i16(self.data.offset(g + 4));
            let x1 = read_i16(self.data.offset(g + 6));
            let y1 = read_i16(self.data.offset(g + 8));

            [
                ( x0 as f32 * scale_x + shift_x).floor() as i32,
                (-y1 as f32 * scale_y + shift_y).floor() as i32,
                ( x1 as f32 * scale_x + shift_x).ceil() as i32,
                (-y0 as f32 * scale_y + shift_y).ceil() as i32,
            ]
        }).unwrap_or_default()
    }

    pub unsafe fn build_glyph_bitmap(
        &self,
        glyph: i32,
        _size: f32,
        scale: f32,
        advance: &mut i32,
        lsb: &mut i32,
    ) -> [i32; 4] {
        self.glyph_h_metrics(glyph, advance, lsb);
        self.glyph_bitmap_box_subpixel(glyph, scale, scale, 0.0, 0.0)
    }

    // Gets the bounding box of the visible part of the glyph, in unscaled coordinates
    pub unsafe fn glyph_h_metrics(
        &self,
        glyph_index: i32,
        advance_width: &mut i32,
        left_side_bearing: &mut i32,
    ) {
        let num_of_long_hor_metrics = read_u16(self.data.offset(self.hhea as isize).offset(34));
        if glyph_index < num_of_long_hor_metrics as i32 {
            *advance_width = read_i16(
                self.data
                    .offset(self.hmtx as isize)
                    .offset((4 * glyph_index) as isize),
            ) as i32;
            *left_side_bearing = read_i16(
                self.data
                    .offset(self.hmtx as isize)
                    .offset((4 * glyph_index) as isize)
                    .offset(2),
            ) as i32;
        } else {
            *advance_width = read_i16(
                self.data
                    .offset(self.hmtx as isize)
                    .offset((4 * (num_of_long_hor_metrics as i32 - 1)) as isize),
            ) as i32;
            *left_side_bearing = read_i16(
                self.data
                    .offset(self.hmtx as isize)
                    .offset((4 * num_of_long_hor_metrics as i32) as isize)
                    .offset((2 * (glyph_index - num_of_long_hor_metrics as i32)) as isize),
            ) as i32;
        }
    }

    pub unsafe fn glyph_index(&self, unicode_codepoint: i32) -> i32 {
        let data: *mut u8 = self.data;
        let index_map: u32 = self.index_map as u32;
        let format: u16 = read_u16(data.offset(index_map as isize).offset(0));
        if format == 0 {
            let bytes: i32 = read_u16(data.offset(index_map as isize).offset(2)) as i32;
            if unicode_codepoint < bytes - 6 {
                return *(data.offset(index_map as isize + 6 + unicode_codepoint as isize) as *mut u8) as i32;
            }
            return 0;
        }

        if format == 6 {
            let first: u32 = read_u16(data.offset(index_map as isize).offset(6)) as u32;
            let count: u32 = read_u16(data.offset(index_map as isize).offset(8)) as u32;
            if unicode_codepoint as u32 >= first && (unicode_codepoint as u32) < first.wrapping_add(count) {
                return read_u16(
                    data.offset(index_map as isize).offset(10).offset(
                        (unicode_codepoint as u32)
                            .wrapping_sub(first)
                            .wrapping_mul(2 as u32) as isize,
                    ),
                ) as i32;
            }
            return 0;
        }

        assert!(format != 2);

        if format == 4 {
            let segcount: u16 =
                (read_u16(data.offset(index_map as isize).offset(6)) as i32 >> 1) as u16;
            let mut search_range: u16 =
                (read_u16(data.offset(index_map as isize).offset(8)) as i32 >> 1) as u16;
            let mut entry_selector: u16 =
                read_u16(data.offset(index_map as isize).offset(10));
            let range_shift: u16 =
                (read_u16(data.offset(index_map as isize).offset(12)) as i32 >> 1) as u16;
            let end_count: u32 = index_map.wrapping_add(14);
            let mut search: u32 = end_count;
            if unicode_codepoint > 0xffff {
                return 0;
            }
            if unicode_codepoint
                >= read_u16(
                    data.offset(search as isize)
                        .offset((range_shift as i32 * 2) as isize),
                ) as i32
            {
                search = (search as u32).wrapping_add((range_shift as i32 * 2) as u32) as u32
                    as u32
            }
            search = (search as u32).wrapping_sub(2 as u32) as u32 as u32;
            while 0 != entry_selector {
                search_range = (search_range as i32 >> 1) as u16;
                let end = read_u16(
                    data.offset(search as isize)
                        .offset((search_range as i32 * 2) as isize),
                );
                if unicode_codepoint > end as i32 {
                    search = (search as u32).wrapping_add((search_range as i32 * 2) as u32)
                        as u32 as u32
                }
                entry_selector = entry_selector.wrapping_sub(1)
            }
            search = (search as u32).wrapping_add(2 as u32) as u32 as u32;
            let offset;
            let start;
            let item: u16 = (search.wrapping_sub(end_count) >> 1) as u16;

            assert!(
                unicode_codepoint
                    <= read_u16(
                        data.offset(end_count as isize)
                            .offset((2 * item as i32) as isize)
                    ) as i32
            );

            start = read_u16(
                data.offset(index_map as isize)
                    .offset(14)
                    .offset((segcount as i32 * 2) as isize)
                    .offset(2)
                    .offset((2 * item as i32) as isize),
            );
            if unicode_codepoint < start as i32 {
                return 0;
            }
            offset = read_u16(
                data.offset(index_map as isize)
                    .offset(14)
                    .offset((segcount as i32 * 6) as isize)
                    .offset(2)
                    .offset((2 * item as i32) as isize),
            );
            if offset as i32 == 0 {
                return (unicode_codepoint
                    + read_i16(
                        data.offset(index_map as isize)
                            .offset(14)
                            .offset((segcount as i32 * 4) as isize)
                            .offset(2)
                            .offset((2 * item as i32) as isize),
                    ) as i32) as u16 as i32;
            }
            return read_u16(
                data.offset(offset as i32 as isize)
                    .offset(((unicode_codepoint - start as i32) * 2) as isize)
                    .offset(index_map as isize)
                    .offset(14)
                    .offset((segcount as i32 * 6) as isize)
                    .offset(2)
                    .offset((2 * item as i32) as isize),
            ) as i32;
        } else if format == 12 || format == 13 {
            let ngroups: u32 = read_u32(data.offset(index_map as isize).offset(12));
            let mut low = 0;
            let mut high = ngroups as i32;
            while low < high {
                let mid: i32 = low + ((high - low) >> 1);
                let start_char: u32 = read_u32(
                    data.offset(index_map as isize)
                        .offset(16)
                        .offset((mid * 12) as isize),
                );
                let end_char: u32 = read_u32(
                    data.offset(index_map as isize)
                        .offset(16)
                        .offset((mid * 12) as isize)
                        .offset(4),
                );
                if (unicode_codepoint as u32) < start_char {
                    high = mid
                } else if unicode_codepoint as u32 > end_char {
                    low = mid + 1
                } else {
                    let start_glyph: u32 = read_u32(
                        data.offset(index_map as isize)
                            .offset(16)
                            .offset((mid * 12) as isize)
                            .offset(8),
                    );
                    if format == 12 {
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

        unreachable!()
    }

    pub unsafe fn render_glyph_bitmap(
        &self,
        output: *mut u8,
        out_w: i32,
        out_h: i32,
        out_stride: i32,
        scale_x: f32,
        scale_y: f32,
        glyph: i32,
    ) {
        let shift_x = 0.0;
        let shift_y = 0.0;

        let mut vertices: *mut Vertex = 0 as *mut Vertex;
        let num_verts: i32 = stbtt_GetGlyphShape(self, glyph, &mut vertices);
        let [ix0, iy0, _, _] = self.glyph_bitmap_box_subpixel(
            glyph,
            scale_x,
            scale_y,
            shift_x,
            shift_y,
        );

        let mut gbm = Bitmap {
            pixels: output,
            w: out_w,
            h: out_h,
            stride: out_stride,
        };

        if 0 != gbm.w && 0 != gbm.h {
            stbtt_Rasterize(
                &mut gbm,
                0.35,
                vertices,
                num_verts,
                scale_x,
                scale_y,
                shift_x,
                shift_y,
                ix0,
                iy0,
                1,
                &mut *self.userdata,
            );
        }
    }
}