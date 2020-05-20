use crate::{
    math::{Offset, PartialClamp, RRect, Rect, Transform},
    paint::{Paint, PaintingStyle, RawPaint, StrokeJoin, Uniforms},
    path::PathCmd,
    picture::{Call, Renderer, Vertex},
};

#[derive(Default)]
struct TransformStack(Transform, Vec<Transform>);

impl TransformStack {
    #[inline]
    fn save_count(&mut self) -> usize {
        self.1.len()
    }

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
    states: TransformStack,
}

impl<'a> Canvas<'a> {
    pub fn new(ctx: &'a mut Renderer) -> Self {
        let states = TransformStack(Transform::default(), Vec::with_capacity(16));
        Self { ctx, states }
    }

    fn fill_or_stroke(&mut self, paint: &Paint, force_stroke: bool) {
        impl Transform {
            #[inline(always)]
            fn average_scale(&self) -> f32 {
                self.re * self.re + self.im * self.im
            }
        }

        let cache = &mut self.ctx.cache;
        let xform = self.states.transform();
        let commands = self.ctx.recorder.transform(xform);
        let pic = &mut self.ctx.picture;

        let mut raw_paint = RawPaint::convert(paint, xform);

        if force_stroke || paint.style == PaintingStyle::Stroke {
            let scale = xform.average_scale();
            let mut stroke_width = (paint.width * scale).clamp(0.0, 200.0);
            let fringe = cache.fringe_width;

            if stroke_width < fringe {
                // If the stroke width is less than pixel size, use alpha to emulate coverage.
                // Since coverage is area, scale by alpha*alpha.
                let alpha = (stroke_width / fringe).clamp(0.0, 1.0);
                raw_paint.inner_color.alpha *= alpha * alpha;
                raw_paint.outer_color.alpha *= alpha * alpha;
                stroke_width = cache.fringe_width;
            }

            cache.flatten_paths(commands);
            cache.expand_stroke(
                stroke_width * 0.5,
                fringe,
                paint.cap,
                paint.join,
                paint.miter,
            );

            // Allocate vertices for all the paths.
            let verts = &mut pic.verts;
            let iter = cache
                .paths
                .iter()
                .filter_map(|path| path.stroke.as_ref().map(|src| verts.extend_with(src)));

            let path = pic.strokes.extend(iter);

            // Fill shader
            let a = Uniforms::fill(&raw_paint, stroke_width, fringe, 1.0 - 0.5 / 255.0);
            let b = Uniforms::fill(&raw_paint, stroke_width, fringe, -1.0);

            let idx = pic.uniforms.push(a);
            let _ = pic.uniforms.push(b);

            pic.calls.push(Call::Stroke { idx, path })
        } else {
            cache.flatten_paths(commands);
            cache.expand_fill(cache.fringe_width, StrokeJoin::Miter, 2.4);
            let fringe = cache.fringe_width;
            let paths = &cache.paths;

            // Bounding box fill quad not needed for convex fill
            let kind = !(paths.len() == 1 && paths[0].convex);

            // Allocate vertices for all the paths.
            let (path, path_dst) = pic.paths.alloc(paths.len());
            for (src, dst) in paths.iter().zip(path_dst.iter_mut()) {
                if let Some(src) = &src.fill {
                    dst.fill = pic.verts.extend_with(src);
                }
                if let Some(src) = &src.stroke {
                    dst.stroke = pic.verts.extend_with(src);
                }
            }

            // Setup uniforms for draw calls
            if kind {
                let quad = pic.verts.extend_with(&[
                    Vertex::new([cache.bounds[2], cache.bounds[3]], [0.5, 1.0]),
                    Vertex::new([cache.bounds[2], cache.bounds[1]], [0.5, 1.0]),
                    Vertex::new([cache.bounds[0], cache.bounds[3]], [0.5, 1.0]),
                    Vertex::new([cache.bounds[0], cache.bounds[1]], [0.5, 1.0]),
                ]);

                let uniform = Uniforms::fill(&raw_paint, fringe, fringe, -1.0);

                let idx = pic.uniforms.push(Default::default());
                let _ = pic.uniforms.push(uniform);

                let quad = quad.start;
                pic.calls.push(Call::Fill { idx, path, quad })
            } else {
                let uniform = Uniforms::fill(&raw_paint, fringe, fringe, -1.0);
                let idx = pic.uniforms.push(uniform);
                pic.calls.push(Call::Convex { idx, path })
            };
        }
    }

    /// Returns the number of items on the save stack, including the initial state.
    /// This means it returns 1 for a clean canvas, and that each call to save and saveLayer increments it,
    /// and that each matching call to restore decrements it. [...]
    pub fn save_count(&mut self) -> usize {
        self.states.save_count()
    }

    /// Saves a copy of the current transform and clip on the save stack. [...]
    pub fn save(&mut self) {
        self.states.save();
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
    pub fn draw_circle(&mut self, c: Offset, radius: f32, paint: Paint) {
        self.ctx.cache.clear();
        self.ctx.recorder.clear();
        self.ctx.recorder.add_circle(c, radius);
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
        self.ctx.recorder.clear();
        let xform = self.states.transform();
        self.ctx.recorder.move_to(xform.apply(p1));
        self.ctx.recorder.line_to(xform.apply(p2));
        self.fill_or_stroke(&paint, true);
    }

    pub fn draw_lines(&mut self, points: &[Offset], paint: Paint) {
        if points.len() < 2 {
            return;
        }
        self.ctx.cache.clear();

        self.ctx.recorder.clear();
        self.ctx.recorder.move_to(points[0]);
        for &p in &points[1..] {
            self.ctx.recorder.line_to(p);
        }
        self.fill_or_stroke(&paint, true);
    }

    /// Draws an axis-aligned oval that fills the given axis-aligned rectangle with the given Paint.
    /// Whether the oval is filled or stroked (or both) is controlled by Paint.style.
    pub fn draw_oval(&mut self, rect: Rect, paint: Paint) {
        self.ctx.cache.clear();
        self.ctx.recorder.clear();
        self.ctx.recorder.add_oval(rect);
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
    pub fn draw_path(&mut self, path: impl AsRef<[PathCmd]>, paint: Paint) {
        let path = path.as_ref();
        self.ctx.cache.clear();
        self.ctx.recorder.clear();
        self.ctx.recorder.extend(path.iter().copied());
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
        self.ctx.recorder.clear();
        self.ctx.recorder.add_rect(rect);
        self.fill_or_stroke(&paint, false);
    }

    /// Draws a rounded rectangle with the given Paint.
    /// Whether the rectangle is filled or stroked (or both) is controlled by Paint.style.
    pub fn draw_rrect(&mut self, rrect: RRect, paint: Paint) {
        self.ctx.cache.clear();
        self.ctx.recorder.clear();
        self.ctx.recorder.add_rrect(rrect);
        self.fill_or_stroke(&paint, false);
    }

    /*
    /// Draws a shadow for a Path representing the given material elevation. [...]
    pub fn draw_shadow(&mut self, Path path, Color color, double elevation, bool transparentOccluder) -> void
    */
}
