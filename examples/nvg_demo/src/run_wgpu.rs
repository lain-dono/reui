#![allow(dead_code, unused_variables)]

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use reui::backend::{Pipeline, Target};
use reui::context::Context;
use reui::math::Offset;

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

    let mut pipeline = Pipeline::new(&device, swapchain_format);

    let mut vg = Context::default();

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Immediate,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);
    let mut depth = create_depth(&device, size.width, size.height);

    let (mut mx, mut my) = (0.0f32, 0.0f32);

    let mut counter = crate::time::Counter::new();

    let mut scale = window.scale_factor() as f32;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                scale = window.scale_factor() as f32;
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
                println!("average by click: {}ms", ms);
            }

            Event::RedrawRequested(_) => {
                let time = counter.update();

                if counter.index == 0 {
                    println!("awerage wgpu: {}ms", counter.average_ms());
                }

                let frame = swap_chain
                    .get_next_texture()
                    .expect("Timeout when acquiring next swap chain texture");

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                clear_pass(&mut encoder, &frame.view, &depth);

                {
                    let win_w = sc_desc.width as f32;
                    let win_h = sc_desc.height as f32;
                    {
                        let mut ctx = vg.begin_frame(win_w, win_h, scale);

                        super::canvas::render_demo(
                            &mut ctx,
                            Offset::new(mx, my) / scale,
                            Offset::new(win_w, win_h) / scale,
                            time as f32,
                            false,
                        );

                        drop(ctx);
                    }

                    //vg.end_frame();

                    let target = Target {
                        width: win_w,
                        height: win_h,
                        scale: scale,
                        color: &frame.view,
                        depth: &depth,
                    };

                    pipeline.draw_commands(&vg.cmd, &mut encoder, &device, target);
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
    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    futures::executor::block_on(run(event_loop, window, format));
}

const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8;

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

fn clear_pass(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    depth: &wgpu::TextureView,
) {
    use palette::{Srgba, LinSrgba};

    let [r, g, b, a] = [0.3, 0.3, 0.32, 1.0];

    let srgb = Srgba::new(r, g, b, a);
    let lin: LinSrgba = srgb.into_encoding();

    let clear_color = wgpu::Color {
        r: lin.color.red as f64,
        g: lin.color.green as f64,
        b: lin.color.blue as f64,
        a: lin.alpha as f64,
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
