use crate::{
    backend::{Paint, Uniforms, Vertex},
    cache::Path,
};

#[inline]
fn max_vert_count(paths: &[Path]) -> usize {
    paths.iter().fold(0, |acc, path| {
        let fill = path.fill.as_ref().map(|v| v.len()).unwrap_or_default();
        let stroke = path.stroke.as_ref().map(|v| v.len()).unwrap_or_default();
        acc + fill + stroke
    })
}

#[derive(Clone, Copy, Default)]
pub struct RawPath {
    pub fill: RawSlice,
    pub stroke: RawSlice,
}

pub enum Call {
    Convex { idx: u32, path: RawSlice },
    Fill { idx: u32, path: RawSlice, quad: u32 },
    Stroke { idx: u32, path: RawSlice },
}

#[derive(Clone, Copy, Default)]
pub struct RawSlice {
    pub start: u32,
    pub len: u32,
}

impl RawSlice {
    #[inline]
    pub fn new(start: u32, len: u32) -> Self {
        Self { start, len }
    }

    #[inline]
    pub fn range32(self) -> std::ops::Range<u32> {
        self.start..self.start + self.len
    }

    #[inline]
    pub fn range(self) -> std::ops::Range<usize> {
        self.start as usize..(self.start + self.len) as usize
    }
}

pub struct VecAlloc<T>(Vec<T>);

impl<T> std::ops::Index<RawSlice> for VecAlloc<T> {
    type Output = [T];

    #[inline]
    fn index(&self, raw: RawSlice) -> &Self::Output {
        &self.0[raw.range()]
    }
}

impl<T> std::ops::IndexMut<RawSlice> for VecAlloc<T> {
    #[inline]
    fn index_mut(&mut self, raw: RawSlice) -> &mut Self::Output {
        &mut self.0[raw.range()]
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
}

impl<T: Default> VecAlloc<T> {
    #[inline]
    fn alloc(&mut self, count: usize) -> (RawSlice, &mut [T]) {
        let start = self.0.len();
        self.0.resize_with(start + count as usize, Default::default);
        let raw = RawSlice::new(start as u32, count as u32);
        (raw, &mut self.0[start..start + count])
    }
}

impl<T: Copy> VecAlloc<T> {
    #[inline]
    fn copy_from_slice(&mut self, slice: RawSlice, src: &[T]) -> u32 {
        (&mut self[slice]).copy_from_slice(src);
        slice.len
    }
}

pub struct Picture {
    pub calls: Vec<Call>,
    pub verts: VecAlloc<Vertex>,
    pub paths: VecAlloc<RawPath>,
    pub uniforms: VecAlloc<Uniforms>,
}

impl Default for Picture {
    fn default() -> Self {
        Self {
            calls: Vec::new(),
            paths: VecAlloc::new(),
            verts: VecAlloc::new(),
            uniforms: VecAlloc::new(),
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
        self.calls.clear();
        self.uniforms.clear();
    }

    pub fn draw_fill(&mut self, paint: Paint, fringe: f32, bounds: [f32; 4], paths: &[Path]) {
        let (kind, triangle_count) = if paths.len() == 1 && paths[0].convex {
            (false, 0u32) // Bounding box fill quad not needed for convex fill
        } else {
            (true, 4u32)
        };

        // Allocate vertices for all the paths.
        let (path, path_dst) = self.paths.alloc(paths.len());
        let maxverts = max_vert_count(paths) + triangle_count as usize;
        let mut offset = self.verts.alloc(maxverts).0.start;

        for (src, dst) in paths.iter().zip(path_dst.iter_mut()) {
            if let Some(src) = &src.fill {
                dst.fill = RawSlice::new(offset, src.len() as u32);
                offset += self.verts.copy_from_slice(dst.fill, src);
            }
            if let Some(src) = &src.stroke {
                dst.stroke = RawSlice::new(offset, src.len() as u32);
                offset += self.verts.copy_from_slice(dst.stroke, src);
            }
        }

        // Setup uniforms for draw calls
        if kind {
            let quad = &mut self.verts[RawSlice::new(offset, 4)];
            quad[0].set([bounds[2], bounds[3]], [0.5, 1.0]);
            quad[1].set([bounds[2], bounds[1]], [0.5, 1.0]);
            quad[2].set([bounds[0], bounds[3]], [0.5, 1.0]);
            quad[3].set([bounds[0], bounds[1]], [0.5, 1.0]);

            let uniform = Uniforms::fill(&paint, fringe, fringe, -1.0);

            let idx = self.uniforms.push(Default::default());
            let _ = self.uniforms.push(uniform);

            let quad = offset;
            self.calls.push(Call::Fill { idx, path, quad })
        } else {
            let uniform = Uniforms::fill(&paint, fringe, fringe, -1.0);
            let idx = self.uniforms.push(uniform);
            self.calls.push(Call::Convex { idx, path })
        };
    }

    pub fn draw_stroke(&mut self, paint: Paint, fringe: f32, stroke_width: f32, paths: &[Path]) {
        // Allocate vertices for all the paths.
        let (path, path_dst) = self.paths.alloc(paths.len());
        let maxverts = max_vert_count(paths);
        let mut offset = self.verts.alloc(maxverts).0.start;

        for (src, dst) in paths.iter().zip(path_dst.iter_mut()) {
            if let Some(src) = &src.stroke {
                dst.stroke = RawSlice::new(offset, src.len() as u32);
                offset += self.verts.copy_from_slice(dst.stroke, src);
            }
        }

        // Fill shader
        let thr = 1.0 - 0.5 / 255.0;
        let (uniform, ab) = self.uniforms.alloc(2);
        ab[0] = Uniforms::fill(&paint, stroke_width, fringe, thr);
        ab[1] = Uniforms::fill(&paint, stroke_width, fringe, -1.0);

        let idx = uniform.start;
        self.calls.push(Call::Stroke { idx, path })
    }
}
