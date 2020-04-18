use crate::math::Transform;


pub fn average_scale(t: &Transform) -> f32 {
    t.re * t.re + t.im * t.im
}

#[inline(always)]
pub const fn pack_uv(u: f32, v: f32) -> [u16; 2] {
    let u = (u * 65535.0) as u16;
    let v = (v * 65535.0) as u16;
    [u, v]
}
