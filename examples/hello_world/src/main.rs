#![warn(clippy::all)]

use reui::{
    app::{self, ControlFlow, Options, Surface, WindowEvent},
    wgpu, Offset, Paint, Recorder, Rect, Renderer,
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
    picture: Recorder,
    img: u32,
}

impl app::Application for Demo {
    type UserEvent = ();

    fn init(device: &wgpu::Device, queue: &wgpu::Queue, surface: &mut Surface) -> Self {
        let mut vg = Renderer::new(&device, surface.format());
        let picture = Recorder::default();

        let img = vg
            .open_image(device, queue, "examples/rust-jerk.jpg")
            .unwrap();

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
            .current_frame()
            .expect("Timeout when acquiring next swap chain texture");

        let mut encoder = device.create_command_encoder(&Default::default());

        self.picture.clear();

        {
            let mut ctx = self
                .vg
                .begin_frame(queue, width, height, scale, &mut self.picture);

            let (width, height) = (width as f32 / scale, height as f32 / scale);
            let size = width.min(height) / 4.0;

            let rect = Rect::from_size(width, height).deflate(size);
            let paint = Paint::default();

            ctx.rotate(f32::to_radians(10.0));
            ctx.translate(100.0, 100.0);
            ctx.draw_rect(rect, paint);
            ctx.draw_image(self.img, Offset::zero());
        }

        let bundle = self.picture.build(&device);
        {
            let mut rpass = frame.clear(&mut encoder, [0.3, 0.3, 0.32, 1.0]);
            self.vg.draw_picture(&mut rpass, &self.picture, &bundle);
        }

        queue.submit(Some(encoder.finish()));
    }
}
