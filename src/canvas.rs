use crate::{
    math::{Corners, Offset, Rect, Transform},
    paint::{LineJoin, Paint, PaintingStyle, RawPaint, Stroke},
    path::{Path, PathCmd},
    picture::{Call, Instance, PictureRecorder, Vertex},
    renderer::Renderer,
};

impl Transform {
    #[inline(always)]
    fn average_scale(&self) -> f32 {
        self.re * self.re + self.im * self.im
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
    picture: &'a mut PictureRecorder,
    states: TransformStack,
    scale: f32,
}

impl<'a> Canvas<'a> {
    pub fn new(ctx: &'a mut Renderer, picture: &'a mut PictureRecorder, scale: f32) -> Self {
        let transform = Transform::default();
        Self {
            ctx,
            picture,
            states: TransformStack(transform, Vec::with_capacity(16)),
            scale,
        }
    }

    pub fn draw_image_rect(&mut self, image: u32, rect: Rect) {
        if self.ctx.images.get(&image).is_some() {
            let Offset { x: x0, y: y0 } = rect.min;
            let Offset { x: x1, y: y1 } = rect.max;

            let xform = self.states.transform();
            let vtx = [
                Vertex::new([x1, y1], [1.0, 1.0]).transform(&xform),
                Vertex::new([x1, y0], [1.0, 0.0]).transform(&xform),
                Vertex::new([x0, y0], [0.0, 0.0]).transform(&xform),
                Vertex::new([x0, y1], [0.0, 1.0]).transform(&xform),
            ];

            let pic = &mut self.picture;

            let idx = pic.instances.push(Instance::image([255; 4]));
            let vtx = pic
                .vertices
                .extend_with(&[vtx[0], vtx[1], vtx[2], vtx[0], vtx[2], vtx[3]]);
            pic.calls.push(Call::Image { idx, vtx, image });
        }
    }

    pub fn draw_image(&mut self, image: u32, offset: Offset) {
        // [012 023] [11 10 00 01]

        if let Some(image_bind) = self.ctx.images.get(&image) {
            let Offset { x: x0, y: y0 } = offset;
            let (x1, y1) = (
                x0 + image_bind.size.width as f32 / self.scale,
                y0 + image_bind.size.height as f32 / self.scale,
            );

            let xform = self.states.transform();
            let vtx = [
                Vertex::new([x1, y1], [1.0, 1.0]).transform(&xform),
                Vertex::new([x1, y0], [1.0, 0.0]).transform(&xform),
                Vertex::new([x0, y0], [0.0, 0.0]).transform(&xform),
                Vertex::new([x0, y1], [0.0, 1.0]).transform(&xform),
            ];

            let pic = &mut self.picture;

            let idx = pic.instances.push(Instance::image([255; 4]));
            let vtx = pic
                .vertices
                .extend_with(&[vtx[0], vtx[1], vtx[2], vtx[0], vtx[2], vtx[3]]);
            pic.calls.push(Call::Image { idx, vtx, image });
        }
    }

    pub fn stroke_path(&mut self, path: &Path, stroke: &Stroke) {
        let xform = self.states.transform();

        let tess = &mut self.ctx.tess;
        let mut width = (stroke.width * xform.average_scale()).clamp(0.0, 200.0);
        let mut color = stroke.color;
        if width < tess.fringe_width {
            // If the stroke width is less than pixel size, use alpha to emulate coverage.
            // Since coverage is area, scale by alpha*alpha.
            let alpha = (width / tess.fringe_width).clamp(0.0, 1.0);
            color.alpha *= alpha * alpha;
            width = tess.fringe_width;
        }

        tess.flatten_paths(path.transform(xform));
        tess.expand_stroke(
            width * 0.5,
            stroke.line_cap,
            stroke.line_join,
            stroke.miter_limit,
        );

        // Allocate vertices for all the paths.
        let pic = &mut self.picture;
        let verts = &mut pic.vertices;
        let iter = tess
            .paths
            .iter()
            .filter_map(|path| path.stroke.as_ref().map(|src| verts.extend_with(src)));

        let path = pic.strokes.extend(iter);

        // Fill shader
        let fringe = tess.fringe_width;
        let a = Instance::from_stroke(&stroke, width, fringe, 1.0 - 0.5 / 255.0);
        let b = Instance::from_stroke(&stroke, width, fringe, -1.0);

        let idx = pic.instances.push(a);
        let _ = pic.instances.push(b);

        pic.calls.push(Call::Stroke { idx, path })
    }

    fn fill_or_stroke(&mut self, paint: &Paint, force_stroke: bool) {
        let cache = &mut self.ctx.tess;
        let xform = self.states.transform();
        let commands = self.ctx.path.transform(xform);
        let pic = &mut self.picture;

        let mut raw_paint = RawPaint::convert(paint, xform);

        if force_stroke || paint.style == PaintingStyle::Stroke {
            let scale = xform.average_scale();
            let mut stroke_width = (paint.width * scale).clamp(0.0, 200.0);

            if stroke_width < cache.fringe_width {
                // If the stroke width is less than pixel size, use alpha to emulate coverage.
                // Since coverage is area, scale by alpha*alpha.
                let alpha = (stroke_width / cache.fringe_width).clamp(0.0, 1.0);
                let coverage = alpha * alpha;
                raw_paint.inner_color.alpha *= coverage;
                raw_paint.outer_color.alpha *= coverage;
                stroke_width = cache.fringe_width;
            }

            cache.flatten_paths(commands);
            cache.expand_stroke(stroke_width * 0.5, paint.cap, paint.join, paint.miter);

            // Allocate vertices for all the paths.
            let verts = &mut pic.vertices;
            let iter = cache
                .paths
                .iter()
                .filter_map(|path| path.stroke.as_ref().map(|src| verts.extend_with(src)));

            let path = pic.strokes.extend(iter);

            // Fill shader
            let fringe = cache.fringe_width;
            let a = raw_paint.to_instance(stroke_width, fringe, 1.0 - 0.5 / 255.0);
            let b = raw_paint.to_instance(stroke_width, fringe, -1.0);

            let idx = pic.instances.push(a);
            let _ = pic.instances.push(b);

            pic.calls.push(Call::Stroke { idx, path })
        } else {
            let w = if paint.antialias {
                cache.fringe_width
            } else {
                0.0
            };

            cache.flatten_paths(commands);
            cache.expand_fill(w, LineJoin::Miter, 2.4);
            let fringe = cache.fringe_width;
            let paths = &cache.paths;

            // Bounding box fill quad not needed for convex fill
            let kind = !(paths.len() == 1 && paths[0].convex);

            // Allocate vertices for all the paths.
            let (path, path_dst) = pic.paths.alloc_with(paths.len(), Default::default);
            for (src, dst) in paths.iter().zip(path_dst.iter_mut()) {
                if let Some(src) = &src.fill {
                    dst.fill = pic.vertices.extend_with(src);
                }
                if let Some(src) = &src.stroke {
                    dst.stroke = pic.vertices.extend_with(src);
                }
            }

            // Setup uniforms for draw calls
            let uniform = raw_paint.to_instance(fringe, fringe, -1.0);
            if kind {
                let quad = pic.vertices.extend_with(&[
                    Vertex::new([cache.bounds[2], cache.bounds[3]], [0.5, 1.0]),
                    Vertex::new([cache.bounds[2], cache.bounds[1]], [0.5, 1.0]),
                    Vertex::new([cache.bounds[0], cache.bounds[3]], [0.5, 1.0]),
                    Vertex::new([cache.bounds[0], cache.bounds[1]], [0.5, 1.0]),
                ]);

                let idx = pic.instances.push(Default::default());
                let _ = pic.instances.push(uniform);

                let quad = quad.start;
                pic.calls.push(Call::Fill { idx, path, quad })
            } else {
                let idx = pic.instances.push(uniform);
                pic.calls.push(Call::Convex { idx, path })
            };
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
        self.ctx.tess.clear();
        self.ctx.path.clear();
        self.ctx.path.circle(center, radius);
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
    pub fn draw_line(&mut self, p0: Offset, p1: Offset, paint: Paint) {
        self.ctx.tess.clear();
        self.ctx.path.clear();
        let xform = self.states.transform();
        let (p0, p1) = (xform.apply(p0), xform.apply(p1));
        self.ctx.path.move_to(p0);
        self.ctx.path.line_to(p1);
        self.fill_or_stroke(&paint, true);
    }

    pub fn draw_lines(&mut self, points: &[Offset], paint: Paint) {
        if points.len() < 2 {
            return;
        }
        self.ctx.tess.clear();

        self.ctx.path.clear();
        self.ctx.path.move_to(points[0]);
        for &p in &points[1..] {
            self.ctx.path.line_to(p);
        }
        self.fill_or_stroke(&paint, true);
    }

    /// Draws an axis-aligned oval that fills the given axis-aligned rectangle with the given Paint.
    /// Whether the oval is filled or stroked (or both) is controlled by Paint.style.
    pub fn draw_oval(&mut self, rect: Rect, paint: Paint) {
        self.ctx.tess.clear();
        self.ctx.path.clear();
        self.ctx.path.oval(rect);
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
        self.ctx.tess.clear();
        self.ctx.path.clear();
        self.ctx.path.extend(path.iter().copied());
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
        self.ctx.tess.clear();
        self.ctx.path.clear();
        self.ctx.path.rect(rect);
        self.fill_or_stroke(&paint, false);
    }

    /// Draws a rounded rectangle with the given Paint.
    /// Whether the rectangle is filled or stroked (or both) is controlled by Paint.style.
    pub fn draw_rrect(&mut self, rect: Rect, radius: Corners, paint: Paint) {
        self.ctx.tess.clear();
        self.ctx.path.clear();
        self.ctx.path.rrect(rect, radius);
        self.fill_or_stroke(&paint, false);
    }

    /*
    /// Draws a shadow for a Path representing the given material elevation. [...]
    pub fn draw_shadow(&mut self, Path path, Color color, double elevation, bool transparentOccluder) -> void
    */
}
