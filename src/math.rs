pub use euclid::{
    self,
    Angle,
    UnknownUnit,
    approxeq::ApproxEq,
};

pub use self::{
    color::Color,
    //transform::Transform,
};

impl self::transform::Transform {
    pub fn transform_point(&self, p: Point) -> Point {
        self.apply(p.into()).into()
    }

    pub fn create_translation(x: f32, y: f32) -> Self {
        Self::translation(x, y)
    }

    pub fn create_rotation(theta: euclid::Angle<f32>) -> Self {
        Self::rotation(theta.get())
    }

    pub fn create_scale(x: f32, y: f32) -> Self {
        assert_eq!(x, y);
        Self::scale(x)
    }

    pub fn post_transform(&self, other: &Self) -> Self {
        *self * *other
    }

    pub fn pre_transform(&self, other: &Self) -> Self {
        *other * *self
    }
}

mod color;
pub mod offset;
pub mod size;
pub mod transform;

pub type Transform = euclid::Transform2D<f32, UnknownUnit, UnknownUnit>;

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