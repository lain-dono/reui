use super::{Color, utils::maxf};
use crate::backend::Image;
use crate::transform;
use slotmap::Key;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Paint {
    pub xform: [f32; 6],
    pub extent: [f32; 2],
    pub radius: f32,
    pub feather: f32,
    pub inner_color: Color,
    pub outer_color: Color,
    pub image: Image,
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
        sx: f32, sy: f32,
        ex: f32, ey: f32,
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
            xform: [
                dy, -dx,
                dx, dy,
                sx - dx*large, sy - dy*large,
            ],
            extent: [large, large + d * 0.5],
            radius: 0.0,
            feather: maxf(1.0, d),
            inner_color,
            outer_color,
            image: Image::null(),
        }
    }

    /// Creates and returns a radial gradient. Parameters (cx,cy) specify the center, inr and outr specify
    /// the inner and outer radius of the gradient, icol specifies the start color and ocol the end color.
    /// The gradient is transformed by the current transform when it is passed to FillPaint() or StrokePaint().
    pub fn radial_gradient(
        cx: f32, cy: f32,
        inr: f32, outr: f32,
        inner_color: Color,
        outer_color: Color,
    ) -> Self {
        let r = (inr+outr)*0.5;
        let f = outr-inr;

        let mut xform = transform::identity();
        xform[4] = cx;
        xform[5] = cy;

        Self {
            xform,
            extent: [r, r],
            radius: r,
            feather: maxf(1.0, f),
            inner_color,
            outer_color,
            image: Image::null(),
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
        x: f32, y: f32,
        w: f32, h: f32,
        r: f32, f: f32,
        inner_color: Color,
        outer_color: Color,
    ) -> Self {
        let mut xform = transform::identity();
        xform[4] = x+w*0.5;
        xform[5] = y+h*0.5;

        Self {
            xform,
            extent: [w*0.5, h*0.5],
            radius: r,
            feather: maxf(1.0, f),
            inner_color,
            outer_color,
            image: Image::null(),
        }
    }

    /// Creates and returns an image patter.
    /// Parameters (ox,oy) specify the left-top location of the image pattern,
    /// (ex,ey) the size of one image, angle rotation around the top-left corner,
    /// image is handle to the image to render.
    /// The gradient is transformed by the current transform when it is passed to FillPaint() or StrokePaint().
    pub fn image_pattern(
        cx: f32, cy: f32,
        w: f32, h: f32, angle: f32,
        image: Image, alpha: f32,
    ) -> Self {
        let mut xform = transform::rotate(angle);
        xform[4] = cx;
        xform[5] = cy;

        let white = Color::rgbaf(1.0, 1.0, 1.0, alpha);

        Self {
            xform,
            extent: [w, h],
            radius: 0.0,
            feather: 0.0,
            image,
            inner_color: white,
            outer_color: white,
        }
    }

    pub fn with_color(color: Color) -> Self {
        Self {
            xform: transform::identity(),
            extent: [0.0, 0.0],
            radius: 0.0,
            feather: 1.0,
            image: Image::null(),
            inner_color: color,
            outer_color: color,
        }
    }

    pub fn set_color(&mut self, color: Color) {
        *self = Self::with_color(color)
    }
}
