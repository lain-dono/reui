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
}

mod color;
pub mod offset;
pub mod size;
pub mod transform;

pub type Point = euclid::Point2D<f32, UnknownUnit>;
pub type Vector = euclid::Vector2D<f32, UnknownUnit>;
//pub type Rect = euclid::Rect<f32, UnknownUnit>;
pub type SideOffsets = euclid::SideOffsets2D<f32, UnknownUnit>;

#[inline]
pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(
        Point::new(x, y),
        euclid::Size2D::new(w, h),
    )
}

#[inline]
pub fn vec2(x: f32, y: f32) -> Vector {
    euclid::vec2(x, y)
}

#[inline]
pub fn point2(x: f32, y: f32) -> Point {
    euclid::point2(x, y)
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    pub fn new(min: Point, size: euclid::Size2D<f32, UnknownUnit>) -> Self {
        Self { min, max: min + size }
    }

    pub fn to_xywh(&self) -> [f32; 4] {
        [
            self.min.x,
            self.min.y,
            self.dx(),
            self.dy(),
        ]
    }

    pub fn from_size(size: euclid::Size2D<f32, UnknownUnit>) -> Self {
        Self { min: point2(0.0, 0.0), max: point2(0.0, 0.0) + size }
    }

    pub fn dx(&self) -> f32 { self.max.x - self.min.x }
    pub fn dy(&self) -> f32 { self.max.y - self.min.y }

    pub fn size(&self) -> euclid::Size2D<f32, UnknownUnit> {
        euclid::size2(self.dx(), self.dy())
    }

    pub fn center(&self) -> Point {
        self.min + self.size() / 2.0
    }

    pub fn translate(&self, v: Vector) -> Self {
        Self {
            min: self.min + v,
            max: self.max + v,
        }
    }

    pub fn inflate(&self, delta: f32) -> Self {
        Self {
            min: euclid::Point2D::new(self.min.x - delta, self.min.y - delta),
            max: euclid::Point2D::new(self.max.x + delta, self.max.y + delta),
        }
    }

    pub fn deflate(&self, delta: f32) -> Self {
        Self {
            min: euclid::Point2D::new(self.min.x + delta, self.min.y + delta),
            max: euclid::Point2D::new(self.max.x - delta, self.max.y - delta),
        }
    }
}