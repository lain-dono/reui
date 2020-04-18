#![feature(clamp)]

use slotmap::Key;

use std::f32::consts::PI;
use std::ptr::null;

use oni2d::{
    canvas::Canvas,
    gl,
    math::{point2, rect, Offset},
    BackendGL, Context, Image, ImageFlags,
};

mod sup;
use self::sup::*;

//#[link(name = "nvg")]
//extern "C" {}

fn main() {
    env_logger::init();
    log::info!("start");

    const GLFW_CONTEXT_VERSION_MAJOR: i32 = 0x0002_2002;
    const GLFW_CONTEXT_VERSION_MINOR: i32 = 0x0002_2003;

    unsafe {
        assert!(glfwInit(), "Failed to init GLFW.");

        glfwSetErrorCallback(errorcb);

        glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 2);
        glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 0);

        let window = glfwCreateWindow(2000, 1200, b"ONI2D\0".as_ptr(), null(), null());
        //window = glfwCreateWindow(1000, 600, "NanoVG", glfwGetPrimaryMonitor(), NULL);
        if window.is_null() {
            glfwTerminate();
            panic!("cant create window");
        }

        glfwSetKeyCallback(window, key);
        glfwMakeContextCurrent(window);

        let mut vg = Context::new(BackendGL::default());

        let data = DemoData::new(&mut vg);

        glfwSwapInterval(0);

        glfwSetTime(0.0);

        while !glfwWindowShouldClose(window) {
            let time = glfwGetTime();

            let (mut mx, mut my) = (0.0, 0.0);
            glfwGetCursorPos(window, &mut mx, &mut my);
            let (mut win_w, mut win_h) = (0, 0);
            glfwGetWindowSize(window, &mut win_w, &mut win_h);
            let (mut fb_w, mut fb_h) = (0, 0);
            glfwGetFramebufferSize(window, &mut fb_w, &mut fb_h);

            // Calculate pixel ration for hi-dpi devices.
            let scale = 2.0;
            let px_ratio = (fb_w as f32) / (win_w as f32);

            // Update and render
            gl::Viewport(0, 0, fb_w, fb_h);
            if PREMULT != 0 {
                gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            } else {
                gl::ClearColor(0.3, 0.3, 0.32, 1.0);
            }

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);

            vg.begin_frame(win_w as f32, win_h as f32, px_ratio * scale);

            render_demo(
                &mut vg,
                point2(mx as f32 / scale, my as f32 / scale),
                (win_w as f32 / scale, win_h as f32 / scale).into(),
                time as f32,
                BLOWUP != 0,
                &data,
            );

            vg.end_frame();

            glfwSwapBuffers(window);
            glfwPollEvents();
        }

        glfwTerminate();
    }
}

#[repr(C)]
struct GLFWwindow(usize);
#[repr(C)]
struct GLFWmonitor(usize);

extern "C" {
    fn glfwInit() -> bool;
    fn glfwTerminate();
    fn glfwSetWindowShouldClose(window: *mut GLFWwindow, value: i32);
    fn glfwSetErrorCallback(errorcb: extern "C" fn(error: i32, desc: *const i8));

    fn glfwWindowHint(hint: i32, value: i32);

    fn glfwCreateWindow(
        width: i32,
        height: i32,
        title: *const u8,
        monitor: *const GLFWmonitor,
        share: *const GLFWwindow,
    ) -> *mut GLFWwindow;
    fn glfwSetKeyCallback(
        window: *mut GLFWwindow,
        key: extern "C" fn(
            window: *mut GLFWwindow,
            key: i32,
            _scancode: i32,
            action: i32,
            _mods: i32,
        ),
    );
    fn glfwMakeContextCurrent(window: *mut GLFWwindow);

    fn glfwSwapInterval(interval: i32);

    fn glfwSetTime(time: f64);
    fn glfwGetTime() -> f64;
    fn glfwWindowShouldClose(window: *mut GLFWwindow) -> bool;
    fn glfwGetCursorPos(window: *mut GLFWwindow, xpos: &mut f64, ypos: &mut f64);
    fn glfwGetWindowSize(window: *mut GLFWwindow, w: &mut i32, h: &mut i32);
    fn glfwGetFramebufferSize(window: *mut GLFWwindow, w: &mut i32, h: &mut i32);
    fn glfwSwapBuffers(window: *mut GLFWwindow);
    fn glfwPollEvents();
}

pub static mut BLOWUP: i32 = 0;
pub static mut SCREENSHOT: i32 = 0;
pub static mut PREMULT: i32 = 0;

extern "C" fn errorcb(error: i32, desc: *const i8) {
    let desc = unsafe { std::ffi::CStr::from_ptr(desc) };
    println!("GLFW error {}: {:?}\n", error, desc);
}

extern "C" fn key(window: *mut GLFWwindow, key: i32, _scancode: i32, action: i32, _mods: i32) {
    const GLFW_KEY_SPACE: i32 = 32;
    const GLFW_KEY_ESCAPE: i32 = 256;
    const GLFW_PRESS: i32 = 1;
    const GLFW_KEY_P: i32 = 80;
    const GLFW_KEY_S: i32 = 83;
    unsafe {
        if key == GLFW_KEY_ESCAPE && action == GLFW_PRESS {
            glfwSetWindowShouldClose(window, 1);
        }
        if key == GLFW_KEY_SPACE && action == GLFW_PRESS {
            BLOWUP = !BLOWUP;
        }
        if key == GLFW_KEY_S && action == GLFW_PRESS {
            SCREENSHOT = 1;
        }
        if key == GLFW_KEY_P && action == GLFW_PRESS {
            PREMULT = !PREMULT;
        }
    }
}

#[repr(C)]
pub struct DemoData {
    pub images: [Image; 12],
}

impl DemoData {
    fn new(vg: &mut Context) -> Self {
        let mut images = [Image::null(); 12];
        for (i, image) in images.iter_mut().enumerate() {
            let file = format!("assets/images/image{}.jpg", i + 1);
            let m = vg.create_image(&file, ImageFlags::REPEAT);
            assert!(!m.is_null(), "Could not load {}.", file);
            *image = m;
        }

        Self { images }
    }
}

pub fn render_demo(
    vg: &mut Context,
    mouse: Offset,
    wsize: Offset,
    time: f32,
    blowup: bool,
    data: &DemoData,
) {
    let (width, height) = wsize.into();

    {
        let mut ctx = Canvas::new(vg);

        draw_colorwheel(
            &mut ctx,
            rect(width - 300.0, height - 300.0, 250.0, 250.0),
            time,
        );
        draw_eyes(
            &mut ctx,
            rect(width - 250.0, 50.0, 150.0, 100.0),
            mouse,
            time,
        );
        draw_graph(&mut ctx, 0.0, height / 2.0, width, height / 2.0, time);
        // Line joints
        draw_lines(&mut ctx, 120.0, height - 50.0, 600.0, 50.0, time);
        // Line caps
        draw_widths(&mut ctx, 10.0, 50.0, 30.0);
        // Line caps
        draw_caps(&mut ctx, 10.0, 300.0, 30.0);
        draw_scissor(&mut ctx, 50.0, height - 80.0, time);

        if blowup {
            ctx.rotate((time * 0.3).sin() * 5.0 / 180.0 * PI);
            ctx.scale(2.0);
        }

        // Widgets
        draw_window(&mut ctx, rect(50.0, 50.0, 300.0, 400.0));

        let (x, mut y) = (60.0, 95.0);
        draw_search_box(&mut ctx, rect(x, y, 280.0, 25.0));
        y += 40.0;
        draw_drop_down(&mut ctx, rect(x, y, 280.0, 28.0));
        let popy = y + 14.0;
        y += 45.0;

        // Form
        //draw_label(&mut ctx, "Login", rect(x, y, 280.0, 20.0));
        y += 25.0;
        draw_edit_box(&mut ctx, rect(x, y, 280.0, 28.0));
        y += 35.0;
        draw_edit_box(&mut ctx, rect(x, y, 280.0, 28.0));
        y += 38.0;
        draw_checkbox(&mut ctx, rect(x, y, 140.0, 28.0));
        draw_button(&mut ctx, rect(x + 138.0, y, 140.0, 28.0), 0xFF_006080);
        y += 45.0;

        // Slider
        //draw_label(&mut ctx, "Diameter", rect(x, y, 280.0, 20.0));
        y += 25.0;
        draw_edit_box_num(&mut ctx, rect(x + 180.0, y, 100.0, 28.0));
        draw_slider(&mut ctx, 0.4, x, y, 170.0, 28.0);
        y += 55.0;

        draw_button(&mut ctx, rect(x, y, 160.0, 28.0), 0xFF_801008);
        draw_button(&mut ctx, rect(x + 170.0, y, 110.0, 28.0), 0x00_000000);

        // Thumbnails box
        draw_thumbnails(
            &mut ctx,
            rect(365.0, popy - 30.0, 160.0, 300.0),
            &data.images[..],
            time,
        );
    }

    if false {
        // Canvas test
        use oni2d::canvas::*;
        let mut ctx = Canvas::new(vg);

        ctx.draw_rect(rect(50.0, 50.0, 100.0, 100.0), Paint::fill(0xFF_000000));
        ctx.draw_rrect(
            RRect::new([50.0, 50.0].into(), [100.0, 100.0].into(), 15.0),
            Paint::fill(0xFF_CC0000),
        );

        ctx.draw_line(
            [60.0, 60.0].into(),
            [140.0, 140.0].into(),
            Paint::stroke(0xFF_00CCCC),
        );
    }

    {
        use oni2d::canvas::*;
        let mut ctx = Canvas::new(vg);
        sup::blendish::run(&mut ctx, time, rect(50.0, 50.0, 200.0, 200.0));
    }
}
