mod gl_shader;
mod params;

pub mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub use self::params::{BackendGL, NFlags};

slotmap::new_key_type! {
    pub struct Image;
}

pub const TEXTURE_ALPHA: i32 = 0x01;
pub const TEXTURE_RGBA: i32 = 0x02;

bitflags::bitflags!(
    #[derive(Default)]
    pub struct ImageFlags: u32 {
        const REPEAT            = 1<<1; // Repeat image in X direction.
        const PREMULTIPLIED     = 1<<4; // Image data has premultiplied alpha.
        const NEAREST           = 1<<5; // Image interpolation is Nearest instead Linear
    }
);

use crate::{
    cache::{Path, Vertex},
    math::Transform,
    vg::{Paint, Scissor},
};

pub trait Backend {
    fn reset(&mut self);

    fn draw_triangles(&mut self, paint: &Paint, scissor: &Scissor, verts: &[Vertex]);

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

    fn texture_size(&self, image: Image) -> Option<(u32, u32)>;

    fn update_texture(
        &mut self,
        image: Image,
        _x: i32,
        y: i32,
        _w: u32,
        h: u32,
        data: &[u8],
    ) -> bool;

    fn delete_texture(&mut self, image: Image) -> bool;

    fn create_texture(
        &mut self,
        kind: i32,
        w: u32,
        h: u32,
        flags: ImageFlags,
        data: *const u8,
    ) -> Image;
}
