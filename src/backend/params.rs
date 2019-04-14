#![allow(dead_code)]

use std::{
    slice::from_raw_parts_mut,
    mem,
    ptr::null,
};

use crate::{
    context::Context,
    cache::{Path, Vertex},
    vg::{Scissor, Paint, CompositeState, BlendFactor, Image},
};

use super::ImageFlags;
use super::gl::*;

fn check_error(msg: &str) {
    if true {
        let err = unsafe { glGetError() };
        if err != GL_NO_ERROR {
            log::debug!("GL Error {:08x} after {}", err, msg);
        }
    }
}

fn xform2mat3(t: [f32; 6]) -> [f32; 12] {
    [
        t[0], t[1], 0.0, 0.0,
        t[2], t[3], 0.0, 0.0,
        t[4], t[5], 1.0, 0.0,
    ]
}

fn copy_verts(dst: &mut [Vertex], offset: usize, count: usize, src: &[Vertex]) {
    (&mut dst[offset..offset+count]).copy_from_slice(src);
}

fn copy_verts_fan(dst: &mut [Vertex], offset: usize, count: usize, src: &[Vertex]) {
    let dst = &mut dst[offset..offset+count];
    for i in 0..dst.len() {
        dst[i] = src[fan2strip(i, src.len())];
    }
}

#[inline(always)]
fn fan2strip(i: usize, len: usize) -> usize {
    // if you need to change winding order
    // - just reverse the test in that "if" (== to !=).
    if i % 2 != 0 {
        i / 2
    } else {
        len - 1 - i / 2
    }
}

fn max_vert_count(paths: &[Path]) -> usize {
    paths.into_iter().fold(0, |acc, path| acc + path.nfill + path.nstroke)
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

bitflags::bitflags!(
    #[repr(transparent)]
    pub struct NFlags: i32 {
        // Flag indicating if geometry based anti-aliasing is used (may not be needed when using MSAA).
        const ANTIALIAS = 1<<0;
        // Flag indicating if strokes should be drawn using stencil buffer. The rendering will be a little
        // slower, but path overlaps (i.e. self-intersecting or sharp turns) will be drawn just once.
        const STENCIL_STROKES = 1<<1;
        // Flag indicating that additional debug checks are done.
        const DEBUG = 1<<2;
    }
);


const TEXTURE_ALPHA: i32 = 0x01;
const TEXTURE_RGBA: i32 = 0x02;

const IMAGE_PREMULTIPLIED: i32 = 1<<4;

const SHADER_FILLGRAD: f32 = 0.0;
const SHADER_FILLIMG: f32 = 1.0;
const SHADER_SIMPLE: f32 = 2.0;
const SHADER_IMG: f32 = 3.0;

const LOC_VIEWSIZE: usize = 0;
const LOC_TEX: usize = 1;
const LOC_FRAG: usize = 2;
const MAX_LOCS: usize = 3;

#[repr(C)]
struct Shader {
    prog: GLuint,
    frag: GLuint,
    vert: GLuint,
    loc: [GLint; MAX_LOCS],
}

#[repr(C, align(4))]
struct FragUniforms {
    array: [f32; 11 * 4 + 1],
}

impl Default for FragUniforms {
    fn default() -> Self {
        Self { array: [0f32; 11 * 4 + 1] }
    }
}

impl FragUniforms {
    fn set_inner_col(&mut self, color: [f32; 4]) {
        self.array[24..28].copy_from_slice(&color)
    }

    fn set_outer_col(&mut self, color: [f32; 4]) {
        self.array[28..32].copy_from_slice(&color)
    }

    fn set_paint_mat(&mut self, mat: [f32; 12]) {
        self.array[12..24].copy_from_slice(&mat);
    }

    fn set_scissor(&mut self, mat: [f32; 12], ext: [f32; 2], scale: [f32; 2]) {
        self.array[0..12].copy_from_slice(&mat);
        self.array[32..34].copy_from_slice(&ext);
        self.array[34..36].copy_from_slice(&scale);
    }

    fn extent(&self) -> [f32; 2] {
        let w = self.array[36];
        let h = self.array[37];
        [w, h]
    }

    fn set_extent(&mut self, ext: [f32; 2]) {
        self.array[36..38].copy_from_slice(&ext);
    }

    fn set_radius(&mut self, radius: f32) {
        self.array[38] = radius;
    }
    fn set_feather(&mut self, feather: f32) {
        self.array[39] = feather;
    }
    fn set_stroke_mul(&mut self, mul: f32) {
        self.array[40] = mul;
    }
    fn set_stroke_thr(&mut self, thr: f32) {
        self.array[41] = thr;
    }
    fn set_tex_type(&mut self, t: f32) {
        self.array[42] = t;
    }
    fn set_type(&mut self, t: f32) {
        self.array[43] = t;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct PathGL {
    fill_offset: usize,
    fill_count: usize,
    stroke_offset: usize,
    stroke_count: usize,
}

#[repr(C)]
#[derive(Default)]
struct Texture {
    id: Image,
    tex: GLuint,

    width: u32,
    height: u32,

    kind: i32,
    flags: ImageFlags,
}

#[repr(i32)]
#[derive(PartialEq, Eq)]
enum CallKind {
    NONE = 0,
    FILL,
    CONVEXFILL,
    STROKE,
    TRIANGLES,
}

impl Default for CallKind {
    fn default() -> Self {
        CallKind::NONE
    }
}

#[repr(C)]
struct Blend {
    src_color: GLenum,
    dst_color: GLenum,
    src_alpha: GLenum,
    dst_alpha: GLenum,
}

#[repr(C)]
#[derive(Default)]
struct Call {
    kind: CallKind,
    image: Image,

    path_offset: usize,
    path_count: usize,

    triangle_offset: usize,
    triangle_count: usize,

    uniform_offset: usize,

    blend_func: Blend,
}

#[repr(C)]
pub struct BackendGL {
    shader: Shader,
    view: [f32; 2],

    texture_id: i32,
    vert_buf: GLuint,
    flags: NFlags,

    textures: Vec<Texture>,

    // Per frame buffers
    calls: Vec<Call>,
    paths: Vec<PathGL>,
    verts: Vec<Vertex>,
    uniforms: Vec<FragUniforms>,
}

impl Drop for BackendGL {
    fn drop(&mut self) {
        if self.vert_buf != 0 {
            unsafe { glDeleteBuffers(1, &self.vert_buf); }
        }
        for t in &mut self.textures {
            if t.tex != 0 && !t.flags.contains(ImageFlags::NODELETE) {
                unsafe { glDeleteTextures(1, &t.tex); }
            }
        }
    }
}

impl BackendGL {
    pub fn new(flags: NFlags) -> Self {
        check_error("init");
        let shader = create_shader();
        check_error("shader & uniform locations");
        let mut vert_buf = 0;
        // Create dynamic vertex array
        unsafe {
            glGenBuffers(1, &mut vert_buf);
            check_error("create done");
            glFinish();
        }

        Self {
            shader,
            view: [0f32; 2],

            textures: Vec::new(),

            texture_id: 0,
            vert_buf,
            flags,

            // Per frame buffers
            calls: Vec::new(),
            paths: Vec::new(),
            verts: Vec::new(),
            uniforms: Vec::new(),
        }
    }

    pub fn edge_aa(&self) -> bool { true }

    pub fn reset(&mut self) {
        self.verts.clear();
        self.paths.clear();
        self.calls.clear();
        self.uniforms.clear();
    }

    fn alloc_call(&mut self) -> *mut Call {
        self.calls.push(Default::default());
        self.calls.last_mut().unwrap()
    }
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
        let size = start + n;
        self.uniforms.resize_with(start + n, Default::default);
        start
    }

    fn alloc_texture(&mut self) -> &mut Texture {
        self.texture_id += 1;
        self.textures.push(Texture {
            id: Image(self.texture_id as u32),
            .. Default::default()
        });
        self.textures.last_mut().unwrap()
    }

    fn set_uniforms(&self, offset: usize, image: Image) {
        let frag = self.frag_uniform(offset);
        unsafe {
            glUniform4fv(self.shader.loc[LOC_FRAG], 11, &(frag.array[0]));

            if image.0 != 0 {
                let tex = self.find_texture(image);
                glBindTexture(GL_TEXTURE_2D, tex.map(|t| t.tex).unwrap_or(0));
                check_error("tex paint tex");
            } else {
                glBindTexture(GL_TEXTURE_2D, 0);
            }
        }
    }

    fn frag_uniform(&self, idx: usize) -> &FragUniforms {
        &self.uniforms[idx]
    }

    fn frag_uniform_mut<'a>(&mut self, idx: usize) -> &'a mut FragUniforms {
        unsafe { &mut from_raw_parts_mut(self.uniforms.as_mut_ptr(), self.uniforms.len())[idx] }
    }

    fn find_texture(&self, image: Image) -> Option<&Texture> {
        self.textures.iter().find(|t| t.id == image)
    }

    fn find_texture_mut<'a>(&mut self, image: Image) -> Option<&'a mut Texture> {
        if let Some(tex) = self.textures.iter_mut().find(|t| t.id == image) {
            let tex: *mut Texture = tex;
            Some(unsafe { &mut *tex })
        } else {
            None
        }
    }

    fn convert_paint(
        &mut self, frag: &mut FragUniforms, paint: &Paint,
        scissor: &Scissor, width: f32, fringe: f32, stroke_thr: f32,
    ) -> bool {
        *frag = Default::default();

        frag.set_inner_col(paint.inner_color.premul());
        frag.set_outer_col(paint.outer_color.premul());

        if scissor.extent[0] < -0.5 || scissor.extent[1] < -0.5 {
            frag.set_scissor([0.0; 12], [1.0, 1.0], [1.0, 1.0]);
        } else {
            let xform = &scissor.xform;
            frag.set_scissor(
                xform2mat3(crate::transform::inverse(xform)),
                scissor.extent,
                [
                    (xform[0]*xform[0] + xform[2]*xform[2]).sqrt() / fringe,
                    (xform[1]*xform[1] + xform[3]*xform[3]).sqrt() / fringe,
                ],
            );
        }

        frag.set_extent(paint.extent);

        frag.set_stroke_mul((width*0.5 + fringe*0.5) / fringe);
        frag.set_stroke_thr(stroke_thr);

        let invxform = if paint.image != Default::default() {
            let tex = match self.find_texture(paint.image) {
                Some(tex) => tex,
                None => return false,
            };
            frag.set_type(SHADER_FILLIMG);

            if tex.kind == TEXTURE_RGBA {
                frag.set_tex_type(if tex.flags.contains(ImageFlags::PREMULTIPLIED) { 0.0 } else { 1.0 });
            } else {
                frag.set_tex_type(2.0);
            }
            //      printf("frag.texType = %d\n", frag.texType);
            if tex.flags.contains(ImageFlags::FLIPY) {
                let mut m1 = crate::transform::translate(0.0, frag.extent()[1] * 0.5);
                crate::transform::mul(&mut m1, &paint.xform);
                let mut m2 = crate::transform::scale(1.0, -1.0);
                crate::transform::mul(&mut m2, &m1);
                let mut m1 = crate::transform::translate(0.0, -frag.extent()[1] * 0.5);
                crate::transform::mul(&mut m1, &m2);
                crate::transform::inverse(&m1)
            } else {
                crate::transform::inverse(&paint.xform)
            }
        } else {
            frag.set_type(SHADER_FILLGRAD);
            frag.set_radius(paint.radius);
            frag.set_feather(paint.feather);
            crate::transform::inverse(&paint.xform)
        };

        frag.set_paint_mat(xform2mat3(invxform));

        true
    }

    unsafe fn convex_fill(&self, call: &Call) {
        let start = call.path_offset as usize;
        let end = start + call.path_count as usize;

        self.set_uniforms(call.uniform_offset, call.image);
        check_error("convex fill");

        for path in &self.paths[start..end] {
            glDrawArrays(GL_TRIANGLE_STRIP, path.fill_offset as i32, path.fill_count as i32);
            // Draw fringes
            if path.stroke_count > 0 {
                glDrawArrays(GL_TRIANGLE_STRIP, path.stroke_offset as i32, path.stroke_count as i32);
            }
        }
    }

    unsafe fn triangles(&self, call: &Call) {
        self.set_uniforms(call.uniform_offset, call.image);
        check_error("triangles fill");

        glDrawArrays(GL_TRIANGLES, call.triangle_offset as i32, call.triangle_count as i32);
    }

    unsafe fn fill(&self, call: &Call) {
        let start = call.path_offset as usize;
        let end = start + call.path_count as usize;

        // Draw shapes
        glEnable(GL_STENCIL_TEST);
        glStencilMask(0xff);
        glStencilFunc(GL_ALWAYS, 0, 0xff);
        glColorMask(GL_FALSE, GL_FALSE, GL_FALSE, GL_FALSE);

        // set bindpoint for solid loc
        self.set_uniforms(call.uniform_offset, Default::default());
        check_error("fill simple");

        glStencilOpSeparate(GL_FRONT, GL_KEEP, GL_KEEP, GL_INCR_WRAP);
        glStencilOpSeparate(GL_BACK, GL_KEEP, GL_KEEP, GL_DECR_WRAP);
        glDisable(GL_CULL_FACE);
        for path in &self.paths[start..end] {
            glDrawArrays(GL_TRIANGLE_STRIP, path.fill_offset as i32, path.fill_count as i32);
        }
        glEnable(GL_CULL_FACE);

        // Draw anti-aliased pixels
        glColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE);

        self.set_uniforms(call.uniform_offset + 1, call.image);
        check_error("fill fill");

        if self.flags.contains(NFlags::ANTIALIAS) {
            glStencilFunc(GL_EQUAL, 0x00, 0xff);
            glStencilOp(GL_KEEP, GL_KEEP, GL_KEEP);
            // Draw fringes
            for path in &self.paths[start..end] {
                glDrawArrays(GL_TRIANGLE_STRIP, path.stroke_offset as i32, path.stroke_count as i32);
            }
        }

        // Draw fill
        glStencilFunc(GL_NOTEQUAL, 0x0, 0xff);
        glStencilOp(GL_ZERO, GL_ZERO, GL_ZERO);
        glDrawArrays(GL_TRIANGLE_STRIP, call.triangle_offset as i32, call.triangle_count as i32);

        glDisable(GL_STENCIL_TEST);
    }

    unsafe fn stroke(&self, call: &Call) {
        let start = call.path_offset as usize;
        let end = start + call.path_count as usize;

        if self.flags.contains(NFlags::STENCIL_STROKES) {
            glEnable(GL_STENCIL_TEST);
            glStencilMask(0xff);

            // Fill the stroke base without overlap
            glStencilFunc(GL_EQUAL, 0x0, 0xff);
            glStencilOp(GL_KEEP, GL_KEEP, GL_INCR);
            self.set_uniforms(call.uniform_offset + 1, call.image);
            check_error("stroke fill 0");
            for path in &self.paths[start..end] {
                glDrawArrays(GL_TRIANGLE_STRIP, path.stroke_offset as i32, path.stroke_count as i32);
            }

            // Draw anti-aliased pixels.
            self.set_uniforms(call.uniform_offset, call.image);
            glStencilFunc(GL_EQUAL, 0x00, 0xff);
            glStencilOp(GL_KEEP, GL_KEEP, GL_KEEP);
            for path in &self.paths[start..end] {
                glDrawArrays(GL_TRIANGLE_STRIP, path.stroke_offset as i32, path.stroke_count as i32);
            }

            // Clear stencil buffer.
            glColorMask(GL_FALSE, GL_FALSE, GL_FALSE, GL_FALSE);
            glStencilFunc(GL_ALWAYS, 0x0, 0xff);
            glStencilOp(GL_ZERO, GL_ZERO, GL_ZERO);
            check_error("stroke fill 1");
            for path in &self.paths[start..end] {
                glDrawArrays(GL_TRIANGLE_STRIP, path.stroke_offset as i32, path.stroke_count as i32);
            }
            glColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE);

            glDisable(GL_STENCIL_TEST);

            //convertPaint(
            //  gl,
            //  self.frag_uniformPtr(gl, call.uniformOffset + 1), paint, scissor, strokeWidth, fringe, 1.0f - 0.5f/255.0f);
        } else {
            self.set_uniforms(call.uniform_offset, call.image);
            check_error("stroke fill");
            // Draw Strokes
            for path in &self.paths[start..end] {
                glDrawArrays(GL_TRIANGLE_STRIP, path.stroke_offset as i32, path.stroke_count as i32);
            }
        }
    }

    pub fn draw_triangles(&mut self, paint: &Paint, op: CompositeState, scissor: &Scissor, verts: &[Vertex]) {
        let call = unsafe { &mut *self.alloc_call() };

        call.kind = CallKind::TRIANGLES;
        call.image = paint.image;
        call.blend_func = op.into();

        // Allocate vertices for all the paths.
        call.triangle_offset = self.alloc_verts(verts.len());
        call.triangle_count = verts.len();

        copy_verts(&mut self.verts, call.triangle_offset, verts.len(), verts);

        // Fill shader
        call.uniform_offset = self.alloc_frag_uniforms(1);
        let frag = self.frag_uniform_mut(call.uniform_offset);
        self.convert_paint(frag, paint, scissor, 1.0, 1.0, -1.0);
        frag.set_type(SHADER_IMG);
    }

    pub fn draw_fill(
        &mut self, paint: &Paint,
        composite: CompositeState , scissor: &Scissor, fringe: f32,
        bounds: &[f32; 4], paths: &[Path],
    ) {
        let call = unsafe { &mut *self.alloc_call() };

        call.kind = CallKind::FILL;
        call.triangle_count = 4;
        call.path_offset = self.alloc_paths(paths.len());
        call.path_count = paths.len();
        call.image = paint.image;
        call.blend_func = blend_composite_op(composite);

        if paths.len() == 1 && paths[0].convex {
            call.kind = CallKind::CONVEXFILL;
            call.triangle_count = 0; // Bounding box fill quad not needed for convex fill
        }

        // Allocate vertices for all the paths.
        let maxverts = max_vert_count(paths) + call.triangle_count;
        let mut offset = self.alloc_verts(maxverts);

        for (i, path) in paths.iter().enumerate() {
            let copy = &mut self.paths[i + call.path_offset];
            *copy = Default::default();
            if path.nfill > 0 {
                copy.fill_offset = offset;
                copy.fill_count = path.nfill;
                copy_verts_fan(&mut self.verts, offset, path.nfill, path.fill());
                offset += path.nfill;
            }
            if path.nstroke > 0 {
                copy.stroke_offset = offset;
                copy.stroke_count = path.nstroke;
                copy_verts(&mut self.verts, offset, path.nstroke, path.stroke());
                offset += path.nstroke;
            }
        }

        // Setup uniforms for draw calls
        let a = if call.kind == CallKind::FILL {
            // Quad
            call.triangle_offset = offset;
            let quad = &mut self.verts[call.triangle_offset..];
            quad[0].set([bounds[2], bounds[3]], [0.5, 1.0]);
            quad[1].set([bounds[2], bounds[1]], [0.5, 1.0]);
            quad[2].set([bounds[0], bounds[3]], [0.5, 1.0]);
            quad[3].set([bounds[0], bounds[1]], [0.5, 1.0]);

            call.uniform_offset = self.alloc_frag_uniforms(2);

            // Simple shader for stencil
            let frag = self.frag_uniform_mut(call.uniform_offset);
            *frag = Default::default();
            frag.set_stroke_thr(-1.0);
            frag.set_type(SHADER_SIMPLE);

            // Fill shader
            self.frag_uniform_mut(call.uniform_offset + 1)
        } else {
            call.uniform_offset = self.alloc_frag_uniforms(1);
            // Fill shader
            self.frag_uniform_mut(call.uniform_offset)
        };

        self.convert_paint(a, paint, scissor, fringe, fringe, -1.0);
    }

    pub fn draw_stroke(
        &mut self, paint: &Paint,
        composite: CompositeState , scissor: &Scissor, fringe: f32,
        stroke_width: f32, paths: &[Path],
    ) {
        let call = unsafe { &mut *self.alloc_call() };

        call.kind = CallKind::STROKE;
        call.path_offset = self.alloc_paths(paths.len());
        call.path_count = paths.len();
        call.image = paint.image;
        call.blend_func = blend_composite_op(composite);

        // Allocate vertices for all the paths.
        let maxverts = max_vert_count(paths);
        let mut offset = self.alloc_verts(maxverts);

        for (i, path) in paths.iter().enumerate() {
            let copy = &mut self.paths[i + call.path_offset];
            *copy = Default::default();

            if path.nstroke != 0 {
                copy.stroke_offset = offset;
                copy.stroke_count = path.nstroke;

                copy_verts(&mut self.verts, offset, path.nstroke, path.stroke());
                offset += path.nstroke;
            }
        }

        // Fill shader
        if self.flags.contains(NFlags::STENCIL_STROKES) {
            call.uniform_offset = self.alloc_frag_uniforms(2);

            let a = self.frag_uniform_mut(call.uniform_offset);
            let b = self.frag_uniform_mut(call.uniform_offset + 1);

            self.convert_paint(a, paint, scissor, stroke_width, fringe, -1.0);
            self.convert_paint(b, paint, scissor, stroke_width, fringe, 1.0 - 0.5/255.0);
        } else {
            call.uniform_offset = self.alloc_frag_uniforms(1);
            let a = self.frag_uniform_mut(call.uniform_offset);
            self.convert_paint(a, paint, scissor, stroke_width, fringe, -1.0);
        }
    }

    pub fn set_viewport(&mut self, width: f32, height: f32, _devicePixelRatio: f32) {
        self.view = [width, height];
    }

    pub fn flush(&mut self) {
        if self.calls.len() == 0 {
            self.reset();
            return;
        }

        unsafe {
            // Setup require GL state.
            glUseProgram(self.shader.prog);

            glEnable(GL_CULL_FACE);
            glCullFace(GL_BACK);
            glFrontFace(GL_CCW);
            glEnable(GL_BLEND);
            glDisable(GL_DEPTH_TEST);
            glDisable(GL_SCISSOR_TEST);
            glColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE);
            glStencilMask(0xffffffff);
            glStencilOp(GL_KEEP, GL_KEEP, GL_KEEP);
            glStencilFunc(GL_ALWAYS, 0, 0xffffffff);
            glActiveTexture(GL_TEXTURE0);
            glBindTexture(GL_TEXTURE_2D, 0);

            // Upload vertex data
            glBindBuffer(GL_ARRAY_BUFFER, self.vert_buf);
            let size = mem::size_of::<Vertex>();
            glBufferData(
                GL_ARRAY_BUFFER,
                self.verts.len() * size,
                self.verts.as_ptr() as *const u8,
                GL_STREAM_DRAW,
            );

            glEnableVertexAttribArray(0);
            glEnableVertexAttribArray(1);

            glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, size as i32, 0);
            glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, size as i32, 2 * mem::size_of::<f32>());

            // Set view and texture just once per frame.
            glUniform1i(self.shader.loc[LOC_TEX], 0);
            glUniform2fv(self.shader.loc[LOC_VIEWSIZE], 1, self.view.as_ptr());

            for call in &self.calls {
                glBlendFuncSeparate(
                    call.blend_func.src_color,
                    call.blend_func.dst_color,
                    call.blend_func.src_alpha,
                    call.blend_func.dst_alpha,
                );

                match call.kind {
                    CallKind::FILL => self.fill(call),
                    CallKind::CONVEXFILL => self.convex_fill(call),
                    CallKind::STROKE => self.stroke(call),
                    CallKind::TRIANGLES => self.triangles(call),
                    CallKind::NONE => (),
                }
            }

            glDisableVertexAttribArray(0);
            glDisableVertexAttribArray(1);

            glDisable(GL_CULL_FACE);
            glBindBuffer(GL_ARRAY_BUFFER, 0);
            glUseProgram(0);

            glBindTexture(GL_TEXTURE_2D, 0);
        }

        // Reset calls
        self.reset();
    }

    pub fn texture_size(&self, image: Image) -> Option<(u32, u32)> {
        self.find_texture(image).map(|t| (t.width, t.height))
    }

    pub fn update_texture(&mut self, image: Image, x: i32, y: i32, w: u32, h: u32, data: *const u8) -> bool {
        let tex = if let Some(tex) = self.find_texture(image) {
            tex
        } else {
            return false;
        };

        unsafe {
            glBindTexture(GL_TEXTURE_2D, tex.tex);
            glPixelStorei(GL_UNPACK_ALIGNMENT, 1);
        }

        // No support for all of skip, need to update a whole row at a time.
        let (kind, stride) = if tex.kind == TEXTURE_RGBA {
            (GL_RGBA, tex.width*4)
        } else {
            (GL_LUMINANCE, tex.width)
        };

        let stride = y * stride as i32;
        let data = unsafe { data.add(stride as usize) };

        let x = 0;
        let w = tex.width;

        unsafe {
            glTexSubImage2D(GL_TEXTURE_2D, 0, x,y, w as i32,h as i32, kind, GL_UNSIGNED_BYTE, data);

            glPixelStorei(GL_UNPACK_ALIGNMENT, 4);
            glBindTexture(GL_TEXTURE_2D, 0);
        }

        true
    }

    pub fn delete_texture(&mut self, id: Image) -> bool {
        for t in &mut self.textures {
            if t.id == id {
                if t.tex != 0 && !t.flags.contains(ImageFlags::NODELETE) {
                    unsafe { glDeleteTextures(1, &t.tex); }
                }
                *t = Default::default();
                return true;
            }
        }
        false
    }

    pub fn create_texture(&mut self, kind: i32, w: u32, h: u32, flags: ImageFlags, data: *const u8) -> Image {
        let tex = self.alloc_texture();
        let id = tex.id;

        tex.width = w;
        tex.height = h;
        tex.kind = kind;
        tex.flags = flags;

        unsafe {
            glGenTextures(1, &mut tex.tex);
            glBindTexture(GL_TEXTURE_2D, tex.tex);
            glPixelStorei(GL_UNPACK_ALIGNMENT,1);
        }

        // GL 1.4 and later has support for generating mipmaps using a tex parameter.
        if flags.contains(ImageFlags::GENERATE_MIPMAPS) {
            unsafe {
                glTexParameteri(GL_TEXTURE_2D, GL_GENERATE_MIPMAP, GL_TRUE as i32);
            }
        }

        let kind = if kind == TEXTURE_RGBA {
            GL_RGBA
        } else {
            GL_LUMINANCE
        };
        unsafe {
            glTexImage2D(
                GL_TEXTURE_2D, 0,
                kind as i32, w as i32, h as i32,
                0, kind, GL_UNSIGNED_BYTE, data);
        }

        let min = if flags.contains(ImageFlags::GENERATE_MIPMAPS) {
            if flags.contains(ImageFlags::NEAREST) {
                GL_NEAREST_MIPMAP_NEAREST
            } else {
                GL_LINEAR_MIPMAP_LINEAR
            }
        } else {
            if flags.contains(ImageFlags::NEAREST) {
                GL_NEAREST
            } else {
                GL_LINEAR
            }
        };
        let mag = if flags.contains(ImageFlags::NEAREST) {
            GL_NEAREST
        } else {
            GL_LINEAR
        };
        let wrap_s = if flags.contains(ImageFlags::REPEATX) {
            GL_REPEAT
        } else {
            GL_CLAMP_TO_EDGE
        };
        let wrap_t = if flags.contains(ImageFlags::REPEATY) {
            GL_REPEAT
        } else {
            GL_CLAMP_TO_EDGE
        };

        unsafe {
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, min);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, mag);

            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, wrap_s);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, wrap_t);

            glPixelStorei(GL_UNPACK_ALIGNMENT, 4);

            check_error("create tex");
            glBindTexture(GL_TEXTURE_2D, 0);
        }

        id
    }
}

fn create_shader() -> Shader {
    let (vshader, fshader) = (VERT.as_ptr(), FRAG.as_ptr());

    unsafe {
        let prog = glCreateProgram();
        let vert = glCreateShader(GL_VERTEX_SHADER);
        let frag = glCreateShader(GL_FRAGMENT_SHADER);
        //str[2] = vshader;
        glShaderSource(vert, 1, &vshader, null());
        //str[2] = fshader;
        glShaderSource(frag, 1, &fshader, null());

        glCompileShader(vert);
        /*
        let mut status = 0;
        glGetShaderiv(vert, GL_COMPILE_STATUS, &status);
        assert!(status == 1);
        */

        glCompileShader(frag);
        /*
        glGetShaderiv(frag, GL_COMPILE_STATUS, &status);
        assert!(status == 1);
        */

        glAttachShader(prog, vert);
        glAttachShader(prog, frag);

        glBindAttribLocation(prog, 0, b"vertex\0".as_ptr());
        glBindAttribLocation(prog, 1, b"tcoord\0".as_ptr());

        glLinkProgram(prog);
        /*
        glGetProgramiv(prog, GL_LINK_STATUS, &status);
        assert!(status == 1);
        */

        Shader {
            prog,
            vert,
            frag,
            loc: [
                glGetUniformLocation(prog, b"viewSize\0".as_ptr()),
                glGetUniformLocation(prog, b"tex\0".as_ptr()),
                glGetUniformLocation(prog, b"frag\0".as_ptr()),
            ]
        }
    }
}

// TODO: mediump float may not be enough for GLES2 in iOS.
// see the following discussion: https://github.com/memononen/nanovg/issues/46
static VERT: &[u8] = b"
uniform vec2 viewSize;
attribute vec2 vertex;
attribute vec2 tcoord;
varying vec2 ftcoord;
varying vec2 fpos;
void main(void) {
    ftcoord = tcoord;
    fpos = vertex;
    gl_Position = vec4(2.0*vertex.x/viewSize.x - 1.0, 1.0 - 2.0*vertex.y/viewSize.y, 0, 1);
}
\0";

static FRAG: &[u8] = b"
#define UNIFORMARRAY_SIZE 11

//precision highp float;

uniform vec4 frag[UNIFORMARRAY_SIZE];
uniform sampler2D tex;
varying vec2 ftcoord;
varying vec2 fpos;

#define scissorMat mat3(frag[0].xyz, frag[1].xyz, frag[2].xyz)
#define paintMat mat3(frag[3].xyz, frag[4].xyz, frag[5].xyz)
#define innerCol frag[6]
#define outerCol frag[7]
#define scissorExt frag[8].xy
#define scissorScale frag[8].zw
#define extent frag[9].xy
#define radius frag[9].z
#define feather frag[9].w
#define strokeMult frag[10].x
#define strokeThr frag[10].y
#define texType int(frag[10].z)
#define type int(frag[10].w)

float sdroundrect(vec2 pt, vec2 ext, float rad) {
    vec2 ext2 = ext - vec2(rad,rad);
    vec2 d = abs(pt) - ext2;
    return min(max(d.x,d.y),0.0) + length(max(d,0.0)) - rad;
}

// Scissoring
float scissorMask(vec2 p) {
    vec2 sc = (abs((scissorMat * vec3(p,1.0)).xy) - scissorExt);
    sc = vec2(0.5,0.5) - sc * scissorScale;
    return clamp(sc.x,0.0,1.0) * clamp(sc.y,0.0,1.0);
}

// Stroke - from [0..1] to clipped pyramid, where the slope is 1px.
float strokeMask() {
    return min(1.0, (1.0-abs(ftcoord.x*2.0-1.0))*strokeMult) * min(1.0, ftcoord.y);
}

void main(void) {
    vec4 result;
    float scissor = scissorMask(fpos);

    float strokeAlpha = strokeMask();
    if (strokeAlpha < strokeThr) discard;

    if (type == 0) {            // Gradient
        // Calculate gradient color using box gradient
        vec2 pt = (paintMat * vec3(fpos,1.0)).xy;
        float d = clamp((sdroundrect(pt, extent, radius) + feather*0.5) / feather, 0.0, 1.0);
        vec4 color = mix(innerCol,outerCol,d);
        // Combine alpha
        color *= strokeAlpha * scissor;
        result = color;
    } else if (type == 1) {        // Image
        // Calculate color fron texture
        vec2 pt = (paintMat * vec3(fpos,1.0)).xy / extent;
        vec4 color = texture2D(tex, pt);
        if (texType == 1) color = vec4(color.xyz*color.w,color.w);
        if (texType == 2) color = vec4(color.x);
        // Apply color tint and alpha.
        color *= innerCol;
        // Combine alpha
        color *= strokeAlpha * scissor;
        result = color;
    } else if (type == 2) {        // Stencil fill
        result = vec4(1,1,1,1);
    } else if (type == 3) {        // Textured tris
        vec4 color = texture2D(tex, ftcoord);
        if (texType == 1) color = vec4(color.xyz*color.w,color.w);
        if (texType == 2) color = vec4(color.x);
        color *= scissor;
        result = color * innerCol;
    }
    gl_FragColor = result;
}\0";
