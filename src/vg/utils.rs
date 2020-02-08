use crate::math::{Transform};

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

pub fn average_scale(t: &Transform) -> f32 {
    (t.re*t.re+ t.im*t.im)
}

#[inline(always)]
pub const fn pack_uv(u: f32, v: f32) -> [u16; 2] {
    let u = (u * 65535.0) as u16;
    let v = (v * 65535.0) as u16;
    [u, v]
}