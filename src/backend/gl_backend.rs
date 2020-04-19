use super::{
    commands::{CallKind, CmdBuffer, RawSlice},
    gl,
    gl::types::{GLint, GLsizei, GLsizeiptr, GLuint},
    gl_shader::Shader,
};
use crate::{
    backend::{Paint, Scissor},
    cache::{Path, Vertex},
};
use std::{mem, ptr::null};

struct Buffer(GLuint);

impl Drop for Buffer {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe { gl::DeleteBuffers(1, &self.0) }
            self.0 = 0;
        }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self(unsafe {
            // Create dynamic vertex array
            let mut buf = 0;
            gl::GenBuffers(1, &mut buf);
            gl::Finish();
            buf
        })
    }
}

impl Buffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub unsafe fn bind_and_upload<T: Sized>(&self, data: &[T]) {
        let size = (data.len() * std::mem::size_of::<T>()) as GLsizeiptr;
        let ptr = data.as_ptr() as *const _;
        gl::BindBuffer(gl::ARRAY_BUFFER, self.0);
        gl::BufferData(gl::ARRAY_BUFFER, size as GLsizeiptr, ptr, gl::STREAM_DRAW);
    }
}

fn check_error(_msg: &str) {
    #[cfg(build = "debug")]
    {
        let err = unsafe { gl::GetError() };
        if err != gl::NO_ERROR {
            println!("GL Error {:08x} after {}", err, _msg);
        }
    }
}

fn gl_draw_strip(slice: RawSlice) {
    if slice.count > 0 {
        let (first, count) = (slice.offset as GLint, slice.count as GLsizei);
        unsafe { gl::DrawArrays(gl::TRIANGLE_STRIP, first, count) }
    }
}

pub struct BackendGL {
    cmd: CmdBuffer,

    shader: Shader,
    view: [f32; 2],

    vert_buf: Buffer,
}

impl BackendGL {
    fn set_uniforms(&self, offset: usize) {
        let uniform = (&self.cmd.uniforms[offset]) as *const _ as *const _;
        unsafe { gl::Uniform4fv(self.shader.loc_frag, 7, uniform) }
        check_error("set_uniforms");
    }
}

impl Default for BackendGL {
    fn default() -> Self {
        check_error("init");
        let shader = Shader::new();
        check_error("shader & uniform locations");

        Self {
            cmd: CmdBuffer::new(),
            shader,
            view: [0f32; 2],
            vert_buf: Buffer::new(),
        }
    }
}

impl super::Backend for BackendGL {
    fn draw_fill(
        &mut self,
        paint: Paint,
        scissor: Scissor,
        fringe: f32,
        bounds: [f32; 4],
        paths: &[Path],
    ) {
        self.cmd.draw_fill(paint, scissor, fringe, bounds, paths);
    }

    fn draw_stroke(
        &mut self,
        paint: Paint,
        scissor: Scissor,
        fringe: f32,
        stroke_width: f32,
        paths: &[Path],
    ) {
        self.cmd
            .draw_stroke(paint, scissor, fringe, stroke_width, paths);
    }

    fn begin_frame(&mut self, width: f32, height: f32, pixel_ratio: f32) {
        self.view = [width / pixel_ratio, height / pixel_ratio];
    }

    fn cancel_frame(&mut self) {
        self.cmd.clear();
    }

    fn end_frame(&mut self) {
        if self.cmd.calls.is_empty() {
            self.cmd.clear();
            return;
        }

        unsafe {
            // Setup require GL state.

            gl::UseProgram(self.shader.prog);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);
            gl::Enable(gl::BLEND);
            gl::Disable(gl::DEPTH_TEST);
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::ActiveTexture(gl::TEXTURE0);

            // upload vertex data
            self.vert_buf.bind_and_upload(&self.cmd.verts);
            let size = mem::size_of::<Vertex>();

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);

            let two = (2 * mem::size_of::<f32>()) as *const _;
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, size as i32, null());
            gl::VertexAttribPointer(1, 2, gl::UNSIGNED_SHORT, gl::TRUE, size as i32, two);

            // set view and texture just once per frame.
            gl::Uniform2fv(self.shader.loc_viewsize, 1, &self.view as *const f32);

            // alpha blending
            let (src, dst) = (gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            gl::BlendFuncSeparate(src, dst, src, dst);

            for call in &self.cmd.calls {
                match call.kind {
                    CallKind::CONVEXFILL => {
                        gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
                        stencil(0, None);
                        self.set_uniforms(call.uniform_offset);
                        for path in &self.cmd.paths[call.path.range()] {
                            gl_draw_strip(path.fill);
                            gl_draw_strip(path.stroke); // fringes
                        }
                    }

                    CallKind::FILL => {
                        let range = call.path.range();

                        // Draw shapes
                        gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
                        gl::Disable(gl::CULL_FACE);
                        self.set_uniforms(call.uniform_offset);
                        stencil(0, Some(FILL_SHAPES));
                        for path in &self.cmd.paths[range.clone()] {
                            gl_draw_strip(path.fill);
                        }

                        // Draw anti-aliased pixels
                        gl::Enable(gl::CULL_FACE);
                        gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);

                        // Draw fringes
                        self.set_uniforms(call.uniform_offset + 1);
                        stencil(0, Some(FILL_FRINGES));
                        for path in &self.cmd.paths[range] {
                            gl_draw_strip(path.stroke);
                        }

                        // Draw fill
                        if call.triangle.count == 4 {
                            stencil(0, Some(FILL_END));
                            gl_draw_strip(call.triangle);
                        }
                    }
                    CallKind::STROKE => {
                        let range = call.path.range();

                        gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);

                        // Fill the stroke base without overlap
                        stencil(0, Some(STROKE_BASE));
                        self.set_uniforms(call.uniform_offset + 1);
                        for path in &self.cmd.paths[range.clone()] {
                            gl_draw_strip(path.stroke);
                        }

                        // Draw anti-aliased pixels.
                        self.set_uniforms(call.uniform_offset);
                        stencil(0, Some(STROKE_AA));
                        for path in &self.cmd.paths[range.clone()] {
                            gl_draw_strip(path.stroke);
                        }

                        // Clear stencil buffer
                        gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
                        stencil(0, Some(STROKE_CLEAR));
                        for path in &self.cmd.paths[range] {
                            gl_draw_strip(path.stroke);
                        }
                    }
                }
            }

            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);
            gl::Disable(gl::CULL_FACE);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::UseProgram(0);
        }

        // Reset calls
        self.cmd.clear();
    }
}

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

macro_rules! stencil_state {
    ($name: ident, $front:expr, $back:expr) => {
        const $name: wgpu::DepthStencilStateDescriptor = stencil_state(FORMAT, $front, $back);
    };
}

const fn stencil_state(
    format: wgpu::TextureFormat,
    stencil_front: wgpu::StencilStateFaceDescriptor,
    stencil_back: wgpu::StencilStateFaceDescriptor,
) -> wgpu::DepthStencilStateDescriptor {
    wgpu::DepthStencilStateDescriptor {
        format,
        depth_write_enabled: false,
        depth_compare: wgpu::CompareFunction::Always,
        stencil_front,
        stencil_back,
        stencil_read_mask: 0xFF,
        stencil_write_mask: 0xFF,
    }
}

stencil_state!(FILL_SHAPES, ALWAYS_KEEP_INCR_WRAP, ALWAYS_KEEP_DECR_WRAP);
stencil_state!(FILL_FRINGES, EQ_KEEP, EQ_KEEP);
stencil_state!(FILL_END, NE_ZERO, NE_ZERO);

stencil_state!(STROKE_BASE, EQ_KEEP_INCR, EQ_KEEP_INCR);
stencil_state!(STROKE_AA, EQ_KEEP, EQ_KEEP);
stencil_state!(STROKE_CLEAR, ALWAYS_ZERO, ALWAYS_ZERO);

macro_rules! stencil_face {
    ($name:ident, $comp:ident, $fail:ident, $pass:ident) => {
        const $name: wgpu::StencilStateFaceDescriptor = wgpu::StencilStateFaceDescriptor {
            compare: wgpu::CompareFunction::$comp,
            fail_op: wgpu::StencilOperation::$fail,
            depth_fail_op: wgpu::StencilOperation::$fail,
            pass_op: wgpu::StencilOperation::$pass,
        };
    };
}

stencil_face!(ALWAYS_ZERO, Always, Zero, Zero);
stencil_face!(NE_ZERO, NotEqual, Zero, Zero);
stencil_face!(EQ_KEEP, Equal, Keep, Keep);
stencil_face!(EQ_KEEP_INCR, Equal, Keep, IncrementClamp);
stencil_face!(ALWAYS_KEEP_INCR_WRAP, Always, Keep, IncrementWrap);
stencil_face!(ALWAYS_KEEP_DECR_WRAP, Always, Keep, DecrementWrap);

unsafe fn stencil(reference: u32, state: Option<wgpu::DepthStencilStateDescriptor>) {
    // ignore: format, depth_write_enabled, depth_compare
    if let Some(state) = state {
        gl::Enable(gl::STENCIL_TEST);

        assert_eq!(state.stencil_write_mask, state.stencil_read_mask);

        let mask = state.stencil_write_mask;
        sep_stencil(gl::FRONT, state.stencil_front, reference, mask);
        sep_stencil(gl::BACK, state.stencil_back, reference, mask);
    } else {
        gl::Disable(gl::STENCIL_TEST);

        gl::StencilMask(0xffff_ffff);
        gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
        gl::StencilFunc(gl::ALWAYS, 0, 0xffff_ffff);
    }
    check_error("stencil");
}

unsafe fn sep_stencil(
    face: gl::types::GLenum,
    state: wgpu::StencilStateFaceDescriptor,
    reference: u32,
    mask: u32,
) {
    let func = match state.compare {
        wgpu::CompareFunction::Undefined => unimplemented!(),
        wgpu::CompareFunction::Never => gl::NEVER,
        wgpu::CompareFunction::Less => gl::LESS,
        wgpu::CompareFunction::Equal => gl::EQUAL,
        wgpu::CompareFunction::LessEqual => gl::LEQUAL,
        wgpu::CompareFunction::Greater => gl::GREATER,
        wgpu::CompareFunction::NotEqual => gl::NOTEQUAL,
        wgpu::CompareFunction::GreaterEqual => gl::GEQUAL,
        wgpu::CompareFunction::Always => gl::ALWAYS,
    };

    let sfail = conv_op(state.fail_op);
    let dpfail = conv_op(state.depth_fail_op);
    let dppass = conv_op(state.pass_op);

    assert_eq!(sfail, dpfail);

    gl::StencilOpSeparate(face, sfail, dpfail, dppass);
    gl::StencilFuncSeparate(face, func, reference as i32, mask);
}

fn conv_op(op: wgpu::StencilOperation) -> gl::types::GLenum {
    match op {
        wgpu::StencilOperation::Keep => gl::KEEP,
        wgpu::StencilOperation::Zero => gl::ZERO,
        wgpu::StencilOperation::Replace => gl::REPLACE,
        wgpu::StencilOperation::Invert => gl::INVERT,
        wgpu::StencilOperation::IncrementClamp => gl::INCR,
        wgpu::StencilOperation::DecrementClamp => gl::DECR,
        wgpu::StencilOperation::IncrementWrap => gl::INCR_WRAP,
        wgpu::StencilOperation::DecrementWrap => gl::DECR_WRAP,
    }
}
