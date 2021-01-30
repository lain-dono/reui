use crate::{
    canvas::Canvas,
    path::Path,
    picture::{Call, Instance, PictureBundle, PictureRecorder, Vertex},
    tessellator::Tessellator,
};
use std::collections::HashMap;

const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;

macro_rules! stencil_face {
    ($name:ident, $comp:ident, $fail:ident, $pass:ident) => {
        const $name: wgpu::StencilFaceState = stencil_face!($comp, $fail, $pass);
    };

    ($comp:ident, $fail:ident, $pass:ident) => {
        wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::$comp,
            fail_op: wgpu::StencilOperation::$fail,
            depth_fail_op: wgpu::StencilOperation::$fail,
            pass_op: wgpu::StencilOperation::$pass,
        }
    };
}

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
        let m = image::open(path)?;

        let m = m.to_rgba8();
        let width = m.width();
        let height = m.height();
        let texels = m.into_raw();

        let size = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED,
        });

        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &texels,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * width,
                rows_per_image: 0,
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
            label: None,
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

pub struct Renderer {
    pub(crate) path: Path,
    pub(crate) tess: Tessellator,
    pub(crate) images: HashMap<u32, ImageBind>,
    images_idx: u32,
    pipeline: Pipeline,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let pipeline = Pipeline::new(device, format);

        let images = HashMap::default();

        Self {
            tess: Tessellator::new(),
            path: Path::new(),

            images,
            images_idx: 1,

            pipeline,
        }
    }

    pub fn open_image(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<std::path::Path>,
    ) -> image::ImageResult<u32> {
        let image = HostImage::open_rgba(device, queue, path)?;
        let bind = image.bind(device, &self.pipeline.image_layout, &self.pipeline.sampler);
        Ok(self.create_image(bind))
    }

    fn create_image(&mut self, bind: ImageBind) -> u32 {
        let idx = self.images_idx;
        let _ = self.images.insert(idx, bind);
        self.images_idx += 1;
        idx
    }

    pub fn begin_frame<'a>(
        &'a mut self,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        scale: f32,
        picture: &'a mut PictureRecorder,
    ) -> Canvas<'a> {
        let w = scale / width as f32;
        let h = scale / height as f32;
        let viewport = [w, h, 0.0, 0.0];

        queue.write_buffer(
            &self.pipeline.buffer,
            0,
            crate::picture::cast_slice(&viewport),
        );

        self.tess.set_scale(scale);
        Canvas::new(self, picture, scale)
    }

    pub fn draw_picture<'rpass>(
        &'rpass self,
        rpass: &mut wgpu::RenderPass<'rpass>,
        picture: &'rpass PictureRecorder,
        bundle: &'rpass PictureBundle,
    ) {
        rpass.set_stencil_reference(0);
        rpass.set_bind_group(0, &self.pipeline.bind_group, &[]);
        rpass.set_vertex_buffer(0, bundle.vertices.slice(..));
        rpass.set_vertex_buffer(1, bundle.instances.slice(..));

        for call in picture.calls.iter().cloned() {
            match call {
                Call::Convex { idx, path } => {
                    let paths = &picture.paths[path];
                    let start = idx;
                    let end = idx + 1;

                    rpass.set_pipeline(&self.pipeline.convex);
                    for path in paths {
                        rpass.draw(path.fill.clone(), start..end);
                        rpass.draw(path.stroke.clone(), start..end); // fringes
                    }
                }
                Call::Fill { idx, path, quad } => {
                    let paths = &picture.paths[path];
                    let instances = idx..idx + 1;

                    rpass.set_pipeline(&self.pipeline.fill_stencil);
                    for path in paths {
                        rpass.draw(path.fill.clone(), instances.clone());
                    }

                    let instances = idx + 1..idx + 2;

                    // Draw fringes
                    rpass.set_pipeline(&self.pipeline.fringes);
                    for path in paths {
                        rpass.draw(path.stroke.clone(), instances.clone());
                    }

                    // Draw fill
                    rpass.set_pipeline(&self.pipeline.fill_quad);
                    rpass.draw(quad..quad + 4, instances.clone());
                }
                Call::Stroke { idx, path } => {
                    let stroke = &picture.strokes[path];
                    let instances = idx..idx + 1;

                    // Fill the stroke base without overlap
                    rpass.set_pipeline(&self.pipeline.stroke_base);
                    for path in stroke {
                        rpass.draw(path.clone(), instances.clone());
                    }

                    let instances = idx + 1..idx + 2;

                    // Draw anti-aliased pixels.
                    rpass.set_pipeline(&self.pipeline.fringes);
                    for path in stroke {
                        rpass.draw(path.clone(), instances.clone());
                    }

                    // Clear stencil buffer
                    rpass.set_pipeline(&self.pipeline.stroke_stencil);
                    for path in stroke {
                        rpass.draw(path.clone(), instances.clone());
                    }
                }

                Call::Image { idx, vtx, image } => {
                    if let Some(image) = self.images.get(&image) {
                        rpass.set_bind_group(1, &image.bind_group, &[]);
                        rpass.set_pipeline(&self.pipeline.image);
                        rpass.draw(vtx, idx..idx + 1);
                    }
                }
            }
        }
    }
}

struct Pipeline {
    image: wgpu::RenderPipeline,

    convex: wgpu::RenderPipeline,
    fringes: wgpu::RenderPipeline,

    fill_stencil: wgpu::RenderPipeline,
    fill_quad: wgpu::RenderPipeline,

    stroke_base: wgpu::RenderPipeline,
    stroke_stencil: wgpu::RenderPipeline,

    image_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,

    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Pipeline {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let viewport_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("VG bind group layout"),
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

        let (buffer, bind_group) = {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Viewport buffer"),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
                size: std::mem::size_of::<[f32; 4]>() as u64,
                mapped_at_creation: false,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Viewport bind group"),
                layout: &viewport_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

            (buffer, bind_group)
        };

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        let image_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("VG bind group layout"),
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
            label: None,
            bind_group_layouts: &[&viewport_layout, &image_layout],
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

        stencil_face!(ALWAYS_ZERO, Always, Zero, Zero);
        stencil_face!(INCR_WRAP, Always, Keep, IncrementWrap);
        stencil_face!(DECR_WRAP, Always, Keep, DecrementWrap);

        Self {
            image: builder.image(stencil_face!(Always, Keep, Keep)),

            convex: builder.base(stencil_face!(Always, Keep, Keep)),
            fringes: builder.base(stencil_face!(Equal, Keep, Keep)),

            fill_stencil: builder.stencil(wgpu::CullMode::None, INCR_WRAP, DECR_WRAP),
            fill_quad: builder.base(stencil_face!(NotEqual, Zero, Zero)),

            stroke_base: builder.base(stencil_face!(Equal, Keep, IncrementClamp)),
            stroke_stencil: builder.stencil(wgpu::CullMode::Back, ALWAYS_ZERO, ALWAYS_ZERO),

            image_layout,
            sampler,

            buffer,
            bind_group,
        }
    }
}

struct Builder<'a> {
    format: wgpu::TextureFormat,
    device: &'a wgpu::Device,
    layout: wgpu::PipelineLayout,
    module: wgpu::ShaderModule,
}

impl<'a> Builder<'a> {
    fn base(&self, stencil: wgpu::StencilFaceState) -> wgpu::RenderPipeline {
        println!("pipeline base");

        let topology = wgpu::PrimitiveTopology::TriangleStrip;
        let (front, back) = (stencil.clone(), stencil);
        let (write_mask, cull_mode) = (wgpu::ColorWrite::ALL, wgpu::CullMode::Back);
        self.pipeline("main", write_mask, cull_mode, front, back, topology)
    }

    fn stencil(
        &self,
        cull_mode: wgpu::CullMode,
        front: wgpu::StencilFaceState,
        back: wgpu::StencilFaceState,
    ) -> wgpu::RenderPipeline {
        println!("pipeline stencil");

        let topology = wgpu::PrimitiveTopology::TriangleStrip;
        let write_mask = wgpu::ColorWrite::empty();
        self.pipeline("stencil", write_mask, cull_mode, front, back, topology)
    }

    fn image(&self, stencil: wgpu::StencilFaceState) -> wgpu::RenderPipeline {
        println!("pipeline image");

        let topology = wgpu::PrimitiveTopology::TriangleList;
        let (front, back) = (stencil.clone(), stencil);
        let (write_mask, cull_mode) = (wgpu::ColorWrite::ALL, wgpu::CullMode::Back);
        self.pipeline("image", write_mask, cull_mode, front, back, topology)
    }

    fn pipeline(
        &self,
        entry_point: &str,
        write_mask: wgpu::ColorWrite,
        cull_mode: wgpu::CullMode,
        front: wgpu::StencilFaceState,
        back: wgpu::StencilFaceState,
        topology: wgpu::PrimitiveTopology,
    ) -> wgpu::RenderPipeline {
        let straight_alpha_blend = wgpu::ColorTargetState {
            format: self.format,
            write_mask,
            color_blend: wgpu::BlendState {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha_blend: wgpu::BlendState {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        };

        let targets = &[straight_alpha_blend];

        let buffers = &[
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float2, 1 => Ushort2Norm],
            },
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![
                    2 => Float4,
                    3 => Uchar4Norm,
                    4 => Uchar4Norm,
                    5 => Float4,
                    6 => Float2
                ],
            },
        ];

        let desc = wgpu::RenderPipelineDescriptor {
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
                topology,
                cull_mode,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front,
                    back,
                    read_mask: 0xFF,
                    write_mask: 0xFF,
                },
                clamp_depth: false,
                bias: wgpu::DepthBiasState::default(),
            }),

            multisample: wgpu::MultisampleState::default(),
        };

        self.device.create_render_pipeline(&desc)
    }
}
