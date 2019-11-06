pub use euclid::{
    self,
    Angle,
    UnknownUnit,
    approxeq::ApproxEq,
};

pub use self::{
    color::Color,
    transform::Transform,
};

impl self::transform::Transform {
    pub fn transform_point(&self, p: Point) -> Point {
        self.apply(p.into()).into()
    }

    pub fn post_transform(&self, other: &Self) -> Self {
        *other * *self
    }

    pub fn pre_transform(&self, other: &Self) -> Self {
        *self * *other
    }
}

mod color;
pub mod offset;
pub mod size;
pub mod transform;

pub type Point = euclid::Point2D<f32, UnknownUnit>;
pub type Vector = euclid::Vector2D<f32, UnknownUnit>;
pub type Rect = euclid::Rect<f32, UnknownUnit>;
pub type SideOffsets = euclid::SideOffsets2D<f32, UnknownUnit>;

#[inline]
pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    euclid::rect(x, y, w, h)
}

#[inline]
pub fn vec2(x: f32, y: f32) -> Vector {
    euclid::vec2(x, y)
}

#[inline]
pub fn point2(x: f32, y: f32) -> Point {
    euclid::point2(x, y)
}

#[inline(always)]
pub fn transform_pt(pt: &mut [f32], t: &Transform) {
    let p = t.transform_point(point2(pt[0], pt[1]));
    pt[0] = p.x;
    pt[1] = p.y;
}

#[inline(always)]
pub fn transform_point(t: &Transform, x: f32, y: f32) -> [f32; 2] {
    t.transform_point(point2(x, y)).to_array()
}