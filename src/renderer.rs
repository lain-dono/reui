use crate::{
    canvas::Canvas,
    paint::Uniforms,
    path::Path,
    picture::{Call, Picture, Vertex},
    shader::Shader,
    tessellator::Tessellator,
};
use std::collections::HashMap;

const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;

macro_rules! stencil_face {
    ($name:ident, $comp:ident, $fail:ident, $pass:ident) => {
        const $name: wgpu::StencilStateFaceDescriptor = stencil_face!($comp, $fail, $pass);
    };

    ($comp:ident, $fail:ident, $pass:ident) => {
        wgpu::StencilStateFaceDescriptor {
            compare: wgpu::CompareFunction::$comp,
            fail_op: wgpu::StencilOperation::$fail,
            depth_fail_op: wgpu::StencilOperation::$fail,
            pass_op: wgpu::StencilOperation::$pass,
        }
    };
}

#[inline(always)]
fn cast_slice<T>(data: &[T]) -> &[u8] {
    use std::{mem::size_of, slice::from_raw_parts};
    unsafe { from_raw_parts(data.as_ptr() as *const u8, data.len() * size_of::<T>()) }
}

fn create_buffer<T>(
    device: &wgpu::Device,
    data: &[T],
    usage: wgpu::BufferUsage,
) -> (u64, wgpu::Buffer) {
    let data = cast_slice(data);
    let len = data.len() as wgpu::BufferAddress;
    (len, device.create_buffer_with_data(&data, usage))
}

pub struct Target<'a> {
    pub color: &'a wgpu::TextureView,
    pub depth: &'a wgpu::TextureView,

    pub width: u32,
    pub height: u32,
}

pub(crate) struct ImageBind {
    pub bind_group: wgpu::BindGroup,
    pub size: wgpu::Extent3d,
}

impl ImageBind {
    fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        view: &wgpu::TextureView,
        size: wgpu::Extent3d,
    ) -> Self {
        let binding = wgpu::Binding {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(view),
        };
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            bindings: &[binding],
        });
        Self { bind_group, size }
    }
}

struct HostImage {
    texture: wgpu::Texture,
    size: wgpu::Extent3d,
}

impl HostImage {
    fn open_rgba(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        path: impl AsRef<std::path::Path>,
    ) -> image::ImageResult<Self> {
        let m = image::open(path)?;

        let m = m.to_rgba();
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
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED,
        });

        let buffer = device.create_buffer_with_data(texels.as_slice(), wgpu::BufferUsage::COPY_SRC);
        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &buffer,
                offset: 0,
                bytes_per_row: 4 * width,
                rows_per_image: 0,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            size,
        );

        Ok(Self { texture, size })
    }
    fn bind(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> ImageBind {
        let view = self.texture.create_default_view();
        ImageBind::new(device, layout, &view, self.size)
    }

    fn create_default(device: &wgpu::Device) -> Self {
        let (width, height) = (1, 1);

        let size = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("default texture"),
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED,
        });

        Self { texture, size }
    }
}

pub struct Renderer {
    pub(crate) recorder: Path,
    pub(crate) tess: Tessellator,
    pub(crate) images: HashMap<u32, ImageBind>,

    pub(crate) scale: f32,

    sampler: wgpu::Sampler,
    images_idx: u32,

    pipeline: Pipeline,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Undefined,
        });

        let pipeline = Pipeline::new(device, format);

        let default_image = HostImage::create_default(&device);
        let mut images = HashMap::default();
        images.insert(0, default_image.bind(device, &pipeline.image_layout));

        Self {
            tess: Tessellator::new(),
            recorder: Path::new(),

            scale: 1.0,

            sampler,
            images,
            images_idx: 1,

            pipeline,
        }
    }

    pub fn open_image(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        path: impl AsRef<std::path::Path>,
    ) -> image::ImageResult<u32> {
        let image = HostImage::open_rgba(device, encoder, path)?;
        let texture_view = image.texture.create_default_view();
        Ok(self.create_image(device, &texture_view, image.size))
    }

    pub fn create_image(
        &mut self,
        device: &wgpu::Device,
        texture_view: &wgpu::TextureView,
        size: wgpu::Extent3d,
    ) -> u32 {
        let bind = ImageBind::new(device, &self.pipeline.image_layout, texture_view, size);

        let idx = self.images_idx;
        let _ = self.images.insert(idx, bind);
        self.images_idx += 1;
        idx
    }

    pub fn begin_frame<'a>(&'a mut self, scale: f32, picture: &'a mut Picture) -> Canvas<'a> {
        self.tess.set_scale(scale);
        self.scale = scale;
        Canvas::new(self, picture)
    }

    pub fn draw_picture(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        target: Target,
        picture: &Picture,
    ) {
        let scale = self.scale;

        const UNIFORM: wgpu::BufferUsage = wgpu::BufferUsage::UNIFORM;
        const VERTEX: wgpu::BufferUsage = wgpu::BufferUsage::VERTEX;

        let (_, vertices) = create_buffer(device, picture.verts.as_ref(), VERTEX);
        let (_, instances) = create_buffer(device, picture.uniforms.as_ref(), VERTEX);

        let bind_group = {
            let w = target.width as f32 / scale;
            let h = target.height as f32 / scale;
            let viewport = [w, h, 0.0, 0.0];

            let (len, buffer) = create_buffer(device, &viewport, UNIFORM);

            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Viewport bind group"),
                layout: &self.pipeline.viewport_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &buffer,
                            range: 0..len,
                        },
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            })
        };

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &target.color,
                resolve_target: None,
                load_op: wgpu::LoadOp::Load,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::TRANSPARENT,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &target.depth,
                depth_load_op: wgpu::LoadOp::Load,
                depth_store_op: wgpu::StoreOp::Store,
                clear_depth: 0.0,
                stencil_load_op: wgpu::LoadOp::Load,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_stencil: 0,
            }),
        });

        rpass.set_stencil_reference(0);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.set_bind_group(1, &self.images[&0].bind_group, &[]);
        rpass.set_vertex_buffer(0, &vertices, 0, 0);
        rpass.set_vertex_buffer(1, &instances, 0, 0);

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
                    }
                    rpass.set_pipeline(&self.pipeline.image);
                    rpass.draw(vtx, idx..idx + 1);
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

    viewport_layout: wgpu::BindGroupLayout,
    image_layout: wgpu::BindGroupLayout,
}

impl Pipeline {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let viewport_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("VG bind group layout"),
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                },
            ],
        });

        let image_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("VG bind group layout"),
            bindings: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    dimension: wgpu::TextureViewDimension::D2,
                    component_type: wgpu::TextureComponentType::Float,
                    multisampled: false,
                },
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&viewport_layout, &image_layout],
        });

        let builder = Builder {
            format,
            device,
            layout,

            base: Shader::base(device).unwrap(),
            stencil: Shader::stencil(device).unwrap(),
            image: Shader::image(device).unwrap(),
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

            viewport_layout,
            image_layout,
        }
    }
}

struct Builder<'a> {
    format: wgpu::TextureFormat,
    device: &'a wgpu::Device,
    layout: wgpu::PipelineLayout,
    base: Shader,
    stencil: Shader,
    image: Shader,
}

impl<'a> Builder<'a> {
    fn image(&self, stencil: wgpu::StencilStateFaceDescriptor) -> wgpu::RenderPipeline {
        let topo = wgpu::PrimitiveTopology::TriangleList;
        let (front, back) = (stencil.clone(), stencil);
        let (write_mask, cull_mode) = (wgpu::ColorWrite::ALL, wgpu::CullMode::Back);
        self.pipeline(&self.image, write_mask, cull_mode, front, back, topo)
    }

    fn base(&self, stencil: wgpu::StencilStateFaceDescriptor) -> wgpu::RenderPipeline {
        let topo = wgpu::PrimitiveTopology::TriangleStrip;
        let (front, back) = (stencil.clone(), stencil);
        let (write_mask, cull_mode) = (wgpu::ColorWrite::ALL, wgpu::CullMode::Back);
        self.pipeline(&self.base, write_mask, cull_mode, front, back, topo)
    }

    fn stencil(
        &self,
        cull_mode: wgpu::CullMode,
        front: wgpu::StencilStateFaceDescriptor,
        back: wgpu::StencilStateFaceDescriptor,
    ) -> wgpu::RenderPipeline {
        let topo = wgpu::PrimitiveTopology::TriangleStrip;
        let write_mask = wgpu::ColorWrite::empty();
        self.pipeline(&self.stencil, write_mask, cull_mode, front, back, topo)
    }

    fn pipeline(
        &self,
        shader: &Shader,
        write_mask: wgpu::ColorWrite,
        cull_mode: wgpu::CullMode,
        stencil_front: wgpu::StencilStateFaceDescriptor,
        stencil_back: wgpu::StencilStateFaceDescriptor,
        primitive_topology: wgpu::PrimitiveTopology,
    ) -> wgpu::RenderPipeline {
        let straight_alpha_blend = wgpu::ColorStateDescriptor {
            format: self.format,
            write_mask,
            color_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        };

        let desc = wgpu::RenderPipelineDescriptor {
            layout: &self.layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &shader.vs,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &shader.fs,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology,
            color_states: &[straight_alpha_blend],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: DEPTH,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil_front,
                stencil_back,
                stencil_read_mask: 0xFF,
                stencil_write_mask: 0xFF,
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    wgpu::VertexBufferDescriptor {
                        stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float2, 1 => Ushort2Norm],
                    },
                    wgpu::VertexBufferDescriptor {
                        stride: std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            2 => Float4,
                            3 => Uchar4Norm,
                            4 => Uchar4Norm,
                            5 => Float4,
                            6 => Float2
                        ],
                    },
                ],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        };

        self.device.create_render_pipeline(&desc)
    }
}
