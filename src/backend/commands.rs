use crate::{
    backend::{Paint, Scissor},
    cache::{Path, Vertex},
};

#[inline]
fn copy_verts(dst: &mut [Vertex], slice: RawSlice, src: &[Vertex]) -> u32 {
    (&mut dst[slice.range()]).copy_from_slice(src);
    slice.count
}

#[inline]
fn max_vert_count(paths: &[Path]) -> usize {
    paths.iter().fold(0, |acc, path| {
        let fill = path.fill.as_ref().map(|v| v.len()).unwrap_or_default();
        let stroke = path.stroke.as_ref().map(|v| v.len()).unwrap_or_default();
        acc + fill + stroke
    })
}

pub const SHADER_FILLGRAD: f32 = 0.0;
pub const SHADER_SIMPLE: f32 = 2.0;

#[repr(C, align(4))]
pub struct FragUniforms {
    pub scissor_mat: [f32; 4],
    pub paint_mat: [f32; 4],
    pub inner_color: [f32; 4],
    pub outer_color: [f32; 4],

    pub scissor_ext: [f32; 2],
    pub scissor_scale: [f32; 2],

    pub extent: [f32; 2],
    pub radius: f32,
    pub feather: f32,

    pub stroke_mul: f32, // scale
    pub stroke_thr: f32, // threshold
    pub padding: [u8; 4],
    pub kind: f32,
}

impl FragUniforms {
    pub fn fill(paint: &Paint, scissor: Scissor, width: f32, fringe: f32, stroke_thr: f32) -> Self {
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

#[derive(Clone, Copy, Default)]
pub struct PathGL {
    pub fill: RawSlice,
    pub stroke: RawSlice,
}

#[repr(u8)]
#[derive(PartialEq, Eq)]
pub enum CallKind {
    CONVEXFILL,
    FILL,
    STROKE,
}

pub struct Call {
    pub kind: CallKind,
    pub path: RawSlice,
    pub triangle: RawSlice,
    pub uniform_offset: usize,
}

#[derive(Clone, Copy, Default)]
pub struct RawSlice {
    pub offset: u32,
    pub count: u32,
}

impl RawSlice {
    #[inline]
    pub fn new(offset: u32, count: u32) -> Self {
        Self { offset, count }
    }

    #[inline]
    pub fn range(self) -> std::ops::Range<usize> {
        let (offset, count) = (self.offset as usize, self.count as usize);
        offset..offset + count
    }
}

pub struct VecAlloc<T: Default>(Vec<T>);

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

pub struct CmdBuffer {
    pub calls: Vec<Call>,
    pub paths: Vec<PathGL>,
    pub verts: Vec<Vertex>,
    pub uniforms: VecAlloc<FragUniforms>,
}

impl CmdBuffer {
    pub fn new() -> Self {
        Self {
            calls: Vec::new(),
            paths: Vec::new(),
            verts: Vec::new(),
            uniforms: VecAlloc::new(),
        }
    }

    pub fn clear(&mut self) {
        self.verts.clear();
        self.paths.clear();
        self.calls.clear();
        self.uniforms.clear();
    }

    pub fn alloc_verts(&mut self, n: usize) -> u32 {
        let start = self.verts.len();
        self.verts.resize_with(start + n, Default::default);
        start as u32
    }

    pub fn alloc_paths(&mut self, count: u32) -> RawSlice {
        let start = self.paths.len();
        self.paths
            .resize_with(start + count as usize, Default::default);
        RawSlice::new(start as u32, count)
    }

    pub fn draw_fill(
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
        let uniform_offset = if kind == CallKind::FILL {
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
            uniform_offset
        } else {
            // Fill shader
            let (uniform_offset, a) = self.uniforms.alloc(1);
            a[0] = FragUniforms::fill(&paint, scissor, fringe, fringe, -1.0);
            uniform_offset
        };

        self.calls.push(Call {
            kind,
            uniform_offset,
            triangle: RawSlice::new(offset, triangle_count),
            path,
        })
    }

    pub fn draw_stroke(
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
}
