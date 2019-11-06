use super::{Point, Size, Vector};

pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(Point::new(x, y), Size::new(w, h))
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    pub fn new(origin: Point, size: Size) -> Self {
        let min = origin;
        let max = origin + size;
        Self { min, max }
    }

    pub fn size(&self) -> Size {
        (self.max - self.min).to_size()
    }

    pub fn center(&self) -> Point {
        self.min + self.size() / 2.0
    }

    pub fn inner_rect(&self, offsets: euclid::SideOffsets2D<f32, euclid::UnknownUnit>) -> Self {
        Self {
            min: Point::new(
                self.min.x + offsets.left,
                self.min.y + offsets.top,
            ),
            max: Point::new(
                self.max.x - offsets.right,
                self.max.y - offsets.bottom,
            ),
        }
    }

    pub fn outer_rect(&self, offsets: euclid::SideOffsets2D<f32, euclid::UnknownUnit>) -> Self {
        Self {
            min: Point::new(
                self.min.x - offsets.left,
                self.min.y - offsets.top,
            ),
            max: Point::new(
                self.max.x + offsets.right,
                self.max.y + offsets.bottom,
            ),
        }
    }

    pub fn translate(&self, by: Vector) -> Self {
        Self {
            min: self.min + by,
            max: self.max + by,
        }
    }
}