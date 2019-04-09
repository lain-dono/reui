use crate::{
    context::Context,
    params::Params,
    cache::{Winding, LineJoin, LineCap},
    vg::*,
};

use std::{
    ffi::{CStr, c_void},
    os::raw::c_char,
    slice::from_raw_parts,
};

pub const TEXTURE_ALPHA: i32 = 0x01;
pub const TEXTURE_RGBA: i32 = 0x02;

/*
#[no_mangle] extern "C"
fn nvgCreateImage(ctx: &mut Context, filename: *const c_char, flags: i32) -> u32 {
    let filename = unsafe { CStr::from_ptr(filename).to_string_lossy().into_owned() };

    //let data = unsafe { from_raw_parts(data, ndata as usize) };
    match image::open(filename) {
        Ok(m) => {
            let m = m.to_rgba();
            let (w, h) = m.dimensions();
            let data = m.into_raw();
            nvgCreateImageRGBA(ctx, w, h, flags, data.as_ptr())
        }
        Err(err) => {
            log::warn!("Failed to load image - {:?}", err);
            0
        }
    }
}

#[no_mangle] extern "C"
fn nvgCreateImageMem(ctx: &mut Context, flags: i32, data: *mut u8, ndata: i32) -> u32 {
    let data = unsafe { from_raw_parts(data, ndata as usize) };
    match image::load_from_memory(data) {
        Ok(m) => {
            let m = m.to_rgba();
            let (w, h) = m.dimensions();
            let data = m.into_raw();
            nvgCreateImageRGBA(ctx, w, h, flags, data.as_ptr())
        }
        Err(err) => {
            log::warn!("Failed to load image - {:?}", err);
            0
        }
    }
}


#[no_mangle] extern "C"
fn nvgCreateImageRGBA(ctx: &mut Context, w: u32, h: u32, flags: i32, data: *const u8) -> u32 {
    ctx.params.create_texture(TEXTURE_RGBA, w, h, flags, data)
}

#[no_mangle] extern "C"
fn nvgUpdateImage(ctx: &mut Context, image: u32, data: *const u8) {
    let (w, h) = ctx.params.texture_size(image).unwrap();
    ctx.params.update_texture(image, 0, 0, w, h, data);
}

#[no_mangle] extern "C"
fn nvgImageSize(ctx: &mut Context, image: u32, w: &mut u32, h: &mut u32) {
    let (_w, _h) = ctx.image_size(image);
    *w = _w;
    *h = _h;
}

#[no_mangle] extern "C"
fn nvgDeleteImage(ctx: &mut Context, image: u32) {
    ctx.delete_image(image);
}
*/

/// Images
///
/// NanoVG allows you to load jpg, png, psd, tga, pic and gif files to be used for rendering.
/// In addition you can upload your own image. The image loading is provided by stb_image.
/// The parameter imageFlags is combination of flags defined in NVGimageFlags.
impl Context {
    /// Creates image by loading it from the disk from specified file name.
    /// Returns handle to the image.
    pub fn create_image(&mut self, name: &str, flags: i32) -> u32 {
        match image::open(name) {
            Ok(m) => {
                let m = m.to_rgba();
                let (w, h) = m.dimensions();
                let data = m.into_raw();
                self.create_image_rgba(w, h, flags, data.as_ptr())
            }
            Err(err) => {
                log::warn!("Failed to load image - {:?}", err);
                0
            }
        }
    }

    /// Creates image by loading it from the specified chunk of memory.
    /// Returns handle to the image.
    pub fn create_image_mem(&mut self, flags: i32, data: &[u8]) -> u32 {
        match image::load_from_memory(data) {
            Ok(m) => {
                let m = m.to_rgba();
                let (w, h) = m.dimensions();
                let data = m.into_raw();
                self.create_image_rgba(w, h, flags, data.as_ptr())
            }
            Err(err) => {
                log::warn!("Failed to load image - {:?}", err);
                0
            }
        }
    }

    /// Creates image from specified image data.
    /// Returns handle to the image.
    pub fn create_image_rgba(&mut self, w: u32, h: u32, flags: i32, data: *const u8) -> u32 {
        self.params.create_texture(TEXTURE_RGBA, w, h, flags, data)
    }

    /// Updates image data specified by image handle.
    pub fn update_image(&mut self, image: u32, data: &[u8]) {
        let (w, h) = self.params.texture_size(image).unwrap();
        self.params.update_texture(image, 0, 0, w, h, data.as_ptr());
    }

    /// Returns the dimensions of a created image.
    pub fn image_size(&mut self, image: u32) -> (u32, u32) {
        self.params.texture_size(image).unwrap()
    }

    /// Deletes created image.
    pub fn delete_image(&mut self, image: u32) {
        self.params.delete_texture(image);
    }
}
