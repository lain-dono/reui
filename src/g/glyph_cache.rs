fn create_sampler(device: &wgpu::Device, filter_mode: wgpu::FilterMode) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: filter_mode,
        min_filter: filter_mode,
        mipmap_filter: filter_mode,
        ..wgpu::SamplerDescriptor::default()
    })
}

fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("glyph::Cache"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED,
        mip_level_count: 1,
        sample_count: 1,
    })
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("glyph::Pipeline cache bind_group"),
        layout,
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

pub struct Cache {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    upload_buffer: wgpu::Buffer,
    upload_buffer_size: u64,

    pub(crate) layout: wgpu::BindGroupLayout,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl Cache {
    const INITIAL_UPLOAD_BUFFER_SIZE: u64 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64 * 100;

    pub fn new(
        device: &wgpu::Device,
        filter_mode: wgpu::FilterMode,
        width: u32,
        height: u32,
    ) -> Self {
        let texture = create_texture(device, width, height);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let upload_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("glyph::Cache upload buffer"),
            size: Self::INITIAL_UPLOAD_BUFFER_SIZE,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
            mapped_at_creation: false,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("glyph::Pipeline image_layout uniforms"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: false,
                        comparison: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let sampler = create_sampler(device, filter_mode);
        let bind_group = create_bind_group(device, &layout, &sampler, &view);

        Self {
            texture,
            view,
            sampler,
            upload_buffer,
            upload_buffer_size: Self::INITIAL_UPLOAD_BUFFER_SIZE,
            layout,
            bind_group,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.texture = create_texture(device, width, height);
        self.view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.upload_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("glyph::Cache upload buffer"),
            size: Self::INITIAL_UPLOAD_BUFFER_SIZE,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
            mapped_at_creation: false,
        });
        self.bind_group = create_bind_group(device, &self.layout, &self.sampler, &self.view)
    }

    /// # Panics
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        offset: [u16; 2],
        size: [u16; 2],
        data: &[u8],
    ) {
        let width = size[0] as usize;
        let height = size[1] as usize;

        // It is a webgpu requirement that:
        //  BufferCopyView.layout.bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0
        // So we calculate padded_width by rounding width
        // up to the next multiple of wgpu::COPY_BYTES_PER_ROW_ALIGNMENT.
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_width_padding = (align - width % align) % align;
        let padded_width = width + padded_width_padding;

        let padded_data_size = (padded_width * height) as u64;

        if self.upload_buffer_size < padded_data_size {
            self.upload_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("glyph::Cache upload buffer"),
                size: padded_data_size,
                usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::COPY_SRC,
                mapped_at_creation: false,
            });

            self.upload_buffer_size = padded_data_size;
        }

        let mut padded_data = staging_belt.write_buffer(
            encoder,
            &self.upload_buffer,
            0,
            wgpu::BufferSize::new(padded_data_size).unwrap(),
            device,
        );

        for row in 0..height {
            padded_data[row * padded_width..row * padded_width + width]
                .copy_from_slice(&data[row * width..(row + 1) * width])
        }

        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &self.upload_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(padded_width as u32),
                    rows_per_image: None,
                },
            },
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: u32::from(offset[0]),
                    y: u32::from(offset[1]),
                    z: 0,
                },
            },
            wgpu::Extent3d {
                width: size[0] as u32,
                height: size[1] as u32,
                depth_or_array_layers: 1,
            },
        );
    }
}
