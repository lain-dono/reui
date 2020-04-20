mod paint;
mod path;
mod picture;

pub use crate::canvas::{
    paint::{Color, Gradient, Paint, PaintingStyle, StrokeCap, StrokeJoin},
    path::Path,
    picture::PictureRecorder,
};
pub use crate::{
    backend::Paint as BackendPaint,
    context::Context,
    math::{clamp_f32, rect, Corners, Offset, Rect, Transform},
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Winding {
    CCW = 1, // Winding for solid shapes
    CW = 2,  // Winding for holes
}

pub type Radius = f32; // TODO: [f32; 2]

#[derive(Clone, Copy)]
pub struct RRect {
    pub rect: Rect,
    pub radius: Corners,
}

impl RRect {
    pub fn from_rect_and_radius(rect: Rect, radius: f32) -> Self {
        Self {
            rect,
            radius: Corners::all_same(radius),
        }
    }

    pub fn new(o: Offset, s: Offset, radius: f32) -> Self {
        Self::from_rect_and_radius(Rect::from_points(o, o + s), radius)
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn width(&self) -> f32 {
        self.rect.dx()
    }
    pub fn height(&self) -> f32 {
        self.rect.dy()
    }

    pub fn inflate(self, v: f32) -> Self {
        Self {
            rect: self.rect.inflate(v),
            radius: self.radius,
        }
    }

    pub fn deflate(self, v: f32) -> Self {
        Self {
            rect: self.rect.deflate(v),
            radius: self.radius,
        }
    }
}

fn gradient_to_paint(gradient: Gradient) -> BackendPaint {
    match gradient {
        Gradient::Linear {
            from,
            to,
            inner_color,
            outer_color,
        } => BackendPaint::linear_gradient(
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
        } => BackendPaint::box_gradient(
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
        } => BackendPaint::radial_gradient(
            center[0],
            center[1],
            inr,
            outr,
            Color::new(inner_color),
            Color::new(outer_color),
        ),
    }
}

fn convert_paint(paint: &Paint, xform: Transform) -> BackendPaint {
    match paint.gradient {
        Some(gradient) => {
            let mut paint = gradient_to_paint(gradient);
            paint.xform.prepend_mut(xform);
            paint
        }
        None => BackendPaint::color(Color::new(paint.color)),
    }
}

pub struct Canvas<'a> {
    ctx: &'a mut Context,
}

impl<'a> Canvas<'a> {
    pub fn new(ctx: &'a mut Context) -> Self {
        Self { ctx }
    }

    fn fill_or_stroke(&mut self, paint: &Paint, force_stroke: bool) {
        let cache = &mut self.ctx.cache;
        let (xform, scissor) = self.ctx.states.decompose();

        if force_stroke || paint.style == PaintingStyle::Stroke {
            let scale = xform.average_scale();
            let mut stroke_width = clamp_f32(paint.stroke_width * scale, 0.0, 200.0);
            let fringe = cache.fringe_width;

            let mut paint_stroke = convert_paint(paint, xform);
            if stroke_width < cache.fringe_width {
                // If the stroke width is less than pixel size, use alpha to emulate coverage.
                // Since coverage is area, scale by alpha*alpha.
                let alpha = clamp_f32(stroke_width / fringe, 0.0, 1.0);
                paint_stroke.inner_color.a *= alpha * alpha;
                paint_stroke.outer_color.a *= alpha * alpha;
                stroke_width = cache.fringe_width;
            }

            // Apply global alpha
            //let alpha = 1.0;
            //paint.inner_color.a *= alpha;
            //paint.outer_color.a *= alpha;

            cache.flatten_paths(&self.ctx.picture.commands);
            cache.expand_stroke(
                stroke_width * 0.5,
                fringe,
                paint.stroke_cap,
                paint.stroke_join,
                paint.stroke_miter_limit,
            );

            self.ctx
                .cmd
                .draw_stroke(paint_stroke, scissor, fringe, stroke_width, &cache.paths);
        } else {
            let w = if paint.is_antialias {
                cache.fringe_width
            } else {
                0.0
            };

            cache.flatten_paths(&self.ctx.picture.commands);
            cache.expand_fill(w, StrokeJoin::Miter, 2.4);

            let paint_fill = convert_paint(paint, xform);
            self.ctx.cmd.draw_fill(
                paint_fill,
                scissor,
                cache.fringe_width,
                cache.bounds,
                &cache.paths,
            );
        }
    }

    pub fn scissor(&mut self, rect: Rect) {
        self.ctx.states.set_scissor(rect);
    }

    pub fn intersect_scissor(&mut self, rect: Rect) {
        self.ctx.states.intersect_scissor(rect);
    }

    pub fn reset_scissor(&mut self) {
        self.ctx.states.reset_scissor();
    }

    /// Returns the number of items on the save stack, including the initial state.
    /// This means it returns 1 for a clean canvas, and that each call to save and saveLayer increments it,
    /// and that each matching call to restore decrements it. [...]
    pub fn save_count(&mut self) -> usize {
        self.ctx.states.save_count()
    }

    /// Saves a copy of the current transform and clip on the save stack. [...]
    pub fn save(&mut self) {
        self.ctx.states.save();
    }

    /// Saves a copy of the current transform and clip on the save stack,
    /// and then creates a new group which subsequent calls will become a part of.
    /// When the save stack is later popped, the group will be flattened into a layer
    /// and have the given paint's Paint.colorFilter and Paint.blendMode applied. [...]
    pub fn _save_layer(&mut self, _bounds: Rect, _paint: Paint) {
        unimplemented!()
    }

    /// Pops the current save stack, if there is anything to pop. Otherwise, does nothing. [...]
    pub fn restore(&mut self) {
        self.ctx.states.restore();
    }

    /// Add a rotation to the current transform. The argument is in radians clockwise.
    pub fn rotate(&mut self, radians: f32) {
        self.ctx.states.pre_transform(Transform::rotation(radians));
    }

    /// Add an axis-aligned scale to the current transform,
    /// scaling by the first argument in the horizontal direction and the second in the vertical direction. [...]
    pub fn scale(&mut self, scale: f32) {
        self.ctx.states.pre_transform(Transform::scale(scale));
    }

    /// Add an axis-aligned skew to the current transform,
    /// with the first argument being the horizontal skew in radians clockwise around the origin,
    /// and the second argument being the vertical skew in radians clockwise around the origin.
    pub fn _skew(&mut self, _sx: f32, _sy: f32) {
        unimplemented!()
    }

    /// Add a translation to the current transform,
    /// shifting the coordinate space horizontally by the first argument and vertically by the second argument.
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.ctx
            .states
            .pre_transform(Transform::translation(dx, dy));
    }

    /// Multiply the current transform by the specified 4⨉4 transformation matrix specified as a list of values in column-major order.
    pub fn _transform(&mut self, _t: Transform) {
        unimplemented!()
    }
}

impl<'a> Canvas<'a> {
    /*
    /*
    /// Reduces the clip region to the intersection of the current clip and the given Path. [...]
    clipPath(Path path, { bool doAntiAlias: true }) -> void
    /// Reduces the clip region to the intersection of the current clip and the given rectangle. [...]
    clipRect(Rect rect, { ClipOp clipOp: ClipOp.intersect, bool doAntiAlias: true }) -> void
    /// Reduces the clip region to the intersection of the current clip and the given rounded rectangle. [...]
    clipRRect(RRect rrect, { bool doAntiAlias: true }) -> void
    */

    /// Draw an arc scaled to fit inside the given rectangle.
    /// It starts from startAngle radians around the oval up to startAngle + sweepAngle radians around the oval,
    /// with zero radians being the point on the right hand side of the oval that crosses the horizontal line
    /// that intersects the center of the rectangle and with positive angles going clockwise around the oval.
    /// If useCenter is true, the arc is closed back to the center, forming a circle sector.
    /// Otherwise, the arc is not closed, forming a circle segment. [...]
    pub fn draw_arc(&mut self, rect: Rect, start_angle: f32, sweep_angle: f32, use_center: bool, paint: Paint) {
        unimplemented!()
    }
    */

    //drawAtlas(Image atlas, List<RSTransform> transforms, List<Rect> rects, List<Color> colors, BlendMode blendMode, Rect cullRect, Paint paint) -> void

    /// Draws a circle centered at the point given by the first argument
    /// and that has the radius given by the second argument, with the Paint given in the third argument.
    /// Whether the circle is filled or stroked (or both) is controlled by Paint.style.
    pub fn draw_circle(&mut self, c: Offset, radius: f32, paint: Paint) {
        self.ctx.cache.clear();
        self.ctx.picture.commands.clear();
        self.ctx.picture.xform = self.ctx.states.transform();
        self.ctx.picture.circle(c.x, c.y, radius);
        self.fill_or_stroke(&paint, false);
    }

    /*
    /// Paints the given Color onto the canvas, applying the given BlendMode,
    /// with the given color being the source and the background being the destination.
    pub fn draw_color(&mut self, color: Color, blend: BlendMode) -> void

    /// Draws a shape consisting of the difference between two rounded rectangles with the given Paint.
    /// Whether this shape is filled or stroked (or both) is controlled by Paint.style. [...]
    pub fn draw_drrect(&mut self, RRect outer, RRect inner, Paint paint) -> void

    /// Draws the given Image into the canvas with its top-left corner at the given Offset.
    /// The image is composited into the canvas using the given Paint.
    pub fn draw_image(&mut self, Image image, Offset p, Paint paint) -> void

    /// Draws the given Image into the canvas using the given Paint. [...]
    pub fn draw_image_nine(&mut self, Image image, Rect center, Rect dst, Paint paint) -> void
    /// Draws the subset of the given image described by the src argument into the canvas
    /// in the axis-aligned rectangle given by the dst argument. [...]
    pub fn draw_image_rect(&mut self, Image image, Rect src, Rect dst, Paint paint) -> void
    */

    /// Draws a line between the given points using the given paint.
    /// The line is stroked, the value of the Paint.style is ignored for this call. [...]
    pub fn draw_line(&mut self, p1: Offset, p2: Offset, paint: Paint) {
        self.ctx.cache.clear();
        self.ctx.picture.commands.clear();
        self.ctx.picture.xform = self.ctx.states.transform();
        self.ctx.picture.move_to(p1);
        self.ctx.picture.line_to(p2);
        self.fill_or_stroke(&paint, true);
    }

    pub fn draw_lines(&mut self, points: &[Offset], paint: Paint) {
        if points.len() < 2 {
            return;
        }
        self.ctx.cache.clear();
        self.ctx.picture.commands.clear();
        self.ctx.picture.xform = self.ctx.states.transform();
        self.ctx.picture.move_to(points[0]);
        for p in points.iter().skip(1) {
            self.ctx.picture.line_to(*p);
        }
        self.fill_or_stroke(&paint, true);
    }

    /// Draws an axis-aligned oval that fills the given axis-aligned rectangle with the given Paint.
    /// Whether the oval is filled or stroked (or both) is controlled by Paint.style.
    pub fn draw_oval(&mut self, rect: Rect, paint: Paint) {
        let (cx, cy) = (rect.min.x, rect.min.y);
        let (rx, ry) = (rect.dx(), rect.dy());
        self.ctx.cache.clear();
        self.ctx.picture.commands.clear();
        self.ctx.picture.xform = self.ctx.states.transform();
        self.ctx.picture.ellipse(cx, cy, rx, ry);
        self.fill_or_stroke(&paint, false);
    }

    /*
    /// Fills the canvas with the given Paint. [...]
    pub fn draw_paint(&mut self, Paint paint) -> void

    /// Draws the text in the given Paragraph into this canvas at the given Offset. [...]
    pub fn draw_paragraph(&mut self, Paragraph paragraph, Offset offset) -> void
    */

    /// Draws the given Path with the given Paint.
    /// Whether this shape is filled or stroked (or both) is controlled by Paint.style.
    /// If the path is filled, then sub-paths within it are implicitly closed (see Path.close).
    pub fn draw_path(&mut self, path: &mut [f32], paint: Paint) {
        self.ctx.picture.xform = self.ctx.states.transform();
        self.ctx.cache.clear();
        self.ctx.picture.commands.clear();
        self.ctx.picture.append_commands(path);
        self.fill_or_stroke(&paint, false);
    }

    pub fn draw_path_cloned(&mut self, path: &[f32], paint: Paint) {
        let mut path = path.to_owned();
        self.ctx.picture.xform = self.ctx.states.transform();
        self.ctx.cache.clear();
        self.ctx.picture.commands.clear();
        self.ctx.picture.append_commands(&mut path);
        self.fill_or_stroke(&paint, false);
    }

    /*
    /// Draw the given picture onto the canvas. To create a picture, see PictureRecorder.
    pub fn draw_picture(&mut self, Picture picture) -> void
    /// Draws a sequence of points according to the given PointMode. [...]
    pub fn draw_points(&mut self, PointMode pointMode, List<Offset> points, Paint paint) -> void

    pub fn draw_raw_atlas(&mut self, Image atlas, Float32List rstTransforms, Float32List rects, Int32List colors, BlendMode blendMode, Rect cullRect, Paint paint) -> void

    /// Draws a sequence of points according to the given PointMode. [...]
    pub fn draw_raw_points(&mut self, PointMode pointMode, Float32List points, Paint paint) -> void
    */

    /// Draws a rectangle with the given Paint.
    /// Whether the rectangle is filled or stroked (or both) is controlled by Paint.style.
    pub fn draw_rect(&mut self, rect: Rect, paint: Paint) {
        self.ctx.cache.clear();
        self.ctx.picture.commands.clear();
        self.ctx.picture.xform = self.ctx.states.transform();
        self.ctx.picture.rect(rect);
        self.fill_or_stroke(&paint, false);
    }

    /// Draws a rounded rectangle with the given Paint.
    /// Whether the rectangle is filled or stroked (or both) is controlled by Paint.style.
    pub fn draw_rrect(&mut self, rrect: RRect, paint: Paint) {
        self.ctx.cache.clear();
        self.ctx.picture.commands.clear();
        self.ctx.picture.xform = self.ctx.states.transform();
        self.ctx.picture.rrect_varying(
            rrect.rect(),
            rrect.radius.tl,
            rrect.radius.tr,
            rrect.radius.br,
            rrect.radius.bl,
        );
        self.fill_or_stroke(&paint, false);
    }

    /*
    /// Draws a shadow for a Path representing the given material elevation. [...]
    pub fn draw_shadow(&mut self, Path path, Color color, double elevation, bool transparentOccluder) -> void
    */
}
