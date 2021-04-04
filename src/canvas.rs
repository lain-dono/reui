use crate::{
    geom::{Corners, Offset, Rect, Transform},
    paint::{Paint, PaintingStyle},
    path::{Command, FillRule, Path},
    picture::Recorder,
    renderer::Renderer,
};

impl Transform {
    #[inline]
    pub(crate) fn average_scale(&self) -> f32 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
}

#[derive(Default)]
struct TransformStack(Transform, Vec<Transform>);

impl TransformStack {
    #[inline]
    fn save(&mut self) {
        self.1.push(self.transform())
    }

    #[inline]
    fn restore(&mut self) -> bool {
        self.1.pop().is_some()
    }

    #[inline]
    fn transform(&self) -> Transform {
        *self.1.last().unwrap_or(&self.0)
    }

    #[inline]
    fn pre_transform(&mut self, m: Transform) {
        *self.1.last_mut().unwrap_or(&mut self.0) *= m;
    }
}

pub struct Canvas<'a> {
    ctx: &'a mut Renderer,
    recorder: &'a mut Recorder,
    states: TransformStack,
    path: Path,
    scale: f32,
}

impl<'a> Canvas<'a> {
    pub fn new(ctx: &'a mut Renderer, recorder: &'a mut Recorder, scale: f32) -> Self {
        let transform = Transform::default();
        Self {
            ctx,
            recorder,
            states: TransformStack(transform, Vec::with_capacity(16)),
            path: Path::new(),
            scale,
        }
    }

    pub fn draw_image_rect(&mut self, image: u32, rect: Rect) {
        if self.ctx.images.get(image).is_some() {
            self.recorder
                .push_image(rect, self.states.transform(), image, [255; 4]);
        }
    }

    pub fn draw_image(&mut self, image: u32, offset: Offset) {
        if let Some(image_bind) = self.ctx.images.get(image) {
            let p0 = offset;
            let p1 = Offset::new(
                p0.x + image_bind.size.width as f32 / self.scale,
                p0.y + image_bind.size.height as f32 / self.scale,
            );
            self.recorder
                .push_image(Rect::new(p0, p1), self.states.transform(), image, [255; 4]);
        }
    }

    fn force_stroke(&mut self, paint: &Paint) {
        let xform = self.states.transform();
        let commands = self.path.into_iter();
        self.recorder
            .stroke_path(commands, paint, xform, self.scale);
    }

    fn fill_or_stroke(&mut self, paint: &Paint) {
        let xform = self.states.transform();
        let commands = self.path.into_iter();
        match paint.style {
            PaintingStyle::Stroke => self
                .recorder
                .stroke_path(commands, paint, xform, self.scale),
            PaintingStyle::FillNonZero => {
                self.recorder
                    .fill_path(commands, paint, xform, self.scale, FillRule::NonZero)
            }
            PaintingStyle::FillEvenOdd => {
                self.recorder
                    .fill_path(commands, paint, xform, self.scale, FillRule::EvenOdd)
            }
        }
    }

    /// Saves a copy of the current transform and clip on the save stack. [...]
    pub fn save(&mut self) {
        self.states.save();
    }

    /// Pops the current save stack, if there is anything to pop. Otherwise, does nothing. [...]
    pub fn restore(&mut self) {
        self.states.restore();
    }

    /// Add a rotation to the current transform. The argument is in radians clockwise.
    pub fn rotate(&mut self, radians: f32) {
        self.transform(Transform::rotation(radians));
    }

    /// Add an axis-aligned scale to the current transform,
    /// scaling by the first argument in the horizontal direction and the second in the vertical direction. [...]
    pub fn scale(&mut self, scale: f32) {
        self.transform(Transform::scale(scale));
    }

    /// Add a translation to the current transform,
    /// shifting the coordinate space horizontally by the first argument and vertically by the second argument.
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.transform(Transform::translation(dx, dy));
    }

    /// Multiply the current transform by the specified 4⨉4 transformation matrix specified as a list of values in column-major order.
    pub fn transform(&mut self, t: Transform) {
        self.states.pre_transform(t);
    }
}

impl<'a> Canvas<'a> {
    /*
    /// Reduces the clip region to the intersection of the current clip and the given Path. [...]
    clipPath(Path path, { bool doAntiAlias: true }) -> void
    /// Reduces the clip region to the intersection of the current clip and the given rectangle. [...]
    clipRect(Rect rect, { ClipOp clipOp: ClipOp.intersect, bool doAntiAlias: true }) -> void
    /// Reduces the clip region to the intersection of the current clip and the given rounded rectangle. [...]
    clipRRect(RRect rrect, { bool doAntiAlias: true }) -> void
    */

    /*
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
    pub fn draw_circle(&mut self, center: Offset, radius: f32, paint: Paint) {
        self.path.clear();
        self.path.circle(center, radius);
        self.fill_or_stroke(&paint);
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
    pub fn draw_line(&mut self, p0: Offset, p1: Offset, paint: Paint) {
        self.path.clear();
        self.path.move_to(p0);
        self.path.line_to(p1);
        self.force_stroke(&paint);
    }

    pub fn draw_lines(&mut self, points: &[Offset], paint: Paint) {
        if points.len() >= 2 {
            self.path.clear();
            self.path.move_to(points[0]);
            for &p in &points[1..] {
                self.path.line_to(p);
            }
            self.force_stroke(&paint);
        }
    }

    /// Draws an axis-aligned oval that fills the given axis-aligned rectangle with the given Paint.
    /// Whether the oval is filled or stroked (or both) is controlled by Paint.style.
    pub fn draw_oval(&mut self, rect: Rect, paint: Paint) {
        self.path.clear();
        self.path.oval(rect);
        self.fill_or_stroke(&paint);
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
    pub fn draw_path(&mut self, path_iter: impl IntoIterator<Item = Command>, paint: Paint) {
        self.path.clear();
        self.path.extend(path_iter);
        self.fill_or_stroke(&paint);
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
        self.path.clear();
        self.path.rect(rect);
        self.fill_or_stroke(&paint);
    }

    /// Draws a rounded rectangle with the given Paint.
    /// Whether the rectangle is filled or stroked (or both) is controlled by Paint.style.
    pub fn draw_rrect(&mut self, rect: Rect, radius: Corners, paint: Paint) {
        self.path.clear();
        if radius.tl <= 0.0 && radius.tr <= 0.0 && radius.br <= 0.0 && radius.bl <= 0.0 {
            self.path.rect(rect);
        } else {
            self.path.rrect(rect, radius);
        }
        self.fill_or_stroke(&paint);
    }

    /*
    /// Draws a shadow for a Path representing the given material elevation. [...]
    pub fn draw_shadow(&mut self, Path path, Color color, double elevation, bool transparentOccluder) -> void
    */
}
