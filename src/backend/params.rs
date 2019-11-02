use std::{
    ptr::null,
    slice::from_raw_parts_mut,
    mem,
};

use crate::{
    cache::{Path, Vertex},
    vg::{Scissor, Paint},
    math::Transform,
};

use super::{Image, ImageFlags, TEXTURE_RGBA};
use super::gl::{self, types::GLuint};
use super::utils::*;
use super::gl_shader::Shader;

use slotmap::Key;

fn check_error(msg: &str) {
    if true {
        let err = unsafe { gl::GetError() };
        if err != gl::NO_ERROR {
            log::debug!("GL Error {:08x} after {}", err, msg);
        }
    }
}

fn xform2mat3(t: Transform) -> [f32; 12] {
    [
        t.m11, t.m12, 0.0, 0.0,
        t.m21, t.m22, 0.0, 0.0,
        t.m31, t.m32, 1.0, 0.0,
    ]
}

fn copy_verts(dst: &mut [Vertex], offset: usize, count: usize, src: &[Vertex]) {
    (&mut dst[offset..offset+count]).copy_from_slice(src);
}

fn max_vert_count(paths: &[Path]) -> usize {
    paths.iter().fold(0, |acc, path| {
        let fill = path.fill.as_ref().map(|v| v.len()).unwrap_or_default();
        let stroke = path.stroke.as_ref().map(|v| v.len()).unwrap_or_default();
        acc + fill + stroke
    })
}

bitflags::bitflags!(
    #[repr(transparent)]
    pub struct NFlags: i32 {
        // Flag indicating if geometry based anti-aliasing is used (may not be needed when using MSAA).
        const ANTIALIAS = 1;
        // Flag indicating if strokes should be drawn using stencil buffer. The rendering will be a little
        // slower, but path overlaps (i.e. self-intersecting or sharp turns) will be drawn just once.
        const STENCIL_STROKES = 1<<1;
        // Flag indicating that additional debug checks are done.
        const DEBUG = 1<<2;
    }
);

//const IMAGE_PREMULTIPLIED: i32 = 1<<4;

const SHADER_FILLGRAD: f32 = 0.0;
const SHADER_FILLIMG: f32 = 1.0;
const SHADER_SIMPLE: f32 = 2.0;
const SHADER_IMG: f32 = 3.0;

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
            unsafe { gl::DeleteTextures(1, &self.tex); }
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

struct Call {
    kind: CallKind,

    path_offset: usize,
    path_count: usize,

    triangle_offset: usize,
    triangle_count: usize,

    data: DrawCallData,
}

impl Call {
    fn triangles(
        image: Image,
        uniform_offset: usize,
        triangle_offset: usize,
        triangle_count: usize,
    ) -> Self {
        Self {
            kind: CallKind::TRIANGLES,
            data: DrawCallData {
                image,
                uniform_offset,
            },
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
                gl::BindTexture(gl::TEXTURE_2D, tex.map(|t| t.tex).unwrap_or(0));
                check_error("tex paint tex");
            } else {
                gl::BindTexture(gl::TEXTURE_2D, 0);
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
                xform2mat3(xform.inverse().unwrap_or_else(Transform::identity)),
                scissor.extent,
                [
                    (xform.m11*xform.m11 + xform.m12*xform.m12).sqrt() / fringe,
                    (xform.m21*xform.m21 + xform.m22*xform.m22).sqrt() / fringe,
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

            paint.xform.inverse().unwrap_or_else(Transform::identity)
        } else {
            frag.set_type(SHADER_FILLGRAD);
            frag.set_radius(paint.radius);
            frag.set_feather(paint.feather);
            paint.xform.inverse().unwrap_or_else(Transform::identity)
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
            gl::Enable(gl::STENCIL_TEST);
            gl::StencilMask(0xff);
            gl::StencilFunc(gl::ALWAYS, 0, 0xff);
            gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
        }

        // set bindpoint for solid loc
        self.set_uniforms(data.uniform_offset, Image::null());
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

        self.set_uniforms(data.uniform_offset + 1, data.image);
        check_error("fill fill");

        if self.flags.contains(NFlags::ANTIALIAS) {
            unsafe {
                gl::StencilFunc(gl::EQUAL, 0x00, 0xff);
                gl::StencilOp(gl::KEEP, gl::KEEP, gl::KEEP);
            }
            // Draw fringes
            for path in &self.paths[start..end] {
                gl_draw_strip(path.stroke_offset, path.stroke_count);
            }
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

    fn stroke(
        &self,
        data: DrawCallData, path_offset: usize, path_count: usize,
    ) {
        let start = path_offset as usize;
        let end = start + path_count as usize;

        if self.flags.contains(NFlags::STENCIL_STROKES) {
            unsafe {
                gl::Enable(gl::STENCIL_TEST);
                gl::StencilMask(0xff);
            }

            // Fill the stroke base without overlap
            unsafe {
                gl::StencilFunc(gl::EQUAL, 0x0, 0xff);
                gl::StencilOp(gl::KEEP, gl::KEEP, gl::INCR);
            }
            self.set_uniforms(data.uniform_offset + 1, data.image);
            check_error("stroke fill 0");
            for path in &self.paths[start..end] {
                gl_draw_strip(path.stroke_offset, path.stroke_count);
            }

            // Draw anti-aliased pixels.
            self.set_uniforms(data.uniform_offset, data.image);
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
            //  self.frag_uniform_mut(gl, uniform_offset + 1), paint, scissor, strokeWidth, fringe, 1.0f - 0.5f/255.0f);
        } else {
            self.set_uniforms(data.uniform_offset, data.image);
            check_error("stroke fill");
            // Draw Strokes
            for path in &self.paths[start..end] {
                gl_draw_strip(path.stroke_offset, path.stroke_count);
            }
        }
    }

    pub fn draw_triangles(&mut self, paint: &Paint, scissor: &Scissor, verts: &[Vertex]) {
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
            triangle_offset,
            triangle_count,
        ))
    }

    pub fn draw_fill(
        &mut self, paint: &Paint,
        scissor: &Scissor, fringe: f32,
        bounds: &[f32; 4], paths: &[Path],
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
            data: DrawCallData {
                image: paint.image,
                uniform_offset,
            },
            triangle_offset,
            triangle_count,
            path_offset,
            path_count,
        })
    }

    pub fn draw_stroke(
        &mut self, paint: &Paint,
        scissor: &Scissor, fringe: f32,
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

            if let Some(stroke) = path.stroke.as_ref() {
                copy.stroke_offset = offset;
                copy.stroke_count = stroke.len();

                copy_verts(&mut self.verts, offset, stroke.len(), stroke);
                offset += stroke.len();
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
            triangle_offset: 0,
            triangle_count: 0,
            path_offset,
            path_count,
        })
    }

    pub fn set_viewport(&mut self, width: f32, height: f32, pixel_ratio: f32) {
        self.view = [width / pixel_ratio, height / pixel_ratio];
    }

    pub fn flush(&mut self) {
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
            gl::VertexAttribPointer(1, 2, gl::UNSIGNED_SHORT, gl::TRUE, size as i32, (2 * mem::size_of::<f32>()) as *const libc::c_void);
        }

        // Set view and texture just once per frame.
        self.shader.bind_view(&self.view);

        Blend::default().bind();
        for call in &self.calls {
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
            (gl::RGBA, tex.width*4)
        } else {
            (gl::LUMINANCE, tex.width)
        };

        let stride = y * stride as i32;
        let data = unsafe { data.add(stride as usize) };

        let x = 0;
        let w = tex.width;

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, tex.tex);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            gl::TexSubImage2D(gl::TEXTURE_2D, 0, x,y, w as i32,h as i32, kind, gl::UNSIGNED_BYTE, data as *const libc::c_void);

            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        true
    }

    pub fn delete_texture(&mut self, image: Image) -> bool {
        self.textures.remove(image).is_some()
    }

    pub fn create_texture(&mut self, kind: i32, w: u32, h: u32, flags: ImageFlags, data: *const u8) -> Image {
        // GL 1.4 and later has support for generating mipmaps using a tex parameter.
        let mipmaps = i32::from(if flags.contains(ImageFlags::GENERATE_MIPMAPS) {
            gl::TRUE
        } else {
            gl::FALSE
        });

        let kind_tex = if kind == TEXTURE_RGBA {
            gl::RGBA
        } else {
            gl::LUMINANCE
        };

        let min = if flags.contains(ImageFlags::GENERATE_MIPMAPS) {
            if flags.contains(ImageFlags::NEAREST) {
                gl::NEAREST_MIPMAP_NEAREST
            } else {
                gl::LINEAR_MIPMAP_LINEAR
            }
        } else if flags.contains(ImageFlags::NEAREST) {
            gl::NEAREST
        } else { gl::LINEAR };

        let mag = if flags.contains(ImageFlags::NEAREST) { gl::NEAREST } else { gl::LINEAR };
        let wrap_s = if flags.contains(ImageFlags::REPEATX) { gl::REPEAT } else { gl::CLAMP_TO_EDGE };
        let wrap_t = if flags.contains(ImageFlags::REPEATY) { gl::REPEAT } else { gl::CLAMP_TO_EDGE };

        let mut tex = 0;
        unsafe {
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT,1);

            gl::TexParameteri(gl::TEXTURE_2D, gl::GENERATE_MIPMAP_HINT, mipmaps);

            gl::TexImage2D(
                gl::TEXTURE_2D, 0,
                kind_tex as i32, w as i32, h as i32,
                0, kind_tex, gl::UNSIGNED_BYTE, data as *const libc::c_void);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag as i32);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_s as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_t as i32);

            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);

            check_error("create tex");
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        self.textures.insert(Texture {
            kind, flags, tex,
            width: w,
            height: h,
        })
    }
}