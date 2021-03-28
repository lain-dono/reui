use crate::{
    math::{Offset, Transform},
    paint::Stroke,
    valloc::VecAlloc,
};
use std::ops::Range;

pub(crate) fn cast_slice<T: Sized>(slice: &[T]) -> &[u8] {
    use std::{mem::size_of, slice::from_raw_parts};
    unsafe { from_raw_parts(slice.as_ptr().cast(), slice.len() * size_of::<T>()) }
}

#[derive(Clone)]
pub enum DrawCall {
    Convex { start: u32, path: Range<u32> },

    FillStencil { start: u32, path: Range<u32> },
    FillQuad { start: u32, quad: Range<u32> },

    StrokeBase { start: u32, path: Range<u32> },
    StrokeStencil { start: u32, path: Range<u32> },

    Fringes { start: u32, path: Range<u32> },

    SelectImage { image: u32 },

    Image { start: u32, vertices: Range<u32> },
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
    pub(crate) vertices: VecAlloc<Vertex>,
    instances: VecAlloc<Instance>,
    pub(crate) ranges: VecAlloc<Range<u32>>,
}

impl PictureRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.calls.clear();
        self.vertices.clear();
        self.instances.clear();
        self.ranges.clear();
    }

    pub fn call(&mut self, call: DrawCall) {
        self.calls.push(call);
    }

    pub fn push_instance(&mut self, instance: Instance) -> u32 {
        self.instances.push(instance)
    }

    pub fn alloc_ranges(&mut self, count: usize) -> (Range<u32>, &mut [Range<u32>]) {
        self.ranges.alloc_with(count, Default::default)
    }

    pub fn build(&mut self, device: &wgpu::Device) -> PictureBundle {
        use wgpu::util::DeviceExt;

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
            vertices,
            instances,
        }
    }
}

pub struct PictureBundle {
    pub(crate) vertices: wgpu::Buffer,
    pub(crate) instances: wgpu::Buffer,
}
