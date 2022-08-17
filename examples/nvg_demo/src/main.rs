#![warn(clippy::all)]
#![allow(clippy::unusual_byte_groupings)]

mod blendish;
mod canvas;

mod time;

use reui::{wgpu, Image, Offset, Rect, Renderer};
use reui_app::{self as app, ControlFlow, WindowEvent};

pub fn main() {
    tracing_subscriber::fmt::init();

    let event_loop = app::EventLoop::new();
    let window = app::Window::new(&event_loop).unwrap();
    window.set_title("Anti-aliased vector graphics (wgpu-rs)");
    window.set_inner_size(app::LogicalSize::new(1000, 600));

    app::run::<Demo>(event_loop, window);
}

/// # Errors
pub fn open_image(
    renderer: &mut Renderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: impl AsRef<std::path::Path>,
) -> image::ImageResult<Image> {
    let path = path.as_ref();
    let m = image::open(path)?;
    let m = m.to_rgba8();

    Ok(renderer.upload_image(
        device,
        queue,
        path.to_str(),
        m.width(),
        m.height(),
        m.as_raw(),
        None,
    ))
}

struct Demo {
    staging_belt: wgpu::util::StagingBelt,
    renderer: Renderer,

    mouse: Offset,
    counter: crate::time::Counter,
    image: u32,
    blowup: bool,
}

impl app::Application for Demo {
    fn init(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        _scale: f32,
    ) -> Self {
        let mut renderer = Renderer::new(device, width, height);

        let image = open_image(&mut renderer, device, queue, "examples/rust-jerk.jpg").unwrap();

        let staging_belt = wgpu::util::StagingBelt::new(0x20_0000);

        Self {
            staging_belt,
            renderer,
            mouse: Offset::zero(),
            counter: crate::time::Counter::new(),
            image,
            blowup: false,
        }
    }

    fn update(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        use reui_app::winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
        match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse = Offset::new(position.x as f32, position.y as f32)
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        state: ElementState::Released,
                        ..
                    },
                ..
            } => self.blowup = !self.blowup,
            _ => {}
        }
    }

    fn render(
        &mut self,
        frame: &wgpu::SurfaceTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        scale: f32,
    ) {
        let time = self.counter.update();
        let view = frame.texture.create_view(&Default::default());

        if self.counter.index == 0 {
            //tracing::info!("average frame time: {}ms", self.counter.average_ms());
        }

        let mut encoder = device.create_command_encoder(&Default::default());

        let mut canvas = self.renderer.start(device, queue, width, height);
        canvas.push_scale(scale);

        {
            let mouse = self.mouse / scale;
            let size = Offset::new(width as f32, height as f32) / scale;

            canvas.image_rect(self.image, Rect::from_size(size.x, size.y));
            canvas::render_demo(&mut canvas, mouse, size, time, self.blowup);
        }

        let clear = reui::wgpu::Color {
            r: 0.3,
            g: 0.3,
            b: 0.32,
            a: 1.0,
        };

        self.renderer.flush(
            &mut encoder,
            &mut self.staging_belt,
            device,
            &view,
            Some(clear),
        );

        self.staging_belt.finish();
        queue.submit(std::iter::once(encoder.finish()));
        self.staging_belt.recall();
    }
}
