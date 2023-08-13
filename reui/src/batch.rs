use crate::pipeline::{Instance, Vertex};
use core::{marker::PhantomData, mem::size_of, ops::RangeBounds};
use std::ops::Range;
use wgpu::util::DeviceExt as _;

#[derive(Default)]
pub struct Batch {
    instances: Vec<Instance>,
    indices: Vec<u32>,
    vertices: Vec<Vertex>,
}

impl std::ops::Index<i32> for Batch {
    type Output = Vertex;
    #[inline]
    fn index(&self, index: i32) -> &Self::Output {
        &self.vertices[index as usize]
    }
}

impl std::ops::IndexMut<i32> for Batch {
    #[inline]
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        &mut self.vertices[index as usize]
    }
}

impl Batch {
    #[inline]
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.instances.clear();
    }

    #[inline(always)]
    pub fn emit(&mut self, pos: impl Into<[f32; 2]>, uv: [f32; 2]) {
        self.vertices.push(Vertex::new(pos, uv))
    }

    #[inline]
    pub fn instance(&mut self, instance: Instance) -> u32 {
        let index = self.instances.len();
        self.instances.push(instance);
        index as u32
    }

    #[inline]
    pub fn base_vertex(&self) -> i32 {
        self.vertices.len() as i32
    }

    #[inline]
    pub fn base_index(&self) -> u32 {
        self.indices.len() as u32
    }

    #[inline]
    pub fn push_strip(&mut self, offset: u32, vertices: &[Vertex]) -> Range<u32> {
        let start = self.base_index();
        self.vertices.extend_from_slice(vertices);
        self.strip(offset, vertices.len() as i32);
        start..self.base_index()
    }

    #[inline]
    pub fn strip(&mut self, offset: u32, num_vertices: i32) {
        for i in 0..num_vertices.saturating_sub(2) as u32 {
            let (a, b) = if 0 == i % 2 { (1, 2) } else { (2, 1) };
            self.indices.push(offset + i);
            self.indices.push(offset + i + a);
            self.indices.push(offset + i + b);
        }
    }

    #[inline]
    pub fn fan(&mut self, offset: u32, num_vertices: i32) {
        for i in 0..num_vertices.saturating_sub(2) as u32 {
            self.indices.push(offset);
            self.indices.push(offset + i + 1);
            self.indices.push(offset + i + 2);
        }
    }
}

#[cfg_attr(feature = "bevy", derive(bevy::prelude::Resource))]
pub struct GpuBatch {
    pub indices: UploadBuffer<u32>,
    pub vertices: UploadBuffer<Vertex>,
    pub instances: UploadBuffer<Instance>,
}

impl GpuBatch {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            indices: UploadBuffer::new(device, wgpu::BufferUsages::INDEX, 128),
            vertices: UploadBuffer::new(device, wgpu::BufferUsages::VERTEX, 128),
            instances: UploadBuffer::new(device, wgpu::BufferUsages::VERTEX, 128),
        }
    }

    pub fn queue(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, batch: &Batch) {
        self.indices.queue(queue, device, &batch.indices);
        self.vertices.queue(queue, device, &batch.vertices);
        self.instances.queue(queue, device, &batch.instances);
    }

    pub fn staging(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        device: &wgpu::Device,
        batch: &Batch,
    ) {
        self.indices.staging(encoder, belt, device, &batch.indices);
        self.vertices
            .staging(encoder, belt, device, &batch.vertices);
        self.instances
            .staging(encoder, belt, device, &batch.instances);
    }

    pub fn bind<'rpass>(&'rpass self, rpass: &mut impl wgpu::util::RenderEncoder<'rpass>) {
        rpass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
        rpass.set_vertex_buffer(0, self.vertices.slice(..));
        rpass.set_vertex_buffer(1, self.instances.slice(..));
    }
}

pub struct UploadBuffer<T> {
    buffer: wgpu::Buffer,
    usage: wgpu::BufferUsages,
    capacity: usize,
    marker: PhantomData<T>,
}

impl<T> AsRef<wgpu::Buffer> for UploadBuffer<T> {
    fn as_ref(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

impl<T: bytemuck::Pod> UploadBuffer<T> {
    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, capacity: usize) -> Self {
        debug_assert_ne!(size_of::<T>(), 0);

        Self {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: size_of::<T>() as u64 * capacity as u64,
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            usage,
            capacity,
            marker: PhantomData,
        }
    }

    pub fn init(device: &wgpu::Device, usage: wgpu::BufferUsages, data: &[T]) -> Self {
        debug_assert_ne!(size_of::<T>(), 0);

        Self {
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(data),
                usage: usage | wgpu::BufferUsages::COPY_DST,
            }),
            usage,
            capacity: data.len(),
            marker: PhantomData,
        }
    }

    pub fn slice<S: RangeBounds<wgpu::BufferAddress>>(&self, bounds: S) -> wgpu::BufferSlice {
        self.buffer.slice(bounds)
    }

    pub fn queue(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, data: &[T]) {
        if data.is_empty() {
            return;
        }

        if data.len() > self.capacity {
            self.capacity = data.len() * 2;
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: size_of::<T>() as u64 * self.capacity as u64,
                usage: self.usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        let src = bytemuck::cast_slice(data);
        queue.write_buffer(&self.buffer, 0, src);
    }

    pub fn staging(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        device: &wgpu::Device,
        data: &[T],
    ) {
        if data.is_empty() {
            return;
        }

        if data.len() > self.capacity {
            self.capacity = data.len() * 2;
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: size_of::<T>() as u64 * self.capacity as u64,
                usage: self.usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        let src = bytemuck::cast_slice(data);
        if let Some(size) = wgpu::BufferSize::new(src.len() as u64) {
            belt.write_buffer(encoder, &self.buffer, 0, size, device)
                .copy_from_slice(src);
        }
    }
}
