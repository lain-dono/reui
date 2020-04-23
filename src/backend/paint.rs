use crate::canvas::Gradient;
use crate::math::{Color, Rect, Transform};
use crate::state::Scissor;

pub const SHADER_SIMPLE: f32 = 0.0;
pub const SHADER_FILLGRAD: f32 = 1.0;

pub fn convert(paint: &crate::canvas::Paint, xform: Transform) -> Paint {
    match paint.gradient {
        Some(gradient) => {
            let mut paint = gradient_to_paint(gradient);
            paint.xform.prepend_mut(xform);
            paint
        }
        None => Paint {
            xform: Transform::identity(),
            extent: [0.0, 0.0],
            radius: 0.0,
            feather: 1.0,
            inner_color: paint.color,
            outer_color: paint.color,
        },
    }
}

fn gradient_to_paint(gradient: Gradient) -> Paint {
    match gradient {
        Gradient::Linear {
            from,
            to,
            inner_color,
            outer_color,
        } => Paint::linear_gradient(
            from[0],
            from[1],
            to[0],
            to[1],
            Color::new(inner_color),
            Color::new(outer_color),
        ),
        Gradient::Box {
            rect,
            radius,
            feather,
            inner_color,
            outer_color,
        } => Paint::box_gradient(
            rect,
            radius,
            feather,
            Color::new(inner_color),
            Color::new(outer_color),
        ),
        Gradient::Radial {
            center,
            inr,
            outr,
            inner_color,
            outer_color,
        } => Paint::radial_gradient(
            center[0],
            center[1],
            inr,
            outr,
            Color::new(inner_color),
            Color::new(outer_color),
        ),
    }
}

#[repr(C, align(4))]
pub struct FragUniforms {
    pub scissor_mat: [f32; 4],
    pub paint_mat: [f32; 4],
    pub inner_color: [f32; 4],
    pub outer_color: [f32; 4],

    pub scissor_ext: [f32; 2],
    pub scissor_scale: [f32; 2],

    pub extent: [f32; 2],
    pub radius: f32,
    pub feather: f32,

    pub stroke_mul: f32, // scale
    pub stroke_thr: f32, // threshold
    pub padding: [u8; 4],
    pub kind: f32,
}

impl FragUniforms {
    pub fn fill(paint: &Paint, scissor: Scissor, width: f32, fringe: f32, stroke_thr: f32) -> Self {
        let (scissor_mat, scissor_ext, scissor_scale);
        if scissor.extent[0] < -0.5 || scissor.extent[1] < -0.5 {
            scissor_mat = [0.0; 4];
            scissor_ext = [1.0, 1.0];
            scissor_scale = [1.0, 1.0];
        } else {
            let xform = scissor.xform;
            let (re, im) = (xform.re, xform.im);
            let scale = (re * re + im * im).sqrt() / fringe;

            scissor_mat = xform.inverse().into();
            scissor_ext = scissor.extent;
            scissor_scale = [scale, scale];
        }

        fn srgb_to_linear(c: Color) -> [f32; 4] {
            use palette::{LinSrgba, Pixel, Srgba};

            let srgb = Srgba::new(c.r, c.g, c.b, c.a);
            let lin: LinSrgba = srgb.into_encoding();

            let [r, g, b, a]: [f32; 4] = lin.into_raw();

            [r * a, g * a, b * a, a]
        }

        //let inner_color = paint.inner_color.premul();
        //let outer_color = paint.outer_color.premul();

        let inner_color = srgb_to_linear(paint.inner_color);
        let outer_color = srgb_to_linear(paint.outer_color);

        Self {
            scissor_mat,
            scissor_ext,
            scissor_scale,

            inner_color,
            outer_color,

            extent: paint.extent,

            stroke_mul: (width * 0.5 + fringe * 0.5) / fringe,
            stroke_thr,
            kind: SHADER_FILLGRAD,
            radius: paint.radius,
            feather: paint.feather,

            paint_mat: paint.xform.inverse().into(),

            padding: [0; 4],
        }
    }
}

impl Default for FragUniforms {
    fn default() -> Self {
        Self {
            scissor_mat: [0.0; 4],
            paint_mat: [0.0; 4],
            inner_color: [0.0; 4],
            outer_color: [0.0; 4],

            scissor_ext: [0.0; 2],
            scissor_scale: [0.0; 2],

            extent: [0.0; 2],
            radius: 0.0,
            feather: 0.0,

            stroke_mul: 0.0,
            stroke_thr: -1.0,
            padding: [0u8; 4],
            kind: SHADER_SIMPLE,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Paint {
    pub xform: Transform,
    pub extent: [f32; 2],
    pub radius: f32,
    pub feather: f32,
    pub inner_color: Color,
    pub outer_color: Color,
}

impl Paint {
    //
    // Paints
    //
    // NanoVG supports four types of paints: linear gradient, box gradient, radial gradient and image pattern.
    // These can be used as paints for strokes and fills.

    /// Creates and returns a linear gradient. Parameters (sx,sy)-(ex,ey) specify the start and end coordinates
    /// of the linear gradient, icol specifies the start color and ocol the end color.
    /// The gradient is transformed by the current transform when it is passed to FillPaint() or StrokePaint().
    pub fn linear_gradient(
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        inner_color: Color,
        outer_color: Color,
    ) -> Self {
        let large = 1e5;

        // Calculate transform aligned to the line
        let dx = ex - sx;
        let dy = ey - sy;
        let d = (dx * dx + dy * dy).sqrt();
        let (dx, dy) = if d > 0.0001 {
            (dx / d, dy / d)
        } else {
            (0.0, 1.0)
        };

        Self {
            xform: Transform {
                re: dy,
                im: -dx,
                tx: sx - dx * large,
                ty: sy - dy * large,
            },
            extent: [large, large + d * 0.5],
            radius: 0.0,
            feather: d.max(1.0),
            inner_color,
            outer_color,
        }
    }

    /// Creates and returns a radial gradient. Parameters (cx,cy) specify the center, inr and outr specify
    /// the inner and outer radius of the gradient, icol specifies the start color and ocol the end color.
    /// The gradient is transformed by the current transform when it is passed to FillPaint() or StrokePaint().
    pub fn radial_gradient(
        cx: f32,
        cy: f32,
        inr: f32,
        outr: f32,
        inner_color: Color,
        outer_color: Color,
    ) -> Self {
        let r = (inr + outr) * 0.5;
        let f = outr - inr;

        Self {
            xform: Transform::translation(cx, cy),
            extent: [r, r],
            radius: r,
            feather: f.max(1.0),
            inner_color,
            outer_color,
        }
    }

    /// Creates and returns a box gradient.
    /// Box gradient is a feathered rounded rectangle, it is useful for rendering
    /// drop shadows or highlights for boxes.
    /// Parameters (x,y) define the top-left corner of the rectangle,
    /// (w,h) define the size of the rectangle, r defines the corner radius, and f feather.
    /// Feather defines how blurry the border of the rectangle is.
    /// Parameter icol specifies the inner color and ocol the outer color of the gradient.
    /// The gradient is transformed by the current transform when it is passed to FillPaint() or StrokePaint().
    pub fn box_gradient(
        rect: Rect,
        radius: f32,
        feather: f32,
        inner_color: Color,
        outer_color: Color,
    ) -> Self {
        let center = rect.center();
        Self {
            xform: Transform::translation(center.x, center.y),
            extent: [rect.dx() * 0.5, rect.dy() * 0.5],
            radius,
            feather: feather.max(1.0),
            inner_color,
            outer_color,
        }
    }
}
