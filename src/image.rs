use std::collections::HashMap;

pub struct ImageBind {
    pub bind_group: wgpu::BindGroup,
    pub size: wgpu::Extent3d,
}

pub struct Images<Key> {
    pub images: HashMap<Key, ImageBind>,
    pub layout: wgpu::BindGroupLayout,
    pub default_sampler: wgpu::Sampler,
}

impl<Key: Eq + std::hash::Hash> std::ops::Index<Key> for Images<Key> {
    type Output = ImageBind;
    fn index(&self, index: Key) -> &Self::Output {
        &self.images[&index]
    }
}

impl<Key: Eq + std::hash::Hash> Images<Key> {
    pub fn new(device: &wgpu::Device) -> Self {
        let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
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
            default_sampler,
            layout,
        }
    }

    pub fn get(&self, image: &Key) -> Option<&ImageBind> {
        self.images.get(image)
    }

    pub fn remove(&mut self, key: Key) -> Option<ImageBind> {
        self.images.remove(&key)
    }

    pub fn insert(&mut self, key: Key, bind: ImageBind) -> Option<ImageBind> {
        self.images.insert(key, bind)
    }

    pub fn bind(
        &self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        sampler: Option<&wgpu::Sampler>,
    ) -> wgpu::BindGroup {
        let sampler = sampler.unwrap_or(&self.default_sampler);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        })
    }
}
