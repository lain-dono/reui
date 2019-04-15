#![allow(dead_code)]

use std::{
    slice::from_raw_parts_mut,
    mem,
};

use crate::{
    cache::{Path, Vertex},
    vg::{Scissor, Paint, CompositeState},
};

use super::{Image, ImageFlags};
use super::gl::*;
use super::gl_shader::Shader;

use slotmap::Key;

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

#[derive(Clone, Copy, Default)]
struct PathGL {
    fill_offset: usize,
    fill_count: usize,
    stroke_offset: usize,
    stroke_count: usize,
}

#[derive(Debug)]
struct Texture {
    tex: GLuint,

    width: u32,
    height: u32,

    kind: i32,
    flags: ImageFlags,
}

impl Drop for Texture {
    fn drop(&mut self) {
        if self.tex != 0 && !self.flags.contains(ImageFlags::NODELETE) {
            unsafe { glDeleteTextures(1, &self.tex); }
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

#[repr(u8)]
#[derive(PartialEq, Eq)]
enum CallKind {
    FILL,
    CONVEXFILL,
    STROKE,
    TRIANGLES,
}

#[derive(Clone, Copy)]
struct DrawCallData {
    image: Image,
    uniform_offset: usize,
}

impl Default for DrawCallData {
    fn default() -> Self {
        Self {
            image: Image::null(),
            uniform_offset: 0,
        }
    }
}

enum DrawCall {
    Fill {
        data: DrawCallData,
        blend_func: Blend,

        path_offset: usize,
        path_count: usize,

        triangle_offset: usize,
    },
    ConvexFill {
        data: DrawCallData,
        blend_func: Blend,

        path_offset: usize,
    },
    Stroke {
        data: DrawCallData,
        blend_func: Blend,

        path_offset: usize,
        path_count: usize,
    },
    Triangles {
        data: DrawCallData,
        blend_func: Blend,

        triangle_offset: usize,
        triangle_count: usize,
    },
}

struct Call {
    kind: CallKind,

    path_offset: usize,
    path_count: usize,

    triangle_offset: usize,
    triangle_count: usize,

    data: DrawCallData,
    blend_func: Blend,
}

impl Call {
    fn triangles(
        image: Image,
        uniform_offset: usize,
        blend_func: Blend,
        triangle_offset: usize,
        triangle_count: usize,
    ) -> Self {
        Self {
            kind: CallKind::TRIANGLES,
            data: DrawCallData {
                image,
                uniform_offset,
            },
            blend_func,
            triangle_offset,
            triangle_count,
            path_offset: 0,
            path_count: 0,
        }
    }
}

pub struct BackendGL {
    // Per frame buffers
    calls: Vec<Call>,
    paths: Vec<PathGL>,
    verts: Vec<Vertex>,
    uniforms: Vec<FragUniforms>,

    textures: slotmap::SlotMap<Image, Texture>,

    shader: Shader,
    view: [f32; 2],

    vert_buf: Buffer,
    flags: NFlags,
}

impl BackendGL {
    pub fn new(flags: NFlags) -> Self {
        check_error("init");
        let shader = Shader::new();
        check_error("shader & uniform locations");

        Self {
            shader,
            flags,
            view: [0f32; 2],
            textures: slotmap::SlotMap::with_key(),
            vert_buf: Buffer::new(),

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

    fn set_uniforms(&self, offset: usize, image: Image) {
        let frag = self.frag_uniform(offset);
        self.shader.bind_frag(&frag.array);
        unsafe {
            if !image.is_null() {
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
        self.textures.get(image)
    }

    fn find_texture_mut<'a>(&mut self, image: Image) -> Option<&'a mut Texture> {
        if let Some(tex) = self.textures.get_mut(image) {
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

        let invxform = if !paint.image.is_null() {
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
                let mut m1 = crate::transform::translate(0.0, paint.extent[1] * 0.5);
                crate::transform::mul(&mut m1, &paint.xform);
                let mut m2 = crate::transform::scale(1.0, -1.0);
                crate::transform::mul(&mut m2, &m1);
                let mut m1 = crate::transform::translate(0.0, -paint.extent[1] * 0.5);
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

    fn convex_fill(
        &self,
        data: DrawCallData,
        path_offset: usize, path_count: usize,
    ) {
        let start = path_offset as usize;
        let end = start + path_count as usize;

        self.set_uniforms(data.uniform_offset, data.image);
        check_error("convex fill");

        for path in &self.paths[start..end] {
            gl_draw_strip(path.fill_offset, path.fill_count);
            // Draw fringes
            if path.stroke_count > 0 {
                gl_draw_strip(path.stroke_offset, path.stroke_count);
            }
        }
    }

    fn triangles(
        &self,
        data: DrawCallData,
        triangle_offset: usize,
        triangle_count: usize,
    ) {
        self.set_uniforms(data.uniform_offset, data.image);
        check_error("triangles fill");
        gl_draw_triangles(triangle_offset, triangle_count);
    }

    fn fill(
        &self,
        data: DrawCallData,
        path_offset: usize, path_count: usize,
        triangle_offset: usize, triangle_count: usize,
    ) {
        let start = path_offset as usize;
        let end = start + path_count as usize;

        // Draw shapes
        unsafe {
            glEnable(GL_STENCIL_TEST);
            glStencilMask(0xff);
            glStencilFunc(GL_ALWAYS, 0, 0xff);
            glColorMask(GL_FALSE, GL_FALSE, GL_FALSE, GL_FALSE);
        }

        // set bindpoint for solid loc
        self.set_uniforms(data.uniform_offset, Image::null());
        check_error("fill simple");

        unsafe {
            glStencilOpSeparate(GL_FRONT, GL_KEEP, GL_KEEP, GL_INCR_WRAP);
            glStencilOpSeparate(GL_BACK, GL_KEEP, GL_KEEP, GL_DECR_WRAP);
            glDisable(GL_CULL_FACE);
        }
        for path in &self.paths[start..end] {
            gl_draw_strip(path.fill_offset, path.fill_count);
        }

        // Draw anti-aliased pixels
        unsafe {
            glEnable(GL_CULL_FACE);
            glColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE);
        }

        self.set_uniforms(data.uniform_offset + 1, data.image);
        check_error("fill fill");

        if self.flags.contains(NFlags::ANTIALIAS) {
            unsafe {
                glStencilFunc(GL_EQUAL, 0x00, 0xff);
                glStencilOp(GL_KEEP, GL_KEEP, GL_KEEP);
            }
            // Draw fringes
            for path in &self.paths[start..end] {
                gl_draw_strip(path.stroke_offset, path.stroke_count);
            }
        }

        // Draw fill
        if triangle_count == 4 {
            unsafe {
                glStencilFunc(GL_NOTEQUAL, 0x00, 0xff);
                glStencilOp(GL_ZERO, GL_ZERO, GL_ZERO);
            }
            gl_draw_strip(triangle_offset, 4);
        }

        unsafe {
            glDisable(GL_STENCIL_TEST);
        }
    }

    fn stroke(
        &self,
        data: DrawCallData, path_offset: usize, path_count: usize,
    ) {
        let start = path_offset as usize;
        let end = start + path_count as usize;

        if self.flags.contains(NFlags::STENCIL_STROKES) {
            unsafe {
                glEnable(GL_STENCIL_TEST);
                glStencilMask(0xff);
            }

            // Fill the stroke base without overlap
            unsafe {
                glStencilFunc(GL_EQUAL, 0x0, 0xff);
                glStencilOp(GL_KEEP, GL_KEEP, GL_INCR);
            }
            self.set_uniforms(data.uniform_offset + 1, data.image);
            check_error("stroke fill 0");
            for path in &self.paths[start..end] {
                gl_draw_strip(path.stroke_offset, path.stroke_count);
            }

            // Draw anti-aliased pixels.
            self.set_uniforms(data.uniform_offset, data.image);
            unsafe {
                glStencilFunc(GL_EQUAL, 0x00, 0xff);
                glStencilOp(GL_KEEP, GL_KEEP, GL_KEEP);
            }
            for path in &self.paths[start..end] {
                gl_draw_strip(path.stroke_offset, path.stroke_count);
            }

            // Clear stencil buffer.
            unsafe {
                glColorMask(GL_FALSE, GL_FALSE, GL_FALSE, GL_FALSE);
                glStencilFunc(GL_ALWAYS, 0x0, 0xff);
                glStencilOp(GL_ZERO, GL_ZERO, GL_ZERO);
            }
            check_error("stroke fill 1");
            for path in &self.paths[start..end] {
                gl_draw_strip(path.stroke_offset, path.stroke_count);
            }

            unsafe {
                glColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE);
                glDisable(GL_STENCIL_TEST);
            }

            //convertPaint(
            //  gl,
            //  self.frag_uniformPtr(gl, uniform_offset + 1), paint, scissor, strokeWidth, fringe, 1.0f - 0.5f/255.0f);
        } else {
            self.set_uniforms(data.uniform_offset, data.image);
            check_error("stroke fill");
            // Draw Strokes
            for path in &self.paths[start..end] {
                gl_draw_strip(path.stroke_offset, path.stroke_count);
            }
        }
    }

    pub fn draw_triangles(&mut self, paint: &Paint, op: CompositeState, scissor: &Scissor, verts: &[Vertex]) {
        // Allocate vertices for all the paths.
        let triangle_offset = self.alloc_verts(verts.len());
        let triangle_count = verts.len();

        copy_verts(&mut self.verts, triangle_offset, verts.len(), verts);

        // Fill shader
        let uniform_offset = self.alloc_frag_uniforms(1);
        let frag = self.frag_uniform_mut(uniform_offset);
        self.convert_paint(frag, paint, scissor, 1.0, 1.0, -1.0);
        frag.set_type(SHADER_IMG);

        self.calls.push(Call::triangles(
            paint.image,
            uniform_offset,
            op.into(),
            triangle_offset,
            triangle_count,
        ))
    }

    pub fn draw_fill(
        &mut self, paint: &Paint,
        op: CompositeState , scissor: &Scissor, fringe: f32,
        bounds: &[f32; 4], paths: &[Path],
    ) {
        let path_offset = self.alloc_paths(paths.len());
        let path_count = paths.len();

        let kind;
        let triangle_count; 
        if paths.len() == 1 && paths[0].convex {
            kind = CallKind::CONVEXFILL;
            triangle_count = 0; // Bounding box fill quad not needed for convex fill
        } else {
            kind = CallKind::FILL;
            triangle_count = 4;
        }

        // Allocate vertices for all the paths.
        let maxverts = max_vert_count(paths) + triangle_count;
        let mut offset = self.alloc_verts(maxverts);

        for (i, path) in paths.iter().enumerate() {
            let copy = &mut self.paths[i + path_offset];
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
            data: DrawCallData {
                image: paint.image,
                uniform_offset,
            },
            blend_func: op.into(),
            triangle_offset,
            triangle_count,
            path_offset,
            path_count,
        })
    }

    pub fn draw_stroke(
        &mut self, paint: &Paint,
        op: CompositeState , scissor: &Scissor, fringe: f32,
        stroke_width: f32, paths: &[Path],
    ) {
        let path_offset = self.alloc_paths(paths.len());
        let path_count = paths.len();

        // Allocate vertices for all the paths.
        let maxverts = max_vert_count(paths);
        let mut offset = self.alloc_verts(maxverts);

        for (i, path) in paths.iter().enumerate() {
            let copy = &mut self.paths[i + path_offset];
            *copy = Default::default();

            if path.nstroke != 0 {
                copy.stroke_offset = offset;
                copy.stroke_count = path.nstroke;

                copy_verts(&mut self.verts, offset, path.nstroke, path.stroke());
                offset += path.nstroke;
            }
        }

        // Fill shader
        let uniform_offset;
        if self.flags.contains(NFlags::STENCIL_STROKES) {
            uniform_offset = self.alloc_frag_uniforms(2);

            let a = self.frag_uniform_mut(uniform_offset);
            let b = self.frag_uniform_mut(uniform_offset + 1);

            self.convert_paint(a, paint, scissor, stroke_width, fringe, -1.0);
            self.convert_paint(b, paint, scissor, stroke_width, fringe, 1.0 - 0.5/255.0);
        } else {
            uniform_offset = self.alloc_frag_uniforms(1);
            let a = self.frag_uniform_mut(uniform_offset);
            self.convert_paint(a, paint, scissor, stroke_width, fringe, -1.0);
        }

        self.calls.push(Call {
            kind: CallKind::STROKE,
            data: DrawCallData {
                image: paint.image,
                uniform_offset,
            },
            blend_func: op.into(),
            triangle_offset: 0,
            triangle_count: 0,
            path_offset,
            path_count,
        })
    }

    pub fn set_viewport(&mut self, width: f32, height: f32, _devicePixelRatio: f32) {
        self.view = [width, height];
    }

    pub fn flush(&mut self) {
        if self.calls.len() == 0 {
            self.reset();
            return;
        }

        self.shader.bind();

        unsafe {
            // Setup require GL state.

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
        }

        // Upload vertex data
        self.vert_buf.bind_and_upload(&self.verts);
        let size = mem::size_of::<Vertex>();

        unsafe {
            glEnableVertexAttribArray(0);
            glEnableVertexAttribArray(1);

            glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, size as i32, 0);
            glVertexAttribPointer(1, 2, GL_UNSIGNED_SHORT, GL_TRUE, size as i32, 2 * mem::size_of::<f32>());
        }

        // Set view and texture just once per frame.
        self.shader.bind_view(&self.view);

        for call in &self.calls {
            call.blend_func.bind();

            match call.kind {
                CallKind::FILL => self.fill(
                    call.data,
                    call.path_offset,
                    call.path_count,
                    call.triangle_offset,
                    call.triangle_count,
                ),
                CallKind::CONVEXFILL => self.convex_fill(
                    call.data,
                    call.path_offset,
                    call.path_count,
                ),
                CallKind::STROKE => self.stroke(
                    call.data,
                    call.path_offset,
                    call.path_count,
                ),
                CallKind::TRIANGLES => self.triangles(
                    call.data,
                    call.triangle_offset,
                    call.triangle_count,
                ),
            }
        }

        unsafe {
            glDisableVertexAttribArray(0);
            glDisableVertexAttribArray(1);

            glDisable(GL_CULL_FACE);

            glBindTexture(GL_TEXTURE_2D, 0);
        }

        self.vert_buf.unbind();
        self.shader.unbind();

        // Reset calls
        self.reset();
    }

    pub fn texture_size(&self, image: Image) -> Option<(u32, u32)> {
        self.find_texture(image).map(|t| (t.width, t.height))
    }

    pub fn update_texture(&mut self, image: Image, _x: i32, y: i32, _w: u32, h: u32, data: *const u8) -> bool {
        let tex = if let Some(tex) = self.find_texture(image) {
            tex
        } else {
            return false;
        };

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
            glBindTexture(GL_TEXTURE_2D, tex.tex);
            glPixelStorei(GL_UNPACK_ALIGNMENT, 1);

            glTexSubImage2D(GL_TEXTURE_2D, 0, x,y, w as i32,h as i32, kind, GL_UNSIGNED_BYTE, data);

            glPixelStorei(GL_UNPACK_ALIGNMENT, 4);
            glBindTexture(GL_TEXTURE_2D, 0);
        }

        true
    }

    pub fn delete_texture(&mut self, image: Image) -> bool {
        self.textures.remove(image).is_some()
    }

    pub fn create_texture(&mut self, kind: i32, w: u32, h: u32, flags: ImageFlags, data: *const u8) -> Image {
        // GL 1.4 and later has support for generating mipmaps using a tex parameter.
        let mipmaps = if flags.contains(ImageFlags::GENERATE_MIPMAPS) {
            GL_TRUE as i32
        } else {
            GL_FALSE as i32
        };

        let kind_tex = if kind == TEXTURE_RGBA {
            GL_RGBA
        } else {
            GL_LUMINANCE
        };

        let min = if flags.contains(ImageFlags::GENERATE_MIPMAPS) {
            if flags.contains(ImageFlags::NEAREST) {
                GL_NEAREST_MIPMAP_NEAREST
            } else {
                GL_LINEAR_MIPMAP_LINEAR
            }
        } else {
            if flags.contains(ImageFlags::NEAREST) { GL_NEAREST } else { GL_LINEAR }
        };

        let mag = if flags.contains(ImageFlags::NEAREST) { GL_NEAREST } else { GL_LINEAR };
        let wrap_s = if flags.contains(ImageFlags::REPEATX) { GL_REPEAT } else { GL_CLAMP_TO_EDGE };
        let wrap_t = if flags.contains(ImageFlags::REPEATY) { GL_REPEAT } else { GL_CLAMP_TO_EDGE };

        let mut tex = 0;
        unsafe {
            glGenTextures(1, &mut tex);
            glBindTexture(GL_TEXTURE_2D, tex);
            glPixelStorei(GL_UNPACK_ALIGNMENT,1);

            glTexParameteri(GL_TEXTURE_2D, GL_GENERATE_MIPMAP, mipmaps);

            glTexImage2D(
                GL_TEXTURE_2D, 0,
                kind_tex as i32, w as i32, h as i32,
                0, kind_tex, GL_UNSIGNED_BYTE, data);

            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, min);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, mag);

            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, wrap_s);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, wrap_t);

            glPixelStorei(GL_UNPACK_ALIGNMENT, 4);

            check_error("create tex");
            glBindTexture(GL_TEXTURE_2D, 0);
        }

        self.textures.insert(Texture {
            kind, flags, tex,
            width: w,
            height: h,
        })
    }
}
