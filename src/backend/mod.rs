mod gl;
mod gl_shader;
mod params;

pub use self::params::{BackendGL, NFlags};

use slotmap::Key;

slotmap::new_key_type! {
    pub struct Image;
}

bitflags::bitflags!(
    #[derive(Default)]
    pub struct ImageFlags: i32 {
        const GENERATE_MIPMAPS  = 1<<0; // Generate mipmaps during creation of the image.
        const REPEATX           = 1<<1; // Repeat image in X direction.
        const REPEATY           = 1<<2; // Repeat image in Y direction.
        const FLIPY             = 1<<3; // Flips (inverses) image in Y direction when rendered.
        const PREMULTIPLIED     = 1<<4; // Image data has premultiplied alpha.
        const NEAREST           = 1<<5; // Image interpolation is Nearest instead Linear

        const NODELETE          = 1<<16;// Do not delete GL texture handle.
    }
);

#[no_mangle] extern "C"
fn nvgDeleteGL2(ctx: *const u8) {
}

#[no_mangle] extern "C"
fn nvgCreateGL2(flags: NFlags) -> Box<crate::context::Context> {
    Box::new(crate::context::Context::new(BackendGL::new(flags)))
}
