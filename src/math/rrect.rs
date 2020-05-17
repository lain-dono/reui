use super::{Corners, Offset, Rect};

#[derive(Clone, Copy)]
pub struct RRect {
    pub rect: Rect,
    pub radius: Corners,
}

impl RRect {
    pub fn from_rect_and_radius(rect: Rect, radius: f32) -> Self {
        let radius = Corners::all_same(radius);
        Self { rect, radius }
    }

    pub fn new(o: Offset, s: Offset, radius: f32) -> Self {
        Self::from_rect_and_radius(Rect::from_points(o, o + s), radius)
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn width(&self) -> f32 {
        self.rect.dx()
    }

    pub fn height(&self) -> f32 {
        self.rect.dy()
    }

    pub fn inflate(self, v: f32) -> Self {
        Self {
            rect: self.rect.inflate(v),
            radius: self.radius,
        }
    }

    pub fn deflate(self, v: f32) -> Self {
        Self {
            rect: self.rect.deflate(v),
            radius: self.radius,
        }
    }
}
