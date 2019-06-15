pub unsafe fn read_i16(p: *const u8) -> i16 {
    (*p.offset(0) as i16).wrapping_mul(256) + (*p.offset(1) as i16)
}
pub unsafe fn read_u16(p: *const u8) -> u16 {
    (*p.offset(0) as u16) * 256 + (*p.offset(1) as u16)
}
pub unsafe fn read_u32(p: *const u8) -> u32 {
    (((*p.offset(0) as i32) << 24)
        + ((*p.offset(1) as i32) << 16)
        + ((*p.offset(2) as i32) << 8)
        + *p.offset(3) as i32) as u32
}

// Copyright (c) 2008-2010 Bjoern Hoehrmann <bjoern@hoehrmann.de>
// See http://bjoern.hoehrmann.de/utf-8/decoder/dfa/ for details.
pub unsafe fn decutf8(state: *mut u32, codep: *mut u32, byte: u32) -> u32 {
    static mut UTF8D: [u8; 364] = [
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
    let type_0: u32 = UTF8D[byte as usize] as u32;
    *codep = if *state != 0 {
        byte & 0x3f | *codep << 6
    } else {
        (0xff >> type_0) as u32 & byte
    };
    *state = UTF8D[256u32.wrapping_add(*state).wrapping_add(type_0) as usize] as u32;
    *state
}

// Based on Exponential blur, Jani Huhtanen, 2006
pub unsafe fn blur(dst: *mut u8, w: i32, h: i32, stride: i32, blur: i32) {
    if blur < 1 {
        return;
    }

    let sigma = blur as f32 * 0.57735;
    let alpha = ((1 << 16) as f32 * (1.0 - (-2.3 / (sigma + 1.0)).exp())) as i32;

    blur_rows(dst, w, h, stride, alpha);
    blur_cols(dst, w, h, stride, alpha);
    blur_rows(dst, w, h, stride, alpha);
    blur_cols(dst, w, h, stride, alpha);
}


unsafe fn blur_cols(mut dst: *mut u8, width: i32, height: i32, stride: i32, alpha: i32) {
    let mut y = 0;
    while y < height {
        let mut z = 0;
        let mut x = 1;
        while x < width {
            let p = i32::from(*dst.add(x as usize));
            z += (alpha * ((p << 7) - z)) >> 16;
            *dst.add(x as usize) = (z >> 7) as u8;
            x += 1;
        }
        *dst.add((width - 1) as usize) = 0;

        let mut z = 0;
        let mut x = width - 2;
        while x >= 0 {
            let p = i32::from(*dst.add(x as usize));
            z += (alpha * ((p << 7) - z)) >> 16;
            *dst.offset(x as isize) = (z >> 7) as u8;
            x -= 1;
        }

        *dst.add(0) = 0;
        dst = dst.add(stride as usize);
        y += 1
    }
}

unsafe fn blur_rows(mut dst: *mut u8, width: i32, height: i32, stride: i32, alpha: i32) {
    let mut x = 0;
    while x < width {
        let mut z = 0;
        let mut y = stride;
        while y < height * stride {
            let p = i32::from(*dst.add(y as usize));
            z += (alpha * ((p << 7) - z)) >> 16;
            *dst.add(y as usize) = (z >> 7) as u8;
            y += stride
        }
        *dst.add(((height - 1) * stride) as usize) = 0;

        let mut z = 0;
        let mut y = (height - 2) * stride;
        while y >= 0 {
            let p = i32::from(*dst.add(y as usize));
            z += (alpha * ((p << 7) - z)) >> 16;
            *dst.add(y as usize) = (z >> 7) as u8;
            y -= stride
        }

        *dst.add(0) = 0;
        dst = dst.add(1);
        x += 1
    }
}