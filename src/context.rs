use crate::{
    backend::Backend,
    cache::PathCache,
    canvas::{Picture, Winding},
    math::{point2, Offset, Rect, Transform},
    vg::*,
};
use arrayvec::ArrayVec;

const INIT_COMMANDS_SIZE: usize = 256;

pub(crate) struct States {
    states: ArrayVec<[State; 32]>,
}

impl States {
    fn new() -> Self {
        let mut states = ArrayVec::<_>::new();
        states.push(State::default());
        Self { states }
    }

    pub(crate) fn last(&self) -> &State {
        self.states.last().expect("last state")
    }
    pub(crate) fn last_mut(&mut self) -> &mut State {
        self.states.last_mut().expect("last_mut state")
    }

    fn save(&mut self) {
        if self.states.len() >= self.states.capacity() {
            panic!("wtf?")
        }
        if let Some(state) = self.states.last().cloned() {
            self.states.push(state);
        }
    }
    fn reset(&mut self) {
        let state = if let Some(state) = self.states.last_mut() {
            state
        } else {
            self.states.push(Default::default());
            self.states.last_mut().expect("last mut state (reset)")
        };
        *state = State::default();
    }
}

pub struct Context {
    pub picture: Picture,

    pub(crate) states: States,
    pub cache: PathCache,

    pub params: Box<dyn Backend>,
}

impl Context {
    pub fn save(&mut self) {
        self.states.save();
    }
    pub fn restore(&mut self) {
        self.states.states.pop();
    }
    pub fn reset(&mut self) {
        self.states.reset();
    }
}

impl Context {
    pub fn new(params: Box<dyn Backend>) -> Self {
        Self {
            params,
            states: States::new(),
            cache: PathCache::new(),
            picture: Picture {
                commands: Vec::with_capacity(INIT_COMMANDS_SIZE),
                cmd: Offset::zero(),
                xform: Transform::identity(),
            },
        }
    }

    pub fn begin_frame(&mut self, width: f32, height: f32, dpi: f32) {
        self.states.states.clear();

        self.save();
        self.reset();
        self.cache.set_dpi(dpi);
        self.params.set_viewport(width, height, dpi);
    }

    pub fn cancel_frame(&mut self) {
        self.params.reset()
    }

    pub fn end_frame(&mut self) {
        self.params.flush();
    }

    pub fn current_transform(&self) -> &Transform {
        &self.states.last().xform
    }
    pub fn pre_transform(&mut self, m: Transform) {
        self.states.last_mut().xform.append_mut(m);
    }
    pub fn post_transform(&mut self, m: Transform) {
        self.states.last_mut().xform.prepend_mut(m);
    }
    pub fn reset_transform(&mut self) {
        self.states.last_mut().xform = Transform::identity();
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.pre_transform(Transform::translation(x, y));
    }
    pub fn rotate(&mut self, angle: f32) {
        self.pre_transform(Transform::rotation(angle));
    }
    pub fn scale(&mut self, scale: f32) {
        self.pre_transform(Transform::scale(scale));
    }

    pub fn begin_path(&mut self) {
        self.picture.commands.clear();
        self.cache.clear();
    }

    pub fn close_path(&mut self) {
        self.picture.close_path();
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.move_to(point2(x, y));
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.line_to(point2(x, y));
    }

    pub fn bezier_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture
            .bezier_to(point2(c1x, c1y), point2(c2x, c2y), point2(x, y));
    }

    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.quad_to(point2(cx, cy), point2(x, y));
    }

    pub fn arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture
            .arc_to(point2(x1, y1), point2(x2, y2), radius, self.cache.dist_tol);
    }

    pub fn arc(&mut self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, dir: Winding) {
        self.picture.xform = self.states.last().xform;
        self.picture.arc(point2(cx, cy), r, a0, a1, dir);
    }

    pub fn rect(&mut self, rect: Rect) {
        self.picture.xform = self.states.last().xform;
        self.picture.rect(rect);
    }

    pub fn rrect(&mut self, rect: Rect, radius: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture
            .rrect_varying(rect, radius, radius, radius, radius);
    }

    pub fn rrect_varying(&mut self, rect: Rect, tl: f32, tr: f32, br: f32, bl: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.rrect_varying(rect, tl, tr, br, bl);
    }

    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.ellipse(cx, cy, rx, ry);
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.circle(cx, cy, r);
    }
}
