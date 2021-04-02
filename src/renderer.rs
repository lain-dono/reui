use crate::{canvas::Canvas, picture::Recorder, pipeline::Pipeline};
use std::collections::HashMap;

pub(crate) struct ImageBind {
    pub bind_group: wgpu::BindGroup,
    pub size: wgpu::Extent3d,
}

struct HostImage {
    texture: wgpu::Texture,
    size: wgpu::Extent3d,
}

impl HostImage {
    fn open_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<std::path::Path>,
    ) -> image::ImageResult<Self> {
        let path = path.as_ref();
        let m = image::open(path)?;

        let m = m.to_rgba8();
        let width = m.width();
        let height = m.height();
        let texels = m.into_raw();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: path.to_str(),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * width),
                rows_per_image: None,
            },
            size,
        );

        Ok(Self { texture, size })
    }

    fn bind(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
    ) -> ImageBind {
        let desc = wgpu::TextureViewDescriptor::default();
        let view = self.texture.create_view(&desc);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("image bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
            ],
        });

        let size = self.size;
        ImageBind { bind_group, size }
    }
}

#[derive(Default)]
pub(crate) struct Images {
    pub(crate) images: HashMap<u32, ImageBind>,
    idx: u32,
}

impl std::ops::Index<u32> for Images {
    type Output = ImageBind;
    fn index(&self, index: u32) -> &Self::Output {
        &self.images[&index]
    }
}

impl Images {
    pub fn get(&self, image: u32) -> Option<&ImageBind> {
        self.images.get(&image)
    }

    pub fn create(&mut self, bind: ImageBind) -> u32 {
        let idx = self.idx;
        drop(self.images.insert(idx, bind));
        self.idx += 1;
        idx
    }
}

pub struct Renderer {
    pub(crate) images: Images,
    pub(crate) pipeline: Pipeline,
    pub(crate) viewport: Viewport,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let viewport = Viewport::new(device);
        let pipeline = Pipeline::new(device, format, &viewport);

        Self {
            images: Images::default(),

            pipeline,
            viewport,
        }
    }

    /// # Errors
    pub fn open_image(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<std::path::Path>,
    ) -> image::ImageResult<u32> {
        let image = HostImage::open_rgba(device, queue, path)?;
        let bind = image.bind(device, &self.pipeline.image_layout, &self.pipeline.sampler);
        Ok(self.images.create(bind))
    }

    pub fn begin_frame<'a>(
        &'a mut self,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        scale: f32,
        recorder: &'a mut Recorder,
    ) -> Canvas<'a> {
        self.viewport.upload(queue, width, height, scale);
        Canvas::new(self, recorder, scale)
    }
}

pub(crate) struct Viewport {
    pub layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Viewport {
    fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Viewport bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<f32>() as u64 * 4),
                },
                count: None,
            }],
        });

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Viewport buffer"),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            size: std::mem::size_of::<[f32; 4]>() as u64,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Viewport bind group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            layout,
            buffer,
            bind_group,
        }
    }

    fn upload(&self, queue: &wgpu::Queue, width: u32, height: u32, scale: f32) {
        let w = scale / width as f32;
        let h = scale / height as f32;
        let viewport = [w, h, 0.0, 0.0];
        queue.write_buffer(&self.buffer, 0, crate::picture::cast_slice(&viewport));
    }
}
