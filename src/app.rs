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
            power_preference: wgpu::PowerPreference::default(),
            backends: wgpu::BackendBit::PRIMARY,

            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),

            swapchain_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            present_mode: wgpu::PresentMode::Mailbox,
        }
    }
}

pub struct Frame {
    pub color: wgpu::SwapChainFrame,
    pub stencil: wgpu::TextureView,
    pub resolve_target: Option<wgpu::TextureView>,
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

        let (view, resolve_target) = match self.resolve_target.as_ref() {
            Some(resolve_target) => (resolve_target, Some(&self.color.output.view)),
            None => (&self.color.output.view, None),
        };

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("frame clear"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view,
                resolve_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.stencil,
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
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
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

    /// # Errors
    pub fn current_frame(&mut self) -> Result<Frame, wgpu::SwapChainError> {
        let desc = wgpu::TextureViewDescriptor::default();
        let stencil = self.stencil.create_view(&desc);
        //let resolve_target = Some(self.resolve_target.create_view(&desc));
        let resolve_target = None;
        let (width, height) = self.size();
        self.swap_chain.get_current_frame().map(|color| Frame {
            color,
            stencil,
            resolve_target,
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
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        })
    }

    fn _create_resolve_target(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("RESOLVE"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
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

/// # Panics
pub async fn run_async<App: Application>(
    event_loop: EventLoop<App::UserEvent>,
    window: Window,
    options: Options,
) {
    let (_instance, _adapter, device, queue, mut surface) = {
        let (width, height) = window.inner_size().into();
        let scale = window.scale_factor();

        let backends = if let Ok(backend) = std::env::var("WGPU_BACKEND") {
            match backend.to_lowercase().as_str() {
                "vulkan" => wgpu::BackendBit::VULKAN,
                "metal" => wgpu::BackendBit::METAL,
                "dx12" => wgpu::BackendBit::DX12,
                "dx11" => wgpu::BackendBit::DX11,
                "gl" => wgpu::BackendBit::GL,
                "webgpu" => wgpu::BackendBit::BROWSER_WEBGPU,
                other => panic!("Unknown backend: {}", other),
            }
        } else {
            wgpu::BackendBit::PRIMARY
        };
        let power_preference = if let Ok(power_preference) = std::env::var("WGPU_POWER_PREF") {
            match power_preference.to_lowercase().as_str() {
                "low" => wgpu::PowerPreference::LowPower,
                "high" => wgpu::PowerPreference::HighPerformance,
                other => panic!("Unknown power preference: {}", other),
            }
        } else {
            wgpu::PowerPreference::default()
        };

        let instance = wgpu::Instance::new(backends);
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("No suitable GPU adapters found on the system!");

        #[cfg(not(target_arch = "wasm32"))]
        {
            let adapter_info = adapter.get_info();
            println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);
        }

        let device = wgpu::DeviceDescriptor {
            label: Some("reui device"),
            features: options.features,
            limits: options.limits,
        };
        let (device, queue) = adapter
            .request_device(&device, None)
            .await
            .expect("Unable to find a suitable GPU adapter!");

        let surface = Surface::new(
            &device,
            surface,
            options.swapchain_format,
            width,
            height,
            options.present_mode,
            scale,
        );

        (instance, adapter, device, queue, surface)
    };

    let mut app = App::init(&device, &queue, &mut surface);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        #[allow(clippy::match_same_arms)]
        match event {
            Event::NewEvents(_) => {}

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
