use crate::{
    backend::Backend,
    cache::PathCache,
    canvas::Picture,
    math::{Offset, Transform},
    state::State,
};

const INIT_COMMANDS_SIZE: usize = 256;

pub(crate) struct States {
    pub states: Vec<State>,
}

impl States {
    fn new() -> Self {
        let mut states = Vec::with_capacity(32);
        states.push(State::default());
        Self { states }
    }

    pub fn last(&self) -> &State {
        self.states.last().expect("last state")
    }
    pub fn last_mut(&mut self) -> &mut State {
        self.states.last_mut().expect("last_mut state")
    }
}

pub struct Context {
    pub(crate) picture: Picture,
    pub(crate) states: States,
    pub(crate) cache: PathCache,
    pub(crate) params: Box<dyn Backend>,
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
        self.states.states.push(Default::default());
        self.cache.set_dpi(dpi);
        self.params.set_viewport(width, height, dpi);
    }

    pub fn cancel_frame(&mut self) {
        self.params.reset()
    }

    pub fn end_frame(&mut self) {
        self.params.flush();
    }

    pub fn pre_transform(&mut self, m: Transform) {
        self.states.last_mut().xform.append_mut(m);
    }

    pub fn begin_path(&mut self) {
        self.picture.commands.clear();
        self.cache.clear();
    }

    /*
    pub fn _close_path(&mut self) {
        self.picture.close_path();
    }

    pub fn _bezier_to(&mut self, p1: Offset, p2: Offset, p3: Offset) {
        self.picture.xform = self.states.last().xform;
        self.picture.bezier_to(p1, p2, p3);
    }

    pub fn _quad_to(&mut self, p1: Offset, p2: Offset) {
        self.picture.xform = self.states.last().xform;
        self.picture.quad_to(p1, p2);
    }

    pub fn _arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture
            .arc_to(point2(x1, y1), point2(x2, y2), radius, self.cache.dist_tol);
    }

    pub fn _arc(&mut self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, dir: Winding) {
        self.picture.xform = self.states.last().xform;
        self.picture.arc(point2(cx, cy), r, a0, a1, dir);
    }
    */
}
