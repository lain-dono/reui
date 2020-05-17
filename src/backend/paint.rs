use crate::{
    canvas::Gradient,
    math::{Color, Transform},
};

#[derive(Default)]
#[repr(C, align(4))]
pub struct Uniforms {
    pub paint_mat: [f32; 4],
    pub inner_color: [f32; 4],
    pub outer_color: [f32; 4],

    pub extent: [f32; 2],
    pub radius: f32,
    pub feather: f32,

    pub stroke_mul: f32, // scale
    pub stroke_thr: f32, // threshold
}

impl Uniforms {
    pub fn fill(paint: &Paint, width: f32, fringe: f32, stroke_thr: f32) -> Self {
        Self {
            paint_mat: paint.xform.inverse().into(),

            inner_color: paint.inner_color.into(),
            outer_color: paint.outer_color.into(),

            extent: paint.extent,
            radius: paint.radius,
            feather: paint.feather,

            stroke_mul: (width * 0.5 + fringe * 0.5) / fringe,
            stroke_thr,
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
