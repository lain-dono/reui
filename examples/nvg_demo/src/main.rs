#![warn(clippy::all)]
#![allow(clippy::unusual_byte_groupings)]

mod blendish;
mod canvas;

mod time;

use reui::{text, wgpu, BatchUpload, Canvas, Images, Offset, Pipeline, Recorder, Rect, Viewport};
use reui_app::{self as app, ControlFlow, WindowEvent};

pub fn main() {
    tracing_subscriber::fmt::init();

    let event_loop = app::EventLoop::new();
    let window = app::Window::new(&event_loop).unwrap();
    window.set_title("Anti-aliased vector graphics (wgpu-rs)");
    window.set_inner_size(app::LogicalSize::new(1000, 600));

    app::run::<Demo>(event_loop, window);
}

struct Demo {
    batch: BatchUpload,
    pipeline: Pipeline,
    viewport: Viewport,
    recorder: Recorder,
    mouse: Offset,
    counter: crate::time::Counter,
    image: u32,
    blowup: bool,

    db: reui::text::FontDatabase,
    font: reui::text::Font,

    staging_belt: wgpu::util::StagingBelt,
    images: Images,
}

impl app::Application for Demo {
    fn init(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        scale: f32,
    ) -> Self {
        let mut images = Images::new(device);
        let batch = BatchUpload::new(device);
        let pipeline = Pipeline::new(device, &images);
        let viewport = pipeline.create_viewport(device, width, height, scale);
        let recorder = Recorder::default();

        let image = images
            .open(device, queue, "examples/rust-jerk.jpg")
            .unwrap();

        let mut db = reui::text::FontDatabase::new();
        db.load_system_fonts();
        db.load_fonts_dir("./assets/fonts/");

        let font = {
            use reui::text::{FontStretch, FontStyle, FontWeight, Query};
            let font = db
                .query(&Query {
                    families: &["Roboto"],
                    weight: FontWeight::NORMAL,
                    stretch: FontStretch::Normal,
                    style: FontStyle::Normal,
                })
                .unwrap();

            db.load_font(font).unwrap()
        };

        let mut families = std::collections::HashMap::new();
        for (id, face) in db.faces() {
            *families.entry(face.family.clone()).or_insert(0) += 1;
        }
        println!("{:#?}", families);

        let staging_belt = wgpu::util::StagingBelt::new(0x20_0000);

        Self {
            batch,
            pipeline,
            viewport,
            recorder,
            mouse: Offset::zero(),
            counter: crate::time::Counter::new(),
            image,
            blowup: false,

            db,
            font,
            staging_belt,
            images,
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
        let inv_scale = scale.recip();
        self.viewport.resize(device, queue, width, height, scale);

        let time = self.counter.update();
        let mouse = self.mouse * inv_scale;
        let wsize = Offset::new(width as f32, height as f32) * inv_scale;

        if self.counter.index == 0 {
            //tracing::info!("average frame time: {}ms", self.counter.average_ms());
        }

        let mut encoder = device.create_command_encoder(&Default::default());

        self.recorder.clear();

        let mut ctx = Canvas::new(&mut self.recorder, self.viewport.scale());

        ctx.draw_image_rect(&self.images, self.image, Rect::from_size(wsize.x, wsize.y));
        canvas::render_demo(&mut ctx, mouse, wsize, time, self.blowup);

        if false {
            use reui::text::{Paragraph, TextAnchor, TextStyle};
            let mut style = TextStyle {
                color: reui::Color::BLACK,
                font: self.font,
                font_size: 50.0,
                decoration: reui::text::TextDecoration::default(),
                letter_spacing: 0.0,
                word_spacing: 0.0,
            };

            let width = 500.0;

            let mut paragraph = Paragraph::new(TextAnchor::Start, style.clone());
            paragraph.add_text("Hello ");
            style.color = reui::Color::GREEN;
            paragraph.push_style(style.clone());
            paragraph.add_text("ÊWorldΐΊ");
            paragraph.add_text(" ");
            paragraph.add_text("金糸");
            paragraph.pop_style();
            paragraph.add_text("雀");
            style.color = reui::Color::RED;
            style.letter_spacing = 0.0;
            paragraph.push_style(style.clone());
            paragraph.add_text(" ");
            //paragraph.add_text(concat!["א", "ב", "ג", "a", "b", "c",]);
            paragraph.add_text("12345");
            paragraph.add_text("\n");
            paragraph.push_style(style);
            paragraph.add_text("The word العربية al-arabiyyah");
            paragraph.add_text("\n");
            paragraph.add_text("12345");

            ctx.save();
            ctx.translate(50.0, 200.0);
            ctx.draw_line(
                Offset::zero(),
                Offset::new(width, 0.0),
                reui::Paint::stroke(reui::Color::WHITE),
            );
            paragraph.draw(&self.db, width, &mut ctx);
            ctx.restore();
        }

        let attachment = frame.texture.create_view(&Default::default());

        let target = self.viewport.target(&attachment);
        let bundle = self.recorder.finish(
            &mut encoder,
            &mut self.staging_belt,
            device,
            &mut self.batch,
            &self.pipeline,
            &target,
            &self.images,
        );
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

        self.staging_belt.finish();
        queue.submit(std::iter::once(encoder.finish()));
        self.staging_belt.recall();
    }
}
