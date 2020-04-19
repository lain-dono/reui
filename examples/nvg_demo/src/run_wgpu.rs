#![allow(dead_code, unused_variables)]

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use wgpu_vg::backend::{CallKind, CmdBuffer, FragUniforms};
use wgpu_vg::cache::Vertex;
use wgpu_vg::canvas::Canvas;
use wgpu_vg::context::Context;
use wgpu_vg::math::{point2, rect};

struct Backend {}

impl wgpu_vg::backend::Backend for Backend {
    fn draw_commands(&mut self, cmd: &CmdBuffer, width: f32, height: f32, pixel_ratio: f32) {
        unimplemented!()
    }
}

struct Shader {
    vs_module: wgpu::ShaderModule,
    fs_module: wgpu::ShaderModule,
}

impl Shader {
    fn new(device: &wgpu::Device) -> std::io::Result<Self> {
        use std::io::Cursor;

        let vs = include_bytes!("shader/shader.vert.spv");
        let vs = wgpu::read_spirv(Cursor::new(&vs[..]))?;
        let vs_module = device.create_shader_module(&vs);

        let fs = include_bytes!("shader/shader.frag.spv");
        let fs = wgpu::read_spirv(Cursor::new(&fs[..]))?;
        let fs_module = device.create_shader_module(&fs);

        Ok(Self {
            vs_module,
            fs_module,
        })
    }
}

struct Pipeline {
    view: wgpu::Buffer,
    frag: [wgpu::Buffer; 2],
    bind: [wgpu::BindGroup; 2],

    bind_layout: wgpu::BindGroupLayout,

    convex: wgpu::RenderPipeline,
    fill_shapes: wgpu::RenderPipeline,

    fill_fringes: wgpu::RenderPipeline,
    fill_end: wgpu::RenderPipeline,

    stroke_base: wgpu::RenderPipeline,
    stroke_aa: wgpu::RenderPipeline,
    stroke_clear: wgpu::RenderPipeline,
}

impl Pipeline {
    const VIEW_SIZE: u64 = 4 * 2;
    const FRAG_SIZE: u64 = std::mem::size_of::<FragUniforms>() as u64;

    fn new(device: &wgpu::Device, shader: &Shader) -> Self {
        let usage = wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST;

        let view = {
            let data = [0.0f32, 0.0f32];
            let data: [u8; Self::VIEW_SIZE as usize] = unsafe { std::mem::transmute(data) };
            device.create_buffer_with_data(&data, usage)
        };

        let frag = {
            let data = [0u8; Self::FRAG_SIZE as usize];
            let a = device.create_buffer_with_data(&data, usage);
            let b = device.create_buffer_with_data(&data, usage);
            [a, b]
        };

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
            ],
            label: None,
        });

        let b0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &view,
                        range: 0..Self::VIEW_SIZE,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &frag[0],
                        range: 0..Self::FRAG_SIZE,
                    },
                },
            ],
            label: None,
        });

        let b1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &view,
                        range: 0..Self::VIEW_SIZE,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &frag[1],
                        range: 0..Self::FRAG_SIZE,
                    },
                },
            ],
            label: None,
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_layout],
        });

        Self {
            view,
            frag,
            bind: [b0, b1],

            bind_layout,

            convex: PipelineBuilder::convex().build(device, &layout, shader),

            fill_shapes: PipelineBuilder::fill_shapes().build(device, &layout, &shader),
            fill_fringes: PipelineBuilder::fill_fringes().build(device, &layout, &shader),
            fill_end: PipelineBuilder::fill_end().build(device, &layout, &shader),

            stroke_base: PipelineBuilder::stroke_base().build(device, &layout, &shader),
            stroke_aa: PipelineBuilder::stroke_aa().build(device, &layout, &shader),
            stroke_clear: PipelineBuilder::stroke_clear().build(device, &layout, &shader),
        }
    }

    fn draw_commands(
        &self,
        cmd: &CmdBuffer,
        width: f32,
        height: f32,
        pixel_ratio: f32,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        depth: &wgpu::TextureView,
    ) {
        let v_size = 4 * 2 + 2 * 2;
        let u_size = std::mem::size_of::<FragUniforms>();

        {
            let data = [width, height, 0.0, 0.0];
            let data: [u8; 4 * 4] = unsafe { std::mem::transmute(data) };
            let tmp = device.create_buffer_with_data(&data, wgpu::BufferUsage::COPY_SRC);
            encoder.copy_buffer_to_buffer(&tmp, 0, &self.view, 0, 4 * 2);
        }

        let (vbytes, vertices) = {
            let ptr = cmd.verts.as_ptr() as *const _;
            let len = cmd.verts.len();
            let data: &[u8] = unsafe { std::slice::from_raw_parts(ptr, len * v_size) };
            (
                data.len() as wgpu::BufferAddress,
                device.create_buffer_with_data(&data, wgpu::BufferUsage::VERTEX),
            )
        };

        let uniforms = {
            let ptr = cmd.uniforms.as_ptr() as *const _;
            let len = cmd.uniforms.len();
            let data: &[u8] = unsafe { std::slice::from_raw_parts(ptr, len * u_size) };
            let usage = wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC;
            device.create_buffer_with_data(&data, usage)
        };

        fn copy_uniform(
            encoder: &mut wgpu::CommandEncoder,
            src: &wgpu::Buffer,
            dst: &wgpu::Buffer,
            offset: usize,
        ) {
            let offset = offset as wgpu::BufferAddress;
            let u_size = std::mem::size_of::<FragUniforms>() as wgpu::BufferAddress;
            encoder.copy_buffer_to_buffer(src, offset * u_size, dst, 0, u_size);
        }

        for call in &cmd.calls {
            match call.kind {
                CallKind::CONVEXFILL => {
                    copy_uniform(encoder, &uniforms, &self.frag[0], call.uniform_offset);

                    let off = call.uniform_offset as u32 * u_size as u32;

                    let mut rpass = create_pass(encoder, view, depth);
                    rpass.set_pipeline(&self.convex);
                    rpass.set_vertex_buffer(0, &vertices, 0, 0);
                    rpass.set_bind_group(0, &self.bind[0], &[]);
                    for path in &cmd.paths[call.path.range()] {
                        rpass.draw(path.fill.range32(), 0..1);
                        rpass.draw(path.stroke.range32(), 0..1); // fringes
                    }
                }
                CallKind::FILL => {
                    let range = call.path.range();

                    copy_uniform(encoder, &uniforms, &self.frag[0], call.uniform_offset);
                    copy_uniform(encoder, &uniforms, &self.frag[1], call.uniform_offset + 1);

                    let mut rpass = create_pass(encoder, view, depth);
                    rpass.set_vertex_buffer(0, &vertices, 0, 0);

                    rpass.set_pipeline(&self.fill_shapes);
                    rpass.set_bind_group(0, &self.bind[0], &[]);
                    for path in &cmd.paths[range.clone()] {
                        rpass.draw(path.fill.range32(), 0..1);
                    }

                    // Draw fringes
                    rpass.set_pipeline(&self.fill_fringes);
                    rpass.set_bind_group(0, &self.bind[1], &[]);
                    for path in &cmd.paths[range] {
                        rpass.draw(path.stroke.range32(), 0..1);
                    }

                    // Draw fill
                    if call.triangle.count == 4 {
                        rpass.set_pipeline(&self.fill_end);
                        rpass.draw(call.triangle.range32(), 0..1);
                    }
                }
                CallKind::STROKE => {
                    let range = call.path.range();

                    copy_uniform(encoder, &uniforms, &self.frag[0], call.uniform_offset);
                    copy_uniform(encoder, &uniforms, &self.frag[1], call.uniform_offset + 1);

                    let mut rpass = create_pass(encoder, view, depth);
                    rpass.set_vertex_buffer(0, &vertices, 0, 0);

                    // Fill the stroke base without overlap
                    rpass.set_pipeline(&self.stroke_base);
                    rpass.set_bind_group(0, &self.bind[1], &[]);
                    for path in &cmd.paths[range.clone()] {
                        rpass.draw(path.stroke.range32(), 0..1);
                    }

                    // Draw anti-aliased pixels.
                    rpass.set_pipeline(&self.stroke_aa);
                    rpass.set_bind_group(0, &self.bind[0], &[]);
                    for path in &cmd.paths[range.clone()] {
                        rpass.draw(path.stroke.range32(), 0..1);
                    }

                    // Clear stencil buffer
                    rpass.set_pipeline(&self.stroke_clear);
                    for path in &cmd.paths[range] {
                        rpass.draw(path.stroke.range32(), 0..1);
                    }
                }
            }
        }
    }
}

async fn run(event_loop: EventLoop<()>, window: Window, swapchain_format: wgpu::TextureFormat) {
    let size = window.inner_size();
    let surface = wgpu::Surface::create(&window);

    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: Some(&surface),
        },
        wgpu::BackendBit::PRIMARY,
    )
    .await
    .unwrap();

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        })
        .await;

    let shader = Shader::new(&device).unwrap();

    let pipeline = Pipeline::new(&device, &shader);

    let mut vg = Context::new(Box::new(Backend {}));

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);
    let mut depth = create_depth(&device, size.width, size.height);

    let (mut mx, mut my) = (0.0f32, 0.0f32);

    let mut counter = crate::time::Counter::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
                depth = create_depth(&device, size.width, size.height);
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                mx = position.x as f32;
                my = position.y as f32;
            }

            Event::WindowEvent {
                event: WindowEvent::MouseInput { .. },
                ..
            } => {
                let ms = counter.average_ms();
                println!("average: {}ms", ms);
            }
            Event::RedrawRequested(_) => {
                let time = counter.update();

                if counter.index == 0 {
                    println!("awerage: {}ms", counter.average_ms());
                }

                let frame = swap_chain
                    .get_next_texture()
                    .expect("Timeout when acquiring next swap chain texture");

                let vert_buffer = {
                    let data = [
                        Vertex::new([250.0, 20.0], [0.0, 0.0]),
                        Vertex::new([100.0, 80.0], [0.0, 0.0]),
                        Vertex::new([300.0, 90.0], [0.0, 0.0]),
                    ];
                    let data: [u8; 12 * 3] = unsafe { std::mem::transmute(data) };
                    device.create_buffer_with_data(&data, wgpu::BufferUsage::VERTEX)
                };

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                clear_pass(&mut encoder, &frame.view, &depth);

                {
                    let mut rpass = create_pass(&mut encoder, &frame.view, &depth);
                    rpass.set_pipeline(&pipeline.convex);
                    rpass.set_bind_group(0, &pipeline.bind[0], &[]);
                    rpass.set_vertex_buffer(0, &vert_buffer, 0, 0);
                    rpass.draw(0..3, 0..1);
                }

                {
                    let scale = window.scale_factor() as f32;
                    let size = window.inner_size();
                    let win_w = size.width as f32 / scale;
                    let win_h = size.height as f32 / scale;

                    vg.begin_frame(win_w, win_h, scale);

                    let mut ctx = Canvas::new(&mut vg);
                    super::canvas::render_demo(
                        &mut ctx,
                        point2(mx as f32, my as f32) / scale,
                        (win_w as f32, win_h as f32).into(),
                        time as f32,
                        false,
                    );

                    if true {
                        super::blendish::run(
                            &mut ctx,
                            time as f32,
                            rect(380.0, 50.0, 200.0, 200.0),
                        );
                    }

                    drop(ctx);

                    //vg.end_frame();
                    pipeline.draw_commands(
                        &vg.cmd,
                        win_w,
                        win_h,
                        scale,
                        &mut encoder,
                        &device,
                        &frame.view,
                        &depth,
                    );
                    vg.cmd.clear();
                }

                queue.submit(&[encoder.finish()]);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

pub fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    window.set_title("Anti-aliased vector graphics (wgpu-rs)");
    window.set_inner_size(winit::dpi::PhysicalSize::new(2000, 1200));

    env_logger::init();
    // Temporarily avoid srgb formats for the swapchain on the web
    futures::executor::block_on(run(event_loop, window, FORMAT));
}

fn clear_pass(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    depth: &wgpu::TextureView,
) {
    let clear_color = wgpu::Color {
        r: 0.3,
        g: 0.3,
        b: 0.32,
        a: 1.0,
    };
    let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
            attachment: view,
            resolve_target: None,
            load_op: wgpu::LoadOp::Clear,
            store_op: wgpu::StoreOp::Store,
            clear_color,
        }],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
            attachment: depth,
            depth_load_op: wgpu::LoadOp::Load,
            depth_store_op: wgpu::StoreOp::Store,
            clear_depth: 0.0,
            stencil_load_op: wgpu::LoadOp::Clear,
            stencil_store_op: wgpu::StoreOp::Store,
            clear_stencil: 0,
        }),
    });
}

fn create_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    view: &'a wgpu::TextureView,
    depth: &'a wgpu::TextureView,
) -> wgpu::RenderPass<'a> {
    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
            attachment: view,
            resolve_target: None,
            load_op: wgpu::LoadOp::Load,
            store_op: wgpu::StoreOp::Store,
            clear_color: wgpu::Color::GREEN,
        }],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
            attachment: depth,
            depth_load_op: wgpu::LoadOp::Load,
            depth_store_op: wgpu::StoreOp::Store,
            clear_depth: 0.0,
            stencil_load_op: wgpu::LoadOp::Load,
            stencil_store_op: wgpu::StoreOp::Store,
            clear_stencil: 0,
        }),
    });
    rpass.set_stencil_reference(0);
    rpass
}

fn create_depth(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("DEPTH"),
        size: wgpu::Extent3d {
            width,
            height,
            depth: 1,
        },
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    });
    texture.create_default_view()
}

struct PipelineBuilder {
    write_mask: wgpu::ColorWrite,
    depth_stencil_state: Option<wgpu::DepthStencilStateDescriptor>,
    cull_mode: wgpu::CullMode,
}

impl PipelineBuilder {
    fn new(
        write_mask: wgpu::ColorWrite,
        depth_stencil_state: Option<wgpu::DepthStencilStateDescriptor>,
    ) -> Self {
        Self {
            depth_stencil_state,
            write_mask,
            cull_mode: wgpu::CullMode::Back,
        }
    }

    fn no_culling(self) -> Self {
        Self {
            cull_mode: wgpu::CullMode::None,
            ..self
        }
    }

    fn convex() -> Self {
        Self::new(wgpu::ColorWrite::ALL, Some(CONVEX))
    }

    fn fill_shapes() -> Self {
        Self::new(wgpu::ColorWrite::empty(), Some(FILL_SHAPES)).no_culling()
    }
    fn fill_fringes() -> Self {
        Self::new(wgpu::ColorWrite::ALL, Some(FILL_FRINGES))
    }
    fn fill_end() -> Self {
        Self::new(wgpu::ColorWrite::ALL, Some(FILL_END))
    }

    fn stroke_base() -> Self {
        Self::new(wgpu::ColorWrite::ALL, Some(STROKE_BASE))
    }
    fn stroke_aa() -> Self {
        Self::new(wgpu::ColorWrite::ALL, Some(STROKE_AA))
    }
    fn stroke_clear() -> Self {
        Self::new(wgpu::ColorWrite::empty(), Some(STROKE_CLEAR))
    }

    fn build(
        self,
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        shader: &Shader,
    ) -> wgpu::RenderPipeline {
        let Self {
            depth_stencil_state,
            write_mask,
            cull_mode,
        } = self;

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &shader.vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &shader.fs_module,
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
                format: FORMAT,
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
                write_mask,
            }],
            depth_stencil_state,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: 12 as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float2, 1 => Ushort2Norm],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }
}

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;
const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;

macro_rules! stencil_state {
    ($name: ident, $front:expr, $back:expr) => {
        const $name: wgpu::DepthStencilStateDescriptor = stencil_state($front, $back);
    };
}

const fn stencil_state(
    stencil_front: wgpu::StencilStateFaceDescriptor,
    stencil_back: wgpu::StencilStateFaceDescriptor,
) -> wgpu::DepthStencilStateDescriptor {
    wgpu::DepthStencilStateDescriptor {
        format: DEPTH,
        depth_write_enabled: false,
        depth_compare: wgpu::CompareFunction::Always,
        stencil_front,
        stencil_back,
        stencil_read_mask: 0xFF,
        stencil_write_mask: 0xFF,
    }
}

stencil_state!(CONVEX, UND_KEEP, UND_KEEP);

stencil_state!(FILL_SHAPES, ALWAYS_KEEP_INCR_WRAP, ALWAYS_KEEP_DECR_WRAP);
stencil_state!(FILL_FRINGES, EQ_KEEP, EQ_KEEP);
stencil_state!(FILL_END, NE_ZERO, NE_ZERO);

stencil_state!(STROKE_BASE, EQ_KEEP_INCR, EQ_KEEP_INCR);
stencil_state!(STROKE_AA, EQ_KEEP, EQ_KEEP);
stencil_state!(STROKE_CLEAR, ALWAYS_ZERO, ALWAYS_ZERO);

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

stencil_face!(UND_KEEP, Always, Keep, Keep);

stencil_face!(ALWAYS_ZERO, Always, Zero, Zero);
stencil_face!(NE_ZERO, NotEqual, Zero, Zero);
stencil_face!(EQ_KEEP, Equal, Keep, Keep);
stencil_face!(EQ_KEEP_INCR, Equal, Keep, IncrementClamp);
stencil_face!(ALWAYS_KEEP_INCR_WRAP, Always, Keep, IncrementWrap);
stencil_face!(ALWAYS_KEEP_DECR_WRAP, Always, Keep, DecrementWrap);
