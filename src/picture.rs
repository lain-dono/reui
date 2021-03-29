use crate::{
    math::{Offset, Transform},
    paint::Stroke,
    tessellator::Contour,
};
use std::ops::Range;

pub(crate) fn cast_slice<T: Sized>(slice: &[T]) -> &[u8] {
    use std::{mem::size_of, slice::from_raw_parts};
    unsafe { from_raw_parts(slice.as_ptr().cast(), slice.len() * size_of::<T>()) }
}

#[derive(Clone)]
pub enum DrawCall {
    Convex {
        instance: u32,
        base_vertex: i32,
        path: Range<usize>,
    },

    FillStencil {
        instance: u32,
        base_vertex: i32,
        path: Range<usize>,
    },
    FillQuad {
        instance: u32,
        base_vertex: i32,
        quad: Range<u32>,
    },
    FillQuadEvenOdd {
        instance: u32,
        base_vertex: i32,
        quad: Range<u32>,
    },

    StrokeBase {
        instance: u32,
        base_vertex: i32,
        path: Range<usize>,
    },
    StrokeStencil {
        instance: u32,
        base_vertex: i32,
        path: Range<usize>,
    },

    Fringes {
        instance: u32,
        base_vertex: i32,
        path: Range<usize>,
    },

    FringesEvenOdd {
        instance: u32,
        base_vertex: i32,
        path: Range<usize>,
    },

    SelectImage {
        image: u32,
    },

    Image {
        instance: u32,
        base_vertex: i32,
        indices: Range<u32>,
    },
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [u16; 2],
}

impl Vertex {
    #[inline]
    pub fn new(pos: impl Into<[f32; 2]>, uv: [f32; 2]) -> Self {
        let pos = pos.into();
        let uv = [(uv[0] * 65535.0) as u16, (uv[1] * 65535.0) as u16];
        Self { pos, uv }
    }

    #[inline]
    pub fn transform(self, transform: &Transform) -> Self {
        let pos = transform.apply(Offset::from(self.pos)).into();
        Self { pos, ..self }
    }
}

#[repr(C, align(4))]
#[derive(Clone, Copy, Default)]
pub struct Instance {
    pub paint_mat: [f32; 4],
    pub inner_color: [u8; 4],
    pub outer_color: [u8; 4],

    pub extent: [f32; 2],
    pub radius: f32,
    pub feather: f32,

    pub stroke_mul: f32, // scale
    pub stroke_thr: f32, // threshold
}

impl Instance {
    pub fn image(color: [u8; 4]) -> Self {
        Self {
            inner_color: color,
            ..Self::default()
        }
    }

    pub fn from_stroke(stroke: &Stroke, width: f32, fringe: f32, stroke_thr: f32) -> Self {
        let color = stroke.color.into();
        Self {
            paint_mat: Transform::identity().into(),

            inner_color: color,
            outer_color: color,

            extent: [0.0, 0.0],
            radius: 0.0,
            feather: 1.0,

            stroke_mul: (width * 0.5 + fringe * 0.5) / fringe,
            stroke_thr,
        }
    }
}

#[derive(Default)]
pub struct PictureRecorder {
    pub(crate) calls: Vec<DrawCall>,
    pub(crate) indices: Vec<u32>,
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) instances: Vec<Instance>,
    pub(crate) ranges: Vec<Range<u32>>,
}

impl PictureRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.calls.clear();
        self.vertices.clear();
        self.indices.clear();
        self.instances.clear();
        self.ranges.clear();
    }

    pub fn call(&mut self, call: DrawCall) {
        self.calls.push(call);
    }

    pub fn push_instance(&mut self, instance: Instance) -> u32 {
        self.instances.push(instance);
        self.instances.len() as u32 - 1
    }

    pub fn push_image(
        &mut self,
        min: Offset,
        max: Offset,
        transform: Transform,
    ) -> (i32, Range<u32>) {
        self.triangle_strip(&[
            Vertex::new([max.x, max.y], [1.0, 1.0]).transform(&transform),
            Vertex::new([max.x, min.y], [1.0, 0.0]).transform(&transform),
            Vertex::new([min.x, max.y], [0.0, 1.0]).transform(&transform),
            Vertex::new([min.x, min.y], [0.0, 0.0]).transform(&transform),
        ])
    }

    pub fn push_quad(&mut self, min: Offset, max: Offset) -> (i32, Range<u32>) {
        self.triangle_strip(&[
            Vertex::new([max.x, max.y], [0.5, 1.0]),
            Vertex::new([max.x, min.y], [0.5, 1.0]),
            Vertex::new([min.x, max.y], [0.5, 1.0]),
            Vertex::new([min.x, min.y], [0.5, 1.0]),
        ])
    }

    pub fn push_fill(&mut self, contours: &[Contour]) -> (i32, Range<usize>) {
        #![allow(clippy::cast_possible_wrap)]

        let base = self.vertices.len() as i32;
        let start = self.ranges.len();
        for contour in contours {
            if !contour.fill.is_empty() {
                let (_, range) = self.triangle_fan(&contour.fill);
                self.ranges.push(range);
            }
        }

        (base, start..self.ranges.len())
    }

    pub fn push_stroke(&mut self, contours: &[Contour]) -> (i32, Range<usize>) {
        #![allow(clippy::cast_possible_wrap)]

        let base = self.vertices.len() as i32;
        let start = self.ranges.len();
        for contour in contours {
            if !contour.stroke.is_empty() {
                let (_, range) = self.triangle_strip(&contour.stroke);
                self.ranges.push(range);
            }
        }

        (base, start..self.ranges.len())
    }

    pub fn build(&mut self, device: &wgpu::Device) -> PictureBundle {
        use wgpu::util::DeviceExt;

        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: cast_slice(self.indices.as_ref()),
            usage: wgpu::BufferUsage::INDEX,
        });

        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: cast_slice(self.vertices.as_ref()),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let instances = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: cast_slice(self.instances.as_ref()),
            usage: wgpu::BufferUsage::VERTEX,
        });

        PictureBundle {
            indices,
            vertices,
            instances,
        }
    }

    fn triangle_strip(&mut self, vertices: &[Vertex]) -> (i32, Range<u32>) {
        #![allow(clippy::cast_possible_wrap)]

        let base_vertex = self.vertices.len() as i32;
        self.vertices.extend_from_slice(vertices);

        let start = self.indices.len() as u32;
        for i in 0..vertices.len().saturating_sub(2) as u32 {
            let (a, b) = if 0 == i % 2 { (1, 2) } else { (2, 1) };
            self.indices.push(i);
            self.indices.push(i + a);
            self.indices.push(i + b);
        }

        (base_vertex, start..self.indices.len() as u32)
    }

    fn triangle_fan(&mut self, vertices: &[Vertex]) -> (i32, Range<u32>) {
        #![allow(clippy::cast_possible_wrap)]

        let base_vertex = self.vertices.len() as i32;
        self.vertices.extend_from_slice(vertices);

        let start = self.indices.len() as u32;
        for i in 0..vertices.len().saturating_sub(2) as u32 {
            self.indices.push(0);
            self.indices.push(i + 1);
            self.indices.push(i + 2);
        }

        (base_vertex, start..self.indices.len() as u32)
    }
}

pub struct PictureBundle {
    pub(crate) indices: wgpu::Buffer,
    pub(crate) vertices: wgpu::Buffer,
    pub(crate) instances: wgpu::Buffer,
}
