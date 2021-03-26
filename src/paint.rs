use crate::math::{Color, Rect, Transform};
use crate::picture::Instance;

#[derive(Clone, PartialEq)]
pub struct Stroke {
    pub color: Color,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f32,
    pub width: f32,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum LineCap {
    Butt = 0,
    Round = 1,
    Square = 2,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum LineJoin {
    Round = 1,
    Bevel = 3,
    Miter = 4,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
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
    pub cap: LineCap,
    pub join: LineJoin,
    pub antialias: bool,
    pub miter: f32,
    pub width: f32,
    pub color: Color,
    pub gradient: Option<Gradient>,
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            style: PaintingStyle::Fill,
            color: Color::BLACK,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
            antialias: true,
            miter: 10.0,
            width: 1.0,
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

    pub fn stroke_cap(self, cap: LineCap) -> Self {
        Self { cap, ..self }
    }

    pub fn stroke_join(self, join: LineJoin) -> Self {
        Self { join, ..self }
    }

    pub fn stroke_miter_limit(self, miter: f32) -> Self {
        Self { miter, ..self }
    }

    pub fn stroke_width(self, width: f32) -> Self {
        Self { width, ..self }
    }

    pub fn antialias(self, antialias: bool) -> Self {
        Self { antialias, ..self }
    }

    pub fn with_gradient(self, gradient: Gradient) -> Self {
        Self {
            gradient: Some(gradient),
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

#[derive(Clone, Copy)]
pub struct RawPaint {
    pub xform: Transform,
    pub extent: [f32; 2],
    pub radius: f32,
    pub feather: f32,
    pub inner_color: Color,
    pub outer_color: Color,
}

impl RawPaint {
    pub fn convert(paint: &Paint, xform: Transform) -> Self {
        paint.gradient.map_or_else(
            || Self {
                xform: Transform::identity(),
                extent: [0.0, 0.0],
                radius: 0.0,
                feather: 1.0,
                inner_color: paint.color,
                outer_color: paint.color,
            },
            |gradient| match gradient {
                Gradient::Linear {
                    from,
                    to,
                    inner_color,
                    outer_color,
                } => {
                    let [sx, sy] = from;
                    let [ex, ey] = to;

                    let large = 1e5;

                    // Calculate transform aligned to the line
                    let dx = ex - sx;
                    let dy = ey - sy;
                    let d = (dx * dx + dy * dy).sqrt();
                    let (im, re) = if d > 0.0001 {
                        (-dx / d, dy / d)
                    } else {
                        (0.0, 1.0)
                    };

                    let tx = sx + im * large;
                    let ty = sy - re * large;

                    Self {
                        xform: xform * Transform { re, im, tx, ty },
                        extent: [large, large + d * 0.5],
                        radius: 0.0,
                        feather: d.max(1.0),
                        inner_color,
                        outer_color,
                    }
                }
                Gradient::Box {
                    rect,
                    radius,
                    feather,
                    inner_color,
                    outer_color,
                } => {
                    let center = rect.center();
                    Self {
                        xform: xform * Transform::translation(center.x, center.y),
                        extent: [rect.dx() * 0.5, rect.dy() * 0.5],
                        radius,
                        feather: feather.max(1.0),
                        inner_color,
                        outer_color,
                    }
                }
                Gradient::Radial {
                    center,
                    inr,
                    outr,
                    inner_color,
                    outer_color,
                } => {
                    let radius = (inr + outr) * 0.5;
                    let feather = (outr - inr).max(1.0);
                    Self {
                        xform: xform * Transform::translation(center[0], center[1]),
                        extent: [radius, radius],
                        radius,
                        feather,
                        inner_color,
                        outer_color,
                    }
                }
            },
        )
    }

    pub fn to_instance(&self, width: f32, fringe: f32, stroke_thr: f32) -> Instance {
        Instance {
            paint_mat: self.xform.inverse().into(),

            inner_color: self.inner_color.into(),
            outer_color: self.outer_color.into(),

            extent: self.extent,
            radius: self.radius,
            feather: self.feather,

            stroke_mul: (width * 0.5 + fringe * 0.5) / fringe,
            stroke_thr,
        }
    }
}
