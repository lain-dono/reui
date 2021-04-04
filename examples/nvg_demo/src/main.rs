#![warn(clippy::all)]
#![allow(clippy::unusual_byte_groupings)]

mod blendish;
mod canvas;

mod time;

use reui::{
    app::{self, ControlFlow, Spawner, WindowEvent},
    wgpu, Offset, Recorder, Rect, Renderer, Viewport,
};

pub fn main() {
    tracing_subscriber::fmt::init();

    let event_loop = app::EventLoop::new();
    let window = app::Window::new(&event_loop).unwrap();
    window.set_title("Anti-aliased vector graphics (wgpu-rs)");
    window.set_inner_size(app::LogicalSize::new(1000, 600));

    app::run::<Demo>(event_loop, window);
}

struct Demo {
    vg: Renderer,
    viewport: Viewport,
    recorder: Recorder,
    mouse: Offset,
    counter: crate::time::Counter,
    image: u32,
    blowup: bool,
}

impl app::Application for Demo {
    //type UserEvent = ();

    fn init(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        scale: f32,
    ) -> Self {
        let mut vg = Renderer::new(&device);
        let viewport = vg.pipeline.create_viewport(device, width, height, scale);
        let recorder = Recorder::default();

        let image = vg
            .open_image(device, queue, "examples/rust-jerk.jpg")
            .unwrap();

        Self {
            vg,
            viewport,
            recorder,
            mouse: Offset::zero(),
            counter: crate::time::Counter::new(),
            image,
            blowup: false,
        }
    }

    fn update(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        use reui::app::winit::event::{ElementState, KeyboardInput, VirtualKeyCode};
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
            } => {
                self.blowup = !self.blowup;
            }
            _ => {}
        }
    }

    fn render(
        &mut self,
        frame: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _spawner: &Spawner,
        staging_belt: &mut wgpu::util::StagingBelt,
        width: u32,
        height: u32,
        scale: f32,
    ) {
        self.viewport.resize(device, queue, width, height, scale);

        let time = self.counter.update();
        let mouse = self.mouse / scale;
        let wsize = Offset::new(width as f32, height as f32) / scale;

        if self.counter.index == 0 {
            tracing::info!("average frame time: {}ms", self.counter.average_ms());
        }

        let mut encoder = device.create_command_encoder(&Default::default());

        self.recorder.clear();
        let mut ctx = self.vg.begin_frame(&self.viewport, &mut self.recorder);

        ctx.draw_image_rect(self.image, Rect::from_size(wsize.x, wsize.y));
        canvas::render_demo(&mut ctx, mouse, wsize, time, self.blowup);

        let target = self.viewport.target(&frame.view);
        let bundle =
            self.recorder
                .finish(&mut encoder, staging_belt, &device, &mut self.vg, &target);
        {
            let mut rpass = target.rpass(
                &mut encoder,
                Some(reui::wgpu::Color {
                    r: 0.3,
                    g: 0.3,
                    b: 0.32,
                    a: 1.0,
                }),
                true,
                true,
            );
            rpass.execute_bundles(bundle.into_iter());
        }

        staging_belt.finish();
        queue.submit(Some(encoder.finish()));
    }
}
