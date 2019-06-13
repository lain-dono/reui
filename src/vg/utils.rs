use std::{
    f32::consts::PI,
    str::from_utf8_unchecked,
    slice::from_raw_parts,
};

use crate::Transform;

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

pub unsafe fn raw_str<'a>(start: *const u8, end: *const u8) -> &'a str {
    from_utf8_unchecked(raw_slice(start, end))
}

pub unsafe fn raw_slice<'a>(start: *const u8, end: *const u8) -> &'a [u8] {
    let len = if end.is_null() {
        libc::strlen(start as *const i8)
    } else {
        end.offset_from(start) as usize
    };
    from_raw_parts(start, len)
}

#[derive(Clone, Copy)]
pub struct Pt {
    pub x: f32,
    pub y: f32,
}

impl Into<[f32; 2]> for Pt {
    fn into(self) -> [f32; 2] {
        [self.x, self.y]
    }
}

impl From<[f32; 2]> for Pt {
    fn from(p: [f32; 2]) -> Self {
        Self::new(p[0], p[1])
    }
}

impl std::ops::Mul<f32> for Pt {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Add for Pt {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Pt {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl std::ops::Mul for Pt {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Pt {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
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

pub fn normalize_pt(x: f32, y: f32) -> (f32, f32) {
    let d = (x * x + y * y).sqrt();
    if d > 1e-6 {
        let id = 1.0 / d;
        (x * id, y * id)
    } else {
        (x, y)
    }
}

pub fn cross(dx0: f32, dy0: f32, dx1: f32, dy1: f32) -> f32 { dx1*dy0 - dx0*dy1 }

pub fn pt_eq(x1: f32, y1: f32, x2: f32, y2: f32, tol: f32) -> bool {
    let dx = x2 - x1;
    let dy = y2 - y1;
    dx*dx + dy*dy < tol*tol
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