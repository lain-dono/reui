use crate::{
    canvas::Canvas,
    picture::Recorder,
    pipeline::{BatchUpload, Pipeline},
    viewport::Viewport,
};
use std::collections::HashMap;

pub struct ImageBind {
    pub bind_group: wgpu::BindGroup,
    pub size: wgpu::Extent3d,
}

pub struct HostImage {
    pub texture: wgpu::Texture,
    pub size: wgpu::Extent3d,
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
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
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
}

pub struct Images {
    pub(crate) images: HashMap<u32, ImageBind>,
    idx: u32,

    pub layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
}

impl std::ops::Index<u32> for Images {
    type Output = ImageBind;
    fn index(&self, index: u32) -> &Self::Output {
        &self.images[&index]
    }
}

impl Images {
    pub fn new(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("reui default sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("reui image bind group"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        Self {
            images: HashMap::default(),
            idx: 0,
            sampler,
            layout,
        }
    }

    pub fn get(&self, image: u32) -> Option<&ImageBind> {
        self.images.get(&image)
    }

    pub fn create(&mut self, bind: ImageBind) -> u32 {
        let idx = self.idx;
        drop(self.images.insert(idx, bind));
        self.idx += 1;
        idx
    }

    pub fn bind_texture_view(
        &self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("image bind group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(view),
                },
            ],
        })
    }

    /// # Errors
    pub fn open(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<std::path::Path>,
    ) -> image::ImageResult<u32> {
        let image = HostImage::open_rgba(device, queue, path)?;

        let desc = wgpu::TextureViewDescriptor::default();
        let view = image.texture.create_view(&desc);
        let bind_group = self.bind_texture_view(device, &view);

        let size = image.size;

        Ok(self.create(ImageBind { bind_group, size }))
    }
}
