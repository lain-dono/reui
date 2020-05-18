use crate::{
    backend::Vertex,
    paint::Uniforms,
    picture::{Call, Picture},
    shader::Shader,
};

const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;

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

pub struct Target<'a> {
    pub color: &'a wgpu::TextureView,
    pub depth: &'a wgpu::TextureView,

    pub width: u32,
    pub height: u32,
    pub scale: f32,
}

pub struct Pipeline {
    bind_group_layout: wgpu::BindGroupLayout,

    convex: wgpu::RenderPipeline,
    fringes: wgpu::RenderPipeline,

    fill_stencil: wgpu::RenderPipeline,
    fill_quad: wgpu::RenderPipeline,

    stroke_base: wgpu::RenderPipeline,
    stroke_stencil: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("VG bind group layout"),
            bindings: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let builder = Builder {
            format,
            device,
            layout,

            base: Shader::base(device).unwrap(),
            stencil: Shader::stencil(device).unwrap(),
        };

        stencil_face!(ALWAYS_ZERO, Always, Zero, Zero);
        stencil_face!(INCR_WRAP, Always, Keep, IncrementWrap);
        stencil_face!(DECR_WRAP, Always, Keep, DecrementWrap);

        Self {
            bind_group_layout,

            convex: builder.base(stencil_face!(Always, Keep, Keep)),
            fringes: builder.base(stencil_face!(Equal, Keep, Keep)),

            fill_stencil: builder.stencil(wgpu::CullMode::None, INCR_WRAP, DECR_WRAP),
            fill_quad: builder.base(stencil_face!(NotEqual, Zero, Zero)),

            stroke_base: builder.base(stencil_face!(Equal, Keep, IncrementClamp)),
            stroke_stencil: builder.stencil(wgpu::CullMode::Back, ALWAYS_ZERO, ALWAYS_ZERO),
        }
    }

    pub fn draw_picture(
        &mut self,
        picture: &Picture,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        target: Target,
    ) {
        const UNIFORM: wgpu::BufferUsage = wgpu::BufferUsage::UNIFORM;
        const VERTEX: wgpu::BufferUsage = wgpu::BufferUsage::VERTEX;

        let (_, vertices) = create_buffer(device, picture.verts.as_ref(), VERTEX);
        let (_, instances) = create_buffer(device, picture.uniforms.as_ref(), VERTEX);

        let bind_group = {
            let w = target.width as f32 / target.scale;
            let h = target.height as f32 / target.scale;
            let viewport = [w, h, 0.0, 0.0];

            let (len, buffer) = create_buffer(device, &viewport, UNIFORM);

            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Viewport bind group"),
                layout: &self.bind_group_layout,
                bindings: &[wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &buffer,
                        range: 0..len,
                    },
                }],
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
        rpass.set_vertex_buffer(0, &vertices, 0, 0);
        rpass.set_vertex_buffer(1, &instances, 0, 0);

        for call in picture.calls.iter().cloned() {
            match call {
                Call::Convex { idx, path } => {
                    let paths = &picture.paths[path];
                    let start = idx;
                    let end = idx + 1;

                    rpass.set_pipeline(&self.convex);
                    for path in paths {
                        rpass.draw(path.fill.clone(), start..end);
                        rpass.draw(path.stroke.clone(), start..end); // fringes
                    }
                }
                Call::Fill { idx, path, quad } => {
                    let paths = &picture.paths[path];
                    let instances = idx..idx + 1;

                    rpass.set_pipeline(&self.fill_stencil);
                    for path in paths {
                        rpass.draw(path.fill.clone(), instances.clone());
                    }

                    let instances = idx + 1..idx + 2;

                    // Draw fringes
                    rpass.set_pipeline(&self.fringes);
                    for path in paths {
                        rpass.draw(path.stroke.clone(), instances.clone());
                    }

                    // Draw fill
                    rpass.set_pipeline(&self.fill_quad);
                    rpass.draw(quad..quad + 4, instances.clone());
                }
                Call::Stroke { idx, path } => {
                    let stroke = &picture.strokes[path];
                    let instances = idx..idx + 1;

                    // Fill the stroke base without overlap
                    rpass.set_pipeline(&self.stroke_base);
                    for path in stroke {
                        rpass.draw(path.clone(), instances.clone());
                    }

                    let instances = idx + 1..idx + 2;

                    // Draw anti-aliased pixels.
                    rpass.set_pipeline(&self.fringes);
                    for path in stroke {
                        rpass.draw(path.clone(), instances.clone());
                    }

                    // Clear stencil buffer
                    rpass.set_pipeline(&self.stroke_stencil);
                    for path in stroke {
                        rpass.draw(path.clone(), instances.clone());
                    }
                }
            }
        }
    }
}

struct Builder<'a> {
    format: wgpu::TextureFormat,
    device: &'a wgpu::Device,
    layout: wgpu::PipelineLayout,
    base: Shader,
    stencil: Shader,
}

impl<'a> Builder<'a> {
    fn base(&self, stencil: wgpu::StencilStateFaceDescriptor) -> wgpu::RenderPipeline {
        let (front, back) = (stencil.clone(), stencil);
        let (write_mask, cull_mode) = (wgpu::ColorWrite::ALL, wgpu::CullMode::Back);
        self.pipeline(&self.base, write_mask, cull_mode, front, back)
    }

    fn stencil(
        &self,
        cull_mode: wgpu::CullMode,
        front: wgpu::StencilStateFaceDescriptor,
        back: wgpu::StencilStateFaceDescriptor,
    ) -> wgpu::RenderPipeline {
        let write_mask = wgpu::ColorWrite::empty();
        self.pipeline(&self.stencil, write_mask, cull_mode, front, back)
    }

    fn pipeline(
        &self,
        shader: &Shader,
        write_mask: wgpu::ColorWrite,
        cull_mode: wgpu::CullMode,
        stencil_front: wgpu::StencilStateFaceDescriptor,
        stencil_back: wgpu::StencilStateFaceDescriptor,
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
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
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
