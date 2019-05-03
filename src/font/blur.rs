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