use crate::backend::{CallKind, CmdBuffer};

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

macro_rules! def_pipeline {
    ($color:expr, $cull:ident, $front:expr, $back:expr) => {
        PipelineBuilder {
            stencil_state: wgpu::DepthStencilStateDescriptor {
                format: DEPTH,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil_front: $front,
                stencil_back: $back,
                stencil_read_mask: 0xFF,
                stencil_write_mask: 0xFF,
            },
            write_mask: $color,
            cull_mode: wgpu::CullMode::$cull,
        }
    };
}

macro_rules! stencil_face {
    ($name:ident, $comp:ident, $fail:ident, $pass:ident) => {
        const $name: wgpu::StencilStateFaceDescriptor = wgpu::StencilStateFaceDescriptor {
            compare: wgpu::CompareFunction::$comp,
            fail_op: wgpu::StencilOperation::$fail,
            depth_fail_op: wgpu::StencilOperation::$fail,
            pass_op: wgpu::StencilOperation::$pass,
        };
    };
}

struct Shader {
    vs: wgpu::ShaderModule,
    fs: wgpu::ShaderModule,
}

impl Shader {
    fn new(device: &wgpu::Device) -> std::io::Result<Self> {
        use std::io::Cursor;

        let vs = include_bytes!("shader/shader.vert.spv");
        let vs = wgpu::read_spirv(Cursor::new(&vs[..]))?;
        let vs = device.create_shader_module(&vs);

        let fs = include_bytes!("shader/shader.frag.spv");
        let fs = wgpu::read_spirv(Cursor::new(&fs[..]))?;
        let fs = device.create_shader_module(&fs);

        Ok(Self { vs, fs })
    }
}

pub struct Target<'a> {
    pub color: &'a wgpu::TextureView,
    pub depth: &'a wgpu::TextureView,

    pub width: f32,
    pub height: f32,
    pub scale: f32,
}

pub struct Pipeline {
    bind_group_layout: wgpu::BindGroupLayout,

    convex: wgpu::RenderPipeline,

    fill_base: wgpu::RenderPipeline,
    fill_fringes: wgpu::RenderPipeline,
    fill_end: wgpu::RenderPipeline,

    stroke_base: wgpu::RenderPipeline,
    stroke_aa: wgpu::RenderPipeline,
    stroke_clear: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    ty: wgpu::BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
            ],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let shader = Shader::new(device).unwrap();

        stencil_face!(UND_KEEP, Always, Keep, Keep);
        stencil_face!(ALWAYS_ZERO, Always, Zero, Zero);
        stencil_face!(NE_ZERO, NotEqual, Zero, Zero);
        stencil_face!(EQ_KEEP, Equal, Keep, Keep);
        stencil_face!(EQ_KEEP_INCR, Equal, Keep, IncrementClamp);
        stencil_face!(ALWAYS_KEEP_INCR_WRAP, Always, Keep, IncrementWrap);
        stencil_face!(ALWAYS_KEEP_DECR_WRAP, Always, Keep, DecrementWrap);

        const COLOR: wgpu::ColorWrite = wgpu::ColorWrite::ALL;
        const STENCIL: wgpu::ColorWrite = wgpu::ColorWrite::empty();

        let convex = def_pipeline!(COLOR, Back, UND_KEEP, UND_KEEP);

        let fill_base = def_pipeline!(STENCIL, None, ALWAYS_KEEP_INCR_WRAP, ALWAYS_KEEP_DECR_WRAP);
        let fill_fringes = def_pipeline!(COLOR, Back, EQ_KEEP, EQ_KEEP);
        let fill_end = def_pipeline!(COLOR, Back, NE_ZERO, NE_ZERO);

        let stroke_base = def_pipeline!(COLOR, Back, EQ_KEEP_INCR, EQ_KEEP_INCR);
        let stroke_aa = def_pipeline!(COLOR, Back, EQ_KEEP, EQ_KEEP);
        let stroke_clear = def_pipeline!(STENCIL, Back, ALWAYS_ZERO, ALWAYS_ZERO);

        Self {
            bind_group_layout,

            convex: convex.build(format, device, &layout, &shader),

            fill_base: fill_base.build(format, device, &layout, &shader),
            fill_fringes: fill_fringes.build(format, device, &layout, &shader),
            fill_end: fill_end.build(format, device, &layout, &shader),

            stroke_base: stroke_base.build(format, device, &layout, &shader),
            stroke_aa: stroke_aa.build(format, device, &layout, &shader),
            stroke_clear: stroke_clear.build(format, device, &layout, &shader),
        }
    }

    pub fn draw_commands(
        &mut self,
        cmd: &CmdBuffer,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        target: Target,
    ) {
        const UNIFORM: wgpu::BufferUsage = wgpu::BufferUsage::UNIFORM;
        const STORAGE_READ: wgpu::BufferUsage = wgpu::BufferUsage::STORAGE_READ;
        const VERTEX: wgpu::BufferUsage = wgpu::BufferUsage::VERTEX;

        let (_, vertices) = create_buffer(device, &cmd.verts, VERTEX);

        let bind_group = {
            let w = target.width / target.scale;
            let h = target.height / target.scale;
            let viewport = [w, h, 0.0, 0.0];
            let uniforms = &cmd.uniforms;

            let (viewport_len, viewport) = create_buffer(device, &viewport, UNIFORM);
            let (uniforms_len, uniforms) = create_buffer(device, &uniforms, STORAGE_READ);

            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("VG bind group"),
                layout: &self.bind_group_layout,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &viewport,
                            range: 0..viewport_len,
                        },
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &uniforms,
                            range: 0..uniforms_len,
                        },
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
        rpass.set_vertex_buffer(0, &vertices, 0, 0);

        for call in &cmd.calls {
            match call.kind {
                CallKind::CONVEXFILL => {
                    let idx = call.uniform_offset as u32;
                    let instance0 = idx..idx + 1;

                    rpass.set_pipeline(&self.convex);
                    for path in &cmd.paths[call.path.range()] {
                        rpass.draw(path.fill.range32(), instance0.clone());
                        rpass.draw(path.stroke.range32(), instance0.clone()); // fringes
                    }
                }
                CallKind::FILL => {
                    let range = call.path.range();
                    let idx = call.uniform_offset as u32;
                    let instance0 = idx..idx + 1;
                    let instance1 = idx + 1..idx + 2;

                    rpass.set_pipeline(&self.fill_base);
                    for path in &cmd.paths[range.clone()] {
                        rpass.draw(path.fill.range32(), instance0.clone());
                    }

                    // Draw fringes
                    rpass.set_pipeline(&self.fill_fringes);
                    for path in &cmd.paths[range] {
                        rpass.draw(path.stroke.range32(), instance1.clone());
                    }

                    // Draw fill
                    rpass.set_pipeline(&self.fill_end);
                    rpass.draw(call.triangle.range32(), instance1.clone());
                }
                CallKind::STROKE => {
                    let range = call.path.range();
                    let idx = call.uniform_offset as u32;
                    let instance0 = idx..idx + 1;
                    let instance1 = idx + 1..idx + 2;

                    // Fill the stroke base without overlap
                    rpass.set_pipeline(&self.stroke_base);
                    for path in &cmd.paths[range.clone()] {
                        rpass.draw(path.stroke.range32(), instance1.clone());
                    }

                    // Draw anti-aliased pixels.
                    rpass.set_pipeline(&self.stroke_aa);
                    for path in &cmd.paths[range.clone()] {
                        rpass.draw(path.stroke.range32(), instance0.clone());
                    }

                    // Clear stencil buffer
                    rpass.set_pipeline(&self.stroke_clear);
                    for path in &cmd.paths[range] {
                        rpass.draw(path.stroke.range32(), instance0.clone());
                    }
                }
            }
        }
    }
}

struct PipelineBuilder {
    write_mask: wgpu::ColorWrite,
    stencil_state: wgpu::DepthStencilStateDescriptor,
    cull_mode: wgpu::CullMode,
}

impl PipelineBuilder {
    fn build(
        self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        shader: &Shader,
    ) -> wgpu::RenderPipeline {
        let Self {
            stencil_state,
            write_mask,
            cull_mode,
        } = self;

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout,
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
            color_states: &[wgpu::ColorStateDescriptor {
                format,
                write_mask,
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            }],
            depth_stencil_state: Some(stencil_state),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    wgpu::VertexBufferDescriptor {
                        stride: 12 as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float2, 1 => Ushort2Norm],
                    },
                    /*
                    wgpu::VertexBufferDescriptor {
                        stride: 112 as wgpu::BufferAddress,
                        step_mode: wgpu::InputStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            0 => Float4,
                            1 => Float4,
                            2 => Float4,
                            3 => Float4,
                            4 => Float4,
                            5 => Float4,
                            6 => Float4,
                        ],
                    },
                    */
                ],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }
}
