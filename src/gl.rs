#![allow(dead_code)]

use std::slice::from_raw_parts;
use crate::cache::{Path, Vertex};
use crate::vg::{Paint, CompositeState, Scissor};

extern "C" {
    fn glnvg__allocFragUniforms(gl: *mut ContextGL, n: i32) -> i32;
    fn nvg__fragUniformPtr<'a>(gl: *mut ContextGL, i: i32) -> &'a mut FragUniforms;
    fn glnvg__blendCompositeOperation(composite: CompositeState) -> Blend;

    /*
    fn glnvg__convertPaint(
        gl: *mut ContextGL, frag: *mut FragUniforms, paint: *const Paint,
        scissor: *const Scissor, width: f32, fringe: f32, stroke_thr: f32,
    ) -> i32;
    */

    fn glnvg__findTexture(gl: *mut ContextGL, id: u32) -> *mut Texture;
}

fn max_vert_count(paths: &[Path]) -> usize {
    paths.into_iter().fold(0, |acc, path| acc + path.nfill + path.nstroke)
}

type GLuint = u32;
type GLint = i32;
type GLenum = i32;

const GLNVG_MAX_LOCS: usize = 3;

// Flag indicating if geometry based anti-aliasing is used (may not be needed when using MSAA).
const ANTIALIAS: i32 = 1<<0;
// Flag indicating if strokes should be drawn using stencil buffer. The rendering will be a little
// slower, but path overlaps (i.e. self-intersecting or sharp turns) will be drawn just once.
const STENCIL_STROKES: i32 = 1<<1;


const TEXTURE_ALPHA: i32 = 0x01;
const TEXTURE_RGBA: i32 = 0x02;

const IMAGE_PREMULTIPLIED: i32 = 1<<4;

const SHADER_FILLGRAD: f32 = 0.0;
const SHADER_FILLIMG: f32 = 1.0;
const SHADER_SIMPLE: f32 = 2.0;
const SHADER_IMG: f32 = 3.0;

fn copy_verts(dst: &mut [Vertex], src: &[Vertex]) {
    dst.copy_from_slice(src);
}

fn copy_verts_fan(dst: &mut [Vertex], src: &[Vertex]) {
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

#[repr(C)]
struct Shader {
    prog: GLuint,
    frag: GLuint,
    vert: GLuint,
    loc: [GLint; GLNVG_MAX_LOCS],
}

#[repr(C)]
struct FragUniforms {
    array: [f32; 11 * 4],
}

impl Default for FragUniforms {
    fn default() -> Self {
        Self { array: [0f32; 11 * 4] }
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

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct PathGL {
    fill_offset: i32,
    fill_count: i32,
    stroke_offset: i32,
    stroke_count: i32,
}

#[repr(C)]
struct Texture {
    id: u32,
    tex: GLuint,
    width: i32,
    height: i32,
    kind: i32,
    flags: i32,
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
#[derive(Default)]
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
    image: u32,

    path_offset: i32,
    path_count: i32,

    triangle_offset: i32,
    triangle_count: i32,

    uniform_offset: i32,

    blend_func: Blend,
}

#[repr(C)]
struct ContextGL {
    shader: Shader,
    view: [f32; 2],

    textures: Vec<Texture>,

    texture_id: i32,
    vert_buf: GLuint,
    frag_size: i32,
    flags: i32,

    // Per frame buffers
    calls: Vec<Call>,
    paths: Vec<PathGL>,
    verts: Vec<Vertex>,
    uniforms: Vec<[u8; 12 * 4]>,
}

impl ContextGL {
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
        let i = unsafe { glnvg__allocFragUniforms(self, n as i32) };
        assert_ne!(i, -1);
        i as usize
    }

    fn frag_uniform<'a>(&mut self, i: usize) -> &'a mut FragUniforms {
        unsafe { nvg__fragUniformPtr(self, i as i32) }
    }
}

fn xform2mat3(t: [f32; 6]) -> [f32; 12] {
    [
        t[0], t[1], 0.0, 0.0,
        t[2], t[3], 0.0, 0.0,
        t[4], t[5], 1.0, 0.0,
    ]
}

fn convert_paint(
    gl: &mut ContextGL, frag: &mut FragUniforms, paint: &Paint,
    scissor: &Scissor, width: f32, fringe: f32, stroke_thr: f32,
) -> i32 {
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

    let invxform = if paint.image != 0 {
        let tex = unsafe { glnvg__findTexture(gl, paint.image) };
        if tex.is_null() { return 0; }
        let tex = unsafe { &*tex };
        /*
        if ((tex.flags & NVG_IMAGE_FLIPY) != 0) {
            float m1[6], m2[6];
            nvgTransformTranslate(m1, 0.0f, frag.extent[1] * 0.5f);
            nvgTransformMultiply(m1, paint.xform);
            nvgTransformScale(m2, 1.0f, -1.0f);
            nvgTransformMultiply(m2, m1);
            nvgTransformTranslate(m1, 0.0f, -frag.extent[1] * 0.5f);
            nvgTransformMultiply(m1, m2);
            nvgTransformInverse(invxform, m1);
        } else {
            nvgTransformInverse(invxform, paint.xform);
        }
        */
        frag.set_type(SHADER_FILLIMG);

        if tex.kind == TEXTURE_RGBA {
            frag.set_tex_type(if tex.flags & IMAGE_PREMULTIPLIED != 0 { 0.0 } else { 1.0 });
        } else {
            frag.set_tex_type(2.0);
        }
//      printf("frag.texType = %d\n", frag.texType);
        crate::transform::inverse(&paint.xform)
    } else {
        frag.set_type(SHADER_FILLGRAD);
        frag.set_radius(paint.radius);
        frag.set_feather(paint.feather);
        crate::transform::inverse(&paint.xform)
    };

    frag.set_paint_mat(xform2mat3(invxform));

    1
}

#[no_mangle] extern "C"
fn glnvg__renderFill(
    gl: &mut ContextGL, paint: &mut Paint,
    composite: CompositeState , scissor: &Scissor, fringe: f32,
    bounds: &[f32; 4], paths: *const Path, npaths: i32,
) {
    let paths = unsafe { from_raw_parts(paths, npaths as usize) };
    let call = unsafe { &mut *gl.alloc_call() };

    call.kind = CallKind::FILL;
    call.triangle_count = 4;
    call.path_offset = gl.alloc_paths(paths.len()) as i32;
    call.path_count = paths.len() as i32;
    call.image = paint.image;
    call.blend_func = unsafe { glnvg__blendCompositeOperation(composite) };

    if paths.len() == 1 && paths[0].convex {
        call.kind = CallKind::CONVEXFILL;
        call.triangle_count = 0; // Bounding box fill quad not needed for convex fill
    }

    // Allocate vertices for all the paths.
    let maxverts = max_vert_count(paths) + call.triangle_count as usize;
    let mut offset = gl.alloc_verts(maxverts);

    for (i, path) in paths.iter().enumerate() {
        let copy = &mut gl.paths[i + call.path_offset as usize];
        *copy = Default::default();
        if path.nfill > 0 {
            copy.fill_offset = offset as i32;
            copy.fill_count = path.nfill as i32;
            copy_verts_fan(&mut gl.verts[offset..offset + path.nfill], path.fill());
            offset += path.nfill;
        }
        if path.nstroke > 0 {
            copy.stroke_offset = offset as i32;
            copy.stroke_count = path.nstroke as i32;
            copy_verts(&mut gl.verts[offset..offset + path.nstroke], path.stroke());
            offset += path.nstroke;
        }
    }

    // Setup uniforms for draw calls
    let a = if call.kind == CallKind::FILL {
        // Quad
        call.triangle_offset = offset as i32;
        let quad = &mut gl.verts[call.triangle_offset as usize..];
        quad[0].set([bounds[2], bounds[3]], [0.5, 1.0]);
        quad[1].set([bounds[2], bounds[1]], [0.5, 1.0]);
        quad[2].set([bounds[0], bounds[3]], [0.5, 1.0]);
        quad[3].set([bounds[0], bounds[1]], [0.5, 1.0]);

        call.uniform_offset = gl.alloc_frag_uniforms(2) as i32;

        // Simple shader for stencil
        let frag = gl.frag_uniform(call.uniform_offset as usize);
        *frag = Default::default();
        frag.set_stroke_thr(-1.0);
        frag.set_type(SHADER_SIMPLE);

        // Fill shader
        gl.frag_uniform((call.uniform_offset + gl.frag_size) as usize)
    } else {
        call.uniform_offset = gl.alloc_frag_uniforms(1) as i32;
        // Fill shader
        gl.frag_uniform(call.uniform_offset as usize)
    };

    convert_paint(gl, a, paint, scissor, fringe, fringe, -1.0);
}

#[no_mangle] extern "C"
fn glnvg__renderStroke(
    gl: &mut ContextGL, paint: &mut Paint,
    composite: CompositeState , scissor: &Scissor, fringe: f32,
    stroke_width: f32, paths: *const Path, npaths: i32,
) {
    let paths = unsafe { from_raw_parts(paths, npaths as usize) };
    let call = unsafe { &mut *gl.alloc_call() };

    call.kind = CallKind::STROKE;
    call.path_offset = gl.alloc_paths(paths.len()) as i32;
    call.path_count = paths.len() as i32;
    call.image = paint.image;
    call.blend_func = unsafe { glnvg__blendCompositeOperation(composite) };

    // Allocate vertices for all the paths.
    let maxverts = max_vert_count(paths);
    let mut offset = gl.alloc_verts(maxverts);

    for (i, path) in paths.iter().enumerate() {
        let copy = &mut gl.paths[i + call.path_offset as usize];
        *copy = Default::default();

        if path.nstroke != 0 {
            copy.stroke_offset = offset as i32;
            copy.stroke_count = path.nstroke as i32;

            copy_verts(&mut gl.verts[offset..offset + path.nstroke], path.stroke());
            offset += path.nstroke;
        }
    }

    // Fill shader
    if gl.flags & STENCIL_STROKES != 0 {
        call.uniform_offset = gl.alloc_frag_uniforms(2) as i32;

        let a = gl.frag_uniform(call.uniform_offset as usize);
        let b = gl.frag_uniform((call.uniform_offset + gl.frag_size) as usize);

        convert_paint(gl, a, paint, scissor, stroke_width, fringe, -1.0);
        convert_paint(gl, b, paint, scissor, stroke_width, fringe, 1.0 - 0.5/255.0);
    } else {
        call.uniform_offset = gl.alloc_frag_uniforms(1) as i32;
        let a = gl.frag_uniform(call.uniform_offset as usize);
        convert_paint(gl, a, paint, scissor, stroke_width, fringe, -1.0);
    }
}


#[no_mangle] extern "C"
fn glnvg__renderTriangles(
    gl: &mut ContextGL, paint: &mut Paint,
    composite: CompositeState , scissor: &Scissor,
    verts: *const Vertex, nverts: i32,
) {
    let verts = unsafe { std::slice::from_raw_parts(verts, nverts as usize) };
    let call = unsafe { &mut *gl.alloc_call() };

    call.kind = CallKind::TRIANGLES;
    call.image = paint.image;
    call.blend_func = unsafe { glnvg__blendCompositeOperation(composite) };

    // Allocate vertices for all the paths.
    call.triangle_offset = gl.alloc_verts(verts.len()) as i32;
    call.triangle_count = verts.len() as i32;

    let offset = call.triangle_offset;
    copy_verts(&mut gl.verts[offset as usize..offset as usize + verts.len()], verts);

    // Fill shader
    call.uniform_offset = gl.alloc_frag_uniforms(1) as i32;
    let frag = gl.frag_uniform(call.uniform_offset as usize);
    convert_paint(gl, frag, paint, scissor, 1.0, 1.0, -1.0);
    frag.set_type(SHADER_IMG);
}
