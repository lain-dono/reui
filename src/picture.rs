use crate::{
    math::{Offset, Transform},
    paint::Stroke,
    valloc::VecAlloc,
};
use std::ops::Range;

pub(crate) fn cast_slice<T: Sized>(slice: &[T]) -> &[u8] {
    use std::{mem::size_of, slice::from_raw_parts};
    unsafe { from_raw_parts(slice.as_ptr() as *const u8, slice.len() * size_of::<T>()) }
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
    Image {
        idx: u32,
        vtx: Range<u32>,
        image: u32,
    },
}

#[repr(C)]
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

    #[inline(always)]
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
            ..Default::default()
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
    pub calls: Vec<Call>,
    pub vertices: VecAlloc<Vertex>,
    pub instances: VecAlloc<Instance>,

    pub paths: VecAlloc<RawPath>,
    pub strokes: VecAlloc<Range<u32>>,
}

impl PictureRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.calls.clear();
        self.vertices.clear();
        self.instances.clear();

        self.paths.clear();
        self.strokes.clear();
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
