#![warn(clippy::all)]

use reui::{
    app::{self, ControlFlow, Options, Surface, WindowEvent},
    wgpu, Offset, Picture, Rect, Renderer,
};

pub fn main() {
    let event_loop = app::EventLoop::new();
    let window = app::Window::new(&event_loop).unwrap();
    window.set_title("Hello world");
    window.set_inner_size(app::LogicalSize::new(1000, 600));

    let options = Options::default();
    app::run::<Demo>(event_loop, window, options);
}

struct Demo {
    vg: Renderer,
    picture: Picture,
    img: u32,
}

impl app::Application for Demo {
    type UserEvent = ();

    fn init(device: &wgpu::Device, queue: &wgpu::Queue, surface: &mut Surface) -> Self {
        let mut vg = Renderer::new(&device, surface.format());
        let picture = Picture::default();

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let img = vg
            .open_image(device, &mut encoder, "examples/hello_world/rust-jerk.jpg")
            .unwrap();

        queue.submit(&[encoder.finish()]);

        Self { vg, picture, img }
    }

    fn update(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        if let WindowEvent::CloseRequested = event {
            *control_flow = ControlFlow::Exit
        }
    }

    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, surface: &mut Surface) {
        let scale = surface.scale() as f32;
        let (width, height) = surface.size();

        let frame = surface
            .next_frame()
            .expect("Timeout when acquiring next swap chain texture");

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let _ = frame.clear(&mut encoder, [0.3, 0.3, 0.32, 1.0]);

        {
            self.picture.clear();

            let mut ctx = self.vg.begin_frame(scale, &mut self.picture);

            let (width, height) = (width as f32 / scale, height as f32 / scale);
            let size = width.min(height) / 4.0;

            let rect = Rect::from_size(width, height).deflate(size);
            let paint = Default::default();

            ctx.rotate(f32::to_radians(10.0));
            ctx.draw_rect(rect, paint);

            ctx.draw_image(self.img, Offset::zero());

            self.vg
                .draw_picture(&mut encoder, &device, frame.target(), &self.picture);
        }

        queue.submit(&[encoder.finish()]);
    }
}
