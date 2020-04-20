use std::ptr::null;
use wgpu_vg::math::{point2, rect};

pub fn main() {
    use wgpu_vg::backend::gles::{gles, BackendGL};
    use wgpu_vg::context::Context;

    const GLFW_CONTEXT_VERSION_MAJOR: i32 = 0x0002_2002;
    const GLFW_CONTEXT_VERSION_MINOR: i32 = 0x0002_2003;

    unsafe {
        assert!(glfwInit(), "Failed to init GLFW.");

        glfwSetErrorCallback(errorcb);

        glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 2);
        glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 0);

        let window = glfwCreateWindow(
            2000,
            1200,
            b"Anti-aliased vector graphics\0".as_ptr(),
            null(),
            null(),
        );
        //window = glfwCreateWindow(1000, 600, "NanoVG", glfwGetPrimaryMonitor(), NULL);
        if window.is_null() {
            glfwTerminate();
            panic!("cant create window");
        }

        glfwSetKeyCallback(window, key);
        glfwMakeContextCurrent(window);

        let mut backend = BackendGL::default();
        let mut vg = Context::default();

        glfwSwapInterval(0);

        glfwSetTime(0.0);

        let mut counter = crate::time::Counter::new();

        while !glfwWindowShouldClose(window) {
            let _time = glfwGetTime();
            let time = counter.update();

            if counter.index == 0 {
                println!("awerage: {}ms", counter.average_ms());
            }

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
            gles::Viewport(0, 0, fb_w, fb_h);
            if PREMULT != 0 {
                gles::ClearColor(0.0, 0.0, 0.0, 0.0);
            } else {
                gles::ClearColor(0.3, 0.3, 0.32, 1.0);
            }

            gles::Clear(gles::COLOR_BUFFER_BIT | gles::DEPTH_BUFFER_BIT | gles::STENCIL_BUFFER_BIT);

            let mut ctx = vg.begin_frame(win_w as f32, win_h as f32, px_ratio * scale);

            super::canvas::render_demo(
                &mut ctx,
                point2(mx as f32 / scale, my as f32 / scale),
                (win_w as f32 / scale, win_h as f32 / scale).into(),
                time as f32,
                BLOWUP != 0,
            );

            if true {
                super::blendish::run(&mut ctx, time as f32, rect(380.0, 50.0, 200.0, 200.0));
            }

            drop(ctx);

            backend.draw_commands(&vg.cmd, win_w as f32, win_h as f32, scale);

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
