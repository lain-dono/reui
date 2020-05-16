use crate::math::{Color, Transform};
use crate::{canvas::Gradient, state::Scissor};

pub const SHADER_SIMPLE: f32 = 0.0;
pub const SHADER_FILLGRAD: f32 = 1.0;

#[repr(C, align(4))]
pub struct FragUniforms {
    pub scissor_mat: [f32; 4],
    pub scissor_ext: [f32; 2],
    pub scissor_scale: [f32; 2],

    pub paint_mat: [f32; 4],
    pub inner_color: [f32; 4],
    pub outer_color: [f32; 4],

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

        Self {
            scissor_mat,
            scissor_ext,
            scissor_scale,

            inner_color: paint.inner_color.into(),
            outer_color: paint.outer_color.into(),

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
    pub fn convert(paint: &crate::canvas::Paint, xform: Transform) -> Self {
        if let Some(gradient) = paint.gradient {
            match gradient {
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
                        }
                        .prepend(xform),
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
                        xform: Transform::translation(center.x, center.y).prepend(xform),
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
                        xform: Transform::translation(center[0], center[1]).prepend(xform),
                        extent: [radius, radius],
                        radius,
                        feather,
                        inner_color,
                        outer_color,
                    }
                }
            }
        } else {
            Self {
                xform: Transform::identity(),
                extent: [0.0, 0.0],
                radius: 0.0,
                feather: 1.0,
                inner_color: paint.color,
                outer_color: paint.color,
            }
        }
    }
}
