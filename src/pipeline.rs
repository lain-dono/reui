use crate::{
    geom::{Offset, Transform},
    image::Images,
    upload_buffer::UploadBuffer,
    viewport::Viewport,
};
use std::ops::Range;

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

#[repr(C, align(4))]
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

#[derive(Default)]
pub struct Batch {
    instances: Vec<Instance>,
    indices: Vec<u32>,
    vertices: Vec<Vertex>,
}

impl std::ops::Index<i32> for Batch {
    type Output = Vertex;
    #[inline]
    fn index(&self, index: i32) -> &Self::Output {
        &self.vertices[index as usize]
    }
}

impl std::ops::IndexMut<i32> for Batch {
    #[inline]
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        &mut self.vertices[index as usize]
    }
}

#[allow(clippy::cast_possible_wrap)]
impl Batch {
    #[inline]
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.instances.clear();
    }

    #[inline]
    pub fn push(&mut self, vertex: Vertex) {
        self.vertices.push(vertex);
    }

    #[inline]
    pub(crate) fn instance(&mut self, instance: Instance) -> u32 {
        let index = self.instances.len();
        self.instances.push(instance);
        index as u32
    }

    #[inline]
    pub(crate) fn base_vertex(&self) -> i32 {
        self.vertices.len() as i32
    }

    #[inline]
    pub(crate) fn base_index(&self) -> u32 {
        self.indices.len() as u32
    }

    #[inline]
    pub(crate) fn push_strip(&mut self, offset: u32, vertices: &[Vertex]) -> Range<u32> {
        let start = self.base_index();
        self.vertices.extend_from_slice(vertices);
        self.strip(offset, vertices.len() as i32);
        start..self.base_index()
    }

    #[inline]
    pub(crate) fn strip(&mut self, offset: u32, num_vertices: i32) {
        for i in 0..num_vertices.saturating_sub(2) as u32 {
            let (a, b) = if 0 == i % 2 { (1, 2) } else { (2, 1) };
            self.indices.push(offset + i);
            self.indices.push(offset + i + a);
            self.indices.push(offset + i + b);
        }
    }

    #[inline]
    pub(crate) fn fan(&mut self, offset: u32, num_vertices: i32) {
        for i in 0..num_vertices.saturating_sub(2) as u32 {
            self.indices.push(offset);
            self.indices.push(offset + i + 1);
            self.indices.push(offset + i + 2);
        }
    }
}

pub struct BatchUpload {
    pub indices: UploadBuffer<u32>,
    pub vertices: UploadBuffer<Vertex>,
    pub instances: UploadBuffer<Instance>,
}

impl BatchUpload {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            indices: UploadBuffer::new(device, wgpu::BufferUsages::INDEX, 128),
            vertices: UploadBuffer::new(device, wgpu::BufferUsages::VERTEX, 128),
            instances: UploadBuffer::new(device, wgpu::BufferUsages::VERTEX, 128),
        }
    }

    pub fn upload_queue(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, batch: &Batch) {
        self.indices.upload_queue(queue, device, &batch.indices);
        self.vertices.upload_queue(queue, device, &batch.vertices);
        self.instances.upload_queue(queue, device, &batch.instances);
    }

    pub fn upload_staging(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        belt: &mut wgpu::util::StagingBelt,
        device: &wgpu::Device,
        batch: &Batch,
    ) {
        self.indices
            .upload_staging(encoder, belt, device, &batch.indices);
        self.vertices
            .upload_staging(encoder, belt, device, &batch.vertices);
        self.instances
            .upload_staging(encoder, belt, device, &batch.instances);
    }

    pub fn bind<'rpass>(&'rpass self, rpass: &mut impl wgpu::util::RenderEncoder<'rpass>) {
        rpass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
        rpass.set_vertex_buffer(0, self.vertices.slice(..));
        rpass.set_vertex_buffer(1, self.instances.slice(..));
    }
}

pub struct Pipeline {
    pub view_layout: wgpu::BindGroupLayout,

    pub image: wgpu::RenderPipeline,

    pub convex: wgpu::RenderPipeline,

    pub fill_stencil: wgpu::RenderPipeline,
    pub fill_quad_non_zero: wgpu::RenderPipeline,
    pub fill_quad_even_odd: wgpu::RenderPipeline,

    pub fringes_non_zero: wgpu::RenderPipeline,
    pub fringes_even_odd: wgpu::RenderPipeline,

    pub stroke_base: wgpu::RenderPipeline,
    pub stroke_stencil: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, images: &Images) -> Self {
        let view_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("reui::view_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<f32>() as u64 * 4),
                },
                count: None,
            }],
        });

        let image_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("reui pipeline layout"),
            bind_group_layouts: &[&view_layout, &images.layout],
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

        let image = Builder::new("image", device, &image_layout, &module);
        let main = Builder::new("main", device, &paint_layout, &module);
        let stencil = Builder::new("stencil", device, &paint_layout, &module);

        Self {
            view_layout,

            image: image.pipeline(true, true, 0xFF, ALWAYS_KEEP, ALWAYS_KEEP),
            convex: main.pipeline(true, true, 0xFF, ALWAYS_KEEP, ALWAYS_KEEP),

            fill_stencil: stencil.pipeline(false, false, 0xFF, INCR_WRAP, DECR_WRAP),
            fill_quad_non_zero: main.pipeline(true, true, 0xFF, NE_ZERO, NE_ZERO),
            fill_quad_even_odd: main.pipeline(true, true, 0x01, NE_ZERO, NE_ZERO),

            stroke_base: main.pipeline(true, true, 0xFF, INCR_CLAMP, INCR_CLAMP),
            stroke_stencil: stencil.pipeline(false, true, 0xFF, ALWAYS_ZERO, ALWAYS_ZERO),

            fringes_non_zero: main.pipeline(true, true, 0xFF, EQ_KEEP, EQ_KEEP),
            fringes_even_odd: main.pipeline(true, true, 0x01, EQ_KEEP, EQ_KEEP),
        }
    }

    pub fn create_viewport(
        &self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        scale: f32,
    ) -> Viewport {
        Viewport::new(device, &self.view_layout, width, height, scale)
    }
}

struct Builder<'a> {
    entry_point: &'a str,
    device: &'a wgpu::Device,
    layout: &'a wgpu::PipelineLayout,
    module: &'a wgpu::ShaderModule,
}

impl<'a> Builder<'a> {
    fn new(
        entry_point: &'a str,
        device: &'a wgpu::Device,
        layout: &'a wgpu::PipelineLayout,
        module: &'a wgpu::ShaderModule,
    ) -> Self {
        Self {
            entry_point,
            device,
            layout,
            module,
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

        let buffers = &[
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Unorm16x2],
            },
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![
                    2 => Float32x4,
                    3 => Unorm8x4,
                    4 => Unorm8x4,
                    5 => Float32x4,
                    6 => Float32x2,
                ],
            },
        ];

        let Self {
            device,
            layout,
            module,
            entry_point,
        } = self;

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(entry_point),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module,
                entry_point: "vertex",
                buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module,
                entry_point,
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

            multisample: wgpu::MultisampleState {
                count: 1,
                ..wgpu::MultisampleState::default()
            },
            multiview: None,
        })
    }
}
