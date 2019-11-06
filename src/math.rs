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
    Rect {
        origin: Point::new(x, y),
        size: euclid::Size2D::new(w, h),
    }
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
    origin: Point,
    size: euclid::Size2D<f32, UnknownUnit>,
}

impl Rect {
    pub fn new(origin: Point, size: euclid::Size2D<f32, UnknownUnit>) -> Self {
        Self { origin, size }
    }

    pub fn to_xywh(&self) -> [f32; 4] {
        [
            self.origin.x,
            self.origin.y,
            self.size.width,
            self.size.height,
        ]
    }

    pub fn from_size(size: euclid::Size2D<f32, UnknownUnit>) -> Self {
        Self { origin: point2(0.0, 0.0), size }
    }

    pub fn dx(&self) -> f32 { self.size.width }
    pub fn dy(&self) -> f32 { self.size.height }

    pub fn min(&self) -> Point { self.origin }
    pub fn max(&self) -> Point { self.origin + self.size }

    pub fn size(&self) -> euclid::Size2D<f32, UnknownUnit> { self.size }

    pub fn min_x(&self) -> f32 { self.origin.x }
    pub fn min_y(&self) -> f32 { self.origin.y }

    pub fn max_x(&self) -> f32 { self.origin.x + self.size.width }
    pub fn max_y(&self) -> f32 { self.origin.y + self.size.height }

    pub fn center(&self) -> Point {
        self.origin + self.size / 2.0
    }

    pub fn translate(&self, v: Vector) -> Self {
        Self {
            origin: self.origin + v,
            size: self.size,
        }
    }

    pub fn inflate(&self, delta: f32) -> Self {
        Rect::new(
            euclid::Point2D::new(self.origin.x - delta, self.origin.y - delta),
            euclid::Size2D::new(
                self.size.width + delta + delta,
                self.size.height + delta + delta,
            ),
        )
    }

    pub fn deflate(&self, delta: f32) -> Self {
        Rect::new(
            euclid::Point2D::new(self.origin.x + delta, self.origin.y + delta),
            euclid::Size2D::new(
                self.size.width - delta - delta,
                self.size.height - delta - delta,
            ),
        )
    }
}