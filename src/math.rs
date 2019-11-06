mod color;
mod offset;
mod transform;

pub use self::{
    color::Color,
    transform::Transform,
    offset::Offset,
};

pub type Point = Offset;
//pub type Vector = Offset;

impl self::transform::Transform {
    pub fn transform_point(&self, p: Offset) -> Offset {
        self.apply(p.into()).into()
    }
}

#[inline]
pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(
        Offset::new(x, y),
        Offset::new(w, h),
    )
}

#[inline]
pub fn vec2(x: f32, y: f32) -> Offset {
    Offset { x, y }
}

#[inline]
pub fn point2(x: f32, y: f32) -> Offset {
    Offset { x, y }
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub min: Offset,
    pub max: Offset,
}

impl Rect {
    pub fn new(min: Offset, size: Offset) -> Self {
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

    pub fn from_size(w: f32, h: f32) -> Self {
        Self { min: point2(0.0, 0.0), max: point2(w, h) }
    }

    pub fn dx(&self) -> f32 { self.max.x - self.min.x }
    pub fn dy(&self) -> f32 { self.max.y - self.min.y }

    pub fn size(&self) -> Offset {
        Offset::new(self.dx(), self.dy())
    }

    pub fn center(&self) -> Offset {
        self.min + self.size() / 2.0
    }

    pub fn translate(&self, v: Offset) -> Self {
        Self {
            min: self.min + v,
            max: self.max + v,
        }
    }

    pub fn inflate(&self, delta: f32) -> Self {
        Self {
            min: point2(self.min.x - delta, self.min.y - delta),
            max: point2(self.max.x + delta, self.max.y + delta),
        }
    }

    pub fn deflate(&self, delta: f32) -> Self {
        Self {
            min: point2(self.min.x + delta, self.min.y + delta),
            max: point2(self.max.x - delta, self.max.y - delta),
        }
    }
}