extern crate oni2d;

mod sup;

//#[link(name = "nvg")]
//extern "C" {}

fn main() {
    env_logger::init();
    log::info!("start");

    use oni2d::{
        perf::{GraphStyle, PerfGraph},
        //Context,
        BackendGL,
        NFlags,
        gl,
    };

    const GLFW_CONTEXT_VERSION_MAJOR: i32 = 0x0002_2002;
    const GLFW_CONTEXT_VERSION_MINOR: i32 = 0x0002_2003;

    unsafe {
        assert!(glfwInit(), "Failed to init GLFW.");

        let mut fps = PerfGraph::new(GraphStyle::Fps, "Frame Time");

        glfwSetErrorCallback(errorcb);

        glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 2);
        glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 0);

        let window = glfwCreateWindow(2000, 1200, b"NanoVG\0".as_ptr(), null(), null());
        //window = glfwCreateWindow(1000, 600, "NanoVG", glfwGetPrimaryMonitor(), NULL);
        if window.is_null() {
            glfwTerminate();
            panic!("cant create window");
        }

        glfwSetKeyCallback(window, key);
        glfwMakeContextCurrent(window);

        let flags = NFlags::ANTIALIAS | NFlags::STENCIL_STROKES | NFlags::DEBUG;
        let mut vg = Context::new(BackendGL::new(flags));

        let data = DemoData::new(&mut vg);

        glfwSwapInterval(0);;

        glfwSetTime(0.0);
        let mut prevt = glfwGetTime();

        while !glfwWindowShouldClose(window) {
            let time = glfwGetTime();
            let dt = time - prevt;
            prevt = time;
        
            fps.update(dt as f32);
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
                gl::ClearColor(0.0,0.0,0.0,0.0);
            } else {
                gl::ClearColor(0.3, 0.3, 0.32, 1.0);
            }

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);

            vg.begin_frame(win_w as f32, win_h as f32, px_ratio * scale);

            render_demo(
                &mut vg,
                point2(mx as f32, my as f32) / scale,
                size2(win_w as f32, win_h as f32) / scale,
                time as f32, BLOWUP != 0, &data,
            );
            fps.render(&mut vg, 5.0, 5.0);

            vg.end_frame();

            /*
            if (screenshot) {
                screenshot = 0;
                save_screenshot(fbWidth, fbHeight, premult, "dump.png");
            }
            */
            
            glfwSwapBuffers(window);
            glfwPollEvents();
        }

        /*
        free_demo_data(vg, &data);
        nvgDeleteGL2(vg);
        */

        glfwTerminate();
    }
}


#[repr(C)] struct GLFWwindow(usize);
#[repr(C)] struct GLFWmonitor(usize);

extern "C" {
    fn glfwInit() -> bool;
    fn glfwTerminate();
    fn glfwSetWindowShouldClose(window: *mut GLFWwindow, value: i32);
    fn glfwSetErrorCallback(errorcb: extern fn(error: i32, desc: *const i8));

    fn glfwWindowHint(hint: i32, value: i32);

    fn glfwCreateWindow(
        width: i32, height: i32, title: *const u8,
        monitor: *const GLFWmonitor, share: *const GLFWwindow,
    ) -> *mut GLFWwindow;
    fn glfwSetKeyCallback(
        window: *mut GLFWwindow,
        key: extern fn(window: *mut GLFWwindow, key: i32, _scancode: i32, action: i32, _mods: i32),
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
    let desc = unsafe {
        std::ffi::CStr::from_ptr(desc)
    };
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


use slotmap::Key;

use std::ptr::null;
use std::f32::consts::PI;

use oni2d::{
    Winding, LineJoin, LineCap,
    Image, ImageFlags,
    Paint, Color,
    Context, Align,
    TextRow, GlyphPosition,
    utils::{
        deg2rad,
        clampf,
        minf,
        maxf,
        slice_start_end,
    },

    rect, Rect,
    size2, Size,
    point2, Point,
};

const ICON_SEARCH: char = '\u{1F50D}';
const ICON_CIRCLED_CROSS: char = '\u{2716}';
const ICON_CHEVRON_RIGHT: char = '\u{E75E}';
const ICON_CHECK: char = '\u{2713}';
const ICON_LOGIN: char = '\u{E740}';
const ICON_TRASH: char = '\u{E729}';

#[repr(C)]
pub struct DemoData {
    pub font_normal: i32,
    pub font_bold: i32,
    pub font_icons: i32,
    pub font_emoji: i32,
    pub images: [Image; 12],
}

impl DemoData {
    fn new(vg: &mut Context) -> Self {
        let mut images = [Image::null(); 12];
        
        for (i, image) in images.iter_mut().enumerate() {
            let file = format!("assets/images/image{}.jpg", i+1);
            let m = vg.create_image(&file, ImageFlags::empty());
            assert!(!m.is_null(), "Could not load {}.", file);
            *image = m;
        }

        let font_icons = vg.create_font("icons", "assets/fonts/entypo.ttf");
        assert_ne!(font_icons, -1, "Could not add font icons.");
        let font_normal = vg.create_font("sans", "assets/fonts/Roboto-Regular.ttf");
        assert_ne!(font_normal, -1, "Could not add font italic.");
        let font_bold = vg.create_font("sans-bold", "assets/fonts/Roboto-Bold.ttf");
        assert_ne!(font_bold, -1, "Could not add font bold.");
        let font_emoji = vg.create_font("emoji", "assets/fonts/NotoEmoji-Regular.ttf");
        assert_ne!(font_emoji, -1, "Could not add font emoji.");

        vg.add_fallback_font_id(font_normal, font_emoji);
        vg.add_fallback_font_id(font_bold, font_emoji);

        Self {
            font_normal,
            font_bold,
            font_icons,
            font_emoji,
            images
        }
    }
}

fn cp2utf8(cp: char, s: &mut [u8; 8]) -> &str {
    cp.encode_utf8(&mut s[..])
}

pub fn render_demo(
    vg: &mut Context, mouse: Point, wsize: Size,
    time: f32, blowup: bool, data: &DemoData,
) {
    let (width, height) = wsize.into();

    draw_eyes(vg, rect(width - 250.0, 50.0, 150.0, 100.0), mouse, time);
    draw_paragraph(vg, rect(width - 450.0, 50.0, 150.0, 100.0), mouse);
    draw_graph(vg, 0.0, height/2.0, width, height/2.0, time);
    draw_colorwheel(vg, rect(width - 300.0, height - 300.0, 250.0, 250.0), time);

    // Line joints
    draw_lines(vg, 120.0, height-50.0, 600.0, 50.0, time);

    // Line caps
    draw_widths(vg, 10.0, 50.0, 30.0);

    // Line caps
    draw_caps(vg, 10.0, 300.0, 30.0);

    draw_scissor(vg, 50.0, height-80.0, time);

    vg.save();
    {
        if blowup {
            vg.rotate((time*0.3).sin()*5.0/180.0*PI);
            vg.scale(2.0, 2.0);
        }

        // Widgets
        draw_window(vg, "Widgets `n Stuff", rect(50.0, 50.0, 300.0, 400.0));
        let (x, mut y) = (60.0, 95.0);
        draw_search_box(vg, "Search", rect(x,y,280.0,25.0));
        y += 40.0;
        draw_drop_down(vg, "Effects", rect(x,y,280.0,28.0));
        let popy = y + 14.0;
        y += 45.0;

        // Form
        draw_label(vg, "Login", rect(x,y, 280.0,20.0));
        y += 25.0;
        draw_edit_box(vg, "Email",  rect(x,y, 280.0,28.0));
        y += 35.0;
        draw_edit_box(vg, "Password", rect(x,y, 280.0,28.0));
        y += 38.0;
        draw_checkbox(vg, "Remember me", rect(x,y, 140.0,28.0));
        draw_button(vg, ICON_LOGIN.into(), "Sign in", rect(x+138.0, y, 140.0, 28.0),
            Color::new(0xFF_006080));
        y += 45.0;

        // Slider
        draw_label(vg, "Diameter", rect(x,y, 280.0,20.0));
        y += 25.0;
        draw_edit_box_num(vg, "123.00", "px", rect(x+180.0,y, 100.0,28.0));
        draw_slider(vg, 0.4, x,y, 170.0,28.0);
        y += 55.0;

        draw_button(vg, ICON_TRASH.into(), "Delete", rect(x, y, 160.0, 28.0), Color::new(0xFF_801008));
        draw_button(vg, None, "Cancel", rect(x+170.0, y, 110.0, 28.0), Color::new(0x00_000000));

        // Thumbnails box
        draw_thumbnails(vg, rect(365.0, popy-30.0, 160.0, 300.0), &data.images[..], time);

        vg.restore();
    }

    {
        // Canvas test
        use oni2d::canvas::*;
        let mut ctx = Canvas::new(vg);

        ctx.draw_rect(Rect {
            top: 50.0,
            left: 50.0,
            bottom: 150.0,
            right: 150.0,
        }, Paint::fill(0xFF_000000));
        ctx.draw_rrect(RRect {
            top: 50.0,
            left: 50.0,
            bottom: 150.0,
            right: 150.0,

            top_right: 15.0,
            top_left: 15.0,
            bottom_right: 15.0,
            bottom_left: 15.0,
        }, Paint::fill(0xFF_CC0000));

        ctx.draw_line([60.0, 60.0], [140.0, 140.0], Paint::stroke(0xFF_00CCCC));
    }
}

fn draw_window(vg: &mut Context, title: &str, rr: Rect) {
    let (x, y, w, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    let corner_radius = 3.0;

    vg.save();
    {
        // vg.ClearState();

        // Window
        vg.begin_path();
        vg.rrect(rr, corner_radius);
        vg.fill_color(Color::new(0xC0_1C1E22));
        //vg.fill_color(Color::rgba(0,0,0,128));
        vg.fill();

        // Drop shadow
        let shadow_paint = Paint::box_gradient(
            rect(x,y+2.0, w,h),
            corner_radius*2.0, 10.0,
            Color::new(0x80_000000),
            Color::new(0x00_000000),
        );
        vg.begin_path();
        vg.rect(rect(x-10.0,y-10.0, w+20.0,h+20.0));
        vg.rrect(rr, corner_radius);
        vg.path_winding(Winding::CW);
        vg.fill_paint(shadow_paint);
        vg.fill();

        // Header
        let header_paint = Paint::linear_gradient(
            x,y,x,y+15.0,
            Color::new(0x08_FFFFFF),
            Color::new(0x10_000000),
        );
        vg.begin_path();
        vg.rrect(rect(x+1.0,y+1.0, w-2.0,30.0), corner_radius-1.0);
        vg.fill_paint(header_paint);
        vg.fill();
        vg.begin_path();
        vg.move_to(x+0.5, y+0.5+30.0);
        vg.line_to(x+0.5+w-1.0, y+0.5+30.0);
        vg.stroke_color(Color::new(0x20_000000));
        vg.stroke();

        vg.font_size(18.0);
        vg.font_face(b"sans-bold\0");
        vg.text_align(Align::CENTER|Align::MIDDLE);

        vg.font_blur(2.0);
        vg.fill_color(Color::rgba(0,0,0,128));
        vg.text(x+w/2.0,y+16.0+1.0, title);

        vg.font_blur(0.0);
        vg.fill_color(Color::rgba(220,220,220,160));
        vg.text(x+w/2.0,y+16.0, title);

        vg.restore();
    }
}

fn draw_search_box(vg: &mut Context, text: &str, rr: Rect) {
    let (x, y, w, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    let mut icon = [0u8; 8];
    let corner_radius = h/2.0-1.0;

    // Edit
    let bg = Paint::box_gradient(
        rect(x,y+1.5, w,h), h/2.0,5.0,
        Color::rgba(0,0,0,16),
        Color::rgba(0,0,0,92),
    );
    vg.begin_path();
    vg.rrect(rr, corner_radius);
    vg.fill_paint(bg);
    vg.fill();

    if false {
        vg.begin_path();
        vg.rrect(rect(x+0.5,y+0.5, w-1.0,h-1.0), corner_radius-0.5);
        vg.stroke_color(Color::rgba(0,0,0,48));
        vg.stroke();
    }

    vg.font_size(h*1.3);
    vg.font_face(b"icons\0");
    vg.fill_color(Color::rgba(255,255,255,64));
    vg.text_align(Align::CENTER|Align::MIDDLE);
    vg.text(x+h*0.55, y+h*0.55, cp2utf8(ICON_SEARCH, &mut icon));

    vg.font_size(20.0);
    vg.font_face(b"sans\0");
    vg.fill_color(Color::rgba(255,255,255,32));

    vg.text_align(Align::LEFT|Align::MIDDLE);
    vg.text(x+h*1.05,y+h*0.5,text);

    vg.font_size(h*1.3);
    vg.font_face(b"icons\0");
    vg.fill_color(Color::rgba(255,255,255,32));
    vg.text_align(Align::CENTER|Align::MIDDLE);
    vg.text(x+w-h*0.55, y+h*0.55, cp2utf8(ICON_CIRCLED_CROSS, &mut icon));
}

fn draw_drop_down(vg: &mut Context, text: &str, bounds: Rect) {
    let (x, y, w, h) = (
        bounds.origin.x, bounds.origin.y,
        bounds.size.width, bounds.size.height,
    );

    let mut icon = [0u8; 8];
    let corner_radius = 4.0;

    let bg = Paint::linear_gradient(
        x,y,x,y+h,
        Color::rgba(255,255,255,16), Color::rgba(0,0,0,16),
    );
    vg.begin_path();
    vg.rrect(rect(x+1.0,y+1.0, w-2.0,h-2.0), corner_radius-1.0);
    vg.fill_paint(bg);
    vg.fill();

    vg.begin_path();
    vg.rrect(rect(x+0.5,y+0.5, w-1.0,h-1.0), corner_radius-0.5);
    vg.stroke_color(Color::rgba(0,0,0,48));
    vg.stroke();

    vg.font_size(20.0);
    vg.font_face(b"sans\0");
    vg.fill_color(Color::rgba(255,255,255,160));
    vg.text_align(Align::LEFT|Align::MIDDLE);
    vg.text(x+h*0.3,y+h*0.5,text);

    vg.font_size(h*1.3);
    vg.font_face(b"icons\0");
    vg.fill_color(Color::rgba(255,255,255,64));
    vg.text_align(Align::CENTER|Align::MIDDLE);
    vg.text(x+w-h*0.5, y+h*0.5, cp2utf8(ICON_CHEVRON_RIGHT, &mut icon));
}

fn draw_label(vg: &mut Context, text: &str, rr: Rect) {
    let (x, y, _, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    vg.font_size(18.0);
    vg.font_face(b"sans\0");

    vg.fill_color(Color::rgba(255, 255, 255, 128));

    vg.text_align(Align::LEFT|Align::MIDDLE);
    vg.text(x, y + h*0.5, text);
}

fn draw_edit_box_base(vg: &mut Context, rr: Rect) {
    let (x, y, w, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    let bg = Paint::box_gradient(
        rect(x+1.0,y+1.0+1.5, w-2.0,h-2.0), 3.0,4.0,
        Color::rgba(255,255,255,32),
        Color::rgba(32,32,32,32),
    );
    vg.begin_path();
    vg.rrect(rect(x+1.0,y+1.0, w-2.0,h-2.0), 4.0-1.0);
    vg.fill_paint(bg);
    vg.fill();

    vg.begin_path();
    vg.rrect(rect(x+0.5,y+0.5, w-1.0,h-1.0), 4.0-0.5);
    vg.stroke_color(Color::rgba(0,0,0,48));
    vg.stroke();
}

fn draw_edit_box(vg: &mut Context, text: &str, rr: Rect) {
    let (x, y, _, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    draw_edit_box_base(vg, rr);

    vg.font_size(20.0);
    vg.font_face(b"sans\0");
    vg.fill_color(Color::rgba(255,255,255,64));
    vg.text_align(Align::LEFT|Align::MIDDLE);
    vg.text(x+h*0.3,y+h*0.5,text);
}

fn draw_edit_box_num(vg: &mut Context, text: &str, units: &str, rr: Rect) {
    let (x, y, w, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);
    draw_edit_box_base(vg, rr);

    let (uw, _) = vg.text_bounds(0.0,0.0, units);

    vg.font_size(18.0);
    vg.font_face(b"sans\0");
    vg.fill_color(Color::rgba(255,255,255,64));
    vg.text_align(Align::RIGHT|Align::MIDDLE);
    vg.text(x+w-h*0.3,y+h*0.5,units);

    vg.font_size(20.0);
    vg.font_face(b"sans\0");
    vg.fill_color(Color::rgba(255,255,255,128));
    vg.text_align(Align::RIGHT|Align::MIDDLE);
    vg.text(x+w-uw-h*0.5,y+h*0.5,text);
}

fn draw_checkbox(vg: &mut Context, text: &str, rr: Rect) {
    let (x, y, _, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    let mut icon = [0u8; 8];

    vg.font_size(18.0);
    vg.font_face(b"sans\0");
    vg.fill_color(Color::rgba(255,255,255,160));

    vg.text_align(Align::LEFT|Align::MIDDLE);
    vg.text(x+28.0,y+h*0.5,text);

    let bg = Paint::box_gradient(
        rect(x+1.0, y+(h*0.5).floor()-9.0+1.0, 18.0, 18.0),
        3.0,3.0,
        Color::rgba(0,0,0,32), Color::rgba(0,0,0,92),
    );
    vg.begin_path();
    vg.rrect(rect(x+1.0,y+(h*0.5).floor()-9.0, 18.0,18.0), 3.0);
    vg.fill_paint(bg);
    vg.fill();

    vg.font_size(40.0);
    vg.font_face(b"icons\0");
    vg.fill_color(Color::rgba(255,255,255,128));
    vg.text_align(Align::CENTER|Align::MIDDLE);
    vg.text(x+9.0+2.0, y+h*0.5, cp2utf8(ICON_CHECK, &mut icon));
}

fn draw_button(
    vg: &mut Context,
    preicon: Option<char>,
    text: &str, rr: Rect, col: Color,
) {
    let (x, y, w, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    let mut icon = [0u8; 8];
    let corner_radius = 4.0;

    let alpha = if col.is_transparent_black() { 16 } else { 32 };
    let bg = Paint::linear_gradient(
        x,y,x,y+h,
        Color::rgba(255,255,255, alpha),
        Color::rgba(0,0,0, alpha),
    );
    vg.begin_path();
    vg.rrect(rect(x+1.0,y+1.0, w-2.0,h-2.0), corner_radius-1.0);
    if !col.is_transparent_black() {
        vg.fill_color(col);
        vg.fill();
    }
    vg.fill_paint(bg);
    vg.fill();

    vg.begin_path();
    vg.rrect(rect(x+0.5,y+0.5, w-1.0,h-1.0), corner_radius-0.5);
    vg.stroke_color(Color::rgba(0,0,0,48));
    vg.stroke();

    vg.font_size(20.0);
    vg.font_face(b"sans-bold\0");
    let (tw, _) = vg.text_bounds(0.0,0.0, text);
    let mut iw = 0.0;

    if let Some(preicon) = preicon {
        vg.font_size(h*1.3);
        vg.font_face(b"icons\0");
        iw = vg.text_bounds(0.0,0.0, cp2utf8(preicon, &mut icon)).0;
        iw += h*0.15;

        vg.font_size(h*1.3);
        vg.font_face(b"icons\0");
        vg.fill_color(Color::rgba(255,255,255,96));
        vg.text_align(Align::LEFT|Align::MIDDLE);
        vg.text(x+w*0.5-tw*0.5-iw*0.75, y+h*0.5, cp2utf8(preicon, &mut icon));
    }

    vg.font_size(20.0);
    vg.font_face(b"sans-bold\0");
    vg.text_align(Align::LEFT|Align::MIDDLE);
    vg.fill_color(Color::rgba(0,0,0,160));
    vg.text(x+w*0.5-tw*0.5+iw*0.25, y+h*0.5-1.0,text);
    vg.fill_color(Color::rgba(255,255,255,160));
    vg.text(x+w*0.5-tw*0.5+iw*0.25,y+h*0.5,text);
}

fn draw_slider(vg: &mut Context, pos: f32, x: f32, y: f32, w: f32, h: f32) {
    let cy = y+(h*0.5).floor();
    let kr = (h*0.25).floor();

    vg.save();
    {
        // vg.clear_state();

        // Slot
        let bg = Paint::box_gradient(
            rect(x,cy-2.0+1.0, w,4.0),
            2.0,2.0,
            Color::rgba(0,0,0,32), Color::rgba(0,0,0,128),
        );
        vg.begin_path();
        vg.rrect(rect(x,cy-2.0, w,4.0), 2.0);
        vg.fill_paint(bg);
        vg.fill();

        // Knob Shadow
        let bg = Paint::radial_gradient(
            x+(pos*w).floor(),cy+1.0,
            kr-3.0,kr+3.0,
            Color::rgba(0,0,0,64), Color::rgba(0,0,0,0),
        );
        vg.begin_path();
        vg.rect(rect(x+(pos*w).floor()-kr-5.0,cy-kr-5.0,kr*2.0+5.0+5.0,kr*2.0+5.0+5.0+3.0));
        vg.circle(x+(pos*w).floor(),cy, kr);
        vg.path_winding(Winding::CW);
        vg.fill_paint(bg);
        vg.fill();

        // Knob
        let knob = Paint::linear_gradient(
            x,cy-kr,x,cy+kr,
            Color::rgba(255,255,255,16), Color::rgba(0,0,0,16),
        );
        vg.begin_path();
        vg.circle(x+(pos*w).floor(),cy, kr-1.0);
        vg.fill_color(Color::rgba(40,43,48,255));
        vg.fill();
        vg.fill_paint(knob);
        vg.fill();

        vg.begin_path();
        vg.circle(x+(pos*w).floor(),cy, kr-0.5);
        vg.stroke_color(Color::rgba(0,0,0,92));
        vg.stroke();

        vg.restore();
    }
}

fn draw_eyes(vg: &mut Context, rr: Rect, mouse: Point, time: f32) {
    let (x, y, w, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    let (mx, my) = mouse.into();

    let ex = w *0.23;
    let ey = h * 0.5;
    let lx = x + ex;
    let ly = y + ey;
    let rx = x + w - ex;
    let ry = y + ey;
    let br = minf(ex, ey) * 0.5;
    let blink = 1.0 - (time*0.5).sin().powf(200.0)*0.8;

    let bg = Paint::linear_gradient(
        x,y+h*0.5,
        x+w*0.1,y+h,
        Color::rgba(0,0,0,32), Color::rgba(0,0,0,16),
    );
    vg.begin_path();
    vg.ellipse(lx+3.0,ly+16.0, ex,ey);
    vg.ellipse(rx+3.0,ry+16.0, ex,ey);
    vg.fill_paint(bg);
    vg.fill();

    let bg = Paint::linear_gradient(
        x,y+h*0.25,x+w*0.1,y+h,
        Color::rgba(220,220,220,255), Color::rgba(128,128,128,255),
    );
    vg.begin_path();
    vg.ellipse(lx,ly, ex,ey);
    vg.ellipse(rx,ry, ex,ey);
    vg.fill_paint(bg);
    vg.fill();

    let mut dx = (mx - rx) / (ex * 10.0);
    let mut dy = (my - ry) / (ey * 10.0);
    let dd = (dx*dx+dy*dy).sqrt();
    if dd > 1.0 {
        dx /= dd; dy /= dd;
    }
    dx *= ex*0.4;
    dy *= ey*0.5;
    vg.begin_path();
    vg.ellipse(lx+dx,ly+dy+ey*0.25*(1.0-blink), br,br*blink);
    vg.fill_color(Color::rgba(32,32,32,255));
    vg.fill();

    let mut dx = (mx - rx) / (ex * 10.0);
    let mut dy = (my - ry) / (ey * 10.0);
    let dd = (dx*dx+dy*dy).sqrt();
    if dd > 1.0 {
        dx /= dd; dy /= dd;
    }
    dx *= ex*0.4;
    dy *= ey*0.5;
    vg.begin_path();
    vg.ellipse(rx+dx,ry+dy+ey*0.25*(1.0-blink), br,br*blink);
    vg.fill_color(Color::rgba(32,32,32,255));
    vg.fill();

    let gloss = Paint::radial_gradient(
        lx-ex*0.25,ly-ey*0.5, ex*0.1,ex*0.75,
        Color::rgba(255,255,255,128), Color::rgba(255,255,255,0),
    );
    vg.begin_path();
    vg.ellipse(lx,ly, ex,ey);
    vg.fill_paint(gloss);
    vg.fill();

    let gloss = Paint::radial_gradient(
        rx-ex*0.25,ry-ey*0.5, ex*0.1,ex*0.75,
        Color::rgba(255,255,255,128), Color::rgba(255,255,255,0),
    );
    vg.begin_path();
    vg.ellipse(rx,ry, ex,ey);
    vg.fill_paint(gloss);
    vg.fill();
}

fn draw_graph(vg: &mut Context, x: f32, y: f32, w: f32, h: f32, time: f32) {
    let samples = [
        (1.0+(time*1.2345  +(time*0.33457).cos()*0.44).sin())*0.5,
        (1.0+(time*0.68363 +(time*1.3).cos()*1.55).sin())*0.5,
        (1.0+(time*1.1642  +(time*0.33457).cos()*1.24).sin())*0.5,
        (1.0+(time*0.56345 +(time*1.63).cos()*0.14).sin())*0.5,
        (1.0+(time*1.6245  +(time*0.254).cos()*0.3).sin())*0.5,
        (1.0+(time*0.345   +(time*0.03).cos()*0.6).sin())*0.5,
    ];

    let dx = w/5.0;

    let mut sx = [0f32; 6];
    let mut sy = [0f32; 6];
    for i in 0..6 {
        sx[i] = x+(i as f32) *dx;
        sy[i] = y+h*samples[i]*0.8;
    }

    // Graph background
    let bg = Paint::linear_gradient(
        x,y,x,y+h,
        Color::rgba(0,160,192,0), Color::rgba(0,160,192,64),
    );
    vg.begin_path();
    vg.move_to(sx[0], sy[0]);
    for i in 1..6 {
        vg.bezier_to(sx[i-1]+dx*0.5,sy[i-1], sx[i]-dx*0.5,sy[i], sx[i],sy[i]);
    }
    vg.line_to(x+w, y+h);
    vg.line_to(x, y+h);
    vg.fill_paint(bg);
    vg.fill();

    // Graph line
    vg.begin_path();
    vg.move_to(sx[0], sy[0]+2.0);
    for i in 1..6 {
        vg.bezier_to(sx[i-1]+dx*0.5,sy[i-1]+2.0, sx[i]-dx*0.5,sy[i]+2.0, sx[i],sy[i]+2.0);
    }
    vg.stroke_color(Color::rgba(0,0,0,32));
    vg.stroke_width(3.0);
    vg.stroke();

    vg.begin_path();
    vg.move_to(sx[0], sy[0]);
    for i in 1..6 {
        vg.bezier_to(sx[i-1]+dx*0.5,sy[i-1], sx[i]-dx*0.5,sy[i], sx[i],sy[i]);
    }
    vg.stroke_color(Color::rgba(0,160,192,255));
    vg.stroke_width(3.0);
    vg.stroke();

    // Graph sample pos
    for i in 0..6 {
        let bg = Paint::radial_gradient(
            sx[i],sy[i]+2.0, 3.0,8.0,
            Color::rgba(0,0,0,32), Color::rgba(0,0,0,0),
        );

        vg.begin_path();
        vg.rect(rect(sx[i]-10.0, sy[i]-10.0+2.0, 20.0,20.0));
        vg.fill_paint(bg);
        vg.fill();
    }

    vg.begin_path();
    for i in 0..6 {
        vg.circle(sx[i], sy[i], 4.0);
    }
    vg.fill_color(Color::rgba(0,160,192,255));
    vg.fill();
    vg.begin_path();
    for i in 0..6 {
        vg.circle(sx[i], sy[i], 2.0);
    }
    vg.fill_color(Color::rgba(220,220,220,255));
    vg.fill();

    vg.stroke_width(1.0);
}

fn draw_spinner(vg: &mut Context, cx: f32, cy: f32, r: f32, time: f32) {
    let a0 = 0.0 + time*6.0;
    let a1 = PI + time*6.0;
    let r0 = r;
    let r1 = r * 0.75;

    vg.save();
    {
        vg.begin_path();
        vg.arc(cx,cy, r0, a0, a1, Winding::CW);
        vg.arc(cx,cy, r1, a1, a0, Winding::CCW);
        vg.close_path();

        let rr = (r0+r1)*0.5;
        let ax = cx + a0.cos() * rr;
        let ay = cy + a0.sin() * rr;
        let bx = cx + a1.cos() * rr;
        let by = cy + a1.sin() * rr;
        let paint = Paint::linear_gradient(
            ax,ay, bx,by,
            Color::rgba(0,0,0,0), Color::rgba(0,0,0,128),
        );

        vg.fill_paint(paint);
        vg.fill();

        vg.restore();
    }
}

fn draw_thumbnails(vg: &mut Context, rr: Rect, images: &[Image], time: f32) {
    let (x, y, width, height) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    let corner_radius = 3.0;
    let thumb = 60.0;
    let arry = 30.5;
    let stackh = ((images.len() / 2) as f32) * (thumb+10.0) + 10.0;
    let u1 = (1.0+(time*0.5).cos()) * 0.5;
    let u2 = (1.0-(time*0.2).cos()) * 0.5;

    vg.save();
    {
        // vg.clear_state();

        // Drop shadow
        let shadow = Paint::box_gradient(
            rect(x,y+4.0, width,height), corner_radius*2.0, 20.0,
            Color::rgba(0,0,0,128), Color::rgba(0,0,0,0),
        );
        vg.begin_path();
        vg.rect(rect(x-10.0,y-10.0, width+20.0,height+20.0));
        vg.rrect(rr, corner_radius);
        vg.path_winding(Winding::CW);
        vg.fill_paint(shadow);
        vg.fill();

        // Window
        vg.begin_path();
        vg.rrect(rr, corner_radius);
        vg.move_to(x-10.0,y+arry);
        vg.line_to(x+1.0,y+arry-11.0);
        vg.line_to(x+1.0,y+arry+11.0);
        vg.fill_color(Color::rgba(200,200,200,255));
        vg.fill();

        vg.save();
        {
            vg.scissor(rr);
            vg.translate(0.0, -(stackh - height)*u1);

            let dv = 1.0 / (images.len()-1) as f32;

            for (i, &image) in images.iter().enumerate() {
                let mut tx = x+10.0;
                let mut ty = y+10.0;
                tx += (thumb+10.0) * (i%2) as f32;
                ty += (thumb+10.0) * (i/2) as f32;

                let (imgw, imgh) = vg.image_size(image).expect("image_size");
                let (iw, ih, ix, iy);
                if imgw < imgh {
                    iw = thumb;
                    ih = iw * (imgh as f32)/(imgw as f32);
                    ix = 0.0;
                    iy = -(ih-thumb)*0.5;
                } else {
                    ih = thumb;
                    iw = ih * (imgw as f32)/(imgh as f32);
                    ix = -(iw-thumb)*0.5;
                    iy = 0.0;
                }

                let v = (i as f32) * dv;
                let a = clampf((u2-v) / dv, 0.0, 1.0);

                if a < 1.0 {
                    draw_spinner(vg, tx+thumb/2.0,ty+thumb/2.0, thumb*0.25, time);
                }

                let img = Paint::image_pattern(tx+ix, ty+iy, iw,ih, 0.0/180.0*PI, image, a);
                vg.begin_path();
                vg.rrect(rect(tx,ty, thumb,thumb), 5.0);
                vg.fill_paint(img);
                vg.fill();

                let shadow = Paint::box_gradient(
                    rect(tx-1.0,ty, thumb+2.0,thumb+2.0), 5.0, 3.0,
                    Color::rgba(0,0,0,128), Color::rgba(0,0,0,0));
                vg.begin_path();
                vg.rect(rect(tx-5.0,ty-5.0, thumb+10.0,thumb+10.0));
                vg.rrect(rect(tx,ty, thumb,thumb), 6.0);
                vg.path_winding(Winding::CW);
                vg.fill_paint(shadow);
                vg.fill();

                vg.begin_path();
                vg.rrect(rect(tx+0.5,ty+0.5, thumb-1.0,thumb-1.0), 4.0-0.5);
                vg.stroke_width(1.0);
                vg.stroke_color(Color::rgba(255,255,255,192));
                vg.stroke();
            }
            vg.restore();
        }

        // Hide fades
        let fade = Paint::linear_gradient(
            x,y,x,y+6.0,
            Color::rgba(200,200,200,255), Color::rgba(200,200,200,0));
        vg.begin_path();
        vg.rect(rect(x+4.0,y,width-8.0,6.0));
        vg.fill_paint(fade);
        vg.fill();

        let fade = Paint::linear_gradient(
            x,y+height,x,y+height-6.0,
            Color::rgba(200,200,200,255), Color::rgba(200,200,200,0));
        vg.begin_path();
        vg.rect(rect(x+4.0,y+height-6.0,width-8.0,6.0));
        vg.fill_paint(fade);
        vg.fill();

        // Scroll bar
        let shadow = Paint::box_gradient(
            rect(x+width-12.0+1.0,y+4.0+1.0, 8.0,height-8.0), 3.0,4.0,
            Color::rgba(0,0,0,32), Color::rgba(0,0,0,92));
        vg.begin_path();
        vg.rrect(rect(x+width-12.0,y+4.0, 8.0,height-8.0), 3.0);
        vg.fill_paint(shadow);
        // vg.fill_color(Color::rgba(255,0,0,128));
        vg.fill();

        let scrollh = (height/stackh) * (height-8.0);
        let shadow = Paint::box_gradient(
            rect(x+width-12.0-1.0,y+4.0+(height-8.0-scrollh)*u1-1.0, 8.0,scrollh), 3.0,4.0,
            Color::rgba(220,220,220,255), Color::rgba(128,128,128,255));
        vg.begin_path();
        vg.rrect(rect(x+width-12.0+1.0,y+4.0+1.0 + (height-8.0-scrollh)*u1, 8.0-2.0,scrollh-2.0), 2.0);
        vg.fill_paint(shadow);
        // vg.fill_color(Color::rgba(0,0,0,128));
        vg.fill();

        vg.restore();
    }
}

fn draw_colorwheel(vg: &mut Context, rr: Rect, time: f32) {
    let (x, y, w, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);
    let hue = (time * 0.12).sin();

    vg.save();

    /*
    vg.BeginPath();
    vg.Rect(x,y,w,h);
    vg.FillColor(Color::rgba(255,0,0,128));
    vg.Fill();
    */

    let cx = x + w*0.5;
    let cy = y + h*0.5;
    let r1 = if w < h { w } else { h } * 0.5 - 5.0;
    let r0 = r1 - 20.0;
    let aeps = 0.5 / r1;    // half a pixel arc length in radians (2pi cancels out).

    for i in 0..6 {
        let a0 = (i as f32) / 6.0 * PI * 2.0 - aeps;
        let a1 = ((i as f32)+1.0) / 6.0 * PI * 2.0 + aeps;
        vg.begin_path();
        vg.arc(cx,cy, r0, a0, a1, Winding::CW);
        vg.arc(cx,cy, r1, a1, a0, Winding::CCW);
        vg.close_path();
        let ax = cx + a0.cos() * (r0+r1)*0.5;
        let ay = cy + a0.sin() * (r0+r1)*0.5;
        let bx = cx + a1.cos() * (r0+r1)*0.5;
        let by = cy + a1.sin() * (r0+r1)*0.5;
        let paint = Paint::linear_gradient(
            ax,ay, bx,by,
            Color::hsla(a0/(PI*2.0),1.0,0.55,255),
            Color::hsla(a1/(PI*2.0),1.0,0.55,255));
        vg.fill_paint(paint);
        vg.fill();
    }

    vg.begin_path();
    vg.circle(cx,cy, r0-0.5);
    vg.circle(cx,cy, r1+0.5);
    vg.stroke_color(Color::rgba(0,0,0,64));
    vg.stroke_width(1.0);
    vg.stroke();

    // Selector
    vg.save();
    vg.translate(cx,cy);
    vg.rotate(hue*PI*2.0);

    // Marker on
    vg.stroke_width(2.0);
    vg.begin_path();
    vg.rect(rect(r0-1.0,-3.0,r1-r0+2.0,6.0));
    vg.stroke_color(Color::rgba(255,255,255,192));
    vg.stroke();

    let paint = Paint::box_gradient(
        rect(r0-3.0,-5.0,r1-r0+6.0,10.0), 2.0,4.0,
        Color::rgba(0,0,0,128), Color::rgba(0,0,0,0));
    vg.begin_path();
    vg.rect(rect(r0-2.0-10.0,-4.0-10.0,r1-r0+4.0+20.0,8.0+20.0));
    vg.rect(rect(r0-2.0,-4.0,r1-r0+4.0,8.0));
    vg.path_winding(Winding::CW);
    vg.fill_paint(paint);
    vg.fill();

    // Center triangle
    let radius = r0 - 6.0;
    let ax = (120.0/180.0*PI).cos() * radius;
    let ay = (120.0/180.0*PI).sin() * radius;
    let bx = (-120.0/180.0*PI).cos() * radius;
    let by = (-120.0/180.0*PI).sin() * radius;
    vg.begin_path();
    vg.move_to(radius,0.0);
    vg.line_to(ax,ay);
    vg.line_to(bx,by);
    vg.close_path();

    let paint = Paint::linear_gradient(
        radius,0.0, ax,ay,
        Color::hsla(hue,1.0,0.5,255), Color::rgba(255,255,255,255));
    vg.fill_paint(paint);
    vg.fill();

    let paint = Paint::linear_gradient(
        (radius+ax)*0.5,(0.0+ay)*0.5, bx,by,
        Color::rgba(0,0,0,0), Color::rgba(0,0,0,255));
    vg.fill_paint(paint);
    vg.fill();
    vg.stroke_color(Color::rgba(0,0,0,64));
    vg.stroke();

    // Select circle on triangle
    let ax = (120.0/180.0*PI).cos() * radius*0.3;
    let ay = (120.0/180.0*PI).sin() * radius*0.4;
    vg.stroke_width(2.0);
    vg.begin_path();
    vg.circle(ax,ay,5.0);
    vg.stroke_color(Color::rgba(255,255,255,192));
    vg.stroke();

    let paint = Paint::radial_gradient(
        ax,ay, 7.0,9.0,
        Color::rgba(0,0,0,64), Color::rgba(0,0,0,0));
    vg.begin_path();
    vg.rect(rect(ax-20.0,ay-20.0,40.0,40.0));
    vg.circle(ax,ay,7.0);
    vg.path_winding(Winding::CW);
    vg.fill_paint(paint);
    vg.fill();

    vg.restore();

    vg.restore();
}

fn draw_lines(vg: &mut Context, x: f32, y: f32, w: f32, _h: f32, t: f32) {
    use oni2d::canvas::*;
    let mut ctx = Canvas::new(vg);

    let pad = 5.0;
    let size = w/9.0 - pad*2.0;

    let joins = [ LineJoin::Miter, LineJoin::Round, LineJoin::Bevel ];
    let caps = [ LineCap::Butt, LineCap::Round, LineCap::Square ];

    let pts = [
        -size*0.25 + (t*0.3).cos() * size*0.5,
        (t*0.3).sin() * size*0.5,
        -size*0.25,
        0.0,
        size*0.25,
        0.0,
        size*0.25 + (-t*0.3).cos() * size*0.5,
        (-t*0.3).sin() * size*0.5,
    ];

    for (i, &cap) in caps.iter().enumerate() {
        for (j, &join) in joins.iter().enumerate() {
            let fx = x + size*0.5 + ((i*3+j) as f32)/9.0*w + pad;
            let fy = y - size*0.5 + pad;

            let mut path = Path::new();
            path.move_to(fx+pts[0], fy+pts[1]);
            path.line_to(fx+pts[2], fy+pts[3]);
            path.line_to(fx+pts[4], fy+pts[5]);
            path.line_to(fx+pts[6], fy+pts[7]);

            ctx.draw_path(&mut path, Paint::stroke(0xA0_000000)
                .with_stroke_width(size*0.3)
                .with_stroke_cap(cap)
                .with_stroke_join(join));

            ctx.draw_path(&mut path, Paint::stroke(0xFF_00C0FF)
                .with_stroke_width(1.0)
                .with_stroke_cap(LineCap::Butt)
                .with_stroke_join(LineJoin::Bevel));
        }
    }

}


fn draw_paragraph(vg: &mut Context, rr: Rect, mouse: Point) {
    let (x, mut y, width, _) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    let (mx, my) = mouse.into();
    let text = "This is longer chunk of text.\n  \n  Would have used lorem ipsum but she    was busy jumping over the lazy dog with the fox and all the men who came to the aid of the party.🎉".as_bytes();

    vg.save();

    vg.font_size(18.0);
    vg.font_face(b"sans\0");
    vg.text_align(Align::LEFT|Align::TOP);
    let lineh = vg.text_metrics().unwrap().line_gap;

    // The text break API can be used to fill a large buffer of rows,
    // or to iterate over the text just few lines (or just one) at a time.
    // The "next" variable of the last returned item tells where to continue.
    let (mut start, end) = slice_start_end(text);

    let mut lnum = 0;

    let mut gx = 0.0;
    let mut gy = 0.0;

    let mut rows: [TextRow; 3] = unsafe { core::mem::zeroed() };
    let mut glyphs: [GlyphPosition; 100] = unsafe { core::mem::zeroed() };

    let mut gutter = 0isize;

    loop {
        let text = unsafe {
            let len = end as usize - start as usize;
            let slice = std::slice::from_raw_parts(start, len);
            std::str::from_utf8_unchecked(slice)
        };
        let rows = vg.text_break_lines(text, width, &mut rows);
        if rows.is_empty() { break }

        for row in rows {
            let hit = mx > x && mx < (x+width) && my >= y && my < (y+lineh);

            vg.begin_path();
            vg.fill_color(Color::rgba(255,255,255, if hit { 64 } else { 16 }));
            vg.rect(rect(x, y, row.width, lineh));
            vg.fill();

            vg.fill_color(Color::rgba(255,255,255,255));
            vg.text(x, y, row.text());

            if hit {
                let mut caretx = if mx < x+row.width/2.0 {
                    x
                } else {
                    x+row.width
                };
                let glyphs = vg.text_glyph_positions(x, y, row.text(), &mut glyphs);

                let mut px = x;
                for j in 0..glyphs.len() {
                    let x0 = glyphs[j].x;
                    let x1 = if j+1 < glyphs.len() { glyphs[j+1].x } else { x+row.width };
                    gx = x0 * 0.3 + x1 * 0.7;
                    if mx >= px && mx < gx {
                        caretx = glyphs[j].x;
                    }
                    px = gx;
                }

                vg.begin_path();
                vg.fill_color(Color::rgba(255,192,0,255));
                vg.rect(rect(caretx, y, 1.0, lineh));
                vg.fill();

                gutter = lnum+1;
                gx = x - 10.0;
                gy = y + lineh/2.0;
            }
            lnum += 1;
            y += lineh;
        }
        // Keep going...
        start = rows[rows.len()-1].next;
    }

    if gutter != 0 {
        use std::fmt::Write;
        let mut txt = arrayvec::ArrayString::<[_; 16]>::new();
        txt.write_fmt(format_args!("{}", gutter)).unwrap();

        vg.font_size(13.0);
        vg.text_align(Align::RIGHT|Align::MIDDLE);

        let (_, bounds) = vg.text_bounds(gx,gy, &txt);

        vg.begin_path();
        vg.fill_color(Color::rgba(255,192,0,255));
        vg.rrect(rect(
                bounds[0].floor()-4.0,
                bounds[1].floor()-2.0,
                (bounds[2]-bounds[0]).floor()+8.0,
                (bounds[3]-bounds[1]).floor()+4.0,
            ), ((bounds[3]-bounds[1]).floor()+4.0)/2.0 - 1.0,
        );
        vg.fill();

        vg.fill_color(Color::rgba(32,32,32,255));
        vg.text(gx,gy, &txt);
    }
    vg.line_height(1.2);

    y += 20.0;

    vg.font_size(13.0);
    vg.text_align(Align::LEFT|Align::TOP);

    let bounds = vg.text_box_bounds(
        x,y, 150.0,
        "Hover your mouse over the text to see calculated caret position.");

    // Fade the tooltip out when close to it.
    let gx = ((mx - (bounds[0]+bounds[2])*0.5) / (bounds[0] - bounds[2])).abs();
    let gy = ((my - (bounds[1]+bounds[3])*0.5) / (bounds[1] - bounds[3])).abs();
    let a = maxf(gx, gy) - 0.5;
    let a = clampf(a, 0.0, 1.0);
    vg.global_alpha(a);

    vg.begin_path();
    vg.fill_color(Color::rgba(220,220,220,255));
    vg.rrect(rect(
            bounds[0]-2.0,bounds[1]-2.0,
            (bounds[2]-bounds[0]).floor()+4.0,
            (bounds[3]-bounds[1]).floor()+4.0,
        ), 3.0);
    let px = ((bounds[2]+bounds[0])/2.0).floor();
    vg.move_to(px,bounds[1] - 10.0);
    vg.line_to(px+7.0,bounds[1]+1.0);
    vg.line_to(px-7.0,bounds[1]+1.0);
    vg.fill();

    vg.fill_color(Color::rgba(0,0,0,220));
    vg.text_box(x,y, 150.0, "Hover your mouse over the text to see calculated caret position.");

    vg.restore();
}

fn draw_widths(vg: &mut Context, x: f32, y: f32, width: f32) {
    vg.save();
    vg.stroke_color(Color::rgba(0,0,0,255));

    let mut y = y;
    for i in 0..20 {
        let w = ((i as f32)+0.5)*0.1;
        vg.stroke_width(w);
        vg.begin_path();
        vg.move_to(x,y);
        vg.line_to(x+width,y+width*0.3);
        vg.stroke();
        y += 10.0;
    }

    vg.restore();
}

fn draw_caps(vg: &mut Context, x: f32, y: f32, width: f32) {
    let caps = [ LineCap::Butt, LineCap::Round, LineCap::Square ];
    let line_width = 8.0;

    use oni2d::canvas::*;

    let mut ctx = Canvas::new(vg);

    ctx.draw_rect(Rect {
        left: x-line_width/2.0,
        top: y,
        right: x + width+line_width/2.0,
        bottom: y + 40.0,
    }, Paint::fill(0x20_FFFFFF));
    ctx.draw_rect(Rect {
        left: x,
        top: y,
        right: x + width,
        bottom: y + 40.0,
    }, Paint::fill(0x20_FFFFFF));

    let paint = Paint::stroke(0xFF_000000)
        .with_stroke_width(line_width);

    for (i, &cap) in caps.iter().enumerate() {
        let y = y + ((i*10) as f32) + 5.0;
        ctx.draw_line([x, y], [x+width, y], paint.with_stroke_cap(cap))
    }
}

fn draw_scissor(vg: &mut Context, x: f32, y: f32, t: f32) {
    vg.save();

    // Draw first rect and set scissor to it's area.
    vg.translate(x, y);
    vg.rotate(deg2rad(5.0));

    let area = rect(-20.0,-20.0,60.0,40.0);
    vg.fill_rect(area, Color::rgba(255,0,0,255));
    vg.scissor(area);

    // Draw second rectangle with offset and rotation.
    vg.translate(40.0,0.0);
    vg.rotate(t);

    // Draw the intended second rectangle without any scissoring.
    {
        vg.save();
        vg.reset_scissor();
        vg.fill_rect(rect(-20.0,-10.0,60.0,30.0), Color::rgba(255,128,0,64));
        vg.restore();
    }

    // Draw second rectangle with combined scissoring.
    vg.intersect_scissor(rect(-20.0,-10.0,60.0,30.0));
    vg.fill_rect(rect(-20.0,-10.0,60.0,30.0), Color::rgba(255,128,0,255));

    vg.restore();
}

/*
fn free_demo_data(vg: &mut Context, data: &mut DemoData) {
    for &m in &data.images {
        vg.delete_image(m);
    }
}

fn unpremultiply(image: &mut [u8], w: usize, h: usize, stride: usize) {
    // Unpremultiply
    for y in 0..h {
        let mut row = &mut image[y*stride..];
        for x in 0..w {
            let r = row[0] as i32;
            let g = row[1] as i32;
            let b = row[2] as i32;
            let a = row[3] as i32;
            if a != 0 {
                row[0] = mini(r*255 / a, 255) as u8;
                row[1] = mini(g*255 / a, 255) as u8;
                row[2] = mini(b*255 / a, 255) as u8;
            }
            row = &mut row[4..];
        }
    }

    // Defringe
    for y in 0..h {
        let mut row = y*stride;
        for x in 0..w {
            let mut r = 0;
            let mut g = 0;
            let mut b = 0;
            let mut a = image[row + 3];
            let mut n = 0;
            if a == 0 {
                if x-1 > 0 && image[row - 1] != 0 {
                    r += image[row - 4];
                    g += image[row - 3];
                    b += image[row - 2];
                    n += 1;
                }
                if x+1 < w && image[row + 7] != 0 {
                    r += image[row + 4];
                    g += image[row + 5];
                    b += image[row + 6];
                    n += 1;
                }
                if y-1 > 0 && image[row-stride+3] != 0 {
                    r += image[row-stride+0];
                    g += image[row-stride+1];
                    b += image[row-stride+2];
                    n += 1;
                }
                if y+1 < h && image[row+stride+3] != 0 {
                    r += image[row+stride+0];
                    g += image[row+stride+1];
                    b += image[row+stride+2];
                    n += 1;
                }
                if n > 0 {
                    image[row+0] = r/n;
                    image[row+1] = g/n;
                    image[row+2] = b/n;
                }
            }
            row += 4;
        }
    }
}

fn set_alpha(image: &mut [u8], w: usize, h: usize, stride: usize, a: u8) {
    for y in 0..h {
        let row = &mut image[y*stride..];
        for x in 0..w {
            row[x*4+3] = a;
        }
    }
}

fn flip_horizontal(image: &mut [u8], w: usize, h: usize, stride: usize) {
    let (mut i, mut j) = (0, h-1);
    while i < j {
        let ri = image[i * stride..].as_mut_ptr();
        let rj = image[j * stride..].as_mut_ptr();
        for k in 0..w*4 {
            unsafe {
                let t = ri.add(k).read();
                ri.add(k).write(rj.add(k).read());
                rj.add(k).write(t);
            }
        }
        i += 1;
        j -= 1;
    }
}

fn save_screenshot(_w: usize, _h: usize, _premult: bool, _name: *const u8) {
    println!("unimplemented: save_screen_shot")
    unsigned char* image = (unsigned char*)malloc(w*h*4);
    if (image == NULL)
            return;
    glReadPixels(0, 0, w, h, GL_RGBA, GL_UNSIGNED_BYTE, image);
    if premult {
        unpremultiplyAlpha(image, w, h, w*4);
    } else {
        setAlpha(image, w, h, w*4, 255);
    }
    flipHorizontal(image, w, h, w*4);
    stbi_write_png(name, w, h, 4, image, w*4);
    free(image);
}
*/