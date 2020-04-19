mod commands;
mod gl_backend;
mod paint;

pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub use self::commands::{CmdBuffer, FragUniforms, CallKind};
pub use self::gl_backend::BackendGL;
pub use self::paint::Paint;
use crate::math::Transform;

#[derive(Clone, Copy)]
pub struct Scissor {
    pub xform: Transform,
    pub extent: [f32; 2],
}

pub trait Backend {
    fn draw_commands(&mut self, cmd: &CmdBuffer, width: f32, height: f32, pixel_ratio: f32);
}
