use crate::{
    backend::Backend,
    cache::PathCache,
    canvas::Picture,
    math::{Offset, Transform},
    state::State,
};

const INIT_COMMANDS_SIZE: usize = 256;

pub struct Context {
    pub picture: Picture,
    pub states: Vec<State>,
    pub cache: PathCache,
    pub cmd: crate::backend::CmdBuffer,
    pub backend: Box<dyn Backend>,

    width: f32,
    height: f32,
    dpi: f32,
}

impl Context {
    pub fn new(backend: Box<dyn Backend>) -> Self {
        Self {
            backend,
            states: Vec::with_capacity(256),
            cache: PathCache::new(),
            picture: Picture {
                commands: Vec::with_capacity(INIT_COMMANDS_SIZE),
                cmd: Offset::zero(),
                xform: Transform::identity(),
            },

            cmd: crate::backend::CmdBuffer::new(),

            width: 1.0,
            height: 1.0,
            dpi: 1.0,
        }
    }

    pub fn begin_frame(&mut self, width: f32, height: f32, dpi: f32) {
        self.states.clear();
        self.states.push(Default::default());
        self.cache.set_dpi(dpi);
        self.width = width;
        self.height = height;
        self.dpi = dpi;
    }

    pub fn end_frame(&mut self) {
        self.backend.draw_commands(&self.cmd, self.width, self.height, self.dpi);
        self.cmd.clear();
    }

    pub fn _begin_path(&mut self) {
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
