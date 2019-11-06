mod color;
mod offset;
mod transform;
mod rect;

pub use self::{
    color::Color,
    transform::Transform,
    offset::Offset,
    rect::Rect,
};

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

fn convert_radius_to_sigma(radius: f32) -> f32 {
    radius * 0.57735 + 0.5
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