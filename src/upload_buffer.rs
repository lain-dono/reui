use core::{marker::PhantomData, mem::size_of, ops::RangeBounds};
use wgpu::util::DeviceExt as _;

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

    pub fn upload_queue(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, data: &[T]) {
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

    pub fn upload_staging(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        staging_belt: &mut wgpu::util::StagingBelt,
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
            staging_belt
                .write_buffer(encoder, &self.buffer, 0, size, device)
                .copy_from_slice(src);
        }
    }
}
