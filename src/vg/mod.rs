mod paint;
mod state;

pub mod utils;

pub use self::{paint::Paint, state::State};

use crate::{
    context::Context,
    math::{Rect, Transform},
};

#[derive(Clone, Copy)]
pub struct Scissor {
    pub xform: Transform,
    pub extent: [f32; 2],
}

impl Context {
    pub fn scissor(&mut self, rect: Rect) {
        self.states.last_mut().set_scissor(rect);
    }

    pub fn intersect_scissor(&mut self, rect: Rect) {
        self.states.last_mut().intersect_scissor(rect);
    }

    pub fn reset_scissor(&mut self) {
        self.states.last_mut().reset_scissor();
    }
}
