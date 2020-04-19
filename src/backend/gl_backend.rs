use super::{
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

#[inline]
fn copy_verts(dst: &mut [Vertex], slice: RawSlice, src: &[Vertex]) -> u32 {
    (&mut dst[slice.range()]).copy_from_slice(src);
    slice.count
}

fn max_vert_count(paths: &[Path]) -> usize {
    paths.iter().fold(0, |acc, path| {
        let fill = path.fill.as_ref().map(|v| v.len()).unwrap_or_default();
        let stroke = path.stroke.as_ref().map(|v| v.len()).unwrap_or_default();
        acc + fill + stroke
    })
}

const SHADER_FILLGRAD: f32 = 0.0;
const SHADER_SIMPLE: f32 = 2.0;

#[repr(C, align(4))]
struct FragUniforms {
    scissor_mat: [f32; 4],
    paint_mat: [f32; 4],
    inner_color: [f32; 4],
    outer_color: [f32; 4],

    scissor_ext: [f32; 2],
    scissor_scale: [f32; 2],

    extent: [f32; 2],
    radius: f32,
    feather: f32,

    stroke_mul: f32, // scale
    stroke_thr: f32, // threshold
    padding: [u8; 4],
    kind: f32,
}

impl Default for FragUniforms {
    fn default() -> Self {
        Self {
            scissor_mat: [0f32; 4],
            paint_mat: [0f32; 4],
            inner_color: [0f32; 4],
            outer_color: [0f32; 4],

            scissor_ext: [0f32; 2],
            scissor_scale: [0f32; 2],

            extent: [0f32; 2],
            radius: 0f32,
            feather: 0f32,

            stroke_mul: 0f32,
            stroke_thr: 0f32,
            padding: [0u8; 4],
            kind: SHADER_FILLGRAD,
        }
    }
}

impl FragUniforms {
    fn fill(
        paint: &Paint,
        scissor: Scissor,
        width: f32,
        fringe: f32,
        stroke_thr: f32,
    ) -> Self {
        let (scissor_mat, scissor_ext, scissor_scale);
        if scissor.extent[0] < -0.5 || scissor.extent[1] < -0.5 {
            scissor_mat = [0.0; 4];
            scissor_ext = [1.0, 1.0];
            scissor_scale = [1.0, 1.0];
        } else {
            let xform = &scissor.xform;
            let (re, im) = (xform.re, xform.im);
            let scale = (re * re + im * im).sqrt() / fringe;

            scissor_mat = xform.inverse().into();
            scissor_ext = scissor.extent;
            scissor_scale = [scale, scale];
        }

        Self {
            scissor_mat,
            scissor_ext,
            scissor_scale,

            inner_color: paint.inner_color.premul(),
            outer_color: paint.outer_color.premul(),

            extent: paint.extent,

            stroke_mul: (width * 0.5 + fringe * 0.5) / fringe,
            stroke_thr: stroke_thr,
            kind: SHADER_FILLGRAD,
            radius: paint.radius,
            feather: paint.feather,

            paint_mat: paint.xform.inverse().into(),

            padding: [0; 4],
        }
    }
}

#[derive(Clone, Copy, Default)]
struct RawSlice {
    offset: u32,
    count: u32,
}

impl RawSlice {
    #[inline]
    fn new(offset: u32, count: u32) -> Self {
        Self { offset, count }
    }

    #[inline]
    fn range(self) -> std::ops::Range<usize> {
        let (offset, count) = (self.offset as usize, self.count as usize);
        offset..offset + count
    }
}

fn gl_draw_strip(slice: RawSlice) {
    if slice.count > 0 {
        let (first, count) = (slice.offset as GLint, slice.count as GLsizei);
        unsafe { gl::DrawArrays(gl::TRIANGLE_STRIP, first, count) }
    }
}

#[derive(Clone, Copy, Default)]
struct PathGL {
    fill: RawSlice,
    stroke: RawSlice,
}

#[repr(u8)]
#[derive(PartialEq, Eq)]
enum CallKind {
    FILL,
    CONVEXFILL,
    STROKE,
}

struct Call {
    kind: CallKind,
    path: RawSlice,
    triangle: RawSlice,
    uniform_offset: usize,
}

struct VecAlloc<T: Default>(Vec<T>);

impl<T: Default> std::ops::Deref for VecAlloc<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Default> std::ops::DerefMut for VecAlloc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Default> VecAlloc<T> {
    #[inline]
    fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    fn alloc(&mut self, n: usize) -> (usize, &mut [T]) {
        let start = self.0.len();
        self.0.resize_with(start + n, Default::default);
        (start, &mut self.0[start..start + n])
    }
}

pub struct BackendGL {
    // Per frame buffers
    calls: Vec<Call>,
    paths: Vec<PathGL>,
    verts: Vec<Vertex>,
    uniforms: VecAlloc<FragUniforms>,

    shader: Shader,
    view: [f32; 2],

    vert_buf: Buffer,
}

impl Default for BackendGL {
    fn default() -> Self {
        check_error("init");
        let shader = Shader::new();
        check_error("shader & uniform locations");

        Self {
            shader,
            view: [0f32; 2],
            vert_buf: Buffer::new(),

            // Per frame buffers
            calls: Vec::new(),
            paths: Vec::new(),
            verts: Vec::new(),
            uniforms: VecAlloc::new(),
        }
    }
}

impl BackendGL {
    fn alloc_verts(&mut self, n: usize) -> u32 {
        let start = self.verts.len();
        self.verts.resize_with(start + n, Default::default);
        start as u32
    }

    fn alloc_paths(&mut self, count: u32) -> RawSlice {
        let start = self.paths.len();
        self.paths.resize_with(start + count as usize, Default::default);
        RawSlice::new(start as u32, count)
    }

    fn set_uniforms(&self, offset: usize) {
        let uniform = (&self.uniforms[offset]) as *const _ as *const _;
        unsafe { gl::Uniform4fv(self.shader.loc_frag, 7, uniform) }
    }

    fn fill(&self, uniform_offset: usize, path: RawSlice, triangle: RawSlice) {
        let range = path.range();

        // Draw shapes
        unsafe {
            gl::Enable(gl::STENCIL_TEST);
            gl::StencilMask(0xff);
            gl::StencilFunc(gl::ALWAYS, 0, 0xff);
            gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
        }

        // set bindpoint for solid loc
        self.set_uniforms(uniform_offset);
        check_error("fill simple");

        unsafe {
            gl::StencilOpSeparate(gl::FRONT, gl::KEEP, gl::KEEP, gl::INCR_WRAP);
            gl::StencilOpSeparate(gl::BACK, gl::KEEP, gl::KEEP, gl::DECR_WRAP);
            gl::Disable(gl::CULL_FACE);
        }
        for path in &self.paths[range.clone()] {
            gl_draw_strip(path.fill);
        }

        // Draw anti-aliased pixels
        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
        }

        self.set_uniforms(uniform_offset + 1);
        check_error("fill fill");

        unsafe {
            gl::StencilFunc(gl::EQUAL, 0x00, 0xff);
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
        }
        // Draw fringes
        for path in &self.paths[range] {
            gl_draw_strip(path.stroke);
        }

        // Draw fill
        if triangle.count == 4 {
            unsafe {
                gl::StencilFunc(gl::NOTEQUAL, 0x00, 0xff);
                gl::StencilOp(gl::ZERO, gl::ZERO, gl::ZERO);
            }
            gl_draw_strip(RawSlice::new(triangle.offset, 4));
        }

        unsafe {
            gl::Disable(gl::STENCIL_TEST);
        }
    }

    fn stroke(&self, uniform_offset: usize, path: RawSlice) {
        let range = path.range();

        unsafe {
            gl::Enable(gl::STENCIL_TEST);
            gl::StencilMask(0xff);
        }

        // Fill the stroke base without overlap
        unsafe {
            gl::StencilFunc(gl::EQUAL, 0x0, 0xff);
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::INCR);
        }
        self.set_uniforms(uniform_offset + 1);
        check_error("stroke fill 0");
        for path in &self.paths[range.clone()] {
            gl_draw_strip(path.stroke);
        }

        // Draw anti-aliased pixels.
        self.set_uniforms(uniform_offset);
        unsafe {
            gl::StencilFunc(gl::EQUAL, 0x00, 0xff);
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
        }
        for path in &self.paths[range.clone()] {
            gl_draw_strip(path.stroke);
        }

        // Clear stencil buffer.
        unsafe {
            gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
            gl::StencilFunc(gl::ALWAYS, 0x0, 0xff);
            gl::StencilOp(gl::ZERO, gl::ZERO, gl::ZERO);
        }
        check_error("stroke fill 1");
        for path in &self.paths[range] {
            gl_draw_strip(path.stroke);
        }

        unsafe {
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::Disable(gl::STENCIL_TEST);
        }
    }
}

impl super::Backend for BackendGL {
    fn cancel_frame(&mut self) {
        self.verts.clear();
        self.paths.clear();
        self.calls.clear();
        self.uniforms.clear();
    }

    fn draw_fill(
        &mut self,
        paint: Paint,
        scissor: Scissor,
        fringe: f32,
        bounds: [f32; 4],
        paths: &[Path],
    ) {
        let path = self.alloc_paths(paths.len() as u32);

        let (kind, triangle_count) = if paths.len() == 1 && paths[0].convex {
            (CallKind::CONVEXFILL, 0u32) // Bounding box fill quad not needed for convex fill
        } else {
            (CallKind::FILL, 4u32)
        };

        // Allocate vertices for all the paths.
        let maxverts = max_vert_count(paths) + triangle_count as usize;
        let mut offset = self.alloc_verts(maxverts);

        for (i, src) in paths.iter().enumerate() {
            let dst = &mut self.paths[i + path.offset as usize];
            if let Some(fill) = &src.fill {
                dst.fill = RawSlice::new(offset, fill.len() as u32);
                offset += copy_verts(&mut self.verts, dst.fill, fill);
            }
            if let Some(stroke) = &src.stroke {
                dst.stroke = RawSlice::new(offset, stroke.len() as u32);
                offset += copy_verts(&mut self.verts, dst.stroke, stroke);
            }
        }

        // Setup uniforms for draw calls
        let (triangle_offset, uniform_offset) = if kind == CallKind::FILL {
            // Quad
            let quad = &mut self.verts[offset as usize..offset as usize + 4];
            quad[0].set([bounds[2], bounds[3]], [0.5, 1.0]);
            quad[1].set([bounds[2], bounds[1]], [0.5, 1.0]);
            quad[2].set([bounds[0], bounds[3]], [0.5, 1.0]);
            quad[3].set([bounds[0], bounds[1]], [0.5, 1.0]);

            let (uniform_offset, ab) = self.uniforms.alloc(2);

            // Simple shader for stencil
            ab[0].stroke_thr = -1.0;
            ab[0].kind = SHADER_SIMPLE;

            // Fill shader
            ab[1] = FragUniforms::fill(&paint, scissor, fringe, fringe, -1.0);
            (offset, uniform_offset)
        } else {
            // Fill shader
            let (uniform_offset, a) = self.uniforms.alloc(1);
            a[0] = FragUniforms::fill(&paint, scissor, fringe, fringe, -1.0);
            (0, uniform_offset)
        };

        self.calls.push(Call {
            kind,
            uniform_offset,
            triangle: RawSlice::new(triangle_offset, triangle_count),
            path,
        })
    }

    fn draw_stroke(
        &mut self,
        paint: Paint,
        scissor: Scissor,
        fringe: f32,
        stroke_width: f32,
        paths: &[Path],
    ) {
        let path = self.alloc_paths(paths.len() as u32);

        // Allocate vertices for all the paths.
        let maxverts = max_vert_count(paths);
        let mut offset = self.alloc_verts(maxverts);

        for (i, src) in paths.iter().enumerate() {
            let dst = &mut self.paths[i + path.offset as usize];
            if let Some(stroke) = &src.stroke {
                dst.stroke = RawSlice::new(offset, stroke.len() as u32);
                offset += copy_verts(&mut self.verts, dst.stroke, stroke);
            }
        }

        // Fill shader
        let thr = 1.0 - 0.5 / 255.0;
        let (uniform_offset, ab) = self.uniforms.alloc(2);
        ab[0] = FragUniforms::fill(&paint, scissor, stroke_width, fringe, -1.0);
        ab[1] = FragUniforms::fill(&paint, scissor, stroke_width, fringe, thr);

        self.calls.push(Call {
            kind: CallKind::STROKE,
            uniform_offset,
            triangle: RawSlice::new(0, 0),
            path,
        })
    }

    fn begin_frame(&mut self, width: f32, height: f32, pixel_ratio: f32) {
        self.view = [width / pixel_ratio, height / pixel_ratio];
    }

    fn end_frame(&mut self) {
        if self.calls.is_empty() {
            self.cancel_frame();
            return;
        }

        unsafe {
            gl::UseProgram(self.shader.prog);

            // Setup require GL state.

            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);
            gl::Enable(gl::BLEND);
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::SCISSOR_TEST);
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::StencilMask(0xffff_ffff);
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
            gl::StencilFunc(gl::ALWAYS, 0, 0xffff_ffff);
            gl::ActiveTexture(gl::TEXTURE0);

            // upload vertex data
            self.vert_buf.bind_and_upload(&self.verts);
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

            for call in &self.calls {
                match call.kind {
                    CallKind::FILL => self.fill(call.uniform_offset, call.path, call.triangle),
                    CallKind::CONVEXFILL => {
                        self.set_uniforms(call.uniform_offset);
                        for path in &self.paths[call.path.range()] {
                            gl_draw_strip(path.fill);
                            gl_draw_strip(path.stroke); // fringes
                        }
                    }
                    CallKind::STROKE => self.stroke(call.uniform_offset, call.path),
                }
            }

            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);
            gl::Disable(gl::CULL_FACE);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::UseProgram(0);
        }

        // Reset calls
        self.cancel_frame();
    }
}
