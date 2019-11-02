use std::f32::{INFINITY, NEG_INFINITY};

use super::offset::Offset;

#[derive(Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}

impl Size {
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const INFINITY: Self = Self::new(INFINITY, INFINITY);
    pub const NEG_INFINITY: Self = Self::new(NEG_INFINITY, NEG_INFINITY);

    #[inline]
    pub const fn new(w: f32, h: f32) -> Self {
        Self { w, h }
    }

    #[inline]
    pub const fn square(dimension: f32) -> Self {
        Self { w: dimension, h: dimension }
    }

    #[inline]
    pub const fn from_radius(radius: f32) -> Self {
        Self { w: radius * 2.0, h: radius * 2.0 }
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.w <= 0.0 || self.h <= 0.0
    }

    #[inline]
    pub fn aspect_ratio(self) -> f32 {
        let Self { w, h } = self;
        if h != 0.0 {
            w / h
        } else if w > 0.0 {
            INFINITY
        } else if w < 0.0 {
            NEG_INFINITY
        } else {
            0.0
        }
    }

    /*
    bottom_center
    center
    top_center
    center_left
    center_right
    */

    #[inline]
    pub fn center(self, origin: Offset) -> Offset {
        Offset::new(origin.x + self.w / 2.0, origin.y + self.h / 2.0)
    }

    #[inline]
    pub fn bl(self, origin: Offset) -> Offset {
        Offset::new(origin.x, origin.y + self.h)
    }

    #[inline]
    pub fn br(self, origin: Offset) -> Offset {
        Offset::new(origin.x + self.w, origin.y + self.h)
    }

    #[inline]
    pub fn tl(self, origin: Offset) -> Offset {
        origin
    }

    #[inline]
    pub fn tr(self, origin: Offset) -> Offset {
        Offset::new(origin.x + self.w, origin.y)
    }

    //contains
}