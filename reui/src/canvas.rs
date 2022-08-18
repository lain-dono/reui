use crate::{
    internals::ImageBind, FillRule, Images, IntoPaint, Offset, Path, Recorder, Rect, Rounding,
    Stroke, Transform,
};

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
}

impl<'a, Key: Eq + std::hash::Hash> Canvas<'a, Key> {
    pub fn new(recorder: &'a mut Recorder<Key>, images: &'a Images<Key>) -> Self {
        Self {
            recorder,
            images,
            states: TransformStack(Transform::default(), Vec::with_capacity(16)),
            path: Path::new(),
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

    pub fn image_rect(&mut self, image: Key, rect: Rect) {
        let transform = self.states.transform();
        self.recorder.blit_premultiplied(rect, transform, image);
    }

    pub fn image(&mut self, image: Key, offset: Offset) {
        if let Some(ImageBind { size, .. }) = self.images.get(&image) {
            let rect = Rect {
                min: offset,
                max: offset + Offset::new(size.width as f32, size.height as f32),
            };
            self.image_rect(image, rect);
        }
    }

    /// Draws a line between the given points using the given paint.
    #[inline]
    pub fn stroke_line(&mut self, p0: Offset, p1: Offset, paint: impl IntoPaint, stroke: Stroke) {
        self.stroke(paint, stroke, |path| path.line(p0, p1));
    }

    #[inline]
    pub fn stroke_polyline(
        &mut self,
        points: &[Offset],
        close: bool,
        paint: impl IntoPaint,
        stroke: Stroke,
    ) {
        self.stroke(paint, stroke, |path| path.polyline(points, close))
    }

    #[inline]
    pub fn stroke_circle(
        &mut self,
        center: Offset,
        radius: f32,
        paint: impl IntoPaint,
        stroke: Stroke,
    ) {
        self.stroke(paint, stroke, |path| path.circle(center, radius));
    }

    #[inline]
    pub fn stroke_oval(&mut self, rect: Rect, paint: impl IntoPaint, stroke: Stroke) {
        self.stroke(paint, stroke, |path| path.oval(rect));
    }

    #[inline]
    pub fn stroke_rect(&mut self, rect: Rect, paint: impl IntoPaint, stroke: Stroke) {
        self.stroke(paint, stroke, |path| path.rect(rect));
    }

    #[inline]
    pub fn stroke_rrect(
        &mut self,
        rect: Rect,
        radius: Rounding,
        paint: impl IntoPaint,
        stroke: Stroke,
    ) {
        self.stroke(paint, stroke, |path| path.rrect(rect, radius));
    }

    #[inline]
    pub fn fill_polyline(&mut self, points: &[Offset], rule: FillRule, paint: impl IntoPaint) {
        self.fill(paint, rule, |path| path.polyline(points, true))
    }

    #[inline]
    pub fn fill_circle(&mut self, center: Offset, radius: f32, paint: impl IntoPaint) {
        self.fill_non_zero(paint, |path| path.circle(center, radius));
    }

    #[inline]
    pub fn fill_oval(&mut self, rect: Rect, paint: impl IntoPaint) {
        self.fill_non_zero(paint, |path| path.oval(rect));
    }

    #[inline]
    pub fn fill_rect(&mut self, rect: Rect, paint: impl IntoPaint) {
        self.fill_non_zero(paint, |path| path.rect(rect));
    }

    #[inline]
    pub fn fill_rrect(&mut self, rect: Rect, radius: Rounding, paint: impl IntoPaint) {
        self.fill_non_zero(paint, |path| path.rrect(rect, radius));
    }

    #[inline]
    pub fn fill_non_zero(&mut self, paint: impl IntoPaint, path: impl FnOnce(&mut Path)) {
        self.fill(paint, FillRule::NonZero, path)
    }

    #[inline]
    pub fn fill_even_odd(&mut self, paint: impl IntoPaint, path: impl FnOnce(&mut Path)) {
        self.fill(paint, FillRule::EvenOdd, path)
    }

    #[inline]
    pub fn stroke_path(&mut self, path: &Path, paint: impl IntoPaint, stroke: Stroke) {
        self.recorder
            .stroke(path, paint, stroke, self.states.transform(), true)
    }

    #[inline]
    pub fn stroke(&mut self, paint: impl IntoPaint, stroke: Stroke, path: impl FnOnce(&mut Path)) {
        self.path.clear();
        path(&mut self.path);
        self.recorder
            .stroke(&self.path, paint, stroke, self.states.transform(), true)
    }

    #[inline]
    pub fn fill_path(&mut self, path: &Path, paint: impl IntoPaint, rule: FillRule) {
        self.recorder
            .fill(path, paint, self.states.transform(), rule, true);
    }

    #[inline]
    pub fn fill(&mut self, paint: impl IntoPaint, rule: FillRule, path: impl FnOnce(&mut Path)) {
        self.path.clear();
        path(&mut self.path);
        self.recorder
            .fill(&self.path, paint, self.states.transform(), rule, true);
    }
}
