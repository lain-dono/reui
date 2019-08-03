use super::gl::{self, types::{GLenum, GLuint}};
use crate::vg::{CompositeState, BlendFactor};

pub fn gl_draw_strip(offset: usize, count: usize) {
    unsafe {
        gl::DrawArrays(gl::TRIANGLE_STRIP, offset as i32, count as i32);
    }
}

pub fn gl_draw_triangles(offset: usize, count: usize) {
    unsafe {
        gl::DrawArrays(gl::TRIANGLES, offset as i32, count as i32);
    }
}

pub struct Buffer(GLuint);

impl Drop for Buffer {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe { gl::DeleteBuffers(1, &self.0); }
        }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        let mut buf = 0;
        // Create dynamic vertex array
        unsafe {
            gl::GenBuffers(1, &mut buf);
            gl::Finish();
        }
        Buffer(buf)
    }
}

impl Buffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind_and_upload<T: Sized>(&self, data: &[T]) {
        let size = std::mem::size_of::<T>();
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.0);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * size) as isize,
                data.as_ptr() as *const libc::c_void,
                gl::STREAM_DRAW,
            );
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}


pub struct Blend {
    src_color: GLenum,
    dst_color: GLenum,
    src_alpha: GLenum,
    dst_alpha: GLenum,
}

impl Blend {
    pub fn bind(&self) {
        unsafe {
            gl::BlendFuncSeparate(
                self.src_color,
                self.dst_color,
                self.src_alpha,
                self.dst_alpha,
            );
        }
    }
}

fn blend_factor(factor: BlendFactor) -> GLenum {
    match factor {
        BlendFactor::Zero               => gl::ZERO,
        BlendFactor::One                => gl::ONE,
        BlendFactor::SrcColor           => gl::SRC_COLOR,
        BlendFactor::OneMinusSrcColor   => gl::ONE_MINUS_SRC_COLOR,
        BlendFactor::SrcAlpha           => gl::SRC_ALPHA,
        BlendFactor::OneMinusSrcAlpha   => gl::ONE_MINUS_SRC_ALPHA,
        BlendFactor::DstAlpha           => gl::DST_ALPHA,
        BlendFactor::OneMinusDstAlpha   => gl::ONE_MINUS_DST_ALPHA,
        BlendFactor::DstColor           => gl::DST_COLOR,
        BlendFactor::OneMinusDstColor   => gl::ONE_MINUS_DST_COLOR,
        BlendFactor::SrcAlphaSaturate   => gl::SRC_ALPHA_SATURATE,
    }
}

impl From<CompositeState> for Blend {
    fn from(op: CompositeState) -> Self {
        Self {
            src_color: blend_factor(op.src_color),
            dst_color: blend_factor(op.dst_color),
            src_alpha: blend_factor(op.src_alpha),
            dst_alpha: blend_factor(op.dst_alpha),
        }
    }
}

/*
impl Default for Blend {
    fn default() -> Self {
        Self::from(CompositeState::default())
    }
}
*/

impl Default for Blend {
    fn default() -> Self {
        Self {
            src_color: gl::ONE,
            dst_color: gl::ONE_MINUS_SRC_ALPHA,
            src_alpha: gl::ONE,
            dst_alpha: gl::ONE_MINUS_SRC_ALPHA,
        }
    }
}
