use std::ops::{Add, Sub, Mul, Div};

/// `pt` is shorthand for Point {x, y}.
pub fn pt(x: f32, y: f32) -> Offset {
    Offset { x, y }
}

// Rect is shorthand for Rectangle{Pt(x0, y0), Pt(x1, y1)}. The returned
// rectangle has minimum and maximum coordinates swapped if necessary so that
// it is well-formed.
pub fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    let (x0, x1) = if x0 > x1 { (x1, x0) } else { (x0, x1) };
    let (y0, y1) = if y0 > y1 { (y1, y0) } else { (y0, y1) };
    Rect { min: pt(x0, y0), max: pt(x1, y1) }
}

/// A Point is an X, Y coordinate pair.
/// The axes increase right and down.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl Add<Self> for Offset {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Offset { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl Sub<Self> for Offset {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Offset { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl Mul<f32> for Offset {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Offset { x: self.x * rhs, y: self.y * rhs }
    }
}

impl Div<f32> for Offset {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Offset { x: self.x / rhs, y: self.y / rhs }
    }
}

impl Offset {
    pub fn new(x: f32, y: f32) -> Self {
        Offset { x, y }
    }
}

// A Rectangle contains the points with min.X <= X < max.X, min.Y <= Y < max.Y.
// It is well-formed if min.X <= max.X and likewise for Y. Points are always
// well-formed. A rectangle's methods always return well-formed outputs for
// well-formed inputs.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub min: Offset,
    pub max: Offset,
}

impl Add<Offset> for Rect {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Offset) -> Self {
        Rect {
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}

impl Sub<Offset> for Rect {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Offset) -> Self {
        Rect {
            min: self.min - rhs,
            max: self.max - rhs,
        }
    }
}

impl Rect {
    pub fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            min: pt(left, top),
            max: pt(right, bottom),
        }
    }

    pub fn from_center(center: Offset, width: f32, height: f32) -> Self {
        let pad = pt(width / 2.0, height / 2.0);
        Self {
            min: center - pad,
            max: center + pad,
        }
    }

    pub fn from_ltwh(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self::from_ltrb(left, top, left + width, top + height)
    }

    pub fn from_points(a: Offset, b: Offset) -> Self {
        Self {
            min: pt(a.x.min(b.x), a.y.min(b.y)),
            max: pt(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    pub fn contains(&self, p: Offset) -> bool {
        p.x >= self.min.x && p.x < self.max.x && p.y >= self.min.y && p.y < self.max.y
    }

    pub fn deflate(&self, delta: f32) -> Self {
        self.inflate(-delta)
    }

    pub fn inflate(&self, delta: f32) -> Self {
        let min = pt(self.min.x - delta, self.min.y - delta);
        let max = pt(self.max.x + delta, self.max.y + delta);
        Self { min, max }
    }

    pub fn shift(&self, p: Offset) -> Self {
        Self {
            min: self.min + p,
            max: self.max + p,
        }
    }

    /// Returns width.
    #[inline]
    pub fn dx(&self) -> f32 { self.max.x - self.min.x }

    /// Returns height.
    #[inline]
    pub fn dy(&self) -> f32 { self.max.y - self.min.y }

    /// Returns width and height.
    #[inline]
    pub fn size(&self) -> (f32, f32) { (self.dx(), self.dy()) }

    pub fn is_empty(&self) -> bool {
        self.min.x >= self.max.x || self.min.y >= self.max.y
    }

    pub fn intersect(r: Self, s: Self) -> Self {
        Self {
            min: Offset {
                x: r.min.x.max(s.min.x),
                y: r.min.y.max(s.min.y),
            },
            max: Offset {
                x: r.max.x.min(s.max.x),
                y: r.max.y.min(s.max.y),
            },
        }
    }

    pub fn union(r: Self, s: Self) -> Self {
        Self {
            min: Offset {
                x: r.min.x.min(s.min.x),
                y: r.min.y.min(s.min.y),
            },
            max: Offset {
                x: r.max.x.max(s.max.x),
                y: r.max.y.max(s.max.y),
            },
        }
    }

    pub fn overlaps(r: Self, s: Self) -> bool {
        r.min.x <= s.max.x && s.min.x <= r.max.x &&
        r.min.y <= s.max.y && s.min.y <= r.max.y
    }
}