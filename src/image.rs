use std::collections::HashMap;
use std::ops::Index;

pub struct ImageBind {
    pub bind: wgpu::BindGroup,
    pub size: wgpu::Extent3d,
}

pub struct Images<Key> {
    pub images: HashMap<Key, ImageBind>,
    pub layout: wgpu::BindGroupLayout,
    pub default_sampler: wgpu::Sampler,
}

impl<Key: Eq + std::hash::Hash> Index<&Key> for Images<Key> {
    type Output = ImageBind;
    fn index(&self, index: &Key) -> &Self::Output {
        self.images.index(index)
    }
}

impl<Key: Eq + std::hash::Hash> Images<Key> {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            images: HashMap::default(),
            layout: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            }),
            default_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
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
            }),
        }
    }

    pub fn get(&self, image: &Key) -> Option<&ImageBind> {
        self.images.get(image)
    }

    pub fn remove(&mut self, key: Key) -> Option<ImageBind> {
        self.images.remove(&key)
    }

    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: Key,
        label: Option<&str>,
        width: u32,
        height: u32,
        data: &[u8],
        sampler: Option<&wgpu::Sampler>,
    ) -> Option<ImageBind> {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let copy_texture = wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        };

        let data_layout = wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(4 * width),
            rows_per_image: None,
        };

        queue.write_texture(copy_texture, data, data_layout, size);

        let desc = wgpu::TextureViewDescriptor::default();
        let view = texture.create_view(&desc);

        self.insert(device, key, &view, size, sampler)
    }

    pub fn insert(
        &mut self,
        device: &wgpu::Device,
        key: Key,
        view: &wgpu::TextureView,
        size: wgpu::Extent3d,
        sampler: Option<&wgpu::Sampler>,
    ) -> Option<ImageBind> {
        let sampler = sampler.unwrap_or(&self.default_sampler);

        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("image bind group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(view),
                },
            ],
        });

        self.images.insert(key, ImageBind { bind, size })
    }
}
