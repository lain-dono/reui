use crate::{Offset, Transform};

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Zeroable, bytemuck::Pod)]
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

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Instance {
    pub paint_mat: [f32; 4],
    pub inner_color: [u8; 4],
    pub outer_color: [u8; 4],

    pub extent: [f32; 2],
    pub radius: f32,
    pub inv_feather: f32,

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

pub struct Pipeline {
    pub view_layout: wgpu::BindGroupLayout,

    pub premultiplied: wgpu::RenderPipeline,
    pub unmultiplied: wgpu::RenderPipeline,
    pub font: wgpu::RenderPipeline,

    pub convex: wgpu::RenderPipeline,
    pub convex_simple: wgpu::RenderPipeline,

    pub fill_stencil: wgpu::RenderPipeline,
    pub fill_quad_non_zero: wgpu::RenderPipeline,
    pub fill_quad_even_odd: wgpu::RenderPipeline,

    pub fringes_non_zero: wgpu::RenderPipeline,
    pub fringes_even_odd: wgpu::RenderPipeline,

    pub stroke_base: wgpu::RenderPipeline,
    pub stroke_stencil: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, image_layout: &wgpu::BindGroupLayout) -> Self {
        let view_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("reui::view_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<[f32; 4]>() as u64),
                },
                count: None,
            }],
        });

        let image_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("reui pipeline layout"),
            bind_group_layouts: &[&view_layout, image_layout],
            push_constant_ranges: &[],
        });

        let paint_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("reui pipeline layout"),
            bind_group_layouts: &[&view_layout],
            push_constant_ranges: &[],
        });

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("reui shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        macro_rules! stencil {
            ($comp:ident, $fail:ident, $pass:ident) => {
                wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::$comp,
                    fail_op: wgpu::StencilOperation::$fail,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                    pass_op: wgpu::StencilOperation::$pass,
                }
            };
        }

        const ALWAYS_ZERO: wgpu::StencilFaceState = stencil!(Always, Zero, Zero);
        const ALWAYS_KEEP: wgpu::StencilFaceState = stencil!(Always, Keep, Keep);
        const NE_ZERO: wgpu::StencilFaceState = stencil!(NotEqual, Zero, Zero);
        const EQ_KEEP: wgpu::StencilFaceState = stencil!(Equal, Keep, Keep);
        const INCR_CLAMP: wgpu::StencilFaceState = stencil!(Equal, Keep, IncrementClamp);
        const INCR_WRAP: wgpu::StencilFaceState = stencil!(Always, Keep, IncrementWrap);
        const DECR_WRAP: wgpu::StencilFaceState = stencil!(Always, Keep, DecrementWrap);

        let premultiplied = Builder::new(
            "vertex_blit",
            "fragment_premultiplied",
            device,
            &image_layout,
            &module,
            false,
        );

        let unmultiplied = Builder::new(
            "vertex_blit",
            "fragment_unmultiplied",
            device,
            &image_layout,
            &module,
            false,
        );

        let font = Builder::new(
            "vertex_blit",
            "fragment_font",
            device,
            &image_layout,
            &module,
            false,
        );

        let main = Builder::new(
            "vertex_main",
            "fragment_main",
            device,
            &paint_layout,
            &module,
            true,
        );

        let convex_simple = Builder::new(
            "vertex_main",
            "fragment_convex_simple",
            device,
            &paint_layout,
            &module,
            true,
        );

        let stencil = Builder::new(
            "vertex_stencil",
            "fragment_stencil",
            device,
            &paint_layout,
            &module,
            false,
        );

        Self {
            view_layout,

            premultiplied: premultiplied.pipeline(true, true, 0xFF, ALWAYS_KEEP, ALWAYS_KEEP),
            unmultiplied: unmultiplied.pipeline(true, true, 0xFF, ALWAYS_KEEP, ALWAYS_KEEP),
            font: font.pipeline(true, true, 0xFF, ALWAYS_KEEP, ALWAYS_KEEP),

            convex: main.pipeline(true, true, 0xFF, ALWAYS_KEEP, ALWAYS_KEEP),
            convex_simple: convex_simple.pipeline(true, true, 0xFF, ALWAYS_KEEP, ALWAYS_KEEP),

            fill_stencil: stencil.pipeline(false, false, 0xFF, INCR_WRAP, DECR_WRAP),
            fill_quad_non_zero: main.pipeline(true, true, 0xFF, NE_ZERO, NE_ZERO),
            fill_quad_even_odd: main.pipeline(true, true, 0x01, NE_ZERO, NE_ZERO),

            stroke_base: main.pipeline(true, true, 0xFF, INCR_CLAMP, INCR_CLAMP),
            stroke_stencil: stencil.pipeline(false, true, 0xFF, ALWAYS_ZERO, ALWAYS_ZERO),

            fringes_non_zero: main.pipeline(true, true, 0xFF, EQ_KEEP, EQ_KEEP),
            fringes_even_odd: main.pipeline(true, true, 0x01, EQ_KEEP, EQ_KEEP),
        }
    }
}

struct Builder<'a> {
    vs_entry_point: &'a str,
    fs_entry_point: &'a str,
    device: &'a wgpu::Device,
    layout: &'a wgpu::PipelineLayout,
    module: &'a wgpu::ShaderModule,
    instances: bool,
}

impl<'a> Builder<'a> {
    fn new(
        vs_entry_point: &'a str,
        fs_entry_point: &'a str,
        device: &'a wgpu::Device,
        layout: &'a wgpu::PipelineLayout,
        module: &'a wgpu::ShaderModule,

        instances: bool,
    ) -> Self {
        Self {
            vs_entry_point,
            fs_entry_point,
            device,
            layout,
            module,
            instances,
        }
    }

    fn pipeline(
        &self,
        write_color: bool,
        back_culling: bool,
        stencil_mask: u32,
        front: wgpu::StencilFaceState,
        back: wgpu::StencilFaceState,
    ) -> wgpu::RenderPipeline {
        let target = wgpu::ColorTargetState {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            write_mask: if write_color {
                wgpu::ColorWrites::all()
            } else {
                wgpu::ColorWrites::empty()
            },
            blend: write_color.then_some(wgpu::BlendState::ALPHA_BLENDING),
        };

        let cull_mode = back_culling.then_some(wgpu::Face::Back);

        let Self {
            device,
            layout,
            module,
            vs_entry_point,
            fs_entry_point,
            instances,
        } = self;

        let vertex_buffer = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Unorm16x2],
        };
        let instance_buffer = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                2 => Float32x4,
                3 => Unorm8x4,
                4 => Unorm8x4,
                5 => Float32x4,
                6 => Float32x2,
            ],
        };

        let buffers = [vertex_buffer, instance_buffer];

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: vs_entry_point,
                buffers: if *instances { &buffers } else { &buffers[..1] },
            },
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point: fs_entry_point,
                targets: &[Some(target)],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode,
                ..wgpu::PrimitiveState::default()
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
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }
}
