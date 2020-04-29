pub use crate::math::{Color, Rect};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StrokeCap {
    Butt = 0,
    Round = 1,
    Square = 2,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StrokeJoin {
    Round = 1,
    Bevel = 3,
    Miter = 4,
}

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
        inner_color: Color,
        outer_color: Color,
    },
    Box {
        rect: Rect,
        radius: f32,
        feather: f32,
        inner_color: Color,
        outer_color: Color,
    },
    Radial {
        center: [f32; 2],
        inr: f32,
        outr: f32,
        inner_color: Color,
        outer_color: Color,
    },
}

#[derive(Clone, Copy)]
pub struct Paint {
    pub style: PaintingStyle,
    pub color: Color,

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
            color: Color::hex(0xFF_000000),
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
    pub fn fill(color: Color) -> Self {
        Self {
            style: PaintingStyle::Fill,
            color,
            ..Self::default()
        }
    }

    pub fn stroke(color: Color) -> Self {
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

    pub fn linear_gradient(
        from: [f32; 2],
        to: [f32; 2],
        inner_color: Color,
        outer_color: Color,
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
        inner_color: Color,
        outer_color: Color,
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
        inner_color: Color,
        outer_color: Color,
    ) -> Self {
        Self::gradient(Gradient::Radial {
            center,
            inr,
            outr,
            inner_color,
            outer_color,
        })
    }

    fn gradient(gradient: Gradient) -> Self {
        Self {
            gradient: Some(gradient),
            ..Self::default()
        }
    }
}
