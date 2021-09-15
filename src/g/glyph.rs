use {
    crate::glyph_cache::Cache,
    crate::upload_buffer::UploadBuffer,
    crate::viewport::TargetDescriptor,
    glyph_brush::ab_glyph::{point, Rect},
    std::mem,
};
pub use {
    glyph_brush::ab_glyph,
    glyph_brush::{
        BuiltInLineBreaker, Extra, FontId, GlyphCruncher, GlyphPositioner, HorizontalAlign, Layout,
        LineBreak, LineBreaker, Section, SectionGeometry, SectionGlyph, SectionGlyphIter,
        SectionText, Text, VerticalAlign,
    },
};

pub struct Pipeline {
    pub cache: Cache,
    pub pipeline: wgpu::RenderPipeline,
    pub instances: UploadBuffer<Instance>,
    pub current_transform: [[f32; 4]; 4],
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        target: &TargetDescriptor,
        filter_mode: wgpu::FilterMode,
        width: u32,
        height: u32,
    ) -> Self {
        let cache = Cache::new(device, filter_mode, width, height);
        let instances = UploadBuffer::new(
            device,
            wgpu::BufferUsage::VERTEX,
            Instance::INITIAL_AMOUNT,
            "reui glyph instances",
        );

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[&target.layout, &cache.layout],
        });

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("glyph.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("glyph.wgsl").into()),
            flags: wgpu::ShaderFlags::all(),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "glyph_vertex",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: mem::size_of::<Instance>() as u64,
                    step_mode: wgpu::InputStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x4,
                        1 => Float32x4,
                        2 => Float32x4,
                        3 => Float32,
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: target.depth,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Greater,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "glyph_fragment",
                targets: &[wgpu::ColorTargetState {
                    format: target.color,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
        });

        Self {
            cache,
            pipeline,
            instances,
            current_transform: [[0.0; 4]; 4],
        }
    }

    pub fn update_cache(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        offset: [u16; 2],
        size: [u16; 2],
        data: &[u8],
    ) {
        self.cache
            .update(device, staging_belt, encoder, offset, size, data);
    }

    pub fn increase_cache_size(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.cache.resize(device, width, height);
    }

    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        instances: &[Instance],
    ) {
        self.instances
            .upload(encoder, staging_belt, device, instances);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Instance {
    position: [f32; 4],
    texcoord: [f32; 4],
    color: [f32; 4],
    z: f32,
}

impl Instance {
    const INITIAL_AMOUNT: usize = 50_000;

    pub fn zindex(&self) -> f32 {
        self.z
    }

    pub fn from_vertex(
        glyph_brush::GlyphVertex {
            mut tex_coords,
            pixel_coords,
            bounds,
            extra,
        }: glyph_brush::GlyphVertex,
    ) -> Self {
        let gl_bounds = bounds;

        let mut gl_rect = Rect {
            min: point(pixel_coords.min.x as f32, pixel_coords.min.y as f32),
            max: point(pixel_coords.max.x as f32, pixel_coords.max.y as f32),
        };

        // handle overlapping bounds, modify uv_rect to preserve texture aspect
        if gl_rect.max.x > gl_bounds.max.x {
            let old_width = gl_rect.width();
            gl_rect.max.x = gl_bounds.max.x;
            tex_coords.max.x = tex_coords.min.x + tex_coords.width() * gl_rect.width() / old_width;
        }

        if gl_rect.min.x < gl_bounds.min.x {
            let old_width = gl_rect.width();
            gl_rect.min.x = gl_bounds.min.x;
            tex_coords.min.x = tex_coords.max.x - tex_coords.width() * gl_rect.width() / old_width;
        }

        if gl_rect.max.y > gl_bounds.max.y {
            let old_height = gl_rect.height();
            gl_rect.max.y = gl_bounds.max.y;
            tex_coords.max.y =
                tex_coords.min.y + tex_coords.height() * gl_rect.height() / old_height;
        }

        if gl_rect.min.y < gl_bounds.min.y {
            let old_height = gl_rect.height();
            gl_rect.min.y = gl_bounds.min.y;
            tex_coords.min.y =
                tex_coords.max.y - tex_coords.height() * gl_rect.height() / old_height;
        }

        Self {
            position: [gl_rect.min.x, gl_rect.max.y, gl_rect.max.x, gl_rect.min.y],
            texcoord: [
                tex_coords.min.x,
                tex_coords.max.y,
                tex_coords.max.x,
                tex_coords.min.y,
            ],
            color: extra.color,
            z: extra.z,
        }
    }
}
