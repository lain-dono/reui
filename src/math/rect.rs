use super::Offset;

#[derive(Clone, Copy, Default)]
pub struct Rect {
    pub min: Offset,
    pub max: Offset,
}

impl Rect {
    #[inline]
    pub fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            min: Offset::new(left, top),
            max: Offset::new(right, bottom),
        }
    }

    #[inline]
    pub fn from_center(center: Offset, width: f32, height: f32) -> Self {
        let pad = Offset::new(width / 2.0, height / 2.0);
        Self {
            min: center - pad,
            max: center + pad,
        }
    }

    #[inline]
    pub fn from_ltwh(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self::from_ltrb(left, top, left + width, top + height)
    }

    #[inline]
    pub fn from_points(a: Offset, b: Offset) -> Self {
        Self {
            min: Offset::new(a.x.min(b.x), a.y.min(b.y)),
            max: Offset::new(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    pub fn to_xywh(&self) -> [f32; 4] {
        [
            self.min.x,
            self.min.y,
            self.dx(),
            self.dy(),
        ]
    }

    #[inline]
    pub fn from_size(w: f32, h: f32) -> Self {
        Self { min: Offset::new(0.0, 0.0), max: Offset::new(w, h) }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min.x >= self.max.x || self.min.y >= self.max.y
    }

    pub fn contains(&self, p: Offset) -> bool {
        p.x >= self.min.x && p.x < self.max.x && p.y >= self.min.y && p.y < self.max.y
    }

    #[inline]
    pub fn dx(&self) -> f32 { self.max.x - self.min.x }

    #[inline]
    pub fn dy(&self) -> f32 { self.max.y - self.min.y }

    #[inline]
    pub fn size(&self) -> Offset {
        Offset::new(self.dx(), self.dy())
    }

    #[inline]
    pub fn center(&self) -> Offset {
        self.min + self.size() / 2.0
    }

    #[inline]
    pub fn translate(&self, v: Offset) -> Self {
        Self {
            min: self.min + v,
            max: self.max + v,
        }
    }

    #[inline]
    pub fn inflate(&self, delta: f32) -> Self {
        Self {
            min: Offset::new(self.min.x - delta, self.min.y - delta),
            max: Offset::new(self.max.x + delta, self.max.y + delta),
        }
    }

    #[inline]
    pub fn deflate(&self, delta: f32) -> Self {
        Self {
            min: Offset::new(self.min.x + delta, self.min.y + delta),
            max: Offset::new(self.max.x - delta, self.max.y - delta),
        }
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
