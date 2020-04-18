mod gl_backend;
mod gl_shader;

pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub use self::gl_backend::BackendGL;

use crate::{
    cache::Path,
    vg::{Paint, Scissor},
};

pub trait Backend {
    fn reset(&mut self);

    fn draw_fill(
        &mut self,
        paint: &Paint,
        scissor: &Scissor,
        fringe: f32,
        bounds: &[f32; 4],
        paths: &[Path],
    );

    fn draw_stroke(
        &mut self,
        paint: &Paint,
        scissor: &Scissor,
        fringe: f32,
        stroke_width: f32,
        paths: &[Path],
    );

    fn set_viewport(&mut self, width: f32, height: f32, pixel_ratio: f32);

    fn flush(&mut self);
}
