use super::{
    gl::{self, types::GLuint},
    ImageFlags, TEXTURE_RGBA,
};

#[derive(Debug)]
pub struct Texture {
    pub tex: GLuint,
    pub kind: i32,
    pub flags: ImageFlags,
    width: u32,
    height: u32,
}

impl Drop for Texture {
    fn drop(&mut self) {
        if self.tex != 0 {
            unsafe {
                gl::DeleteTextures(1, &self.tex);
            }
            self.tex = 0;
        }
    }
}

impl Default for Texture {
    fn default() -> Self {
        Self {
            tex: 0,
            width: 0,
            height: 0,
            kind: 0,
            flags: ImageFlags::empty(),
        }
    }
}

pub struct TextureManager<K: slotmap::Key> {
    textures: slotmap::SlotMap<K, Texture>,
}

impl<K: slotmap::Key + Copy> TextureManager<K> {
    pub fn new() -> Self {
        Self {
            textures: slotmap::SlotMap::with_key(),
        }
    }

    pub fn texture_size(&self, image: K) -> Option<(u32, u32)> {
        self.find_texture(image).map(|t| (t.width, t.height))
    }

    pub fn find_texture(&self, image: K) -> Option<&Texture> {
        self.textures.get(image)
    }

    pub fn bind(&self, image: K) {
        unsafe {
            let tex = self.find_texture(image);
            gl::BindTexture(gl::TEXTURE_2D, tex.map(|t| t.tex).unwrap_or(0));
            super::check_error("tex paint tex");
        }
    }

    pub fn update_texture(
        &mut self,
        image: K,
        _x: i32,
        y: i32,
        _w: u32,
        h: u32,
        data: &[u8],
    ) -> bool {
        let tex = if let Some(tex) = self.find_texture(image) {
            tex
        } else {
            return false;
        };

        // No support for all of skip, need to update a whole row at a time.
        let (kind, stride) = if tex.kind == TEXTURE_RGBA {
            (gl::RGBA, tex.width * 4)
        } else {
            (gl::LUMINANCE, tex.width)
        };

        let stride = y * stride as i32;
        let data = &data[stride as usize..];

        let x = 0;
        let w = tex.width;

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, tex.tex);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x,
                y,
                w as i32,
                h as i32,
                kind,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const libc::c_void,
            );

            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        true
    }

    pub fn delete_texture(&mut self, image: K) -> bool {
        self.textures.remove(image).is_some()
    }

    pub fn create_texture(
        &mut self,
        kind: i32,
        w: u32,
        h: u32,
        flags: ImageFlags,
        data: *const u8,
    ) -> K {
        // GL 1.4 and later has support for generating mipmaps using a tex parameter.
        let mipmaps = gl::FALSE as i32;

        let kind_tex = if kind == TEXTURE_RGBA {
            gl::RGBA
        } else {
            gl::LUMINANCE
        };

        let min = if flags.contains(ImageFlags::NEAREST) {
            gl::NEAREST
        } else {
            gl::LINEAR
        };

        let mag = if flags.contains(ImageFlags::NEAREST) {
            gl::NEAREST
        } else {
            gl::LINEAR
        };
        let wrap = if flags.contains(ImageFlags::REPEAT) {
            gl::REPEAT
        } else {
            gl::CLAMP_TO_EDGE
        };

        let mut tex = 0;
        unsafe {
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            gl::TexParameteri(gl::TEXTURE_2D, gl::GENERATE_MIPMAP_HINT, mipmaps);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                kind_tex as i32,
                w as i32,
                h as i32,
                0,
                kind_tex,
                gl::UNSIGNED_BYTE,
                data as *const libc::c_void,
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag as i32);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap as i32);

            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);

            super::check_error("create tex");
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        self.textures.insert(Texture {
            kind,
            flags,
            tex,
            width: w,
            height: h,
        })
    }
}
