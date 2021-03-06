use core::{marker::PhantomData, mem::size_of, ops::RangeBounds, slice::from_raw_parts};
use wgpu::util::DeviceExt as _;

pub fn bytes_of<T>(data: &[T]) -> &[u8] {
    debug_assert_ne!(size_of::<T>(), 0);
    unsafe { from_raw_parts(data.as_ptr().cast(), data.len() * size_of::<T>()) }
}

pub struct UploadBuffer<T> {
    buffer: wgpu::Buffer,
    usage: wgpu::BufferUsage,
    len: usize,
    capacity: usize,
    label: &'static str,
    marker: PhantomData<T>,
}

impl<T> AsRef<wgpu::Buffer> for UploadBuffer<T> {
    fn as_ref(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

impl<T> UploadBuffer<T> {
    pub fn new(
        device: &wgpu::Device,
        usage: wgpu::BufferUsage,
        capacity: usize,
        label: &'static str,
    ) -> Self {
        assert_ne!(size_of::<T>(), 0);

        Self {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: size_of::<T>() as u64 * capacity as u64,
                usage: usage | wgpu::BufferUsage::COPY_DST,
                mapped_at_creation: false,
            }),
            usage,
            capacity,
            len: 0,
            label,
            marker: PhantomData,
        }
    }

    pub fn init(
        device: &wgpu::Device,
        usage: wgpu::BufferUsage,
        data: &[T],
        label: &'static str,
    ) -> Self {
        assert_ne!(size_of::<T>(), 0);

        Self {
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytes_of(data),
                usage: usage | wgpu::BufferUsage::COPY_DST,
            }),
            usage,
            capacity: data.len(),
            len: 0,
            label,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn slice<S: RangeBounds<wgpu::BufferAddress>>(&self, bounds: S) -> wgpu::BufferSlice {
        self.buffer.slice(bounds)
    }

    pub fn upload(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        staging_belt: &mut wgpu::util::StagingBelt,
        device: &wgpu::Device,
        data: &[T],
    ) {
        if data.is_empty() {
            self.len = 0;
            return;
        }

        if data.len() > self.capacity {
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label),
                size: size_of::<T>() as u64 * data.len() as u64,
                usage: self.usage | wgpu::BufferUsage::COPY_DST,
                mapped_at_creation: false,
            });
            self.capacity = data.len();
        }

        let src = bytes_of(data);
        if let Some(size) = wgpu::BufferSize::new(src.len() as u64) {
            staging_belt
                .write_buffer(encoder, &self.buffer, 0, size, device)
                .copy_from_slice(src);
        }

        self.len = data.len();
    }
}
