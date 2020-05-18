use crate::{
    backend::Vertex,
    cache::Path,
    paint::{InternalPaint, Uniforms},
};
use std::ops::Range;

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
}

pub struct VecAlloc<T>(Vec<T>);

impl<T> std::ops::Index<Range<u32>> for VecAlloc<T> {
    type Output = [T];
    #[inline]
    fn index(&self, raw: Range<u32>) -> &Self::Output {
        &self.0[raw.start as usize..raw.end as usize]
    }
}

impl<T> std::ops::IndexMut<Range<u32>> for VecAlloc<T> {
    #[inline]
    fn index_mut(&mut self, raw: Range<u32>) -> &mut Self::Output {
        &mut self.0[raw.start as usize..raw.end as usize]
    }
}

impl<T> AsRef<[T]> for VecAlloc<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T> VecAlloc<T> {
    #[inline]
    fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    fn push(&mut self, value: T) -> u32 {
        let start = self.0.len();
        self.0.push(value);
        start as u32
    }

    #[inline]
    fn alloc_with<F: FnMut() -> T>(&mut self, count: usize, f: F) -> (Range<u32>, &mut [T]) {
        let start = self.0.len();
        self.0.resize_with(start + count as usize, f);
        (
            start as u32..start as u32 + count as u32,
            &mut self.0[start..start + count],
        )
    }

    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) -> Range<u32> {
        let start = self.0.len() as u32;
        self.0.extend(iter);
        let end = self.0.len() as u32;
        start..end
    }
}

impl<T: Default> VecAlloc<T> {
    #[inline]
    fn alloc(&mut self, count: usize) -> (Range<u32>, &mut [T]) {
        self.alloc_with(count, Default::default)
    }
}

impl<T: Copy> VecAlloc<T> {
    #[inline]
    fn extend_with(&mut self, src: &[T]) -> Range<u32> {
        let start = self.0.len() as u32;
        self.0.extend_from_slice(src);
        let end = self.0.len() as u32;
        start..end
    }
}

pub struct Picture {
    pub calls: Vec<Call>,
    pub verts: VecAlloc<Vertex>,
    pub uniforms: VecAlloc<Uniforms>,

    pub paths: VecAlloc<RawPath>,
    pub strokes: VecAlloc<Range<u32>>,
}

impl Default for Picture {
    fn default() -> Self {
        Self {
            calls: Vec::new(),
            verts: VecAlloc::new(),
            uniforms: VecAlloc::new(),

            paths: VecAlloc::new(),
            strokes: VecAlloc::new(),
        }
    }
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

    pub fn draw_fill(
        &mut self,
        paint: InternalPaint,
        fringe: f32,
        bounds: [f32; 4],
        paths: &[Path],
    ) {
        // Bounding box fill quad not needed for convex fill
        let kind = !(paths.len() == 1 && paths[0].convex);

        // Allocate vertices for all the paths.
        let (path, path_dst) = self.paths.alloc(paths.len());
        for (src, dst) in paths.iter().zip(path_dst.iter_mut()) {
            if let Some(src) = &src.fill {
                dst.fill = self.verts.extend_with(src);
            }
            if let Some(src) = &src.stroke {
                dst.stroke = self.verts.extend_with(src);
            }
        }

        // Setup uniforms for draw calls
        if kind {
            let quad = self.verts.extend_with(&[
                Vertex::new([bounds[2], bounds[3]], [0.5, 1.0]),
                Vertex::new([bounds[2], bounds[1]], [0.5, 1.0]),
                Vertex::new([bounds[0], bounds[3]], [0.5, 1.0]),
                Vertex::new([bounds[0], bounds[1]], [0.5, 1.0]),
            ]);

            let uniform = Uniforms::fill(&paint, fringe, fringe, -1.0);

            let idx = self.uniforms.push(Default::default());
            let _ = self.uniforms.push(uniform);

            let quad = quad.start;
            self.calls.push(Call::Fill { idx, path, quad })
        } else {
            let uniform = Uniforms::fill(&paint, fringe, fringe, -1.0);
            let idx = self.uniforms.push(uniform);
            self.calls.push(Call::Convex { idx, path })
        };
    }

    pub fn draw_stroke(
        &mut self,
        paint: InternalPaint,
        fringe: f32,
        stroke_width: f32,
        paths: &[Path],
    ) {
        // Allocate vertices for all the paths.
        let verts = &mut self.verts;
        let iter = paths
            .iter()
            .filter_map(|path| path.stroke.as_ref().map(|src| verts.extend_with(src)));

        let path = self.strokes.extend(iter);

        // Fill shader
        let a = Uniforms::fill(&paint, stroke_width, fringe, 1.0 - 0.5 / 255.0);
        let b = Uniforms::fill(&paint, stroke_width, fringe, -1.0);

        let idx = self.uniforms.push(a);
        let _ = self.uniforms.push(b);

        self.calls.push(Call::Stroke { idx, path })
    }
}
