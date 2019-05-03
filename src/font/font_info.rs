use super::utils::*;

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

impl FontInfo {
    pub unsafe fn pixel_height_scale(&self, height: f32) -> f32 {
        let fheight: i32 = read_i16(self.data.offset(self.hhea as isize).offset(4)) as i32
            - read_i16(self.data.offset(self.hhea as isize).offset(6)) as i32;
        height as f32 / fheight as f32
    }

    pub unsafe fn glyph_kern_advance(&self, glyph1: i32, glyph2: i32) -> i32 {
        let mut data: *mut u8 = self.data.offset(self.kern as isize);
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
            let m = l + r >> 1;
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
}

pub unsafe fn stbtt__GetGlyfOffset(mut info: *const FontInfo, mut glyph_index: i32) -> i32 {
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
                    .offset((*info).loca as isize + (glyph_index * 2i32) as isize),
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

pub unsafe fn stbtt_GetGlyphBitmapBoxSubpixel(
    info: *const FontInfo,
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
    let g = stbtt__GetGlyfOffset(info, glyph) as isize;
    if g < 0 {
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
        let x0 = read_i16((*info).data.offset(g + 2)) as i32;
        let y0 = read_i16((*info).data.offset(g + 4)) as i32;
        let x1 = read_i16((*info).data.offset(g + 6)) as i32;
        let y1 = read_i16((*info).data.offset(g + 8)) as i32;

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

pub unsafe fn fons__tt_buildGlyphBitmap(
    font: &mut FontInfo,
    glyph: i32,
    _size: f32,
    scale: f32,
    advance: &mut i32,
    lsb: &mut i32,
    x0: &mut i32,
    y0: &mut i32,
    x1: &mut i32,
    y1: &mut i32,
) -> i32 {
    stbtt_GetGlyphHMetrics(font, glyph, advance, lsb);
    stbtt_GetGlyphBitmapBoxSubpixel(font, glyph, scale, scale, 0.0, 0.0, x0, y0, x1, y1);
    1
}

// Gets the bounding box of the visible part of the glyph, in unscaled coordinates
pub unsafe fn stbtt_GetGlyphHMetrics(
    info: &FontInfo,
    glyph_index: i32,
    advance_width: &mut i32,
    left_side_bearing: &mut i32,
) {
    let numOfLongHorMetrics = read_u16(info.data.offset(info.hhea as isize).offset(34));
    if glyph_index < numOfLongHorMetrics as i32 {
        *advance_width = read_i16(
            info.data
                .offset(info.hmtx as isize)
                .offset((4 * glyph_index) as isize),
        ) as i32;
        *left_side_bearing = read_i16(
            info.data
                .offset(info.hmtx as isize)
                .offset((4 * glyph_index) as isize)
                .offset(2),
        ) as i32;
    } else {
        *advance_width = read_i16(
            info.data
                .offset(info.hmtx as isize)
                .offset((4 * (numOfLongHorMetrics as i32 - 1)) as isize),
        ) as i32;
        *left_side_bearing = read_i16(
            info.data
                .offset(info.hmtx as isize)
                .offset((4 * numOfLongHorMetrics as i32) as isize)
                .offset((2 * (glyph_index - numOfLongHorMetrics as i32)) as isize),
        ) as i32;
    }
}

pub unsafe fn fons__tt_getGlyphIndex(info: *mut FontInfo, unicode_codepoint: i32) -> i32 {
    let data: *mut u8 = (*info).data;
    let index_map: u32 = (*info).index_map as u32;
    let format: u16 = read_u16(data.offset(index_map as isize).offset(0isize));
    if format as i32 == 0 {
        let bytes: i32 = read_u16(data.offset(index_map as isize).offset(2isize)) as i32;
        if unicode_codepoint < bytes - 6i32 {
            return *(data
                .offset(index_map as isize)
                .offset(6)
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
            if format == 2 {
                panic!("ff")
            } else {
                if format == 4 {
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

                    assert!(unicode_codepoint <= read_u16(data.offset(endCount as isize)
                                .offset((2i32 * item as i32) as isize)) as i32);

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
                            let mid: i32 = low + ((high - low) >> 1);
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
                                        .offset(16)
                                        .offset((mid * 12i32) as isize)
                                        .offset(8),
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

    unreachable!()
}