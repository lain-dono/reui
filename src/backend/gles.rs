use super::commands::{CallKind, CmdBuffer, RawSlice};
use crate::cache::Vertex;
use std::{mem, ptr};

use self::gles::types::{GLint, GLsizei, GLsizeiptr, GLuint};

#[allow(clippy::module_inception)]
pub mod gles {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

// TODO: mediump float may not be enough for GLES2 in iOS.
// see the following discussion: https://github.com/memononen/nanovg/issues/46
static VERT: &[u8] = b"
uniform vec2 viewSize;

attribute vec2 a_Position;
attribute vec2 a_TexCoord;

varying vec2 v_Position;
varying vec2 v_TexCoord;

void main() {
    v_TexCoord = a_TexCoord;
    v_Position = a_Position;
    gl_Position = vec4(
        2.0 * a_Position.x / viewSize.x - 1.0,
        1.0 - 2.0 * a_Position.y / viewSize.y,
        0.0, 1.0);
}
\0";

static FRAG: &[u8] = b"
#define UNIFORMARRAY_SIZE 7

//precision highp float;

varying vec2 v_Position;
varying vec2 v_TexCoord;

uniform vec4 frag[UNIFORMARRAY_SIZE];

#define scissorTransform frag[0]
#define paintTransform frag[1]

#define innerCol frag[2]
#define outerCol frag[3]
#define scissorExt frag[4].xy
#define scissorScale frag[4].zw
#define extent frag[5].xy
#define radius frag[5].z
#define feather frag[5].w
#define strokeMult frag[6].x
#define strokeThr frag[6].y

#define type int(frag[6].w)

float sdroundrect(vec2 pt, vec2 ext, float rad) {
    vec2 ext2 = ext - vec2(rad,rad);
    vec2 d = abs(pt) - ext2;
    return min(max(d.x,d.y),0.0) + length(max(d,0.0)) - rad;
}

vec2 applyTransform(vec4 transform, vec2 pt) {
    float re = transform.x;
    float im = transform.y;
    return transform.zw + vec2(pt.x * re - pt.y * im, pt.x * im + pt.y * re);
}

// Scissoring
float scissorMask(vec2 p) {
    vec2 sc = vec2(0.5,0.5) -
        (abs(applyTransform(scissorTransform, p)) - scissorExt) * scissorScale;
    return clamp(sc.x,0.0,1.0) * clamp(sc.y,0.0,1.0);
}

// Stroke - from [0..1] to clipped pyramid, where the slope is 1px.
float strokeMask() {
    return min(1.0, (1.0-abs(v_TexCoord.x*2.0-1.0))*strokeMult) * min(1.0, v_TexCoord.y);
}

void main(void) {
    float scissor = scissorMask(v_Position);

    float strokeAlpha = strokeMask();
    if (strokeAlpha < strokeThr) {
        discard;
    }

    vec4 result;
    if (type == 1) {            // Gradient
        // Calculate gradient color using box gradient
        vec2 pt = applyTransform(paintTransform, v_Position);
        float d = clamp((sdroundrect(pt, extent, radius) + feather*0.5) / feather, 0.0, 1.0);
        vec4 color = mix(innerCol,outerCol,d);
        // Combine alpha
        color *= strokeAlpha * scissor;
        result = color;
    } else if (type == 0) {        // Stencil fill
        result = vec4(1,1,1,1);
    }

    gl_FragColor = result;
}\0";

fn check_error(_msg: &str) {
    #[cfg(build = "debug")]
    {
        let err = unsafe { gles::GetError() };
        if err != gles::NO_ERROR {
            println!("GL Error {:08x} after {}", err, _msg);
        }
    }
}

pub struct BackendGL {
    vert_buf: GLuint,
    prog: GLuint,
    loc_viewsize: GLint,
    loc_frag: GLint,
}

impl Drop for BackendGL {
    fn drop(&mut self) {
        if self.vert_buf != 0 {
            unsafe { gles::DeleteBuffers(1, &self.vert_buf) }
            self.vert_buf = 0;
        }
    }
}

impl Default for BackendGL {
    fn default() -> Self {
        check_error("init");

        let (vshader, fshader) = (VERT.as_ptr() as *const i8, FRAG.as_ptr() as *const i8);

        unsafe {
            let prog = gles::CreateProgram();
            let vert = gles::CreateShader(gles::VERTEX_SHADER);
            let frag = gles::CreateShader(gles::FRAGMENT_SHADER);
            gles::ShaderSource(vert, 1, &vshader, ptr::null());
            gles::ShaderSource(frag, 1, &fshader, ptr::null());

            gles::CompileShader(vert);
            let mut status = 0i32;
            gles::GetShaderiv(vert, gles::COMPILE_STATUS, &mut status);
            assert_eq!(status, 1);

            gles::CompileShader(frag);
            gles::GetShaderiv(frag, gles::COMPILE_STATUS, &mut status);
            assert_eq!(status, 1);

            gles::AttachShader(prog, vert);
            gles::AttachShader(prog, frag);

            gles::BindAttribLocation(prog, 0, b"a_Position\0".as_ptr() as *const i8);
            gles::BindAttribLocation(prog, 1, b"a_TexCoord\0".as_ptr() as *const i8);

            gles::LinkProgram(prog);
            gles::GetProgramiv(prog, gles::LINK_STATUS, &mut status);
            assert_eq!(status, 1);

            check_error("shader & uniform locations");

            // Create dynamic vertex array
            let mut vert_buf = 0;
            gles::GenBuffers(1, &mut vert_buf);
            gles::Finish();

            Self {
                vert_buf,
                prog,
                loc_viewsize: gles::GetUniformLocation(prog, b"viewSize\0".as_ptr() as *const i8),
                loc_frag: gles::GetUniformLocation(prog, b"frag\0".as_ptr() as *const i8),
            }
        }
    }
}

impl BackendGL {
    fn set_uniforms(&self, cmd: &CmdBuffer, offset: usize) {
        let uniform = (&cmd.uniforms[offset]) as *const _ as *const _;
        unsafe { gles::Uniform4fv(self.loc_frag, 7, uniform) }
        check_error("set_uniforms");
    }

    pub fn draw_commands(&mut self, cmd: &CmdBuffer, width: f32, height: f32, pixel_ratio: f32) {
        if cmd.calls.is_empty() {
            return;
        }

        unsafe fn gl_draw_strip(slice: RawSlice) {
            if slice.count > 0 {
                let (first, count) = (slice.offset as GLint, slice.count as GLsizei);
                gles::DrawArrays(gles::TRIANGLE_STRIP, first, count)
            }
        }

        let view = [width / pixel_ratio, height / pixel_ratio];

        unsafe {
            // Setup require GL state.

            gles::UseProgram(self.prog);
            gles::Enable(gles::CULL_FACE);
            gles::CullFace(gles::BACK);
            gles::FrontFace(gles::CCW);
            gles::Enable(gles::BLEND);
            gles::Disable(gles::DEPTH_TEST);
            gles::ColorMask(gles::TRUE, gles::TRUE, gles::TRUE, gles::TRUE);
            gles::ActiveTexture(gles::TEXTURE0);

            // upload vertex data

            {
                let size = (cmd.verts.len() * mem::size_of::<Vertex>()) as GLsizeiptr;
                let ptr = cmd.verts.as_ptr() as *const _;
                gles::BindBuffer(gles::ARRAY_BUFFER, self.vert_buf);
                gles::BufferData(
                    gles::ARRAY_BUFFER,
                    size as GLsizeiptr,
                    ptr,
                    gles::STREAM_DRAW,
                );
            }

            let size = mem::size_of::<Vertex>();

            gles::EnableVertexAttribArray(0);
            gles::EnableVertexAttribArray(1);

            let two = (2 * mem::size_of::<f32>()) as *const _;
            gles::VertexAttribPointer(0, 2, gles::FLOAT, gles::FALSE, size as i32, ptr::null());
            gles::VertexAttribPointer(1, 2, gles::UNSIGNED_SHORT, gles::TRUE, size as i32, two);

            // set view and texture just once per frame.
            gles::Uniform2fv(self.loc_viewsize, 1, &view as *const f32);

            // alpha blending
            let (src, dst) = (gles::ONE, gles::ONE_MINUS_SRC_ALPHA);
            gles::BlendFuncSeparate(src, dst, src, dst);

            for call in &cmd.calls {
                match call.kind {
                    CallKind::CONVEXFILL => {
                        gles::ColorMask(gles::TRUE, gles::TRUE, gles::TRUE, gles::TRUE);
                        stencil(0, None);
                        self.set_uniforms(cmd, call.uniform_offset);
                        for path in &cmd.paths[call.path.range()] {
                            gl_draw_strip(path.fill);
                            gl_draw_strip(path.stroke); // fringes
                        }
                    }

                    CallKind::FILL => {
                        let range = call.path.range();

                        // Draw shapes
                        gles::ColorMask(gles::FALSE, gles::FALSE, gles::FALSE, gles::FALSE);
                        gles::Disable(gles::CULL_FACE);
                        self.set_uniforms(cmd, call.uniform_offset);
                        stencil(0, Some(FILL_SHAPES));
                        for path in &cmd.paths[range.clone()] {
                            gl_draw_strip(path.fill);
                        }
                        gles::Enable(gles::CULL_FACE);

                        // Draw anti-aliased pixels
                        gles::ColorMask(gles::TRUE, gles::TRUE, gles::TRUE, gles::TRUE);

                        // Draw fringes
                        self.set_uniforms(cmd, call.uniform_offset + 1);
                        stencil(0, Some(FILL_FRINGES));
                        for path in &cmd.paths[range] {
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

                        gles::ColorMask(gles::TRUE, gles::TRUE, gles::TRUE, gles::TRUE);

                        // Fill the stroke base without overlap
                        stencil(0, Some(STROKE_BASE));
                        self.set_uniforms(cmd, call.uniform_offset + 1);
                        for path in &cmd.paths[range.clone()] {
                            gl_draw_strip(path.stroke);
                        }

                        // Draw anti-aliased pixels.
                        self.set_uniforms(cmd, call.uniform_offset);
                        stencil(0, Some(STROKE_AA));
                        for path in &cmd.paths[range.clone()] {
                            gl_draw_strip(path.stroke);
                        }

                        // Clear stencil buffer
                        gles::ColorMask(gles::FALSE, gles::FALSE, gles::FALSE, gles::FALSE);
                        stencil(0, Some(STROKE_CLEAR));
                        for path in &cmd.paths[range] {
                            gl_draw_strip(path.stroke);
                        }
                    }
                }
            }

            gles::DisableVertexAttribArray(0);
            gles::DisableVertexAttribArray(1);
            gles::Disable(gles::CULL_FACE);
            gles::BindBuffer(gles::ARRAY_BUFFER, 0);
            gles::UseProgram(0);
        }
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
        gles::Enable(gles::STENCIL_TEST);

        assert_eq!(state.stencil_write_mask, state.stencil_read_mask);

        let mask = state.stencil_write_mask;
        sep_stencil(gles::FRONT, state.stencil_front, reference, mask);
        sep_stencil(gles::BACK, state.stencil_back, reference, mask);
    } else {
        gles::Disable(gles::STENCIL_TEST);

        gles::StencilMask(0xffff_ffff);
        gles::StencilOp(gles::KEEP, gles::KEEP, gles::KEEP);
        gles::StencilFunc(gles::ALWAYS, 0, 0xffff_ffff);
    }
    check_error("stencil");
}

unsafe fn sep_stencil(
    face: gles::types::GLenum,
    state: wgpu::StencilStateFaceDescriptor,
    reference: u32,
    mask: u32,
) {
    let func = match state.compare {
        wgpu::CompareFunction::Undefined => unimplemented!(),
        wgpu::CompareFunction::Never => gles::NEVER,
        wgpu::CompareFunction::Less => gles::LESS,
        wgpu::CompareFunction::Equal => gles::EQUAL,
        wgpu::CompareFunction::LessEqual => gles::LEQUAL,
        wgpu::CompareFunction::Greater => gles::GREATER,
        wgpu::CompareFunction::NotEqual => gles::NOTEQUAL,
        wgpu::CompareFunction::GreaterEqual => gles::GEQUAL,
        wgpu::CompareFunction::Always => gles::ALWAYS,
    };

    let sfail = conv_op(state.fail_op);
    let dpfail = conv_op(state.depth_fail_op);
    let dppass = conv_op(state.pass_op);

    assert_eq!(sfail, dpfail);

    gles::StencilOpSeparate(face, sfail, dpfail, dppass);
    gles::StencilFuncSeparate(face, func, reference as i32, mask);
}

fn conv_op(op: wgpu::StencilOperation) -> gles::types::GLenum {
    match op {
        wgpu::StencilOperation::Keep => gles::KEEP,
        wgpu::StencilOperation::Zero => gles::ZERO,
        wgpu::StencilOperation::Replace => gles::REPLACE,
        wgpu::StencilOperation::Invert => gles::INVERT,
        wgpu::StencilOperation::IncrementClamp => gles::INCR,
        wgpu::StencilOperation::DecrementClamp => gles::DECR,
        wgpu::StencilOperation::IncrementWrap => gles::INCR_WRAP,
        wgpu::StencilOperation::DecrementWrap => gles::DECR_WRAP,
    }
}
