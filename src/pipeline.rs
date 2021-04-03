use crate::{
    math::{Offset, Transform},
    renderer::Viewport,
};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [u16; 2],
}

impl Vertex {
    #[inline]
    pub fn new(pos: impl Into<[f32; 2]>, uv: [f32; 2]) -> Self {
        let pos = pos.into();
        let uv = [(uv[0] * 65535.0) as u16, (uv[1] * 65535.0) as u16];
        Self { pos, uv }
    }

    #[inline]
    pub fn transform(self, transform: Transform) -> Self {
        let pos = transform.apply(Offset::from(self.pos)).into();
        Self { pos, ..self }
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
            ..Self::default()
        }
    }
}

pub(crate) struct Pipeline {
    pub image: wgpu::RenderPipeline,

    pub convex: wgpu::RenderPipeline,

    pub fill_stencil: wgpu::RenderPipeline,
    pub fill_quad_non_zero: wgpu::RenderPipeline,
    pub fill_quad_even_odd: wgpu::RenderPipeline,

    pub fringes_non_zero: wgpu::RenderPipeline,
    pub fringes_even_odd: wgpu::RenderPipeline,

    pub stroke_base: wgpu::RenderPipeline,
    pub stroke_stencil: wgpu::RenderPipeline,

    pub image_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, viewport: &Viewport) -> Self {
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

        let image_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("reui image bind group"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
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

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("reui pipeline layout"),
            bind_group_layouts: &[&viewport.layout, &image_layout],
            push_constant_ranges: &[],
        });

        let builder = Builder {
            format,
            device,
            layout,

            module: device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("reui shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
                flags: wgpu::ShaderFlags::all(),
            }),
        };

        macro_rules! stencil_face {
            ($comp:ident, Fail::$fail:ident, Pass::$pass:ident) => {
                wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::$comp,
                    fail_op: wgpu::StencilOperation::$fail,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::$pass,
                }
            };
        }

        const ALWAYS_ZERO: wgpu::StencilFaceState = stencil_face!(Always, Fail::Zero, Pass::Zero);
        const INCR_WRAP: wgpu::StencilFaceState =
            stencil_face!(Always, Fail::Keep, Pass::IncrementWrap);
        const DECR_WRAP: wgpu::StencilFaceState =
            stencil_face!(Always, Fail::Keep, Pass::DecrementWrap);

        Self {
            image: builder.image(stencil_face!(Always, Fail::Keep, Pass::Keep)),
            convex: builder.base(stencil_face!(Always, Fail::Keep, Pass::Keep)),

            fill_stencil: builder.stencil(None, INCR_WRAP, DECR_WRAP),
            fill_quad_non_zero: builder.base(stencil_face!(NotEqual, Fail::Zero, Pass::Zero)),
            fill_quad_even_odd: builder.even_odd(stencil_face!(NotEqual, Fail::Zero, Pass::Zero)),

            stroke_base: builder.base(stencil_face!(Equal, Fail::Keep, Pass::IncrementClamp)),
            stroke_stencil: builder.stencil(Some(wgpu::Face::Back), ALWAYS_ZERO, ALWAYS_ZERO),

            fringes_non_zero: builder.base(stencil_face!(Equal, Fail::Keep, Pass::Keep)),
            fringes_even_odd: builder.even_odd(stencil_face!(Equal, Fail::Keep, Pass::Keep)),

            image_layout,
            sampler,
        }
    }
}

struct Builder<'a> {
    device: &'a wgpu::Device,
    format: wgpu::TextureFormat,
    layout: wgpu::PipelineLayout,
    module: wgpu::ShaderModule,
}

impl<'a> Builder<'a> {
    fn base(&self, stencil: wgpu::StencilFaceState) -> wgpu::RenderPipeline {
        self.pipeline(
            "main",
            wgpu::ColorWrite::ALL,
            Some(wgpu::Face::Back),
            stencil.clone(),
            stencil,
            false,
        )
    }

    fn even_odd(&self, stencil: wgpu::StencilFaceState) -> wgpu::RenderPipeline {
        self.pipeline(
            "main",
            wgpu::ColorWrite::ALL,
            Some(wgpu::Face::Back),
            stencil.clone(),
            stencil,
            true,
        )
    }

    fn image(&self, stencil: wgpu::StencilFaceState) -> wgpu::RenderPipeline {
        self.pipeline(
            "image",
            wgpu::ColorWrite::ALL,
            Some(wgpu::Face::Back),
            stencil.clone(),
            stencil,
            false,
        )
    }

    fn stencil(
        &self,
        cull_mode: Option<wgpu::Face>,
        front: wgpu::StencilFaceState,
        back: wgpu::StencilFaceState,
    ) -> wgpu::RenderPipeline {
        self.pipeline(
            "stencil",
            wgpu::ColorWrite::empty(),
            cull_mode,
            front,
            back,
            false,
        )
    }

    fn pipeline(
        &self,
        entry_point: &str,
        write_mask: wgpu::ColorWrite,
        cull_mode: Option<wgpu::Face>,
        front: wgpu::StencilFaceState,
        back: wgpu::StencilFaceState,
        one_mask: bool,
    ) -> wgpu::RenderPipeline {
        let targets = &[wgpu::ColorTargetState {
            format: self.format,
            write_mask,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
        }];

        let buffers = &[
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Unorm16x2],
            },
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![
                    2 => Float32x4,
                    3 => Unorm8x4,
                    4 => Unorm8x4,
                    5 => Float32x4,
                    6 => Float32x2
                ],
            },
        ];

        let stencil_mask = if one_mask { 0x01 } else { 0xFF };

        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&self.layout),
                vertex: wgpu::VertexState {
                    module: &self.module,
                    entry_point: "vertex",
                    buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.module,
                    entry_point,
                    targets,
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    stencil: wgpu::StencilState {
                        front,
                        back,
                        read_mask: stencil_mask,
                        write_mask: stencil_mask,
                    },
                    clamp_depth: false,
                    bias: wgpu::DepthBiasState::default(),
                }),

                multisample: wgpu::MultisampleState {
                    count: 1,
                    ..wgpu::MultisampleState::default()
                },
            })
    }
}
