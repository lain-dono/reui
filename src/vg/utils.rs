use std::f32::consts::PI;

use euclid::approxeq::ApproxEq;

use crate::{Transform, Vector, Point};

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

pub fn vec_mul(lhs: Vector, rhs: Vector) -> Vector {
    Vector::new(
        lhs.x * rhs.x,
        lhs.y * rhs.y,
    )
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

pub fn pt_eq(x1: f32, y1: f32, x2: f32, y2: f32, tol: f32) -> bool {
    let p1 = Point::new(x1, y1);
    let p2 = Point::new(x2, y2);
    let tol = Point::new(tol, tol);
    p1.approx_eq_eps(&p2, &tol)

    /*
    let dx = x2 - x1;
    let dy = y2 - y1;
    dx*dx + dy*dy < tol*tol
    */
}

pub fn dist_pt_seg(x: f32, y: f32, px: f32, py: f32, qx: f32, qy: f32) -> f32 {
    let pqx = qx-px;
    let pqy = qy-py;
    let dx = x-px;
    let dy = y-py;
    let d = pqx*pqx + pqy*pqy;

    let mut t = pqx*dx + pqy*dy;
    if d > 0.0 { t /= d }
    let t = if t < 0.0 {
        0.0
    } else if t > 1.0 {
        1.0
    } else { t };

    let dx = px + t*pqx - x;
    let dy = py + t*pqy - y;
    dx*dx + dy*dy
}

pub fn average_scale(t: &Transform) -> f32 {
    let sx = (t.m11*t.m11 + t.m12*t.m12).sqrt();
    let sy = (t.m21*t.m21 + t.m22*t.m22).sqrt();
    (sx + sy) * 0.5
}

#[inline(always)]
pub const fn pack_uv(u: f32, v: f32) -> [u16; 2] {
    let u = (u * 65535.0) as u16;
    let v = (v * 65535.0) as u16;
    [u, v]
}

pub fn deg2rad(deg: f32) -> f32 { deg / 180.0 * PI }
pub fn rad2deg(rad: f32) -> f32 { rad / PI * 180.0 }

pub fn _xfloor(f: f32) -> f32 {
    f - (f % 1.0)
}

pub fn minf(a: f32, b: f32) -> f32 { min(a, b) }
pub fn maxf(a: f32, b: f32) -> f32 { max(a, b) }
pub fn clampf(a: f32, mn: f32, mx: f32) -> f32 { clamp(a, mn, mx) }

pub fn mini(a: i32, b: i32) -> i32 { min(a, b) }
pub fn maxi(a: i32, b: i32) -> i32 { max(a, b) }
pub fn clampi(a: i32, mn: i32, mx: i32) -> i32 { clamp(a, mn, mx) }

pub fn min<I: PartialOrd>(a: I, b: I) -> I {
    if a < b { a } else { b }
}

pub fn max<I: PartialOrd>(a: I, b: I) -> I {
    if a > b { a } else { b }
}

pub fn clamp<I: PartialOrd>(a: I, mn: I, mx: I) -> I {
    if a < mn { mn } else if a > mx { mx } else { a }
}