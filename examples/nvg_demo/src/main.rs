#![warn(clippy::all)]
#![allow(clippy::unusual_byte_groupings)]

mod blendish;
mod canvas;

mod time;

use reui::{
    app::{self, ControlFlow, Options, Surface, WindowEvent},
    wgpu, Offset, PictureRecorder, Rect, Renderer,
};

pub fn main() {
    let event_loop = app::EventLoop::new();
    let window = app::Window::new(&event_loop).unwrap();
    window.set_title("Anti-aliased vector graphics (wgpu-rs)");
    window.set_inner_size(app::LogicalSize::new(1000, 600));

    let options = Options::default();
    app::run::<Demo>(event_loop, window, options);
}

struct Demo {
    vg: Renderer,
    picture: PictureRecorder,
    mouse: Offset,
    counter: crate::time::Counter,
    image: u32,
}

impl app::Application for Demo {
    type UserEvent = ();

    fn init(device: &wgpu::Device, queue: &wgpu::Queue, surface: &mut Surface) -> Self {
        let mut vg = Renderer::new(&device, surface.format());
        let picture = PictureRecorder::default();

        let image = vg
            .open_image(device, queue, "examples/rust-jerk.jpg")
            .unwrap();

        Self {
            vg,
            picture,
            mouse: Offset::zero(),
            counter: crate::time::Counter::new(),
            image,
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

        let frame = match surface.current_frame() {
            Ok(frame) => frame,
            Err(_) => {
                println!("Timeout when acquiring next swap chain texture");
                return;
            }
        };

        let mut encoder = device.create_command_encoder(&Default::default());

        self.picture.clear();
        let mut ctx = self
            .vg
            .begin_frame(&queue, width, height, scale, &mut self.picture);
        ctx.draw_image_rect(self.image, Rect::from_size(wsize.x, wsize.y));
        canvas::render_demo(&mut ctx, mouse, wsize, time);

        let bundle = self.picture.build(&device);
        {
            let mut rpass = frame.clear(&mut encoder, [0.3, 0.3, 0.32, 1.0]);
            self.vg.draw_picture(&mut rpass, &self.picture, &bundle);
        }

        queue.submit(Some(encoder.finish()));
    }
}
