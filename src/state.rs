use crate::math::{Rect, Transform};

#[derive(Clone, Copy)]
pub struct Scissor {
    pub xform: Transform,
    pub extent: [f32; 2],
}

#[derive(Default)]
pub struct States(State, Vec<State>);

impl States {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Default::default(), Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0 = Default::default();
        self.1.clear();
    }

    #[inline]
    pub fn save_count(&mut self) -> usize {
        self.1.len()
    }

    #[inline]
    pub fn save(&mut self) {
        self.1.push(self.last().clone())
    }

    #[inline]
    pub fn restore(&mut self) -> bool {
        self.1.pop().is_some()
    }

    #[inline]
    pub fn transform(&self) -> Transform {
        self.last().xform
    }

    #[inline]
    pub fn decompose(&self) -> (Transform, Scissor) {
        let State { xform, scissor } = self.last().clone();
        (xform, scissor)
    }

    #[inline]
    pub fn pre_transform(&mut self, m: Transform) {
        self.last_mut().xform.append_mut(m);
    }

    #[inline]
    pub fn set_scissor(&mut self, rect: Rect) {
        self.last_mut().set_scissor(rect);
    }

    #[inline]
    pub fn intersect_scissor(&mut self, rect: Rect) {
        self.last_mut().intersect_scissor(rect)
    }

    #[inline]
    pub fn reset_scissor(&mut self) {
        self.last_mut().reset_scissor()
    }

    #[inline(always)]
    fn last(&self) -> &State {
        self.1.last().unwrap_or(&self.0)
    }

    #[inline(always)]
    fn last_mut(&mut self) -> &mut State {
        self.1.last_mut().unwrap_or(&mut self.0)
    }
}

#[derive(Clone)]
struct State {
    xform: Transform,
    scissor: Scissor,
}

impl Default for State {
    fn default() -> Self {
        Self {
            xform: Transform::identity(),
            scissor: Scissor {
                extent: [-1.0, -1.0],
                xform: Transform::identity(),
            },
        }
    }
}

impl State {
    fn set_scissor(&mut self, rect: Rect) {
        let [x, y, w, h] = rect.to_xywh();

        let translate = Transform::translation(x + w * 0.5, y + h * 0.5);
        self.scissor.xform = self.xform.append(translate);

        self.scissor.extent[0] = w * 0.5;
        self.scissor.extent[1] = h * 0.5;
    }

    fn intersect_scissor(&mut self, r: Rect) {
        // If no previous scissor has been set, set the scissor as current scissor.
        if self.scissor.extent[0] < 0.0 {
            self.set_scissor(r);
            return;
        }

        // Transform the current scissor rect into current transform space.
        // If there is difference in rotation, this will be approximation.
        let inv = self.xform.inverse();
        let xform = self.scissor.xform.prepend(inv);

        let ex = self.scissor.extent[0];
        let ey = self.scissor.extent[1];

        let tex = ex * xform.re.abs() + ey * xform.im.abs();
        let tey = ex * xform.im.abs() + ey * xform.re.abs();

        // Intersect rects.
        let (ax, ay) = (r.min.x, r.min.y);
        let (aw, ah) = (r.dx(), r.dy());

        let (bx, by) = (xform.tx - tex, xform.ty - tey);
        let (bw, bh) = (tex * 2.0, tey * 2.0);

        let minx = f32::max(ax, bx);
        let miny = f32::max(ay, by);
        let maxx = f32::min(ax + aw, bx + bw);
        let maxy = f32::min(ay + ah, by + bh);
        self.set_scissor(Rect {
            min: (minx, miny).into(),
            max: (maxx, maxy).into(),
        });
    }

    fn reset_scissor(&mut self) {
        self.scissor.xform = Transform::identity();
        self.scissor.extent = [-1.0, -1.0];
    }
}
