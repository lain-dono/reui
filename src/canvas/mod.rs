mod path;
mod paint;
mod picture;

mod geom;

pub use slotmap::Key;

pub use crate::{
    Context,
    backend::Image,
    font::Align,
    Winding,
    math::{
        Vector,
        Point,
        Transform,
        Rect,
        rect,
    },
};

pub use self::{
    path::Path,
    picture::Picture,
    paint::{
        Paint,
        PaintingStyle,
        StrokeCap,
        StrokeJoin,
        Gradient,
        Color,
    },
};

#[derive(Clone, Copy)]
pub struct TextStyle<'a> {
    pub font_size: f32,
    pub font_face: &'a [u8],
    pub font_blur: f32,
    pub color: u32,
    pub text_align: Align,
}

impl<'a> TextStyle<'a> {
    pub fn text_align(self, align: Align) -> Self {
        Self {
            text_align: align,
            .. self
        }
    }
}

pub type Offset = [f32; 2];
pub type Size = [f32; 2];
pub type Radius = f32; // TODO: [f32; 2]

#[derive(Clone, Copy, Default)]
pub struct CornerRadius {
    pub tl: f32,
    pub tr: f32,
    pub br: f32,
    pub bl: f32,
}

impl CornerRadius {
    pub fn all_same(radius: f32) -> Self {
        Self {
            tr: radius,
            tl: radius,
            br: radius,
            bl: radius,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct RRect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,

    pub radius: CornerRadius,
}

impl RRect {
    pub fn new(o: Offset, s: Size, radius: f32) -> Self {
        Self {
            left: o[0],
            top: o[1],
            right: o[0] + s[0],
            bottom: o[1] + s[1],
            radius: CornerRadius::all_same(radius),
        }
    }

    pub fn rect(&self) -> Rect {
        let origin = [self.left, self.top].into();
        let size = [self.width(), self.height()].into();
        Rect::new(origin, size)
    }

    pub fn width(&self) -> f32 { self.right - self.left }
    pub fn height(&self) -> f32 { self.bottom - self.top }

    pub fn add(self, v: f32) -> Self {
        Self {
            left: self.left - v,
            top: self.left - v,
            bottom: self.left + v,
            right: self.left + v,
            .. self
        }
    }
}

pub struct Canvas<'a> {
    ctx: &'a mut Context,
    save_count: usize,
}

impl<'a> Drop for Canvas<'a> {
    fn drop(&mut self) {
        self.ctx.restore();
    }
}

fn gradient_to_paint(gradient: Gradient) -> crate::vg::Paint {
    match gradient {
        Gradient::Linear { from, to, inner_color, outer_color } =>
            crate::vg::Paint::linear_gradient(
                from[0], from[1],
                to[0], to[1],
                Color::new(inner_color),
                Color::new(outer_color),
            ),
        Gradient::Box { rect, radius, feather, inner_color, outer_color } =>
            crate::vg::Paint::box_gradient(
                rect,
                radius,
                feather,
                Color::new(inner_color),
                Color::new(outer_color),
            ),
        Gradient::Radial { center, inr, outr, inner_color, outer_color } =>
            crate::vg::Paint::radial_gradient(
                center[0], center[1],
                inr, outr,
                Color::new(inner_color),
                Color::new(outer_color),
            ),
        Gradient::ImagePattern { center, size, image, alpha } =>
            crate::vg::Paint::image_pattern(
                center[0], center[1],
                size[0], size[1],
                image, alpha,
            ),
    }
}

impl<'a> Canvas<'a> {
    pub fn new(ctx: &'a mut Context) -> Self {
        ctx.save();

        Self {
            ctx,
            save_count: 1,
        }
    }

    fn sync_fill(&mut self, paint: &Paint) {
        match paint.gradient {
            Some(gradient) => self.ctx.fill_paint(gradient_to_paint(gradient)),
            None => self.ctx.fill_color(paint.color),
        }
    }

    fn sync_stroke(&mut self, paint: &Paint) {
        match paint.gradient {
            Some(gradient) => self.ctx.stroke_paint(gradient_to_paint(gradient)),
            None => self.ctx.stroke_color(paint.color),
        }
        self.ctx.line_cap(paint.stroke_cap);
        self.ctx.line_join(paint.stroke_join);
        self.ctx.miter_limit(paint.stroke_miter_limit);
        self.ctx.stroke_width(paint.stroke_width);
    }

    fn fill_or_stroke(&mut self, paint: &Paint) {
        self.ctx.shape_anti_alias(paint.aa);
        match paint.style {
            PaintingStyle::Fill => {
                self.sync_fill(paint);
                self.ctx.fill();
            }
            PaintingStyle::Stroke => {
                self.sync_stroke(paint);
                self.ctx.stroke();
            }
        }
    }

    pub fn text_bounds(&mut self, text: &str, font_size: f32, font_face: &[u8]) -> (f32, [f32; 4]) {
        self.ctx.font_size(font_size);
        self.ctx.font_face(font_face);
        self.ctx.text_bounds(0.0, 0.0, text)
    }

    pub fn text(&mut self, o: Offset, text: &str, style: TextStyle) {
        self.ctx.font_size(style.font_size);
        self.ctx.font_face(style.font_face);
        self.ctx.font_blur(style.font_blur);
        self.ctx.text_align(style.text_align);
        self.ctx.fill_color(style.color);
        self.ctx.text(o[0], o[1], text);
    }

    pub fn scissor(&mut self, r: Rect) {
        self.ctx.scissor(r)
    }

    pub fn intersect_scissor(&mut self, r: Rect) {
        self.ctx.intersect_scissor(r)
    }

    pub fn reset_scissor(&mut self) {
        self.ctx.reset_scissor();
    }

    pub fn image_size(&mut self, image: Image) -> Option<(u32, u32)> {
        self.ctx.image_size(image)
    }

    /// Returns the number of items on the save stack, including the initial state.
    /// This means it returns 1 for a clean canvas, and that each call to save and saveLayer increments it,
    /// and that each matching call to restore decrements it. [...] 
    pub fn save_count(&mut self) -> usize { self.save_count }

    /// Saves a copy of the current transform and clip on the save stack. [...] 
    pub fn save(&mut self) {
        self.save_count += 1;
        self.ctx.save();
    }

    /// Saves a copy of the current transform and clip on the save stack,
    /// and then creates a new group which subsequent calls will become a part of.
    /// When the save stack is later popped, the group will be flattened into a layer
    /// and have the given paint's Paint.colorFilter and Paint.blendMode applied. [...] 
    pub fn save_layer(&mut self, _bounds: Rect, _paint: Paint) { unimplemented!() }

    /// Pops the current save stack, if there is anything to pop. Otherwise, does nothing. [...] 
    pub fn restore(&mut self) {
        if self.save_count > 1 {
            self.save_count -= 1;
            self.ctx.restore();
        }
    }

    /// Add a rotation to the current transform. The argument is in radians clockwise. 
    pub fn rotate(&mut self, radians: f32) {
        self.ctx.rotate(radians)
    }

    /// Add an axis-aligned scale to the current transform,
    /// scaling by the first argument in the horizontal direction and the second in the vertical direction. [...] 
    pub fn scale(&mut self, scale: f32) {
        self.ctx.scale(scale)
    }

    /// Add an axis-aligned skew to the current transform,
    /// with the first argument being the horizontal skew in radians clockwise around the origin,
    /// and the second argument being the vertical skew in radians clockwise around the origin. 
    pub fn skew(&mut self, _sx: f32, _sy: f32) { unimplemented!() }

    /// Add a translation to the current transform,
    /// shifting the coordinate space horizontally by the first argument and vertically by the second argument. 
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.ctx.translate(dx, dy)
    }

    /// Multiply the current transform by the specified 4⨉4 transformation matrix specified as a list of values in column-major order. 
    pub fn transform(&mut self, t: Transform) { unimplemented!() }
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
        self.ctx.begin_path();
        self.ctx.circle(c[0], c[1], radius);
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
    pub fn draw_line(&mut self, p1: Offset, p2: Offset, paint: Paint) {
        self.sync_stroke(&paint);

        self.ctx.begin_path();
        self.ctx.move_to(p1[0], p1[1]);
        self.ctx.line_to(p2[0], p2[1]);
        self.ctx.stroke();
    }

    pub fn draw_lines(&mut self, points: &[Offset], paint: Paint) {
        if points.len() < 2 { return }

        self.sync_stroke(&paint);

        self.ctx.begin_path();
        self.ctx.move_to(points[0][0], points[0][1]);
        for p in points.iter().skip(1) {
            self.ctx.line_to(p[0], p[1]);
        }
        self.ctx.stroke();
    }

    /// Draws an axis-aligned oval that fills the given axis-aligned rectangle with the given Paint.
    /// Whether the oval is filled or stroked (or both) is controlled by Paint.style. 
    pub fn draw_oval(&mut self, rect: Rect, paint: Paint) {
        self.ctx.begin_path();
        self.ctx.ellipse(rect.min.x, rect.min.y, rect.dx(), rect.dy());
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
    pub fn draw_path(&mut self, path: &mut [f32], paint: Paint) {
        self.ctx.picture.xform = self.ctx.states.last().xform;

        self.ctx.begin_path();
        self.ctx.picture.append_commands(path);
        self.fill_or_stroke(&paint);
    }

    pub fn draw_path_cloned(&mut self, path: &[f32], paint: Paint) {
        let mut path = path.to_owned();
        self.ctx.picture.xform = self.ctx.states.last().xform;

        self.ctx.begin_path();
        self.ctx.picture.append_commands(&mut path);
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
        self.ctx.begin_path();
        self.ctx.rect(rect);
        self.fill_or_stroke(&paint);
    }

    /// Draws a rounded rectangle with the given Paint.
    /// Whether the rectangle is filled or stroked (or both) is controlled by Paint.style. 
    pub fn draw_rrect(&mut self, rrect: RRect, paint: Paint) {
        self.ctx.begin_path();
        self.ctx.rrect_varying(
            rrect.rect(),
            rrect.radius.tl,
            rrect.radius.tr,
            rrect.radius.br,
            rrect.radius.bl,
        );
        self.fill_or_stroke(&paint);
    }

    /*
    /// Draws a shadow for a Path representing the given material elevation. [...] 
    pub fn draw_shadow(&mut self, Path path, Color color, double elevation, bool transparentOccluder) -> void
    */
}