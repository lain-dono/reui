mod gl_backend;
mod gl_shader;

mod paint;

pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub use self::gl_backend::BackendGL;
pub use self::paint::Paint;

use crate::{cache::Path, math::Transform};

#[derive(Clone, Copy)]
pub struct Scissor {
    pub xform: Transform,
    pub extent: [f32; 2],
}

pub trait Backend {
    fn draw_fill(
        &mut self,
        paint: Paint,
        scissor: Scissor,
        fringe: f32,
        bounds: [f32; 4],
        paths: &[Path],
    );

    fn draw_stroke(
        &mut self,
        paint: Paint,
        scissor: Scissor,
        fringe: f32,
        stroke_width: f32,
        paths: &[Path],
    );

    fn begin_frame(&mut self, width: f32, height: f32, pixel_ratio: f32);
    fn cancel_frame(&mut self);
    fn end_frame(&mut self);
}
