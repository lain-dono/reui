use arrayvec::ArrayVec;

use crate::{
    backend::{Backend, BackendGL},
    cache::{LineCap, LineJoin, PathCache},
    canvas::Picture,
    math::{Color, Offset, Transform},
    vg::*,
};

const INIT_COMMANDS_SIZE: usize = 256;

pub(crate) struct States {
    states: ArrayVec<[State; 32]>,
}

impl States {
    pub(crate) fn new() -> Self {
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
    pub(crate) fn clear(&mut self) {
        self.states.clear();
    }
    fn save(&mut self) {
        if self.states.len() >= self.states.capacity() {
            return;
        }
        if let Some(state) = self.states.last().cloned() {
            self.states.push(state);
        }
    }
    fn restore(&mut self) {
        self.states.pop();
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
    pub device_px_ratio: f32,

    pub params: BackendGL,
}

// FIXME
unsafe impl Sync for Context {}
unsafe impl Send for Context {}

impl Context {
    pub fn save(&mut self) {
        self.states.save();
    }
    pub fn restore(&mut self) {
        self.states.restore();
    }
    pub fn reset(&mut self) {
        self.states.reset();
    }

    // State setting

    pub(crate) fn shape_anti_alias(&mut self, enabled: bool) {
        self.states.last_mut().shape_aa = enabled;
    }
    pub(crate) fn stroke_width(&mut self, width: f32) {
        self.states.last_mut().stroke_width = width;
    }
    pub(crate) fn miter_limit(&mut self, limit: f32) {
        self.states.last_mut().miter_limit = limit;
    }
    pub(crate) fn line_cap(&mut self, cap: LineCap) {
        self.states.last_mut().line_cap = cap;
    }
    pub(crate) fn line_join(&mut self, join: LineJoin) {
        self.states.last_mut().line_join = join;
    }
    pub(crate) fn global_alpha(&mut self, alpha: f32) {
        self.states.last_mut().alpha = alpha;
    }
    pub(crate) fn stroke_color(&mut self, color: u32) {
        self.states.last_mut().stroke = Paint::color(Color::new(color))
    }
    pub(crate) fn fill_color(&mut self, color: u32) {
        self.states.last_mut().fill = Paint::color(Color::new(color))
    }
    pub(crate) fn stroke_paint(&mut self, paint: Paint) {
        let state = self.states.last_mut();
        state.stroke = paint;
        state.stroke.xform.prepend_mut(state.xform);
    }
    pub(crate) fn fill_paint(&mut self, paint: Paint) {
        let state = self.states.last_mut();
        state.fill = paint;
        state.fill.xform.prepend_mut(state.xform);
    }
}

impl Context {
    pub fn begin_frame(&mut self, width: f32, height: f32, dpi: f32) {
        self.states.clear();
        self.save();
        self.reset();
        self.set_dpi(dpi);

        self.params.set_viewport(width, height, dpi);
    }

    pub fn cancel_frame(&mut self) {
        self.params.reset()
    }

    pub fn end_frame(&mut self) {
        self.params.flush();
    }

    pub fn set_dpi(&mut self, ratio: f32) {
        self.cache.set_dpi(ratio);
        self.device_px_ratio = ratio;
    }

    pub fn new(params: BackendGL) -> Self {
        Self {
            params,

            states: States::new(),
            cache: PathCache::new(),

            picture: Picture {
                commands: Vec::with_capacity(INIT_COMMANDS_SIZE),
                cmd: Offset::zero(),
                xform: Transform::identity(),
            },

            device_px_ratio: 1.0,
        }
    }
}

impl Context {
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
}
