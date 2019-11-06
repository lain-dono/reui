#![feature(clamp)]

mod sup;
use self::sup::*;

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

        let window = glfwCreateWindow(2000, 1200, b"ONI2D\0".as_ptr(), null(), null());
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
                (win_w as f32 / scale, win_h as f32 / scale).into(),
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
    Image, ImageFlags,
    Context, Align,
    TextRow, GlyphPosition,

    canvas::Canvas,

    utils::slice_start_end,

    math::{
        Color,
        rect, Rect,
        point2, Point,
        Vector,
    },
};

/*
const ICON_SEARCH: char = '\u{1F50D}';
const ICON_CIRCLED_CROSS: char = '\u{2716}';
const ICON_CHEVRON_RIGHT: char = '\u{E75E}';
const ICON_CHECK: char = '\u{2713}';
const ICON_LOGIN: char = '\u{E740}';
const ICON_TRASH: char = '\u{E729}';
*/

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
            let m = vg.create_image(&file, ImageFlags::REPEATX | ImageFlags::REPEATY);
            assert!(!m.is_null(), "Could not load {}.", file);
            *image = m;
        }

        let font_icons = vg.create_font("icons", "assets/fonts/entypo.ttf");
        let font_normal = vg.create_font("sans", "assets/fonts/Roboto-Regular.ttf");
        let font_bold = vg.create_font("sans-bold", "assets/fonts/Roboto-Bold.ttf");
        let font_emoji = vg.create_font("emoji", "assets/fonts/NotoEmoji-Regular.ttf");

        assert_ne!(font_icons, -1, "Could not add font icons.");
        assert_ne!(font_normal, -1, "Could not add font italic.");
        assert_ne!(font_bold, -1, "Could not add font bold.");
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
    vg: &mut Context, mouse: Point, wsize: Vector,
    time: f32, blowup: bool, data: &DemoData,
) {
    let (width, height) = wsize.into();

    draw_paragraph(vg, rect(width - 450.0, 50.0, 150.0, 100.0), mouse);

    {
        let mut ctx = Canvas::new(vg);
        draw_colorwheel(&mut ctx, rect(width - 300.0, height - 300.0, 250.0, 250.0), time);
        draw_eyes(&mut ctx, rect(width - 250.0, 50.0, 150.0, 100.0), mouse, time);
        draw_graph(&mut ctx, 0.0, height/2.0, width, height/2.0, time);
        // Line joints
        draw_lines(&mut ctx, 120.0, height-50.0, 600.0, 50.0, time);
        // Line caps
        draw_widths(&mut ctx, 10.0, 50.0, 30.0);
        // Line caps
        draw_caps(&mut ctx, 10.0, 300.0, 30.0);
        draw_scissor(&mut ctx, 50.0, height-80.0, time);
    }

    {
        let mut ctx = Canvas::new(vg);
        if blowup {
            ctx.rotate((time*0.3).sin()*5.0/180.0*PI);
            ctx.scale(2.0);
        }

        // Widgets
        draw_window(&mut ctx, "Widgets `n Stuff", rect(50.0, 50.0, 300.0, 400.0));

        let (x, mut y) = (60.0, 95.0);
        draw_search_box(&mut ctx, "Search", rect(x,y,280.0,25.0));
        y += 40.0;
        draw_drop_down(&mut ctx, "Effects", rect(x,y,280.0,28.0));
        let popy = y + 14.0;
        y += 45.0;

        // Form
        draw_label(&mut ctx, "Login", rect(x,y, 280.0,20.0));
        y += 25.0;
        draw_edit_box(&mut ctx, "Email",  rect(x,y, 280.0,28.0));
        y += 35.0;
        draw_edit_box(&mut ctx, "Password", rect(x,y, 280.0,28.0));
        y += 38.0;
        draw_checkbox(&mut ctx, "Remember me", rect(x,y, 140.0,28.0));
        draw_button(&mut ctx, ICON_LOGIN, "Sign in", rect(x+138.0, y, 140.0, 28.0), 0xFF_006080);
        y += 45.0;

        // Slider
        draw_label(&mut ctx, "Diameter", rect(x,y, 280.0,20.0));
        y += 25.0;
        draw_edit_box_num(&mut ctx, "123.00", "px", rect(x+180.0,y, 100.0,28.0));
        draw_slider(&mut ctx, 0.4, x,y, 170.0,28.0);
        y += 55.0;

        draw_button(&mut ctx, ICON_TRASH, "Delete", rect(x, y, 160.0, 28.0), 0xFF_801008);
        draw_button(&mut ctx, None, "Cancel", rect(x+170.0, y, 110.0, 28.0), 0x00_000000);

        // Thumbnails box
        draw_thumbnails(&mut ctx, rect(365.0, popy-30.0, 160.0, 300.0), &data.images[..], time);
    }

    if false {
        // Canvas test
        use oni2d::canvas::*;
        let mut ctx = Canvas::new(vg);

        ctx.draw_rect(rect(50.0, 50.0, 100.0, 100.0), Paint::fill(0xFF_000000));
        ctx.draw_rrect(RRect::new([50.0, 50.0], [100.0, 100.0], 15.0), Paint::fill(0xFF_CC0000));

        ctx.draw_line([60.0, 60.0], [140.0, 140.0], Paint::stroke(0xFF_00CCCC));
    }

    /*
    {
        use oni2d::canvas::*;
        let mut ctx = Canvas::new(vg);
        sup::blendish::run(&mut ctx, time, rect(50.0, 50.0, 200.0, 200.0));
    }
    */

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

            vg.fill_rect(rect(x, y, row.width, lineh), if hit { 0x40_FFFFFF } else { 0x10_FFFFFF });

            vg.fill_color(0xFF_FFFFFF);
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

                vg.fill_rect(rect(caretx, y, 1.0, lineh), Color::rgba(255,192,0,255).to_bgra());

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
        vg.fill_color(Color::rgba(255,192,0,255).to_bgra());
        vg.rrect(rect(
                bounds[0].floor()-4.0,
                bounds[1].floor()-2.0,
                (bounds[2]-bounds[0]).floor()+8.0,
                (bounds[3]-bounds[1]).floor()+4.0,
            ), ((bounds[3]-bounds[1]).floor()+4.0)/2.0 - 1.0,
        );
        vg.fill();

        vg.fill_color(0xFF_202020);
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
    let a = f32::max(gx, gy) - 0.5;
    let a = a.clamp(0.0, 1.0);
    vg.global_alpha(a);

    vg.begin_path();
    vg.fill_color(0xFF_DCDCDC);
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

    vg.fill_color(0xDC_000000);
    vg.text_box(x,y, 150.0, "Hover your mouse over the text to see calculated caret position.");

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