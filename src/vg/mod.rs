mod composite;
mod paint;
mod color;
mod state;

pub mod utils;

pub use self::{
    composite::{CompositeState, CompositeOp, BlendFactor},
    paint::Paint,
    color::Color,
    state::State,
};

use crate::{context::Context, Rect, Transform};


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

/*
impl Context {
    pub fn global_composite(&mut self, op: CompositeOp) {
        self.global_composite_state(op.into());
    }

    pub fn global_blend(&mut self, sfactor: BlendFactor, dfactor: BlendFactor) {
        self.global_blend_separate(sfactor, dfactor, sfactor, dfactor);
    }

    pub fn global_blend_separate(
        &mut self,
        src_color: BlendFactor,
        dst_color: BlendFactor,
        src_alpha: BlendFactor,
        dst_alpha: BlendFactor,
    ) {
        self.global_composite_state(CompositeState {
            src_color,
            dst_color,
            src_alpha,
            dst_alpha,
        });
    }

    pub fn global_composite_state(&mut self, composite: CompositeState) {
        self.states.last_mut().set_composite(composite);
    }
}
*/