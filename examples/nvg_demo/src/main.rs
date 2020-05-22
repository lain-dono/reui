#![warn(clippy::all)]
#![allow(unstable_name_collisions)] // TODO: clamp

mod blendish;
mod canvas;

mod time;

use reui::{
    app::{self, ControlFlow, Options, Surface, WindowEvent},
    wgpu, Offset, Renderer,
};

pub fn main() {
    let event_loop = app::EventLoop::new();
    let window = app::Window::new(&event_loop).unwrap();
    window.set_title("Anti-aliased vector graphics (wgpu-rs)");
    window.set_inner_size(app::PhysicalSize::new(2000, 1200));

    let options = Options::default();
    app::run::<Demo>(event_loop, window, options);
}

struct Demo {
    vg: Renderer,
    mouse: Offset,
    counter: crate::time::Counter,
}

impl app::Application for Demo {
    type UserEvent = ();

    fn init(device: &wgpu::Device, _queue: &wgpu::Queue, surface: &mut Surface) -> Self {
        Self {
            vg: Renderer::new(&device, surface.format()),
            mouse: Offset::zero(),
            counter: crate::time::Counter::new(),
        }
    }

    fn update(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse = Offset::new(position.x as f32, position.y as f32)
            }
            _ => {}
        }
    }

    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, surface: &mut Surface) {
        let time = self.counter.update();
        let scale = surface.scale() as f32;
        let (width, height) = surface.size();
        let mouse = self.mouse / scale;
        let wsize = Offset::new(width as f32, height as f32) / scale;

        if self.counter.index == 0 {
            println!("average wgpu: {}ms", self.counter.average_ms());
        }

        let frame = surface
            .next_frame()
            .expect("Timeout when acquiring next swap chain texture");

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let _ = frame.clear(&mut encoder, [0.3, 0.3, 0.32, 1.0]);

        {
            {
                let mut ctx = self.vg.begin_frame(scale);
                canvas::render_demo(&mut ctx, mouse, wsize, time);
                drop(ctx);
            }

            self.vg.draw(&mut encoder, &device, frame.target());
        }

        queue.submit(&[encoder.finish()]);
    }
}
