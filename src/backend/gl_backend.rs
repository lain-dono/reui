use std::{mem, ptr::null, slice::from_raw_parts_mut};

use crate::{
    cache::{Path, Vertex},
    math::Transform,
    vg::{Paint, Scissor},
};

use super::{
    gl::{
        self,
        types::{GLint, GLsizei, GLsizeiptr, GLuint},
    },
    gl_shader::Shader,
};

fn gl_draw_strip(offset: usize, count: usize) {
    unsafe {
        gl::DrawArrays(gl::TRIANGLE_STRIP, offset as GLint, count as GLsizei);
    }
}

struct Buffer(GLuint);

impl Drop for Buffer {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe {
                gl::DeleteBuffers(1, &self.0);
            }
            self.0 = 0;
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
        let size = (data.len() * std::mem::size_of::<T>()) as GLsizeiptr;
        let ptr = data.as_ptr() as *const libc::c_void;
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.0);
            gl::BufferData(gl::ARRAY_BUFFER, size as GLsizeiptr, ptr, gl::STREAM_DRAW);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}

fn check_error(msg: &str) {
    if true {
        let err = unsafe { gl::GetError() };
        if err != gl::NO_ERROR {
            log::debug!("GL Error {:08x} after {}", err, msg);
        }
    }
}

#[inline]
fn xform2mat3(t: Transform) -> [f32; 4] {
    [t.re, t.im, t.tx, t.ty]
}

fn copy_verts(dst: &mut [Vertex], offset: usize, count: usize, src: &[Vertex]) {
    (&mut dst[offset..offset + count]).copy_from_slice(src);
}

fn max_vert_count(paths: &[Path]) -> usize {
    paths.iter().fold(0, |acc, path| {
        let fill = path.fill.as_ref().map(|v| v.len()).unwrap_or_default();
        let stroke = path.stroke.as_ref().map(|v| v.len()).unwrap_or_default();
        acc + fill + stroke
    })
}

const SHADER_FILLGRAD: f32 = 0.0;
//const SHADER_FILLIMG: f32 = 1.0;
const SHADER_SIMPLE: f32 = 2.0;
//const SHADER_IMG: f32 = 3.0;

#[repr(C, align(4))]
struct FragUniforms {
    array: [f32; 7 * 4 + 1],
}

impl Default for FragUniforms {
    fn default() -> Self {
        Self {
            array: [0.0; 7 * 4 + 1],
        }
    }
}

impl FragUniforms {
    fn set_paint_mat(&mut self, t: Transform) {
        self.array[4..8].copy_from_slice(&[t.re, t.im, t.tx, t.ty]);
    }

    fn set_inner_col(&mut self, color: [f32; 4]) {
        self.array[8..12].copy_from_slice(&color)
    }

    fn set_outer_col(&mut self, color: [f32; 4]) {
        self.array[12..16].copy_from_slice(&color)
    }

    fn set_scissor(&mut self, mat: [f32; 4], ext: [f32; 2], scale: [f32; 2]) {
        self.array[0..4].copy_from_slice(&mat);
        self.array[16..18].copy_from_slice(&ext);
        self.array[18..20].copy_from_slice(&scale);
    }

    fn set_extent(&mut self, ext: [f32; 2]) {
        self.array[20..22].copy_from_slice(&ext);
    }

    fn set_radius(&mut self, radius: f32) {
        self.array[22] = radius;
    }
    fn set_feather(&mut self, feather: f32) {
        self.array[23] = feather;
    }
    fn set_stroke_mul(&mut self, mul: f32) {
        self.array[24] = mul;
    }
    fn set_stroke_thr(&mut self, thr: f32) {
        self.array[25] = thr;
    }
    /*
    fn set_tex_type(&mut self, t: f32) {
        self.array[26] = t;
    }
    */
    fn set_type(&mut self, t: f32) {
        self.array[27] = t;
    }
}

#[derive(Clone, Copy, Default)]
struct PathGL {
    fill_offset: usize,
    fill_count: usize,
    stroke_offset: usize,
    stroke_count: usize,
}

#[repr(u8)]
#[derive(PartialEq, Eq)]
enum CallKind {
    FILL,
    CONVEXFILL,
    STROKE,
}

#[derive(Clone, Copy)]
struct DrawCallData {
    uniform_offset: usize,
}

impl Default for DrawCallData {
    fn default() -> Self {
        Self { uniform_offset: 0 }
    }
}

struct Call {
    kind: CallKind,

    path_offset: usize,
    path_count: usize,

    triangle_offset: usize,
    triangle_count: usize,

    data: DrawCallData,
}

pub struct BackendGL {
    // Per frame buffers
    calls: Vec<Call>,
    paths: Vec<PathGL>,
    verts: Vec<Vertex>,
    uniforms: Vec<FragUniforms>,

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
            uniforms: Vec::new(),
        }
    }
}

impl BackendGL {
    fn alloc_verts(&mut self, n: usize) -> usize {
        let start = self.verts.len();
        self.verts.resize_with(start + n, Default::default);
        start
    }
    fn alloc_paths(&mut self, n: usize) -> usize {
        let start = self.paths.len();
        self.paths.resize_with(start + n, Default::default);
        start
    }

    fn alloc_frag_uniforms(&mut self, n: usize) -> usize {
        let start = self.uniforms.len();
        self.uniforms.resize_with(start + n, Default::default);
        start
    }

    fn set_uniforms(&self, offset: usize) {
        let frag = self.frag_uniform(offset);
        self.shader.bind_frag(&frag.array);
    }

    fn frag_uniform(&self, idx: usize) -> &FragUniforms {
        &self.uniforms[idx]
    }

    fn frag_uniform_mut<'a>(&mut self, idx: usize) -> &'a mut FragUniforms {
        unsafe { &mut from_raw_parts_mut(self.uniforms.as_mut_ptr(), self.uniforms.len())[idx] }
    }

    fn convert_paint(
        &mut self,
        frag: &mut FragUniforms,
        paint: &Paint,
        scissor: &Scissor,
        width: f32,
        fringe: f32,
        stroke_thr: f32,
    ) -> bool {
        *frag = Default::default();

        frag.set_inner_col(paint.inner_color.premul());
        frag.set_outer_col(paint.outer_color.premul());

        if scissor.extent[0] < -0.5 || scissor.extent[1] < -0.5 {
            frag.set_scissor([0.0; 4], [1.0, 1.0], [1.0, 1.0]);
        } else {
            let xform = &scissor.xform;
            let (re, im) = (xform.re, xform.im);
            let scale = (re * re + im * im).sqrt() / fringe;
            frag.set_scissor(xform2mat3(xform.inverse()), scissor.extent, [scale, scale]);
        }

        frag.set_extent(paint.extent);

        frag.set_stroke_mul((width * 0.5 + fringe * 0.5) / fringe);
        frag.set_stroke_thr(stroke_thr);
        frag.set_type(SHADER_FILLGRAD);
        frag.set_radius(paint.radius);
        frag.set_feather(paint.feather);

        frag.set_paint_mat(paint.xform.inverse());

        true
    }

    fn convex_fill(&self, data: DrawCallData, path_offset: usize, path_count: usize) {
        let start = path_offset as usize;
        let end = start + path_count as usize;

        self.set_uniforms(data.uniform_offset);
        check_error("convex fill");

        for path in &self.paths[start..end] {
            gl_draw_strip(path.fill_offset, path.fill_count);
            // Draw fringes
            if path.stroke_count > 0 {
                gl_draw_strip(path.stroke_offset, path.stroke_count);
            }
        }
    }

    fn fill(
        &self,
        data: DrawCallData,
        path_offset: usize,
        path_count: usize,
        triangle_offset: usize,
        triangle_count: usize,
    ) {
        let start = path_offset as usize;
        let end = start + path_count as usize;

        // Draw shapes
        unsafe {
            gl::Enable(gl::STENCIL_TEST);
            gl::StencilMask(0xff);
            gl::StencilFunc(gl::ALWAYS, 0, 0xff);
            gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
        }

        // set bindpoint for solid loc
        self.set_uniforms(data.uniform_offset);
        check_error("fill simple");

        unsafe {
            gl::StencilOpSeparate(gl::FRONT, gl::KEEP, gl::KEEP, gl::INCR_WRAP);
            gl::StencilOpSeparate(gl::BACK, gl::KEEP, gl::KEEP, gl::DECR_WRAP);
            gl::Disable(gl::CULL_FACE);
        }
        for path in &self.paths[start..end] {
            gl_draw_strip(path.fill_offset, path.fill_count);
        }

        // Draw anti-aliased pixels
        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
        }

        self.set_uniforms(data.uniform_offset + 1);
        check_error("fill fill");

        unsafe {
            gl::StencilFunc(gl::EQUAL, 0x00, 0xff);
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
        }
        // Draw fringes
        for path in &self.paths[start..end] {
            gl_draw_strip(path.stroke_offset, path.stroke_count);
        }

        // Draw fill
        if triangle_count == 4 {
            unsafe {
                gl::StencilFunc(gl::NOTEQUAL, 0x00, 0xff);
                gl::StencilOp(gl::ZERO, gl::ZERO, gl::ZERO);
            }
            gl_draw_strip(triangle_offset, 4);
        }

        unsafe {
            gl::Disable(gl::STENCIL_TEST);
        }
    }

    fn stroke(&self, data: DrawCallData, path_offset: usize, path_count: usize) {
        let start = path_offset as usize;
        let end = start + path_count as usize;

        unsafe {
            gl::Enable(gl::STENCIL_TEST);
            gl::StencilMask(0xff);
        }

        // Fill the stroke base without overlap
        unsafe {
            gl::StencilFunc(gl::EQUAL, 0x0, 0xff);
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::INCR);
        }
        self.set_uniforms(data.uniform_offset + 1);
        check_error("stroke fill 0");
        for path in &self.paths[start..end] {
            gl_draw_strip(path.stroke_offset, path.stroke_count);
        }

        // Draw anti-aliased pixels.
        self.set_uniforms(data.uniform_offset);
        unsafe {
            gl::StencilFunc(gl::EQUAL, 0x00, 0xff);
            gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
        }
        for path in &self.paths[start..end] {
            gl_draw_strip(path.stroke_offset, path.stroke_count);
        }

        // Clear stencil buffer.
        unsafe {
            gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
            gl::StencilFunc(gl::ALWAYS, 0x0, 0xff);
            gl::StencilOp(gl::ZERO, gl::ZERO, gl::ZERO);
        }
        check_error("stroke fill 1");
        for path in &self.paths[start..end] {
            gl_draw_strip(path.stroke_offset, path.stroke_count);
        }

        unsafe {
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::Disable(gl::STENCIL_TEST);
        }

        //convertPaint(
        //  gl,
        //  self.frag_uniform_mut(gl, uniform_offset + 1), paint,
        // scissor, strokeWidth, fringe, 1.0 - 0.5/255.0);
    }
}

impl super::Backend for BackendGL {
    fn reset(&mut self) {
        self.verts.clear();
        self.paths.clear();
        self.calls.clear();
        self.uniforms.clear();
    }

    fn draw_fill(
        &mut self,
        paint: &Paint,
        scissor: &Scissor,
        fringe: f32,
        bounds: &[f32; 4],
        paths: &[Path],
    ) {
        let path_offset = self.alloc_paths(paths.len());
        let path_count = paths.len();

        let (kind, triangle_count) = if paths.len() == 1 && paths[0].convex {
            (CallKind::CONVEXFILL, 0) // Bounding box fill quad not needed for convex fill
        } else {
            (CallKind::FILL, 4)
        };

        // Allocate vertices for all the paths.
        let maxverts = max_vert_count(paths) + triangle_count;
        let mut offset = self.alloc_verts(maxverts);

        for (i, path) in paths.iter().enumerate() {
            let copy = &mut self.paths[i + path_offset];
            *copy = Default::default();
            if let Some(fill) = path.fill.as_ref() {
                copy.fill_offset = offset;
                copy.fill_count = fill.len();
                copy_verts(&mut self.verts, offset, fill.len(), fill);
                offset += fill.len();
            }
            if let Some(stroke) = path.stroke.as_ref() {
                copy.stroke_offset = offset;
                copy.stroke_count = stroke.len();
                copy_verts(&mut self.verts, offset, stroke.len(), stroke);
                offset += stroke.len();
            }
        }

        // Setup uniforms for draw calls
        let triangle_offset;
        let uniform_offset;
        let a = if kind == CallKind::FILL {
            // Quad
            triangle_offset = offset;
            let quad = &mut self.verts[triangle_offset..];
            quad[0].set([bounds[2], bounds[3]], [0.5, 1.0]);
            quad[1].set([bounds[2], bounds[1]], [0.5, 1.0]);
            quad[2].set([bounds[0], bounds[3]], [0.5, 1.0]);
            quad[3].set([bounds[0], bounds[1]], [0.5, 1.0]);

            uniform_offset = self.alloc_frag_uniforms(2);

            // Simple shader for stencil
            let frag = self.frag_uniform_mut(uniform_offset);
            *frag = Default::default();
            frag.set_stroke_thr(-1.0);
            frag.set_type(SHADER_SIMPLE);

            // Fill shader
            self.frag_uniform_mut(uniform_offset + 1)
        } else {
            triangle_offset = 0;
            uniform_offset = self.alloc_frag_uniforms(1);
            // Fill shader
            self.frag_uniform_mut(uniform_offset)
        };

        self.convert_paint(a, paint, scissor, fringe, fringe, -1.0);

        self.calls.push(Call {
            kind,
            data: DrawCallData { uniform_offset },
            triangle_offset,
            triangle_count,
            path_offset,
            path_count,
        })
    }

    fn draw_stroke(
        &mut self,
        paint: &Paint,
        scissor: &Scissor,
        fringe: f32,
        stroke_width: f32,
        paths: &[Path],
    ) {
        let path_offset = self.alloc_paths(paths.len());
        let path_count = paths.len();

        // Allocate vertices for all the paths.
        let maxverts = max_vert_count(paths);
        let mut offset = self.alloc_verts(maxverts);

        for (i, path) in paths.iter().enumerate() {
            let copy = &mut self.paths[i + path_offset];
            *copy = Default::default();

            if let Some(stroke) = path.stroke.as_ref() {
                copy.stroke_offset = offset;
                copy.stroke_count = stroke.len();

                copy_verts(&mut self.verts, offset, stroke.len(), stroke);
                offset += stroke.len();
            }
        }

        // Fill shader
        let uniform_offset = self.alloc_frag_uniforms(2);

        let a = self.frag_uniform_mut(uniform_offset);
        let b = self.frag_uniform_mut(uniform_offset + 1);

        self.convert_paint(a, paint, scissor, stroke_width, fringe, -1.0);
        self.convert_paint(b, paint, scissor, stroke_width, fringe, 1.0 - 0.5 / 255.0);

        self.calls.push(Call {
            kind: CallKind::STROKE,
            data: DrawCallData { uniform_offset },
            triangle_offset: 0,
            triangle_count: 0,
            path_offset,
            path_count,
        })
    }

    fn set_viewport(&mut self, width: f32, height: f32, pixel_ratio: f32) {
        self.view = [width / pixel_ratio, height / pixel_ratio];
    }

    fn flush(&mut self) {
        if self.calls.is_empty() {
            self.reset();
            return;
        }

        self.shader.bind();

        unsafe {
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
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        // Upload vertex data
        self.vert_buf.bind_and_upload(&self.verts);
        let size = mem::size_of::<Vertex>();

        unsafe {
            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);

            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, size as i32, null());
            gl::VertexAttribPointer(
                1,
                2,
                gl::UNSIGNED_SHORT,
                gl::TRUE,
                size as i32,
                (2 * mem::size_of::<f32>()) as *const libc::c_void,
            );
        }

        // Set view and texture just once per frame.
        self.shader.bind_view(&self.view);

        unsafe {
            gl::BlendFuncSeparate(
                gl::ONE,
                gl::ONE_MINUS_SRC_ALPHA,
                gl::ONE,
                gl::ONE_MINUS_SRC_ALPHA,
            );
        }

        for call in &self.calls {
            match call.kind {
                CallKind::FILL => self.fill(
                    call.data,
                    call.path_offset,
                    call.path_count,
                    call.triangle_offset,
                    call.triangle_count,
                ),
                CallKind::CONVEXFILL => {
                    self.convex_fill(call.data, call.path_offset, call.path_count)
                }
                CallKind::STROKE => self.stroke(call.data, call.path_offset, call.path_count),
            }
        }

        unsafe {
            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);

            gl::Disable(gl::CULL_FACE);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        self.vert_buf.unbind();
        self.shader.unbind();

        // Reset calls
        self.reset();
    }
}
