use crate::{Command, FillRule, Images, Offset, Paint, Path, Recorder, Rect, Rounding, Transform};

#[derive(Default)]
pub struct TransformStack(Transform, Vec<Transform>);

impl TransformStack {
    #[inline]
    pub fn save(&mut self) {
        self.1.push(self.transform());
    }

    #[inline]
    pub fn restore(&mut self) -> bool {
        self.1.pop().is_some()
    }

    #[inline]
    pub fn transform(&self) -> Transform {
        *self.1.last().unwrap_or(&self.0)
    }

    #[inline]
    pub fn pre_transform(&mut self, m: Transform) {
        *self.1.last_mut().unwrap_or(&mut self.0) *= m;
    }
}

pub struct Canvas<'a, Key = u32> {
    recorder: &'a mut Recorder<Key>,
    images: &'a Images<Key>,
    states: TransformStack,
    path: Path,
    scale: f32,
}

impl<'a, Key: Eq + std::hash::Hash> Canvas<'a, Key> {
    pub fn new(recorder: &'a mut Recorder<Key>, images: &'a Images<Key>, scale: f32) -> Self {
        Self {
            recorder,
            images,
            states: TransformStack(Transform::default(), Vec::with_capacity(16)),
            path: Path::new(),
            scale,
        }
    }

    pub fn dpi(&self) -> f32 {
        self.scale
    }

    pub fn image_rect(&mut self, image: Key, rect: Rect) {
        if self.images.get(&image).is_some() {
            let transform = self.states.transform();
            self.recorder.draw_image(rect, transform, image, [255; 4]);
        }
    }

    pub fn image(&mut self, image: Key, offset: Offset) {
        if let Some(image_bind) = self.images.get(&image) {
            let size = image_bind.size;
            let rect = Rect {
                min: offset,
                max: offset + Offset::new(size.width as f32, size.height as f32) / self.scale,
            };
            let transform = self.states.transform();
            self.recorder.draw_image(rect, transform, image, [255; 4]);
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
    pub fn push_rotate(&mut self, radians: f32) {
        self.push_transform(Transform::rotation(radians));
    }

    /// Add an axis-aligned scale to the current transform,
    /// scaling by the first argument in the horizontal direction and the second in the vertical direction. [...]
    pub fn push_scale(&mut self, scale: f32) {
        self.push_transform(Transform::scale(scale));
    }

    /// Add a translation to the current transform,
    /// shifting the coordinate space horizontally by the first argument and vertically by the second argument.
    pub fn push_translate(&mut self, dx: f32, dy: f32) {
        self.push_transform(Transform::translation(dx, dy));
    }

    /// Multiply the current transform by the specified 4⨉4 transformation matrix specified as a list of values in column-major order.
    pub fn push_transform(&mut self, t: Transform) {
        self.states.pre_transform(t);
    }
}

impl<'a, Key: Eq + std::hash::Hash> Canvas<'a, Key> {
    /// Draws a line between the given points using the given paint.
    /// The line is stroked, the value of the Paint.style is ignored for this call.
    ///
    /// The p1 and p2 arguments are interpreted as offsets from the origin.
    #[inline]
    pub fn stroke_line(&mut self, p0: Offset, p1: Offset, paint: impl Into<Paint>) {
        self.stroke(paint, |path| path.line(p0, p1));
    }

    #[inline]
    pub fn stroke_polyline(&mut self, points: &[Offset], close: bool, paint: impl Into<Paint>) {
        self.stroke(paint, |path| path.polyline(points, close))
    }

    #[inline]
    pub fn stroke_circle(&mut self, center: Offset, radius: f32, paint: impl Into<Paint>) {
        self.stroke(paint, |path| path.circle(center, radius));
    }

    #[inline]
    pub fn stroke_oval(&mut self, rect: Rect, paint: impl Into<Paint>) {
        self.stroke(paint, |path| path.oval(rect));
    }

    #[inline]
    pub fn stroke_rect(&mut self, rect: Rect, paint: impl Into<Paint>) {
        self.stroke(paint, |path| path.rect(rect));
    }

    #[inline]
    pub fn stroke_rrect(&mut self, rect: Rect, radius: Rounding, paint: impl Into<Paint>) {
        self.stroke(paint, |path| path.rrect(rect, radius));
    }

    #[inline]
    pub fn stroke_path<P>(&mut self, path_iter: P, paint: impl Into<Paint>)
    where
        P: IntoIterator<Item = Command>,
    {
        self.stroke(paint, |path| path.extend(path_iter))
    }

    #[inline]
    pub fn stroke(&mut self, paint: impl Into<Paint>, path: impl FnOnce(&mut Path)) {
        self.path.clear();
        path(&mut self.path);

        let xform = self.states.transform();
        let scale = self.scale;
        self.recorder.stroke_path(&self.path, paint, xform, scale)
    }

    #[inline]
    pub fn fill_polyline(&mut self, points: &[Offset], rule: FillRule, paint: impl Into<Paint>) {
        self.fill(paint, rule, |path| path.polyline(points, true))
    }

    #[inline]
    pub fn fill_circle(&mut self, center: Offset, radius: f32, paint: impl Into<Paint>) {
        self.fill_non_zero(paint, |path| path.circle(center, radius));
    }

    #[inline]
    pub fn fill_oval(&mut self, rect: Rect, paint: impl Into<Paint>) {
        self.fill_non_zero(paint, |path| path.oval(rect));
    }

    #[inline]
    pub fn fill_rect(&mut self, rect: Rect, paint: impl Into<Paint>) {
        self.fill_non_zero(paint, |path| path.rect(rect));
    }

    #[inline]
    pub fn fill_rrect(&mut self, rect: Rect, radius: Rounding, paint: impl Into<Paint>) {
        self.fill_non_zero(paint, |path| path.rrect(rect, radius));
    }

    #[inline]
    pub fn fill_path<P>(&mut self, path_iter: P, paint: impl Into<Paint>, fill_rule: FillRule)
    where
        P: IntoIterator<Item = Command>,
    {
        self.fill(paint, fill_rule, |path| path.extend(path_iter))
    }

    #[inline]
    pub fn fill_non_zero(&mut self, paint: impl Into<Paint>, path: impl FnOnce(&mut Path)) {
        self.fill(paint, FillRule::NonZero, path)
    }

    #[inline]
    pub fn fill_even_odd(&mut self, paint: impl Into<Paint>, path: impl FnOnce(&mut Path)) {
        self.fill(paint, FillRule::EvenOdd, path)
    }

    #[inline]
    pub fn fill(&mut self, paint: impl Into<Paint>, rule: FillRule, path: impl FnOnce(&mut Path)) {
        self.path.clear();
        path(&mut self.path);

        let xform = self.states.transform();
        let scale = self.scale;
        self.recorder
            .fill_path(&self.path, paint, xform, scale, rule);
    }
}
