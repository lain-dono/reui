use crate::{backend::Image, math::{Transform, Rect, Color}};
use slotmap::Key;

#[derive(Clone, Copy)]
pub struct Paint {
    pub xform: Transform,
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
            xform: Transform::row_major(
                dy, -dx,
                dx, dy,
                sx - dx*large,
                sy - dy*large,
            ),
            /*
            xform: Transform::new(
                dy, -dx,
                sx - dx*large,
                sy - dy*large,
            ),
            */
            extent: [large, large + d * 0.5],
            radius: 0.0,
            feather: d.max(1.0),
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

        Self {
            xform: Transform::create_translation(cx, cy),
            extent: [r, r],
            radius: r,
            feather: f.max(1.0),
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
        rect: Rect,
        radius: f32, feather: f32,
        inner_color: Color,
        outer_color: Color,
    ) -> Self {
        Self {
            xform: Transform::create_translation(
                rect.origin.x + rect.size.width*0.5,
                rect.origin.y + rect.size.height*0.5,
            ),
            extent: [rect.size.width*0.5, rect.size.height*0.5],
            radius,
            feather: feather.max(1.0),
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
        w: f32, h: f32,
        image: Image, alpha: f32,
    ) -> Self {
        let white = Color::rgbaf(1.0, 1.0, 1.0, alpha);
        Self {
            xform: Transform::create_translation(cx, cy),
            extent: [w, h],
            radius: 0.0,
            feather: 0.0,
            image,
            inner_color: white,
            outer_color: white,
        }
    }

    pub fn color(color: Color) -> Self {
        Self {
            xform: Transform::identity(),
            extent: [0.0, 0.0],
            radius: 0.0,
            feather: 1.0,
            image: Image::null(),
            inner_color: color,
            outer_color: color,
        }
    }
}