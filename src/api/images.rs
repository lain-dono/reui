use crate::{
    context::Context,
    backend::ImageFlags,
    vg::Image,
};

pub const TEXTURE_ALPHA: i32 = 0x01;
pub const TEXTURE_RGBA: i32 = 0x02;

/// Images
///
/// NanoVG allows you to load jpg, png, psd, tga, pic and gif files to be used for rendering.
/// In addition you can upload your own image. The image loading is provided by stb_image.
/// The parameter imageFlags is combination of flags defined in NVGimageFlags.
impl Context {
    /// Creates image by loading it from the disk from specified file name.
    /// Returns handle to the image.
    pub fn create_image(&mut self, name: &str, flags: ImageFlags) -> Image {
        match image::open(name) {
            Ok(m) => {
                let m = m.to_rgba();
                let (w, h) = m.dimensions();
                let data = m.into_raw();
                self.create_image_rgba(w, h, flags, data.as_ptr())
            }
            Err(err) => {
                log::warn!("Failed to load image - {:?}", err);
                Image(0)
            }
        }
    }

    /// Creates image by loading it from the specified chunk of memory.
    /// Returns handle to the image.
    pub fn create_image_mem(&mut self, flags: ImageFlags, data: &[u8]) -> Image {
        match image::load_from_memory(data) {
            Ok(m) => {
                let m = m.to_rgba();
                let (w, h) = m.dimensions();
                let data = m.into_raw();
                self.create_image_rgba(w, h, flags, data.as_ptr())
            }
            Err(err) => {
                log::warn!("Failed to load image - {:?}", err);
                Image(0)
            }
        }
    }

    /// Creates image from specified image data.
    /// Returns handle to the image.
    pub fn create_image_rgba(&mut self, w: u32, h: u32, flags: ImageFlags, data: *const u8) -> Image {
        self.params.create_texture(TEXTURE_RGBA, w, h, flags, data)
    }

    /// Updates image data specified by image handle.
    pub fn update_image(&mut self, image: Image, data: &[u8]) {
        let (w, h) = self.params.texture_size(image).unwrap();
        self.params.update_texture(image, 0, 0, w, h, data.as_ptr());
    }

    /// Returns the dimensions of a created image.
    pub fn image_size(&mut self, image: Image) -> (u32, u32) {
        self.params.texture_size(image).unwrap()
    }

    /// Deletes created image.
    pub fn delete_image(&mut self, image: Image) {
        self.params.delete_texture(image);
    }
}
