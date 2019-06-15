mod gl_shader;
mod params;
mod utils;

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
    pub struct ImageFlags: i32 {
        const GENERATE_MIPMAPS  = 1; // Generate mipmaps during creation of the image.
        const REPEATX           = 1<<1; // Repeat image in X direction.
        const REPEATY           = 1<<2; // Repeat image in Y direction.
        const FLIPY             = 1<<3; // Flips (inverses) image in Y direction when rendered.
        const PREMULTIPLIED     = 1<<4; // Image data has premultiplied alpha.
        const NEAREST           = 1<<5; // Image interpolation is Nearest instead Linear

        const NODELETE          = 1<<16;// Do not delete GL texture handle.
    }
);