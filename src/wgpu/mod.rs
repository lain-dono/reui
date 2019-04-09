const VERTEX_BUFFER_DESCRIPTOR: wgpu::VertexBufferDescriptor = wgpu::VertexBufferDescriptor {
    step_mode: wgpu::InputStepMode::Vertex,
    stride: 4 * 3,
    attributes: &[
        wgpu::VertexAttributeDescriptor {
            attribute_index: 0,
            format: wgpu::VertexFormat::Float2,
            offset: 0,
        },
        wgpu::VertexAttributeDescriptor {
            attribute_index: 1,
            format: wgpu::VertexFormat::Ushort2Norm,
            offset: 8,
        },
    ],
};

const BLENGING: wgpu::ColorStateDescriptor = wgpu::ColorStateDescriptor {
    format: wgpu::TextureFormat::Bgra8UnormSrgb,
    color: wgpu::BlendDescriptor {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    },
    alpha: wgpu::BlendDescriptor {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    },
    write_mask: wgpu::ColorWriteFlags::ALL,
};

const COMBINED_LAYOUT: wgpu::BindGroupLayoutDescriptor = wgpu::BindGroupLayoutDescriptor {
    bindings: &[
        wgpu::BindGroupLayoutBinding {
            binding: 0,
            visibility: wgpu::ShaderStageFlags::FRAGMENT,
            ty: wgpu::BindingType::SampledTexture,
        },
        wgpu::BindGroupLayoutBinding {
            binding: 1,
            visibility: wgpu::ShaderStageFlags::FRAGMENT,
            ty: wgpu::BindingType::Sampler,
        },
    ],
};


fn new_shader(device: &mut wgpu::Device) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    let vs_bytes = include_bytes!("../../hello_triangle.vert.spv");
    let vs_module = device.create_shader_module(vs_bytes);
    let fs_bytes = include_bytes!("../../hello_triangle.frag.spv");
    let fs_module = device.create_shader_module(fs_bytes);
    (vs_module, fs_module)
}

fn new_pipeline(device: &mut wgpu::Device) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
    let (vs_module, fs_module) = new_shader(device);

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        bindings: &[],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &layout,
        vertex_stage: wgpu::PipelineStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: wgpu::PipelineStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        },
        rasterization_state: wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        },
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[BLENGING],
        depth_stencil_state: None,
        index_format: wgpu::IndexFormat::Uint16,
        //vertex_buffers: &[VERTEX_BUFFER_DESCRIPTOR],
        vertex_buffers: &[],
        sample_count: 1,
    });

    (pipeline, bind_group)
}

pub fn run() {
    let instance = wgpu::Instance::new();
    let adapter = instance.get_adapter(&wgpu::AdapterDescriptor {
        power_preference: wgpu::PowerPreference::LowPower,
    });
    let mut device = adapter.create_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
    });

    let (render_pipeline, bind_group) = new_pipeline(&mut device);

    use wgpu::winit::{
        ControlFlow, ElementState, Event, EventsLoop, KeyboardInput, VirtualKeyCode, Window,
        WindowEvent,
    };

    let mut events_loop = EventsLoop::new();
    let window = Window::new(&events_loop).unwrap();
    let size = window
        .get_inner_size()
        .unwrap()
        .to_physical(window.get_hidpi_factor());

    let surface = instance.create_surface(&window);
    let mut swap_chain = device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsageFlags::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width.round() as u32,
            height: size.height.round() as u32,
        },
    );

    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(code),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match code {
                    VirtualKeyCode::Escape => return ControlFlow::Break,
                    _ => {}
                },
                WindowEvent::CloseRequested => return ControlFlow::Break,
                _ => {}
            },
            _ => {}
        }

        let frame = swap_chain.get_next_texture();

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::GREEN,
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&render_pipeline);
            rpass.set_bind_group(0, &bind_group);
            rpass.draw(0..3, 0..1);
        }

        device.get_queue().submit(&[encoder.finish()]);

        ControlFlow::Continue
    });
}
