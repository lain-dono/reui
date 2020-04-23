mod color;
mod offset;
mod rect;
mod rrect;
mod transform;

pub use self::{color::Color, offset::Offset, rect::Rect, rrect::RRect, transform::Transform};

impl self::transform::Transform {
    pub fn transform_point(&self, p: Offset) -> Offset {
        self.apply(p.into()).into()
    }
}

pub trait PartialClamp {
    fn clamp(self, min: Self, max: Self) -> Self;
}

impl<T: std::cmp::PartialOrd> PartialClamp for T {
    fn clamp(self, min: Self, max: Self) -> Self {
        assert!(min <= max);
        let mut x = self;
        if x < min {
            x = min;
        }
        if x > max {
            x = max;
        }
        x
    }
}

// reduce #![feature(clamp)]

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
