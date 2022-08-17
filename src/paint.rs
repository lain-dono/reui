use crate::{internals::Instance, Color, Rect, Transform};

pub trait IntoPaint {
    fn into_paint(self, transform: Transform) -> RawPaint;
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
    pub fn to_instance(self, width: f32, fringe: f32, stroke_thr: f32) -> Instance {
        Instance {
            paint_mat: self.xform.inverse().into(),

            inner_color: self.inner_color.into(),
            outer_color: self.outer_color.into(),

            extent: self.extent,
            radius: self.radius,
            inv_feather: self.feather.recip(),

            stroke_mul: (width + fringe) / fringe * 0.5,
            stroke_thr,
        }
    }
}

/// Styles to use for line endings.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

/// Styles to use for line segment joins.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum LineJoin {
    Round,
    Bevel,
    Miter,
}

#[derive(Clone, Copy)]
pub enum Gradient {
    Linear(LinearGradient),
    Box(BoxGradient),
    Radial(RadialGradient),
}

#[derive(Clone, Copy)]
pub struct Stroke {
    pub start: LineCap,
    pub end: LineCap,
    pub join: LineJoin,
    pub miter: f32,
    pub width: f32,
}

impl Stroke {
    pub fn width(width: f32) -> Self {
        Self {
            width,
            ..Default::default()
        }
    }
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            start: LineCap::Butt,
            end: LineCap::Butt,
            join: LineJoin::Miter,
            miter: 4.0,
            width: 1.0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Paint {
    pub stroke: Stroke,
    pub color: Color,
    pub gradient: Option<Gradient>,
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            stroke: Stroke::default(),
            gradient: None,
        }
    }
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Self {
            color,
            ..Self::default()
        }
    }
}

impl Paint {
    pub fn color(color: Color) -> Self {
        Self {
            color,
            ..Self::default()
        }
    }

    pub fn cap(self, cap: LineCap) -> Self {
        Self {
            stroke: Stroke {
                start: cap,
                end: cap,
                ..self.stroke
            },
            ..self
        }
    }

    pub fn joint(self, join: LineJoin) -> Self {
        Self {
            stroke: Stroke {
                join,
                ..self.stroke
            },
            ..self
        }
    }

    pub fn miter_limit(self, miter: f32) -> Self {
        Self {
            stroke: Stroke {
                miter,
                ..self.stroke
            },
            ..self
        }
    }

    pub fn stroke_width(self, width: f32) -> Self {
        Self {
            stroke: Stroke {
                width,
                ..self.stroke
            },
            ..self
        }
    }
}

impl IntoPaint for Paint {
    fn into_paint(self, transform: Transform) -> RawPaint {
        self.gradient.map_or_else(
            || self.color.into_paint(transform),
            |gradient| match gradient {
                Gradient::Linear(gradient) => gradient.into_paint(transform),
                Gradient::Box(gradient) => gradient.into_paint(transform),
                Gradient::Radial(gradient) => gradient.into_paint(transform),
            },
        )
    }
}

impl IntoPaint for Color {
    fn into_paint(self, transform: Transform) -> RawPaint {
        RawPaint {
            xform: transform,
            extent: [0.0, 0.0],
            radius: 0.0,
            feather: 1.0,
            inner_color: self,
            outer_color: self,
        }
    }
}

#[derive(Clone, Copy)]
pub struct LinearGradient {
    pub from: [f32; 2],
    pub to: [f32; 2],
    pub inner: Color,
    pub outer: Color,
}

impl LinearGradient {
    pub fn new(from: [f32; 2], to: [f32; 2], inner: Color, outer: Color) -> Self {
        Self {
            from,
            to,
            inner,
            outer,
        }
    }
}

impl IntoPaint for LinearGradient {
    fn into_paint(self, transform: Transform) -> RawPaint {
        let Self {
            from,
            to,
            inner,
            outer,
        } = self;
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

        RawPaint {
            xform: transform * Transform { re, im, tx, ty },
            extent: [large, large + d * 0.5],
            radius: 0.0,
            feather: d.max(1.0),
            inner_color: inner,
            outer_color: outer,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BoxGradient {
    pub rect: Rect,
    pub radius: f32,
    pub feather: f32,
    pub inner: Color,
    pub outer: Color,
}

impl BoxGradient {
    pub fn new(rect: Rect, radius: f32, feather: f32, inner: Color, outer: Color) -> Self {
        Self {
            rect,
            radius,
            feather,
            inner,
            outer,
        }
    }
}

impl IntoPaint for BoxGradient {
    fn into_paint(self, transform: Transform) -> RawPaint {
        let Self {
            rect,
            radius,
            feather,
            inner,
            outer,
        } = self;
        let center = rect.center();
        RawPaint {
            xform: transform * Transform::translation(center.x, center.y),
            extent: [rect.dx() * 0.5, rect.dy() * 0.5],
            radius,
            feather: feather.max(1.0),
            inner_color: inner,
            outer_color: outer,
        }
    }
}

#[derive(Clone, Copy)]
pub struct RadialGradient {
    pub center: [f32; 2],
    pub inr: f32,
    pub outr: f32,
    pub inner: Color,
    pub outer: Color,
}

impl RadialGradient {
    pub fn new(center: [f32; 2], inr: f32, outr: f32, inner: Color, outer: Color) -> Self {
        Self {
            center,
            inr,
            outr,
            inner,
            outer,
        }
    }
}

impl IntoPaint for RadialGradient {
    fn into_paint(self, transform: Transform) -> RawPaint {
        let Self {
            center,
            inr,
            outr,
            inner,
            outer,
        } = self;
        let radius = (inr + outr) * 0.5;
        RawPaint {
            xform: transform * Transform::translation(center[0], center[1]),
            extent: [radius, radius],
            radius,
            feather: (outr - inr).max(1.0),
            inner_color: inner,
            outer_color: outer,
        }
    }
}
