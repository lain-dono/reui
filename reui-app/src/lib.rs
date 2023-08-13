#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

pub use futures;
pub use winit;

pub use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub trait Application: 'static + Sized {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::default()
    }

    fn init(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        scale: f32,
    ) -> Self;

    fn update(&mut self, event: WindowEvent, control_flow: &mut ControlFlow);

    fn render(
        &mut self,
        frame: &wgpu::SurfaceTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        scale: f32,
    );
}

struct Setup {
    window: winit::window::Window,
    event_loop: EventLoop<()>,
    instance: wgpu::Instance,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

fn backend() -> wgpu::Backends {
    if let Ok(backend) = std::env::var("WGPU_BACKEND") {
        match backend.to_lowercase().as_str() {
            "vulkan" => wgpu::Backends::VULKAN,
            "metal" => wgpu::Backends::METAL,
            "dx12" => wgpu::Backends::DX12,
            "dx11" => wgpu::Backends::DX11,
            "gl" => wgpu::Backends::GL,
            "webgpu" => wgpu::Backends::BROWSER_WEBGPU,
            other => panic!("Unknown backend: {}", other),
        }
    } else {
        wgpu::Backends::PRIMARY
    }
}

fn power_preference() -> wgpu::PowerPreference {
    if let Ok(power_preference) = std::env::var("WGPU_POWER_PREF") {
        match power_preference.to_lowercase().as_str() {
            "low" => wgpu::PowerPreference::LowPower,
            "high" => wgpu::PowerPreference::HighPerformance,
            other => panic!("Unknown power preference: {}", other),
        }
    } else {
        wgpu::PowerPreference::default()
    }
}

async fn setup<E: Application>(event_loop: EventLoop<()>, window: Window) -> Setup {
    #[cfg(not(target_arch = "wasm32"))]
    if false {
        let chrome_tracing_dir = std::env::var("WGPU_CHROME_TRACE");
        wgpu_subscriber::initialize_default_subscriber(
            chrome_tracing_dir.as_ref().map(std::path::Path::new).ok(),
        );
    };

    /*
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        console_log::init().expect("could not initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
    }
    */

    tracing::info!("Initializing the surface...");

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: backend(),
        dx12_shader_compiler: wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default(),
    });
    let (size, surface) = unsafe {
        let size = window.inner_size();
        let surface = instance.create_surface(&window);
        (size, surface.unwrap())
    };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: power_preference(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("No suitable GPU adapters found on the system!");

    #[cfg(not(target_arch = "wasm32"))]
    {
        let info = adapter.get_info();
        tracing::info!(
            "Using {:?} [{}:{}] {} ({:?})",
            info.device_type,
            info.vendor,
            info.device,
            info.name,
            info.backend,
        );
    }

    let optional_features = E::optional_features();
    let required_features = E::required_features();
    let adapter_features = adapter.features();
    assert!(
        adapter_features.contains(required_features),
        "Adapter does not support required features for this example: {:?}",
        required_features - adapter_features
    );

    let needed_limits = E::required_limits();

    let trace_dir = std::env::var("WGPU_TRACE");
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: (optional_features & adapter_features) | required_features,
                limits: needed_limits,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .expect("Unable to find a suitable GPU adapter!");

    tracing::info!("Adapter Features: {:?}", adapter_features);
    tracing::info!("Result Features: {:?}", device.features());
    tracing::info!("Limits: {:#?}", device.limits());

    Setup {
        window,
        event_loop,
        instance,
        size,
        surface,
        adapter,
        device,
        queue,
    }
}

fn start<App: Application>(
    Setup {
        window,
        event_loop,
        instance,
        size,
        surface,
        adapter,
        device,
        queue,
    }: Setup,
) -> ! {
    /*
    let format = surface
        .get_preferred_format(&adapter)
        .unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb);
    */

    let mut sc_desc = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        //view_formats: vec![wgpu::TextureFormat::Rgba8UnormSrgb],
        view_formats: vec![wgpu::TextureFormat::Rgba8UnormSrgb],
    };
    surface.configure(&device, &sc_desc);

    tracing::info!("Initializing the app...");
    let mut state = App::init(
        &device,
        &queue,
        sc_desc.width,
        sc_desc.height,
        window.scale_factor() as f32,
    );

    tracing::info!("Entering render loop...");
    let start_time = Instant::now();

    let target_frametime = Duration::from_secs_f64(1.0 / 60.0);

    #[cfg(not(target_arch = "wasm32"))]
    let mut last_update_inst = Instant::now();
    #[cfg(not(target_arch = "wasm32"))]
    let mut last_frame_inst = Instant::now();
    #[cfg(not(target_arch = "wasm32"))]
    let (mut frame_count, mut accum_time) = (0, 0.0);

    event_loop.run(move |event, _event_loop, control_flow| {
        let _ = (&instance, &adapter); // force ownership by the closure
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Wait
        };
        match event {
            Event::NewEvents(start_cause) => trace_start_cause(start_cause, start_time),

            Event::WindowEvent { event, .. } => {
                if let WindowEvent::Resized(size) = event {
                    tracing::info!("Resizing to {:?}", size);
                    sc_desc.width = size.width.max(1);
                    sc_desc.height = size.height.max(1);

                    surface.configure(&device, &sc_desc);
                }
                state.update(event, control_flow);
            }

            Event::DeviceEvent { .. } => tracing::trace!("DeviceEvent"),
            Event::UserEvent(_) => tracing::trace!("user event"),

            Event::Suspended => tracing::debug!("Suspended"),
            Event::Resumed => tracing::debug!("Resume"),

            Event::MainEventsCleared => {
                tracing::trace!("MainEventsCleared");
            }
            Event::RedrawRequested(_window_id) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    accum_time += last_frame_inst.elapsed().as_secs_f32();
                    last_frame_inst = Instant::now();
                    frame_count += 1;
                    if frame_count == 1000 {
                        println!(
                            "Avg frame time {}ms",
                            accum_time * 1000.0 / frame_count as f32
                        );
                        accum_time = 0.0;
                        frame_count = 0;
                    }
                }

                let (width, height) = window.inner_size().into();

                let frame = if let Ok(frame) = surface.get_current_texture() {
                    frame
                } else {
                    sc_desc.width = width;
                    sc_desc.height = height;
                    surface.configure(&device, &sc_desc);
                    surface
                        .get_current_texture()
                        .expect("Failed to acquire next swap chain texture!")
                };

                state.render(
                    &frame,
                    &device,
                    &queue,
                    width,
                    height,
                    window.scale_factor() as f32,
                );

                frame.present();
            }

            Event::RedrawEventsCleared => {
                tracing::trace!("RedrawEventsCleared");

                #[cfg(not(target_arch = "wasm32"))]
                if false {
                    // Clamp to some max framerate to avoid busy-looping too much
                    // (we might be in wgpu::PresentMode::Mailbox, thus discarding superfluous frames)
                    //
                    // winit has window.current_monitor().video_modes() but that is a list of all full screen video modes.
                    // So without extra dependencies it's a bit tricky to get the max refresh rate we can run the window on.
                    // Therefore we just go with 60fps - sorry 120hz+ folks!
                    let time_since_last_frame = last_update_inst.elapsed();
                    if time_since_last_frame >= target_frametime {
                        window.request_redraw();
                        last_update_inst = Instant::now();
                    } else {
                        *control_flow = ControlFlow::WaitUntil(
                            Instant::now() + target_frametime - time_since_last_frame,
                        );
                    }
                } else {
                    window.request_redraw();
                }

                #[cfg(target_arch = "wasm32")]
                window.request_redraw();
            }
            Event::LoopDestroyed => tracing::warn!("Event loop destroyed"),
        }
    })
}

fn trace_start_cause(start_cause: StartCause, start_time: Instant) {
    match start_cause {
        StartCause::Init => tracing::info!("Init event loop"),
        StartCause::Poll => tracing::info!("Pool event loop"),
        StartCause::ResumeTimeReached {
            start,
            requested_resume,
        } => {
            tracing::trace!(
                "Resume time reached: {:?} {:?}",
                start.duration_since(start_time),
                requested_resume.duration_since(start_time)
            )
        }
        StartCause::WaitCancelled {
            start,
            requested_resume,
        } => {
            tracing::trace!(
                "Wait cancelled: {:?} {:?}",
                start.duration_since(start_time),
                requested_resume.map(|i| i.duration_since(start_time))
            )
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run<E: Application>(event_loop: EventLoop<()>, window: Window) {
    start::<E>(pollster::block_on(setup::<E>(event_loop, window)));
}
