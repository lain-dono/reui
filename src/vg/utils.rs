use crate::math::{Transform, Offset};

pub fn slice_start_end(b: &[u8]) -> (*const u8, *const u8) {
    unsafe {
        let start = b.as_ptr();
        let end = start.add(b.len());
        (start, end)
    }
}

pub fn str_start_end(s: &str) -> (*const u8, *const u8) {
    slice_start_end(s.as_bytes())
}

pub fn normalize(x: &mut f32, y: &mut f32) -> f32 {
    let xx = (*x) * (*x);
    let yy = (*y) * (*y);
    let d = (xx + yy).sqrt();
    if d > 1e-6 {
        let id = 1.0 / d;
        *x *= id;
        *y *= id;
    }
    d
}

/*
pub fn pt_eq(x1: f32, y1: f32, x2: f32, y2: f32, tol: f32) -> bool {
    let p1 = Point::new(x1, y1);
    let p2 = Point::new(x2, y2);
    p1.approx_eq_eps(p2, tol)
}
*/

pub fn average_scale(t: &Transform) -> f32 {
    (t.re*t.re+ t.im*t.im)
}

#[inline(always)]
pub const fn pack_uv(u: f32, v: f32) -> [u16; 2] {
    let u = (u * 65535.0) as u16;
    let v = (v * 65535.0) as u16;
    [u, v]
}