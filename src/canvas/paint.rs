pub use crate::cache::{LineCap as StrokeCap, LineJoin as StrokeJoin};
pub use crate::math::{Color, Rect};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PaintingStyle {
    Fill = 0,
    Stroke = 1,
}

#[derive(Clone, Copy)]
pub enum Gradient {
    Linear {
        from: [f32; 2],
        to: [f32; 2],
        inner_color: u32,
        outer_color: u32,
    },
    Box {
        rect: Rect,
        radius: f32,
        feather: f32,
        inner_color: u32,
        outer_color: u32,
    },
    Radial {
        center: [f32; 2],
        inr: f32,
        outr: f32,
        inner_color: u32,
        outer_color: u32,
    },
}

#[derive(Clone, Copy)]
pub struct Paint {
    pub style: PaintingStyle,
    pub color: u32,

    pub is_antialias: bool,

    pub stroke_cap: StrokeCap,
    pub stroke_join: StrokeJoin,
    pub stroke_miter_limit: f32,
    pub stroke_width: f32,

    pub gradient: Option<Gradient>,
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            style: PaintingStyle::Fill,
            color: 0xFF_000000,
            is_antialias: true,
            stroke_cap: StrokeCap::Butt,
            stroke_join: StrokeJoin::Miter,
            stroke_miter_limit: 10.0,
            stroke_width: 1.0,
            gradient: None,
        }
    }
}

impl Paint {
    pub fn fill(color: u32) -> Self {
        Self {
            style: PaintingStyle::Fill,
            color,
            ..Self::default()
        }
    }

    pub fn stroke(color: u32) -> Self {
        Self {
            style: PaintingStyle::Stroke,
            color,
            ..Self::default()
        }
    }

    pub fn stroke_cap(self, stroke_cap: StrokeCap) -> Self {
        Self { stroke_cap, ..self }
    }
    pub fn stroke_join(self, stroke_join: StrokeJoin) -> Self {
        Self {
            stroke_join,
            ..self
        }
    }
    pub fn stroke_miter_limit(self, stroke_miter_limit: f32) -> Self {
        Self {
            stroke_miter_limit,
            ..self
        }
    }
    pub fn stroke_width(self, stroke_width: f32) -> Self {
        Self {
            stroke_width,
            ..self
        }
    }

    pub fn with_gradient(self, gradient: Gradient) -> Self {
        Self {
            gradient: Some(gradient),
            ..self
        }
    }

    pub fn antialias(self, is_antialias: bool) -> Self {
        Self {
            is_antialias,
            ..self
        }
    }

    pub fn gradient(gradient: Gradient) -> Self {
        Self {
            gradient: Some(gradient),
            ..Self::default()
        }
    }

    pub fn linear_gradient(
        from: [f32; 2],
        to: [f32; 2],
        inner_color: u32,
        outer_color: u32,
    ) -> Self {
        Self::gradient(Gradient::Linear {
            from,
            to,
            inner_color,
            outer_color,
        })
    }
    pub fn box_gradient(
        rect: Rect,
        radius: f32,
        feather: f32,
        inner_color: u32,
        outer_color: u32,
    ) -> Self {
        Self::gradient(Gradient::Box {
            rect,
            radius,
            feather,
            inner_color,
            outer_color,
        })
    }
    pub fn radial_gradient(
        center: [f32; 2],
        inr: f32,
        outr: f32,
        inner_color: u32,
        outer_color: u32,
    ) -> Self {
        Self::gradient(Gradient::Radial {
            center,
            inr,
            outr,
            inner_color,
            outer_color,
        })
    }
}
