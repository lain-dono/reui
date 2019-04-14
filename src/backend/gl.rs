
use crate::vg::{CompositeState, BlendFactor};

pub type GLuint = u32;
pub type GLint = i32;
pub type GLenum = u32;

pub type GLchar = u8;

pub type GLboolean = u8;

pub type GLsizei = i32;

pub type GLsizeiptr = usize;
pub type GLfloat = f32;

pub const GL_TRIANGLES: GLenum      = 0x0004;
pub const GL_TRIANGLE_STRIP: GLenum = 0x0005;

pub const GL_FRONT: GLenum = 0x0404;
pub const GL_BACK: GLenum = 0x0405;

pub const GL_KEEP: GLenum = 0x1E00;

pub const GL_FALSE: GLboolean = 0;
pub const GL_TRUE: GLboolean = 1;

pub const GL_ZERO: GLenum = 0;
pub const GL_ONE: GLenum = 1;
pub const GL_SRC_COLOR: GLenum              = 0x0300;
pub const GL_ONE_MINUS_SRC_COLOR: GLenum    = 0x0301;
pub const GL_SRC_ALPHA: GLenum              = 0x0302;
pub const GL_ONE_MINUS_SRC_ALPHA: GLenum    = 0x0303;
pub const GL_DST_ALPHA: GLenum              = 0x0304;
pub const GL_ONE_MINUS_DST_ALPHA: GLenum    = 0x0305;
pub const GL_DST_COLOR: GLenum              = 0x0306;
pub const GL_ONE_MINUS_DST_COLOR: GLenum    = 0x0307;
pub const GL_SRC_ALPHA_SATURATE: GLenum     = 0x0308;

pub const GL_EQUAL: GLenum    = 0x0202;
pub const GL_NOTEQUAL: GLenum = 0x0205;
pub const GL_ALWAYS: GLenum   = 0x0207;

pub const GL_CULL_FACE: GLenum    = 0x0B44;
pub const GL_STENCIL_TEST: GLenum = 0x0B90;
pub const GL_BLEND: GLenum        = 0x0BE2;
pub const GL_DEPTH_TEST: GLenum   = 0x0B71;
pub const GL_SCISSOR_TEST: GLenum = 0x0C11;

pub const GL_INCR: GLenum      = 0x1E02;
pub const GL_INCR_WRAP: GLenum = 0x8507;
pub const GL_DECR_WRAP: GLenum = 0x8508;

pub const GL_ARRAY_BUFFER: GLenum = 0x8892;
pub const GL_CCW: GLenum = 0x0901;
pub const GL_STREAM_DRAW: GLenum = 0x88E0;

pub const GL_TEXTURE0: GLenum = 0x84C0;
pub const GL_TEXTURE_2D: GLenum = 0x0DE1;

pub const GL_UNSIGNED_BYTE: GLenum = 0x1401;
pub const GL_UNSIGNED_SHORT: GLenum = 0x1403;
pub const GL_FLOAT: GLenum         = 0x1406;

pub const GL_UNPACK_ALIGNMENT: GLenum = 0x0CF5;
pub const GL_RGBA: GLenum = 0x1908;
pub const GL_LUMINANCE: GLenum = 0x1909;

pub const GL_REPEAT: i32 = 0x2901;
pub const GL_CLAMP_TO_EDGE: i32 = 0x812F;

pub const GL_NEAREST: i32 = 0x2600;
pub const GL_LINEAR: i32 = 0x2601;
pub const GL_NEAREST_MIPMAP_NEAREST: i32 = 0x2700;
pub const GL_LINEAR_MIPMAP_NEAREST : i32 = 0x2701;
pub const GL_NEAREST_MIPMAP_LINEAR : i32 = 0x2702;
pub const GL_LINEAR_MIPMAP_LINEAR  : i32 = 0x2703;

pub const GL_TEXTURE_MAG_FILTER    : GLenum = 0x2800;
pub const GL_TEXTURE_MIN_FILTER: GLenum = 0x2801;

pub const GL_TEXTURE_WRAP_S: GLenum = 0x2802;
pub const GL_TEXTURE_WRAP_T: GLenum = 0x2803;
pub const GL_GENERATE_MIPMAP: GLenum = 0x8191;

pub const GL_NO_ERROR: GLenum = 0;

pub const GL_FRAGMENT_SHADER: GLenum = 0x8B30;
pub const GL_VERTEX_SHADER: GLenum = 0x8B31;

extern "C" {
    pub fn glDrawArrays(mode: GLenum, first: GLint, count: GLsizei);
    pub fn glEnable(cap: GLenum);
    pub fn glDisable(cap: GLenum);

    pub fn glStencilFunc(func: GLenum, _ref: GLint, mask: GLuint);
    //pub fn glStencilFuncSeparate(face: GLenum, func: GLenum, _ref: GLint, mask: GLuint);
    pub fn glStencilMask(mask: GLuint);
    //pub fn glStencilMaskSeparate(face: GLenum, mask: GLuint);
    pub fn glStencilOp(fail: GLenum, zfail: GLenum, zpass: GLenum);
    pub fn glStencilOpSeparate(face: GLenum, sfail: GLenum, dpfail: GLenum, dppass: GLenum);

    pub fn glColorMask(red: GLboolean, green: GLboolean, blue: GLboolean, alpha: GLboolean);
    pub fn glUseProgram(program: GLuint);
    pub fn glBindBuffer(target: GLenum, buffer: GLuint);
    pub fn glBindTexture(target: GLenum, texture: GLuint);
    pub fn glActiveTexture(texture: GLenum);

    fn glBufferData(target: GLenum, size: GLsizeiptr, data: *const u8, usage: GLenum);

    pub fn glEnableVertexAttribArray(index: GLuint);
    pub fn glDisableVertexAttribArray(index: GLuint);

    pub fn glCullFace(mode: GLenum);
    pub fn glFrontFace(mode: GLenum);

    pub fn glUniform1i(location: GLint, v0: GLint);
    pub fn glUniform2fv(location: GLint, count: GLsizei, value: *const GLfloat);
    pub fn glUniform4fv(location: GLint, count: GLsizei, value: *const GLfloat);

    pub fn glBlendFuncSeparate(sc: GLenum, dc: GLenum, sa: GLenum, da: GLenum);

    pub fn glVertexAttribPointer(
        index: GLuint,
        size: GLint,
        _type: GLenum,
        normalized: GLboolean,
        stride: GLsizei,
        pointer: usize);

    pub fn glGenTextures(n: GLsizei, textures: *mut GLuint);
    fn glGenBuffers(n: GLsizei, buffers: *mut GLuint);

    pub fn glDeleteTextures(n: GLsizei, textures: *const GLuint);
    fn glDeleteBuffers(n: GLsizei, buffers: *const GLuint);

    pub fn glPixelStorei(pname: GLenum, param: GLint);

    pub fn glTexSubImage2D(
        target: GLenum, level: GLint,
        xoffset: GLint, yoffset: GLint, width: GLsizei, height: GLsizei,
        format: GLenum, _type: GLenum, pixels: *const u8,
    );

    pub fn glTexParameteri(target: GLenum, pname: GLenum, param: GLint);

    pub fn glTexImage2D(
        target: GLenum, level: GLint, internalformat: GLint,
        width: GLsizei, height: GLsizei,
        border: GLint, format: GLenum, _type: GLenum, pixels: *const u8,
    );

    pub fn glGetError() -> GLenum;
    pub fn glFinish();
    pub fn glGetUniformLocation(a: GLuint, ptr: *const u8) -> GLint;

    pub fn glShaderSource(
        shader: GLuint,
        count: GLsizei,
        string: *const *const GLchar,
        length: *const GLint,
    );

    pub fn glCreateProgram() -> GLuint;
    pub fn glCreateShader(kind: GLenum) -> GLuint;
    pub fn glLinkProgram(p: GLuint);

    pub fn glCompileShader(p: GLuint);

    pub fn glBindAttribLocation(program: GLuint, index: GLuint, name: *const u8);
    pub fn glAttachShader(program: GLuint, shader: GLuint);
}


pub struct Buffer(GLuint);

impl Drop for Buffer {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe { glDeleteBuffers(1, &self.0); }
        }
    }
}

impl Buffer {
    pub fn new() -> Self {
        let mut buf = 0;
        // Create dynamic vertex array
        unsafe {
            glGenBuffers(1, &mut buf);
            glFinish();
        }
        Buffer(buf)
    }

    pub fn bind_and_upload<T: Sized>(&self, data: &[T]) {
        let size = std::mem::size_of::<T>();
        unsafe {
            glBindBuffer(GL_ARRAY_BUFFER, self.0);
            glBufferData(
                GL_ARRAY_BUFFER,
                data.len() * size,
                data.as_ptr() as *const u8,
                GL_STREAM_DRAW,
            );
        }
    }

    pub fn unbind(&self) {
        unsafe {
            glBindBuffer(GL_ARRAY_BUFFER, 0);
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
            glBlendFuncSeparate(
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
        BlendFactor::ZERO                   => GL_ZERO,
        BlendFactor::ONE                    => GL_ONE,

        BlendFactor::SRC_COLOR              => GL_SRC_COLOR,
        BlendFactor::ONE_MINUS_SRC_COLOR    => GL_ONE_MINUS_SRC_COLOR,

        BlendFactor::SRC_ALPHA              => GL_SRC_ALPHA,
        BlendFactor::ONE_MINUS_SRC_ALPHA    => GL_ONE_MINUS_SRC_ALPHA,

        BlendFactor::DST_ALPHA              => GL_DST_ALPHA,
        BlendFactor::ONE_MINUS_DST_ALPHA    => GL_ONE_MINUS_DST_ALPHA,

        BlendFactor::DST_COLOR              => GL_DST_COLOR,
        BlendFactor::ONE_MINUS_DST_COLOR    => GL_ONE_MINUS_DST_COLOR,

        BlendFactor::SRC_ALPHA_SATURATE     => GL_SRC_ALPHA_SATURATE,
    }
}

fn blend_composite_op(op: CompositeState) -> Blend {
    op.into()
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

impl Default for Blend {
    fn default() -> Self {
        Self {
            src_color: GL_ONE,
            dst_color: GL_ONE_MINUS_SRC_ALPHA,
            src_alpha: GL_ONE,
            dst_alpha: GL_ONE_MINUS_SRC_ALPHA,
        }
    }
}
