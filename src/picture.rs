use crate::{paint::Uniforms, valloc::VecAlloc};
use std::ops::Range;

#[derive(Clone)]
pub enum Call {
    Convex {
        idx: u32,
        path: Range<u32>,
    },
    Fill {
        idx: u32,
        path: Range<u32>,
        quad: u32,
    },
    Stroke {
        idx: u32,
        path: Range<u32>,
    },

    Image {
        idx: u32,
        vtx: Range<u32>,
        image: u32,
    },
}

#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [u16; 2],
}

impl Vertex {
    #[inline(always)]
    pub fn new(pos: [f32; 2], uv: [f32; 2]) -> Self {
        let uv = [(uv[0] * 65535.0) as u16, (uv[1] * 65535.0) as u16];
        Self { pos, uv }
    }
}

#[derive(Clone)]
pub struct RawPath {
    pub fill: Range<u32>,
    pub stroke: Range<u32>,
}

impl Default for RawPath {
    #[inline]
    fn default() -> Self {
        Self {
            fill: 0..0,
            stroke: 0..0,
        }
    }
}

#[derive(Default)]
pub struct Picture {
    pub calls: Vec<Call>,
    pub verts: VecAlloc<Vertex>,
    pub uniforms: VecAlloc<Uniforms>,

    pub paths: VecAlloc<RawPath>,
    pub strokes: VecAlloc<Range<u32>>,
}

impl Picture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.verts.clear();
        self.paths.clear();
        self.strokes.clear();
        self.calls.clear();
        self.uniforms.clear();
    }
}
