pub use futures;
pub use winit;

pub use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Options {
    pub power_preference: wgpu::PowerPreference,
    pub backends: wgpu::BackendBit,

    pub features: wgpu::Features,
    pub limits: wgpu::Limits,

    pub swapchain_format: wgpu::TextureFormat,
    pub present_mode: wgpu::PresentMode,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::Default,
            backends: wgpu::BackendBit::PRIMARY,

            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),

            swapchain_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            present_mode: wgpu::PresentMode::Mailbox,
        }
    }
}

impl Options {
    pub async fn create(
        self,
        instance: &wgpu::Instance,
        surface: wgpu::Surface,
        width: u32,
        height: u32,
        scale: f64,
    ) -> Option<(wgpu::Adapter, wgpu::Device, wgpu::Queue, Surface)> {
        let options = wgpu::RequestAdapterOptions {
            power_preference: self.power_preference,
            compatible_surface: Some(&surface),
        };

        let adapter = instance.request_adapter(&options).await?;

        let device = wgpu::DeviceDescriptor {
            features: self.features,
            limits: self.limits,
            shader_validation: true,
        };
        let (device, queue) = adapter.request_device(&device, None).await.ok()?;

        let surface = Surface::new(
            &device,
            surface,
            self.swapchain_format,
            width,
            height,
            self.present_mode,
            scale,
        );

        Some((adapter, device, queue, surface))
    }
}

pub struct Frame {
    pub color: wgpu::SwapChainFrame,
    pub stencil: wgpu::TextureView,
    pub width: u32,
    pub height: u32,
}

impl Frame {
    pub fn clear<'a>(
        &'a self,
        encoder: &'a mut wgpu::CommandEncoder,
        color: [f64; 4],
    ) -> wgpu::RenderPass<'a> {
        use palette::{LinSrgba, Srgba};

        let [r, g, b, a] = color;

        let lin: LinSrgba<f64> = Srgba::new(r, g, b, a).into_encoding();
        let clear_color = wgpu::Color {
            r: lin.color.red,
            g: lin.color.green,
            b: lin.color.blue,
            a: lin.alpha,
        };

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &self.color.output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.stencil,
                depth_ops: None,
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                }),
            }),
        })
    }
}

pub struct Surface {
    swap_chain: wgpu::SwapChain,
    stencil: wgpu::Texture,
    surface: wgpu::Surface,
    desc: wgpu::SwapChainDescriptor,
    scale: f64,
}

impl Surface {
    pub fn new(
        device: &wgpu::Device,
        surface: wgpu::Surface,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        present_mode: wgpu::PresentMode,

        scale: f64,
    ) -> Self {
        let desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format,
            width,
            height,
            present_mode,
        };

        Self {
            swap_chain: device.create_swap_chain(&surface, &desc),
            stencil: Surface::create_stencil(&device, width, height),
            surface,
            desc,
            scale,
        }
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn width(&self) -> u32 {
        self.desc.width
    }

    pub fn height(&self) -> u32 {
        self.desc.height
    }

    pub fn size(&self) -> (u32, u32) {
        (self.desc.width, self.desc.height)
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.desc.format
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.desc.width = width;
        self.desc.height = height;

        self.swap_chain = device.create_swap_chain(&self.surface, &self.desc);
        self.stencil = Self::create_stencil(device, width, height);
    }

    pub fn current_frame(&mut self) -> Result<Frame, wgpu::SwapChainError> {
        let desc = wgpu::TextureViewDescriptor::default();
        let stencil = self.stencil.create_view(&desc);
        let (width, height) = self.size();
        self.swap_chain.get_current_frame().map(|color| Frame {
            color,
            stencil,
            width,
            height,
        })
    }

    fn create_stencil(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("DEPTH"),
            size: wgpu::Extent3d {
                width,
                height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        })
    }
}

pub trait Application: 'static {
    type UserEvent: 'static;

    fn init(device: &wgpu::Device, queue: &wgpu::Queue, surface: &mut Surface) -> Self;
    fn update(&mut self, event: WindowEvent, control_flow: &mut ControlFlow);
    fn user_event(&mut self, _event: Self::UserEvent) {}
    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, surface: &mut Surface);
}

pub fn run<App: Application>(
    event_loop: EventLoop<App::UserEvent>,
    window: Window,
    options: Options,
) {
    futures::executor::block_on(run_async::<App>(event_loop, window, options));
}

pub async fn run_async<App: Application>(
    event_loop: EventLoop<App::UserEvent>,
    window: Window,
    options: Options,
) {
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

    let (_adapter, device, queue, mut surface) = {
        let size = window.inner_size();
        let scale = window.scale_factor();
        let surface = unsafe { instance.create_surface(&window) };

        options
            .create(&instance, surface, size.width, size.height, scale)
            .await
            .unwrap()
    };

    let mut app = App::init(&device, &queue, &mut surface);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::NewEvents(StartCause::Init) => {}
            Event::NewEvents(StartCause::Poll) => {}
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {}
            Event::NewEvents(StartCause::WaitCancelled { .. }) => {}

            Event::WindowEvent { event, window_id } => {
                if window.id() == window_id {
                    match event {
                        WindowEvent::Resized(size) => {
                            surface.resize(&device, size.width, size.height)
                        }
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            surface.scale = scale_factor
                        }
                        _ => {}
                    }
                    app.update(event, control_flow);
                }
            }

            Event::DeviceEvent { .. } => {}
            Event::UserEvent(event) => app.user_event(event),
            Event::Suspended => {}
            Event::Resumed => {}
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(window_id) => {
                if window.id() == window_id {
                    app.render(&device, &queue, &mut surface);
                }
            }
            Event::RedrawEventsCleared => {}
            Event::LoopDestroyed => {}
        }
    })
}
