use std::f32::consts::PI;

use oni2d::{
    Context, Point, Winding, Image,
    utils::{minf, clampf, deg2rad},
    canvas::*,
};

const ICON_SEARCH: char = '\u{1F50D}';
const ICON_CIRCLED_CROSS: char = '\u{2716}';
const ICON_CHEVRON_RIGHT: char = '\u{E75E}';
const ICON_CHECK: char = '\u{2713}';
const ICON_LOGIN: char = '\u{E740}';
const ICON_TRASH: char = '\u{E729}';

use crate::cp2utf8;

pub fn draw_window(ctx: &mut Canvas, title: &str, rr: Rect) {
    let (x, y, w, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    let corner_radius = 3.0;

    // Window
    let rrect = RRect::new(rr.origin.into(), rr.size.into(), corner_radius);
    ctx.draw_rrect(rrect, Paint::fill(0xC0_1C1E22));
    //vg.fill_color(Color::rgba(0,0,0,128));

    // Drop shadow
    let mut path: Path<[_; 128]> = Path::new();
    path.add_rect(rect(x-10.0,y-10.0, w+20.0,h+20.0));
    path.add_rrect(rrect);
    path._path_winding(Winding::CW);
    ctx.draw_path(&mut path, Paint::gradient(Gradient::Box {
        rect: Rect::new([x, y+2.0].into(), rr.size),
        radius: corner_radius*2.0,
        feather: 10.0,
        inner_color: Color::new(0x80_000000),
        outer_color: Color::new(0x00_000000),
    }));

    // Header
    let header_paint = Paint::gradient(Gradient::Linear {
        from: [x,y], to: [x,y+15.0],
        inner_color: Color::new(0x08_FFFFFF),
        outer_color: Color::new(0x10_000000),
    });

    let rrect = RRect::new([x+1.0,y+1.0], [w-2.0,30.0], corner_radius-1.0);
    ctx.draw_rrect(rrect, header_paint);

    ctx.draw_line([x+0.5, y+0.5+30.0], [x+0.5+w-1.0, y+0.5+30.0], Paint::stroke(0x20_000000));

    ctx.text([x+w/2.0,y+16.0+1.0], title, TextStyle {
        font_size: 18.0,
        font_face: b"sans-bold\0",
        font_blur: 2.0,
        color: Color::rgba(0,0,0,128),
        text_align: Align::CENTER|Align::MIDDLE,
    });

    ctx.text([x+w/2.0,y+16.0], title, TextStyle {
        font_size: 18.0,
        font_face: b"sans-bold\0",
        font_blur: 0.0,
        color: Color::rgba(220,220,220,160),
        text_align: Align::CENTER|Align::MIDDLE,
    });
}

pub fn draw_search_box(ctx: &mut Canvas, text: &str, rr: Rect) {
    let (x, y, w, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    let corner_radius = h/2.0-1.0;

    // Edit
    let rrect = RRect::new(rr.origin.into(), rr.size.into(), corner_radius);
    ctx.draw_rrect(rrect, Paint::gradient(Gradient::Box {
        rect: rect(x,y+1.5, rr.size.width, rr.size.height),
        radius: h/2.0, feather: 5.0,
        inner_color: Color::rgba(0,0,0,16),
        outer_color: Color::rgba(0,0,0,92),
    }));

    ctx.text([x+h*1.05,y+h*0.5], text, TextStyle {
        font_size: 20.0,
        font_face: b"sans\0",
        font_blur: 0.0,
        color: Color::rgba(255,255,255,32),
        text_align: Align::LEFT|Align::MIDDLE,
    });

    let mut style = TextStyle {
        font_size: h*1.3,
        font_face: b"icons\0",
        font_blur: 0.0,
        color: Color::rgba(255,255,255,64),
        text_align: Align::CENTER|Align::MIDDLE,
    };

    let mut icon = [0u8; 8];
    ctx.text([x+h*0.55, y+h*0.55], cp2utf8(ICON_SEARCH, &mut icon), style);
    style.color = Color::rgba(255,255,255,32);
    ctx.text([x+w-h*0.55, y+h*0.55], cp2utf8(ICON_CIRCLED_CROSS, &mut icon), style);
}

pub fn draw_label(ctx: &mut Canvas, text: &str, rr: Rect) {
    let mut origin = rr.origin;
    origin.y += rr.size.height * 0.5;
    ctx.text(origin.into(), text, TextStyle {
        font_size: 18.0,
        font_face: b"sans\0",
        font_blur: 0.0,
        text_align: Align::LEFT|Align::MIDDLE,
        color: Color::rgba(255, 255, 255, 128),
    });
}

pub fn draw_button<I: Into<Option<char>>>(ctx: &mut Canvas, preicon: I, text: &str, rr: Rect, col: u32) {
    let (x, y, w, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    let corner_radius = 4.0;

    let col = Color::new(col);
    let alpha = if col.is_transparent_black() { 16 } else { 32 };

    let rrect = RRect::new([x+1.0,y+1.0], [w-2.0,h-2.0], corner_radius-1.0);
    if !col.is_transparent_black() {
        ctx.draw_rrect(rrect, Paint::fill(col.to_bgra()));
    }
    ctx.draw_rrect(rrect, Paint::gradient(Gradient::Linear {
        from: [x,y], to: [x,y+h],
        inner_color: Color::rgba(255,255,255, alpha),
        outer_color: Color::rgba(0,0,0, alpha),
    }));

    let rrect = RRect::new([x+0.5,y+0.5], [w-1.0,h-1.0], corner_radius-0.5);
    ctx.draw_rrect(rrect, Paint::stroke(0x30_000000));

    let (tw, _) = ctx.text_bounds(text, 20.0, b"sans-bold\0");
    let mut iw = 0.0;
    if let Some(preicon) = preicon.into() {
        let mut icon = [0u8; 8];
        let icon = cp2utf8(preicon, &mut icon);
        iw = ctx.text_bounds(icon, h*1.3, b"icons\0").0;
        iw += h*0.15;

        ctx.text([x+w*0.5-tw*0.5-iw*0.75, y+h*0.5], icon, TextStyle {
            font_size: h*1.3,
            font_face: b"icons\0",
            font_blur: 0.0,
            color: Color::rgba(255,255,255,96),
            text_align: Align::LEFT|Align::MIDDLE,
        });
    }

    let mut style =  TextStyle {
        font_size: 20.0,
        font_face: b"sans-bold\0",
        font_blur: 0.0,
        color: Color::rgba(0,0,0,160),
        text_align: Align::LEFT|Align::MIDDLE,
    };
    ctx.text([x+w*0.5-tw*0.5+iw*0.25, y+h*0.5-1.0], text, style);
    style.color = Color::rgba(255,255,255,160);
    ctx.text([x+w*0.5-tw*0.5+iw*0.25,y+h*0.5], text, style);
}

pub fn draw_checkbox(ctx: &mut Canvas, text: &str, rr: Rect) {
    let (x, y, _, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    let rrect = RRect::new([x+1.0,y+(h*0.5).floor()-9.0], [18.0,18.0], 3.0);
    ctx.draw_rrect(rrect, Paint::gradient(Gradient::Box {
        rect: rect(x+1.0, y+(h*0.5).floor()-9.0+1.0, 18.0, 18.0),
        radius: 3.0, feather: 3.0,
        inner_color: Color::rgba(0,0,0,32),
        outer_color: Color::rgba(0,0,0,92),
    }));

    let mut icon = [0u8; 8];
    ctx.text([x+9.0+2.0, y+h*0.5], cp2utf8(ICON_CHECK, &mut icon), TextStyle {
        font_size: 40.0,
        font_face: b"icons\0",
        font_blur: 0.0,
        color: Color::rgba(255,255,255,128),
        text_align: Align::CENTER|Align::MIDDLE,
    });
    ctx.text([x+28.0,y+h*0.5], text, TextStyle {
        font_size: 18.0,
        font_face: b"sans\0",
        font_blur: 0.0,
        color: Color::rgba(255,255,255,160),
        text_align: Align::LEFT|Align::MIDDLE,
    });
}

pub fn draw_drop_down(ctx: &mut Canvas, text: &str, bounds: Rect) {
    let (x, y, w, h) = (
        bounds.origin.x, bounds.origin.y,
        bounds.size.width, bounds.size.height,
    );

    let corner_radius = 4.0;

    let rrect = RRect::new([x+1.0,y+1.0], [w-2.0,h-2.0], corner_radius-1.0);
    ctx.draw_rrect(rrect, Paint::gradient(Gradient::Linear {
        from: bounds.origin.into(), to: [x,y+h],
        inner_color: Color::rgba(255,255,255,16),
        outer_color: Color::rgba(0,0,0,16),
    }));

    let rrect = RRect::new([x+0.5,y+0.5], [w-1.0,h-1.0], corner_radius-0.5);
    ctx.draw_rrect(rrect, Paint::stroke(0x30_000000));

    let mut icon = [0u8; 8];
    ctx.text([x+w-h*0.5, y+h*0.5], cp2utf8(ICON_SEARCH, &mut icon), TextStyle {
        font_size: h*1.3,
        font_face: b"icons\0",
        font_blur: 0.0,
        color: Color::rgba(255,255,255,64),
        text_align: Align::CENTER|Align::MIDDLE,
    });
    ctx.text([x+h*0.3,y+h*0.5], text, TextStyle {
        font_size: 20.0,
        font_face: b"sans\0",
        font_blur: 0.0,
        color: Color::rgba(255,255,255,160),
        text_align: Align::LEFT|Align::MIDDLE,
    });
}

pub fn draw_eyes(ctx: &mut Canvas, rr: Rect, mouse: Point, time: f32) {
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

    let bg = Paint::gradient(Gradient::Linear {
        from: [x,y+h*0.5],
        to: [x+w*0.1,y+h],
        inner_color: Color::rgba(0,0,0,32),
        outer_color: Color::rgba(0,0,0,16),
    });
    ctx.draw_oval(rect(lx+3.0, ly+16.0, ex, ey), bg);
    ctx.draw_oval(rect(rx+3.0, ry+16.0, ex, ey), bg);

    let bg = Paint::gradient(Gradient::Linear {
        from: [x,y+h*0.25],
        to: [x+w*0.1,y+h],
        inner_color: Color::rgba(220,220,220,255), outer_color: Color::rgba(128,128,128,255),
    });
    ctx.draw_oval(rect(lx, ly, ex, ey), bg);
    ctx.draw_oval(rect(rx, ry, ex, ey), bg);

    let mut dx = (mx - rx) / (ex * 10.0);
    let mut dy = (my - ry) / (ey * 10.0);
    let dd = (dx*dx+dy*dy).sqrt();
    if dd > 1.0 {
        dx /= dd; dy /= dd;
    }
    dx *= ex*0.4;
    dy *= ey*0.5;
    ctx.draw_oval(rect(lx+dx,ly+dy+ey*0.25*(1.0-blink), br, br*blink), Paint::fill(0xFF_202020));

    let mut dx = (mx - rx) / (ex * 10.0);
    let mut dy = (my - ry) / (ey * 10.0);
    let dd = (dx*dx+dy*dy).sqrt();
    if dd > 1.0 {
        dx /= dd; dy /= dd;
    }
    dx *= ex*0.4;
    dy *= ey*0.5;
    ctx.draw_oval(rect(rx+dx,ry+dy+ey*0.25*(1.0-blink), br, br*blink), Paint::fill(0xFF_202020));

    ctx.draw_oval(rect(lx, ly, ex, ey), Paint::gradient(Gradient::Radial {
        center: [lx-ex*0.25,ly-ey*0.5],
        inr: ex*0.1, outr: ex*0.75,
        inner_color: Color::rgba(255,255,255,128),
        outer_color: Color::rgba(255,255,255,0),
    }));

    ctx.draw_oval(rect(rx, ry, ex, ey), Paint::gradient(Gradient::Radial {
        center: [rx-ex*0.25,ry-ey*0.5],
        inr: ex*0.1, outr: ex*0.75,
        inner_color: Color::rgba(255,255,255,128),
        outer_color: Color::rgba(255,255,255,0),
    }));
}

pub fn draw_graph(ctx: &mut Canvas, x: f32, y: f32, w: f32, h: f32, time: f32) {
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
    let mut path: Path<[_; 128]> = Path::new();
    path.move_to(sx[0], sy[0]);
    for i in 1..6 {
        path.bezier_to(sx[i-1]+dx*0.5,sy[i-1], sx[i]-dx*0.5,sy[i], sx[i],sy[i]);
    }
    path.line_to(x+w, y+h);
    path.line_to(x, y+h);
    ctx.draw_path(&mut path, Paint::gradient(Gradient::Linear {
        from: [x,y], to: [x,y+h],
        inner_color: Color::rgba(0,160,192,0),
        outer_color: Color::rgba(0,160,192,64),
    }));

    // Graph line
    path.clear();
    path.move_to(sx[0], sy[0]+2.0);
    for i in 1..6 {
        path.bezier_to(sx[i-1]+dx*0.5,sy[i-1]+2.0, sx[i]-dx*0.5,sy[i]+2.0, sx[i],sy[i]+2.0);
    }
    ctx.draw_path(&mut path, Paint::stroke(0x20_000000).with_stroke_width(3.0));

    path.clear();
    path.move_to(sx[0], sy[0]);
    for i in 1..6 {
        path.bezier_to(sx[i-1]+dx*0.5,sy[i-1], sx[i]-dx*0.5,sy[i], sx[i],sy[i]);
    }
    ctx.draw_path(&mut path, Paint::stroke(0xFF_00A0C0).with_stroke_width(3.0));

    // Graph sample pos
    for i in 0..6 {
        let [x, y] = [sx[i]-10.0, sy[i]-10.0+2.0];
        ctx.draw_rect(rect(x, y, 20.0, 20.0), Paint::gradient(Gradient::Radial {
            center: [sx[i],sy[i]+2.0],
            inr: 3.0,
            outr: 8.0,
            inner_color: Color::rgba(0,0,0,32),
            outer_color: Color::rgba(0,0,0,0),
        }));
    }

    for i in 0..6 {
        ctx.draw_circle([sx[i], sy[i]], 4.0, Paint::fill(0xFF_00A0C0));
        ctx.draw_circle([sx[i], sy[i]], 2.0, Paint::fill(0xFF_DCDCDC));
    }
}

pub fn draw_spinner(ctx: &mut Canvas, cx: f32, cy: f32, r: f32, time: f32) {
    let a0 = 0.0 + time*6.0;
    let a1 = PI + time*6.0;
    let r0 = r;
    let r1 = r * 0.75;

    let mut path: Path<[_; 128]> = Path::new();
    path.arc(cx,cy, r0, a0, a1, Winding::CW);
    path.arc(cx,cy, r1, a1, a0, Winding::CCW);
    path.close();

    let rr = (r0+r1)*0.5;
    let ax = cx + a0.cos() * rr;
    let ay = cy + a0.sin() * rr;
    let bx = cx + a1.cos() * rr;
    let by = cy + a1.sin() * rr;

    ctx.draw_path(&mut path, Paint::gradient(Gradient::Linear {
        from: [ax, ay],
        to: [bx, by],
        inner_color: Color::rgba(0,0,0,0),
        outer_color: Color::rgba(0,0,0,128),
    }));
}

pub fn draw_widths(ctx: &mut Canvas, x: f32, y: f32, width: f32) {
    let paint = Paint::stroke(0xFF_000000);

    let mut y = y;
    for i in 0..20 {
        let paint = paint.with_stroke_width(((i as f32)+0.5)*0.1);
        ctx.draw_line([x, y], [x+width,y+width*0.3], paint);
        y += 10.0;
    }
}

pub fn draw_caps(ctx: &mut Canvas, x: f32, y: f32, width: f32) {
    let caps = [ StrokeCap::Butt, StrokeCap::Round, StrokeCap::Square ];
    let line_width = 8.0;

    ctx.draw_rect(rect(x-line_width/2.0, y, width+line_width, 40.0), Paint::fill(0x20_FFFFFF));
    ctx.draw_rect(rect(x, y, width, 40.0), Paint::fill(0x20_FFFFFF));

    let paint = Paint::stroke(0xFF_000000)
        .with_stroke_width(line_width);

    for (i, &cap) in caps.iter().enumerate() {
        let y = y + ((i*10) as f32) + 5.0;
        ctx.draw_line([x, y], [x+width, y], paint.with_stroke_cap(cap))
    }
}

pub fn draw_lines(ctx: &mut Canvas, x: f32, y: f32, w: f32, _h: f32, t: f32) {
    let pad = 5.0;
    let size = w/9.0 - pad*2.0;

    let joins = [ StrokeJoin::Miter, StrokeJoin::Round, StrokeJoin::Bevel ];
    let caps = [ StrokeCap::Butt, StrokeCap::Round, StrokeCap::Square ];

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

    let mut path: Path<[_; 128]> = Path::new();
    for (i, &cap) in caps.iter().enumerate() {
        for (j, &join) in joins.iter().enumerate() {
            let fx = x + size*0.5 + ((i*3+j) as f32)/9.0*w + pad;
            let fy = y - size*0.5 + pad;

            path.clear();
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
                .with_stroke_cap(StrokeCap::Butt)
                .with_stroke_join(StrokeJoin::Bevel));
        }
    }

}

fn draw_edit_box_base(ctx: &mut Canvas, rr: Rect) {
    let (x, y, w, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    let bg = Paint::gradient(Gradient::Box {
        rect: rect(x+1.0,y+1.0+1.5, w-2.0,h-2.0),
        radius: 3.0,
        feather: 4.0,
        inner_color: Color::rgba(255,255,255,32),
        outer_color: Color::rgba(32,32,32,32),
    });

    ctx.draw_rrect(RRect::new([x+1.0,y+1.0], [w-2.0,h-2.0], 4.0-1.0), bg);
    ctx.draw_rrect(RRect::new([x+0.5,y+0.5], [w-1.0,h-1.0], 4.0-0.5), Paint::stroke(0x30_000000));
}

pub fn draw_edit_box(ctx: &mut Canvas, text: &str, rr: Rect) {
    let (x, y, _, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    draw_edit_box_base(ctx, rr);
    ctx.text([x+h*0.3,y+h*0.5], text, TextStyle {
        font_size: 20.0,
        font_face: b"sans\0",
        font_blur: 0.0,
        color: Color::rgba(255,255,255,64),
        text_align: Align::LEFT|Align::MIDDLE,
    });
}

pub fn draw_edit_box_num(ctx: &mut Canvas, text: &str, units: &str, rr: Rect) {
    draw_edit_box_base(ctx, rr);

    let (x, y, w, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);
    let (uw, _) = ctx.text_bounds(units, 18.0, b"sans\0");

    ctx.text([x+w-h*0.3,y+h*0.5], units, TextStyle {
        font_size: 18.0,
        font_face: b"sans\0",
        font_blur: 0.0,
        color: Color::rgba(255,255,255,64),
        text_align: Align::RIGHT|Align::MIDDLE,
    });
    ctx.text([x+w-uw-h*0.5,y+h*0.5], text, TextStyle {
        font_size: 20.0,
        font_face: b"sans\0",
        font_blur: 0.0,
        color: Color::rgba(255,255,255,128),
        text_align: Align::RIGHT|Align::MIDDLE,
    });
}



pub fn draw_slider(ctx: &mut Canvas, pos: f32, x: f32, y: f32, w: f32, h: f32) {
    let cy = y+(h*0.5).floor();
    let kr = (h*0.25).floor();

    // vg.clear_state();

    // Slot
    ctx.draw_rrect(RRect::new([x,cy-2.0], [w,4.0], 2.0), Paint::gradient(Gradient::Box {
        rect: rect(x,cy-2.0+1.0, w,4.0),
        radius: 2.0,feather: 2.0,
        inner_color: Color::rgba(0,0,0,32),
        outer_color: Color::rgba(0,0,0,128),
    }));

    // Knob Shadow
    let mut path: Path<[_; 128]> = Path::new();
    path.add_rect(rect(x+(pos*w).floor()-kr-5.0,cy-kr-5.0, kr*2.0+5.0+5.0,kr*2.0+5.0+5.0+3.0));
    path.add_circle([x+(pos*w).floor(),cy], kr);
    path._path_winding(Winding::CW);
    ctx.draw_path(&mut path, Paint::gradient(Gradient::Radial {
        center: [x+(pos*w).floor(),cy+1.0],
        inr: kr-3.0, outr: kr+3.0,
        inner_color: Color::rgba(0,0,0,64),
        outer_color: Color::rgba(0,0,0,0),
    }));

    // Knob
    let knob = Paint::gradient(Gradient::Linear {
        from: [x,cy-kr], to: [x,cy+kr],
        inner_color: Color::rgba(255,255,255,16),
        outer_color: Color::rgba(0,0,0,16),
    });

    let center = [x+(pos*w).floor(),cy];
    ctx.draw_circle(center, kr-1.0, Paint::fill(0xFF_282B30));
    ctx.draw_circle(center, kr-1.0, knob);
    ctx.draw_circle(center, kr-0.5, Paint::stroke(0x5C_000000));
}

pub fn draw_thumbnails(ctx: &mut Canvas, rr: Rect, images: &[Image], time: f32) {
    let (x, y, width, height) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);

    let corner_radius = 3.0;
    let thumb = 60.0;
    let arry = 30.5;
    let stackh = ((images.len() / 2) as f32) * (thumb+10.0) + 10.0;
    let u1 = (1.0+(time*0.5).cos()) * 0.5;
    let u2 = (1.0-(time*0.2).cos()) * 0.5;

    // vg.clear_state();

    // Drop shadow
    let mut path: Path<[_; 128]> = Path::new();
    path.add_rect(rect(x-10.0,y-10.0, width+20.0,height+20.0));
    path.add_rrect(RRect::new(rr.origin.into(), rr.size.into(), corner_radius));
    path._path_winding(Winding::CW);
    path.close();
    ctx.draw_path(&mut path, Paint::gradient(Gradient::Box {
        rect: rect(x,y+4.0, width,height),
        radius: corner_radius*2.0, feather: 20.0,
        inner_color: Color::rgba(0,0,0,128),
        outer_color: Color::rgba(0,0,0,0),
    }));

    // Window
    path.clear();
    path.add_rrect(RRect::new(rr.origin.into(), rr.size.into(), corner_radius));
    path.move_to(x-10.0,y+arry);
    path.line_to(x+1.0,y+arry-11.0);
    path.line_to(x+1.0,y+arry+11.0);
    ctx.draw_path(&mut path, Paint::fill(0xFF_C8C8C8));

    ctx.save();
    ctx.scissor(Rect::new(rr.origin.into(), rr.size.into()));
    ctx.translate(0.0, -(stackh - height)*u1);

    let dv = 1.0 / (images.len()-1) as f32;

    for (i, &image) in images.iter().enumerate() {
        let mut tx = x+10.0;
        let mut ty = y+10.0;
        tx += (thumb+10.0) * (i%2) as f32;
        ty += (thumb+10.0) * (i/2) as f32;

        let (imgw, imgh) = ctx.image_size(image).expect("image_size");
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
            draw_spinner(ctx, tx+thumb/2.0,ty+thumb/2.0, thumb*0.25, time);
        }

        ctx.draw_rrect(RRect::new([tx,ty], [thumb,thumb], 5.0), Paint::gradient(Gradient::ImagePattern {
            center: [tx+ix, ty+iy],
            size: [iw,ih],
            angle: 0.0/180.0*PI, image,
            alpha: a,
        }));

        path.clear();
        path.add_rect(rect(tx-5.0,ty-5.0, thumb+10.0,thumb+10.0));
        path.add_rrect(RRect::new([tx,ty], [thumb,thumb], 6.0));
        path._path_winding(Winding::CW);
        ctx.draw_path(&mut path, Paint::gradient(Gradient::Box {
            rect: rect(tx-1.0,ty, thumb+2.0,thumb+2.0),
            radius: 5.0, feather: 3.0,
            inner_color: Color::rgba(0,0,0,128),
            outer_color: Color::rgba(0,0,0,0),
        }));

        let rrect = RRect::new([tx+0.5,ty+0.5], [thumb-1.0,thumb-1.0], 4.0-0.5);
        ctx.draw_rrect(rrect, Paint::stroke(0xC0_FFFFFF).with_stroke_width(1.0));
    }
    ctx.restore();

    // Hide fades
    ctx.draw_rect(rect(x+4.0,y, width-8.0,6.0), Paint::gradient(Gradient::Linear {
        from: [x,y], to: [x,y+6.0],
        inner_color: Color::rgba(200,200,200,255),
        outer_color: Color::rgba(200,200,200,0),
    }));
    ctx.draw_rect(rect(x+4.0,y+height-6.0, width-8.0,6.0), Paint::gradient(Gradient::Linear {
        from: [x,y+height],
        to: [x,y+height-6.0],
        inner_color: Color::rgba(200,200,200,255),
        outer_color: Color::rgba(200,200,200,0),
    }));

    // Scroll bar
    let rrect = RRect::new([x+width-12.0,y+4.0], [8.0,height-8.0], 3.0);
    ctx.draw_rrect(rrect, Paint::gradient(Gradient::Box {
        rect: rect(x+width-12.0+1.0,y+4.0+1.0, 8.0,height-8.0),
        radius: 3.0, feather: 4.0,
        inner_color: Color::rgba(0,0,0,32),
        outer_color: Color::rgba(0,0,0,92),
    }));

    let scrollh = (height/stackh) * (height-8.0);
    let rrect = RRect::new([x+width-12.0+1.0,y+4.0+1.0 + (height-8.0-scrollh)*u1], [8.0-2.0,scrollh-2.0], 2.0);
    ctx.draw_rrect(rrect, Paint::gradient(Gradient::Box {
        rect: rect(x+width-12.0-1.0,y+4.0+(height-8.0-scrollh)*u1-1.0, 8.0,scrollh),
        radius: 3.0, feather: 4.0,
        inner_color: Color::rgba(220,220,220,255),
        outer_color: Color::rgba(128,128,128,255),
    }));
}

pub fn draw_scissor(ctx: &mut Canvas, x: f32, y: f32, t: f32) {
    ctx.save();

    // Draw first rect and set scissor to it's area.
    ctx.translate(x, y);
    ctx.rotate(deg2rad(5.0));

    let area = rect(-20.0,-20.0, 60.0,40.0);
    ctx.draw_rect(area, Paint::fill(0xFF_FF0000));
    ctx.scissor(area);

    // Draw second rectangle with offset and rotation.
    ctx.translate(40.0,0.0);
    ctx.rotate(t);

    // Draw the intended second rectangle without any scissoring.
    ctx.save();
    ctx.reset_scissor();
    ctx.draw_rect(rect(-20.0,-10.0, 60.0,30.0), Paint::fill(0x40_FF8000));
    ctx.restore();

    // Draw second rectangle with combined scissoring.
    let r = rect(-20.0,-10.0, 60.0,30.0);
    ctx.intersect_scissor(r);
    ctx.draw_rect(r, Paint::fill(0xFF_FF8000));

    ctx.restore();
}