mod color;
mod offset;
mod rect;
mod transform;

pub use self::{color::Color, offset::Offset, rect::Rect, transform::Transform};

impl self::transform::Transform {
    pub fn transform_point(&self, p: Offset) -> Offset {
        self.apply(p.into()).into()
    }
}

#[inline]
pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_ltwh(x, y, w, h)
}

#[inline]
pub fn vec2(x: f32, y: f32) -> Offset {
    Offset { x, y }
}

#[inline]
pub fn point2(x: f32, y: f32) -> Offset {
    Offset { x, y }
}

/*
fn convert_radius_to_sigma(radius: f32) -> f32 {
    radius * 0.57735 + 0.5
}
*/

// reduce #![feature(clamp)]

#[doc(hidden)]
#[inline]
pub fn clamp_f32(mut x: f32, min: f32, max: f32) -> f32 {
    assert!(min <= max);
    if x < min {
        x = min;
    }
    if x > max {
        x = max;
    }
    x
}

#[doc(hidden)]
#[inline]
pub fn clamp_i32(mut x: i32, min: i32, max: i32) -> i32 {
    assert!(min <= max);
    if x < min {
        x = min;
    }
    if x > max {
        x = max;
    }
    x
}

#[derive(Clone, Copy, Default)]
pub struct Corners {
    pub tl: f32,
    pub tr: f32,
    pub br: f32,
    pub bl: f32,
}

impl Corners {
    pub fn all_same(radius: f32) -> Self {
        Self {
            tr: radius,
            tl: radius,
            br: radius,
            bl: radius,
        }
    }
}
