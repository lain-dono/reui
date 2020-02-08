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

impl Default for FontInfo {
    fn default() -> Self {
        Self {
            userdata: std::ptr::null_mut(),
            data: std::ptr::null_mut(),
            fontstart: 0,
            num_glyphs: 0,
            loca: 0,
            head: 0,
            glyf: 0,
            hhea: 0,
            hmtx: 0,
            kern: 0,
            index_map: 0,
            index2loc_format: 0,
        }
    }
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
            match needle.cmp(&straw) {
                std::cmp::Ordering::Less => r = m - 1,
                std::cmp::Ordering::Greater => l = m + 1,
                std::cmp::Ordering::Equal => {
                    return read_i16(data.offset(22).offset((m * 6) as isize)) as i32;
                }
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
            let x0 = read_i16(self.data.offset(g + 2)) as f32;
            let y0 = read_i16(self.data.offset(g + 4)) as f32;
            let x1 = read_i16(self.data.offset(g + 6)) as f32;
            let y1 = read_i16(self.data.offset(g + 8)) as f32;

            [
                ( x0 * scale_x + shift_x).floor() as i32,
                (-y1 * scale_y + shift_y).floor() as i32,
                ( x1 * scale_x + shift_x).ceil() as i32,
                (-y0 * scale_y + shift_y).ceil() as i32,
            ]
        }).unwrap_or_default()
    }

    // Gets the bounding box of the visible part of the glyph, in unscaled coordinates
    pub unsafe fn glyph_h_metrics(
        &self,
        glyph_index: i32,
        advance_width: &mut i32,
        left_side_bearing: &mut i32,
    ) {
        let glyph_index = glyph_index as usize;
        let num_of_long_hor_metrics =
            read_u16(self.data.add(self.hhea as usize + 34)) as usize;
        let hmtx = self.hmtx as usize;
        let (a, b);
        if glyph_index < num_of_long_hor_metrics {
            a = hmtx + 4 * glyph_index;
            b = hmtx + 4 * glyph_index + 2;
        } else {
            a = hmtx + 4 * (num_of_long_hor_metrics - 1);
            b = hmtx + 4 * num_of_long_hor_metrics
                     + 2 * (glyph_index - num_of_long_hor_metrics);
        }

        *advance_width = read_i16(self.data.add(a)) as i32;
        *left_side_bearing = read_i16(self.data.add(b)) as i32;
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
            let first: u32 = read_u16(data.offset(index_map as isize + 6)) as u32;
            let count: u32 = read_u16(data.offset(index_map as isize + 8)) as u32;
            return if unicode_codepoint as u32 >= first && (unicode_codepoint as u32) < first.wrapping_add(count) {
                read_u16(
                    data.offset(index_map as isize + 10 +
                        (unicode_codepoint as u32)
                            .wrapping_sub(first)
                            .wrapping_mul(2 as u32) as isize,
                    ),
                ) as i32
            } else {
                0
            };
        }

        assert!(format != 2);

        if format == 4 {
            let segcount: u16 = (read_u16(data.offset(index_map as isize + 6)) as i32 >> 1) as u16;
            let mut search_range: u16 = (read_u16(data.offset(index_map as isize + 8)) as i32 >> 1) as u16;
            let mut entry_selector: u16 = read_u16(data.offset(index_map as isize + 10));
            let range_shift: u16 = (read_u16(data.offset(index_map as isize + 12)) as i32 >> 1) as u16;

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
                    data.offset(search as isize + (search_range as i32 * 2) as isize),
                );
                if unicode_codepoint > end as i32 {
                    search = (search as u32).wrapping_add((search_range as i32 * 2) as u32) as u32 as u32
                }
                entry_selector = entry_selector.wrapping_sub(1)
            }
            search = (search as u32).wrapping_add(2) as u32;
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
                data.offset(index_map as isize + 14)
                    .offset((segcount as i32 * 2) as isize)
                    .offset(2 + (2 * item as i32) as isize),
            );
            if unicode_codepoint < start as i32 {
                return 0;
            }
            offset = read_u16(
                data.offset(index_map as isize + 14)
                    .offset((segcount as i32 * 6) as isize)
                    .offset(2 + (2 * item as i32) as isize),
            );
            if offset as i32 == 0 {
                return (unicode_codepoint
                    + read_i16(
                        data.offset(index_map as isize + 14)
                            .offset((segcount as i32 * 4) as isize)
                            .offset(2 + (2 * item as i32) as isize),
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
                    data.offset(index_map as isize + 16)
                        .offset((mid * 12) as isize),
                );
                let end_char: u32 = read_u32(
                    data.offset(index_map as isize + 16)
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
        pixels: *mut u8,
        out_w: i32,
        out_h: i32,
        stride: i32,
        scale: [f32; 2],
        glyph: i32,
    ) {
        let shift_x = 0.0;
        let shift_y = 0.0;
        let [scale_x, scale_y] = scale;

        let mut vertices: *mut Vertex = std::ptr::null_mut();
        let num_verts = self.glyph_shape(glyph, &mut vertices);
        let [ix0, iy0, _, _] = self.glyph_bitmap_box_subpixel(
            glyph,
            scale_x,
            scale_y,
            shift_x,
            shift_y,
        );

        let mut gbm = Bitmap {
            pixels,
            w: out_w,
            h: out_h,
            stride,
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

    pub unsafe fn glyph_shape(
        &self,
        glyph_index: i32,
        pvertices: *mut *mut Vertex,
    ) -> i32 {
        let data: *mut u8 = self.data;
        let mut vertices: *mut Vertex = std::ptr::null_mut();
        let mut num_vertices: i32 = 0;
        *pvertices = std::ptr::null_mut();

        let g = match self.glyf_offset(glyph_index) {
            Some(g) => g as isize,
            None => return 0,
        };

        let num_contours = read_i16(data.offset(g)) as isize;
        if num_contours > 0 {
            let mut j = 0;
            let mut was_off = 0;
            let mut start_off = 0;

            let end_pts_of_contours = data.offset(g + 10);
            let ins = read_u16(data.offset(g + 10 + num_contours * 2)) as isize;

            let mut points = data
                .offset(g + 10 + num_contours * 2 + 2 + ins);

            let n = 1 + read_u16(end_pts_of_contours.offset(num_contours * 2 - 2)) as i32;

            let m = n + 2 * num_contours as i32;
            vertices = (*self.userdata).calloc(m as usize);
            if vertices.is_null() {
                return 0;
            }

            let mut next_move = 0;
            let mut flagcount = 0;
            let off = m - n;

            let mut flags = 0;
            for i in 0..n {
                if flagcount == 0 {
                    flags = *points;
                    points = points.offset(1);
                    if 0 != flags & 8 {
                        flagcount = *points;
                        points = points.offset(1);
                    }
                } else {
                    flagcount = flagcount.wrapping_sub(1)
                }
                (*vertices.offset((off + i) as isize)).type_0 = flags;
            }

            let mut x = 0;
            for i in 0..n {
                let flags = (*vertices.offset((off + i) as isize)).type_0;
                if 0 != flags & 2 {
                    let dx = *points as i32;
                    points = points.offset(1);
                    x += if 0 != flags & 16 { dx } else { -dx };
                } else if 0 == flags & 16 {
                    x += *points.offset(0) as i32 * 256 + *points.offset(1) as i32;
                    points = points.offset(2)
                }
                (*vertices.offset((off + i) as isize)).x = x as i16;
            }

            let mut y = 0;
            for i in 0..n {
                let flags = (*vertices.offset((off + i) as isize)).type_0;
                if 0 != flags & 4 {
                    let dy = *points as i32;
                    points = points.offset(1);
                    y += if 0 != flags & 32 { dy } else { -dy }
                } else if 0 == flags & 32 {
                    y += *points.offset(0) as i32 * 256 + *points.offset(1) as i32;
                    points = points.offset(2)
                }
                (*vertices.offset((off + i) as isize)).y = y as i16;
            }

            num_vertices = 0;
            let mut scy = 0;
            let mut scx = 0;
            let mut cy = 0;
            let mut cx = 0;
            let mut sy = 0;
            let mut sx = 0;
            let mut i = 0;
            while i < n {
                let flags = (*vertices.offset((off + i) as isize)).type_0;
                x = (*vertices.offset((off + i) as isize)).x as i32;
                y = (*vertices.offset((off + i) as isize)).y as i32;
                if next_move == i {
                    if i != 0 {
                        num_vertices = close_shape(
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
                            sx = (x + (*vertices.offset((off + i + 1) as isize)).x as i32) >> 1;
                            sy = (y + (*vertices.offset((off + i + 1) as isize)).y as i32) >> 1;
                        } else {
                            sx = (*vertices.offset((off + i + 1) as isize)).x as i32;
                            sy = (*vertices.offset((off + i + 1) as isize)).y as i32;
                            i += 1
                        }
                    } else {
                        sx = x;
                        sy = y;
                    }
                    setvertex(
                        &mut *vertices.offset(num_vertices as isize),
                        VMOVE as i32 as u8,
                        sx, sy, 0, 0,
                    );
                    num_vertices += 1;
                    was_off = 0;
                    next_move = 1 + read_u16(end_pts_of_contours.offset((j * 2) as isize)) as i32;
                    j += 1
                } else if 0 == flags as i32 & 1 {
                    if 0 != was_off {
                        setvertex(
                            &mut *vertices.offset(num_vertices as isize),
                            VCURVE as i32 as u8,
                            (cx + x) >> 1,
                            (cy + y) >> 1,
                            cx, cy,
                        );
                        num_vertices += 1;
                    }
                    cx = x;
                    cy = y;
                    was_off = 1
                } else {
                    if 0 != was_off {
                        setvertex(
                            &mut *vertices.offset(num_vertices as isize),
                            VCURVE as i32 as u8,
                            x, y, cx, cy,
                        );
                        num_vertices += 1;
                    } else {
                        setvertex(
                            &mut *vertices.offset(num_vertices as isize),
                            VLINE as i32 as u8,
                            x, y, 0, 0,
                        );
                        num_vertices += 1;
                    }
                    was_off = 0
                }
                i += 1
            }
            num_vertices = close_shape(
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
        } else if num_contours == -1 {
            return self.bad_contour(data, pvertices, g);
        } else if num_contours < 0 {
            unimplemented!();
        }

        *pvertices = vertices;
        num_vertices
    }

    unsafe fn bad_contour(
        &self,
        data: *mut u8,
        pvertices: *mut *mut Vertex,
        g: isize,
    ) -> i32 {
        let mut more: i32 = 1;
        let mut comp: *mut u8 = data.offset(g).offset(10);
        let mut num_vertices = 0;
        let mut vertices = std::ptr::null_mut();
        while 0 != more {
            let mut mtx: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
            let flags_0 = read_i16(comp) as u16;
            comp = comp.offset(2);
            let gidx = read_i16(comp) as u16;
            comp = comp.offset(2);

            assert!(0 != flags_0 & 2);
            if 0 != flags_0 as i32 & 1 {
                mtx[4] = read_i16(comp) as f32;
                comp = comp.offset(2);
                mtx[5] = read_i16(comp) as f32;
                comp = comp.offset(2)
            } else {
                mtx[4] = *(comp as *mut i8) as f32;
                comp = comp.offset(1);
                mtx[5] = *(comp as *mut i8) as f32;
                comp = comp.offset(1)
            }

            if 0 != flags_0 & 1 << 3 {
                mtx[0] = read_i16(comp) as f32 / 16384.0;
                comp = comp.offset(2);
                mtx[1] = 0.0;
                mtx[2] = 0.0;
                mtx[3] = mtx[0];
            } else if 0 != flags_0 & 1 << 6 {
                mtx[0] = read_i16(comp) as f32 / 16384.0;
                comp = comp.offset(2);
                mtx[1] = 0.0;
                mtx[2] = 0.0;
                mtx[3] = read_i16(comp) as f32 / 16384.0;
                comp = comp.offset(2)
            } else if 0 != flags_0 as i32 & 1 << 7 {
                mtx[0] = read_i16(comp) as f32 / 16384.0;
                comp = comp.offset(2);
                mtx[1] = read_i16(comp) as f32 / 16384.0;
                comp = comp.offset(2);
                mtx[2] = read_i16(comp) as f32 / 16384.0;
                comp = comp.offset(2);
                mtx[3] = read_i16(comp) as f32 / 16384.0;
                comp = comp.offset(2)
            }

            let m_0 = (mtx[0] * mtx[0] + mtx[1] * mtx[1]).sqrt();
            let n_0 = (mtx[2] * mtx[2] + mtx[3] * mtx[3]).sqrt();
            let mut comp_verts = std::ptr::null_mut();
            let comp_num_verts = self.glyph_shape(gidx as i32, &mut comp_verts);
            if comp_num_verts > 0 {
                for i_0 in 0..comp_num_verts {
                    let mut v = &mut *comp_verts.offset(i_0 as isize);
                    let x = v.x as f32;
                    let y = v.y as f32;
                    v.x = (m_0 * (mtx[0] * x + mtx[2] * y + mtx[4])) as i16;
                    v.y = (n_0 * (mtx[1] * x + mtx[3] * y + mtx[5])) as i16;
                    let x = v.cx as f32;
                    let y = v.cy as f32;
                    v.cx = (m_0 * (mtx[0] * x + mtx[2] * y + mtx[4])) as i16;
                    v.cy = (n_0 * (mtx[1] * x + mtx[3] * y + mtx[5])) as i16;
                }
                let tmp: *mut Vertex = (*self.userdata).calloc((num_vertices + comp_num_verts) as usize);
                if tmp.is_null() {
                    return 0;
                }
                if num_vertices > 0 {
                    std::ptr::copy(vertices, tmp, num_vertices as usize);
                }
                std::ptr::copy(comp_verts, tmp.offset(num_vertices as isize), comp_num_verts as usize);
                vertices = tmp;
                num_vertices += comp_num_verts
            }
            more = flags_0 as i32 & 1 << 5
        }

        *pvertices = vertices;
        num_vertices
    }

}

unsafe fn close_shape(
    vertices: *mut Vertex,
    mut num_vertices: i32,
    was_off: i32,
    start_off: i32,
    sx: i32,
    sy: i32,
    scx: i32,
    scy: i32,
    cx: i32,
    cy: i32,
) -> i32 {
    if 0 != start_off {
        if 0 != was_off {
            setvertex(
                &mut *vertices.offset(num_vertices as isize),
                VCURVE as u8,
                (cx + scx) >> 1,
                (cy + scy) >> 1,
                cx, cy,
            );
            num_vertices += 1;
        }
        setvertex(
            &mut *vertices.offset(num_vertices as isize),
            VCURVE as u8,
            sx, sy,
            scx, scy,
        );
    } else if 0 != was_off {
        setvertex(
            &mut *vertices.offset(num_vertices as isize),
            VCURVE as u8,
            sx, sy,
            cx, cy,
        );
    } else {
        setvertex(
            &mut *vertices.offset(num_vertices as isize),
            VLINE as u8,
            sx, sy,
            0, 0,
        );
    }
    num_vertices + 1
}

#[inline(always)]
unsafe fn setvertex(mut v: *mut Vertex, type_0: u8, x: i32, y: i32, cx: i32, cy: i32) {
    (*v).type_0 = type_0;
    (*v).x = x as i16;
    (*v).y = y as i16;
    (*v).cx = cx as i16;
    (*v).cy = cy as i16;
}