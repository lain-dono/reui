use reui::{Canvas, Color, Corners, LineCap, LineJoin, Offset, Paint, Path, Rect, Solidity};
use std::f32::consts::PI;

pub fn render_demo(ctx: &mut Canvas, mouse: Offset, wsize: Offset, time: f32, blowup: bool) {
    let (width, height) = wsize.into();

    {
        let rect = Rect::from_ltwh(width - 250.0, 50.0, 150.0, 100.0);
        draw_eyes(ctx, rect, mouse, time);
        draw_graph(ctx, 0.0, height / 2.0, width, height / 2.0, time);
        let rect = Rect::from_ltwh(width - 300.0, height - 300.0, 250.0, 250.0);
        draw_colorwheel(ctx, rect, time);

        // Line joints
        draw_lines(ctx, 120.0, height - 100.0, 600.0, 50.0, time);
        // Line caps
        draw_widths(ctx, 10.0, 50.0, 30.0);
        // Line caps
        draw_caps(ctx, 10.0, 400.0, 30.0);

        if blowup {
            ctx.rotate((time * 0.3).sin() * 5.0 / 180.0 * PI);
            ctx.scale(2.0);
        }

        // Widgets
        let win = Rect::from_ltwh(50.0, 50.0, 300.0, 400.0);
        draw_window(ctx, win, |ctx, window| {
            let window = window.deflate(10.0);
            let width = window.dx();

            let item_h = 28.0;
            let line_height = 40.0;

            let mut offset = window.min;
            draw_search_box(ctx, Rect::from_size(width, item_h).translate(offset));
            offset.y += line_height;
            draw_drop_down(ctx, Rect::from_size(width, item_h).translate(offset));
            offset.y += line_height;

            // Form
            draw_edit_box(ctx, Rect::from_size(width, item_h).translate(offset));
            offset.y += line_height;
            offset.y += line_height;
            {
                let mut offset = offset;
                let item = Rect::from_size(width / 2.0, item_h);
                draw_checkbox(ctx, item.translate(offset).deflate(3.0));
                offset.x += width / 2.0;
                draw_edit_box(ctx, item.translate(offset).deflate(3.0));
            }
            offset.y += line_height;

            // Slider
            {
                let mut offset = offset;
                let item = Rect::from_size(width / 2.0, item_h);
                draw_slider(ctx, 0.4, item.translate(offset).deflate(3.0));
                offset.x += width / 2.0;
                draw_edit_box_num(ctx, item.translate(offset).deflate(3.0));
            }

            offset.y = window.max.y - item_h;
            {
                let mut offset = offset;
                let btn = Rect::from_size(width / 3.0, item_h);
                draw_button(ctx, btn.translate(offset).deflate(3.0), 0xFF_801008);
                offset.x += width / 3.0;
                draw_button(ctx, btn.translate(offset).deflate(3.0), 0x00_000000);
                offset.x += width / 3.0;
                draw_button(ctx, btn.translate(offset).deflate(3.0), 0xFF_006080);
            }
        });
    }

    if true {
        ctx.save();
        super::blendish::run(ctx, time, Rect::from_ltwh(380.0, 50.0, 200.0, 200.0));
        ctx.restore();
    }

    if false {
        // Canvas test

        ctx.draw_rect(
            Rect::from_ltwh(50.0, 50.0, 100.0, 100.0),
            Paint::fill(Color::BLACK),
        );
        let radius = Corners::all_same(15.0);
        ctx.draw_rrect(
            Rect::new(Offset::new(50.0, 50.0), Offset::new(100.0, 100.0)),
            radius,
            Paint::fill(Color::hex(0xFF_CC0000)),
        );

        ctx.draw_line(
            [60.0, 60.0].into(),
            [140.0, 140.0].into(),
            Paint::stroke(Color::hex(0xFF_00CCCC)),
        );
    }

    draw_palette(ctx, Offset::new(10.0, 10.0));

    {
        // blending test

        fn srgba_fill(c: u32) -> Paint {
            Paint::fill(Color::hex(c))
        }

        let x = 260.0;
        let y = 5.0;
        let width = 85.0;
        let height = 40.0;

        let bg_paint = Paint::fill(Color::hex(0xFF_FFFFFF));
        ctx.draw_rect(Rect::from_ltwh(x, y, width, height), bg_paint);

        let vg = srgba_fill(0xFF_18EA22);
        let vy = srgba_fill(0xFF_EAE818);
        let vc = srgba_fill(0xFF_18EAC8);
        let vm = srgba_fill(0xFF_EE46A6);

        let w = 10.0;
        let rect = Rect::from_size(w, height);
        ctx.draw_rect(rect.shift(x + w * 0.5, y), vc);
        ctx.draw_rect(rect.shift(x + w * 2.5, y), vm);
        ctx.draw_rect(rect.shift(x + w * 4.5, y), vy);
        ctx.draw_rect(rect.shift(x + w * 6.5, y), vg);

        let hr: [_; 2] = [srgba_fill(0xFF_FF0000), srgba_fill(0x7F_FF0000)];
        let hg: [_; 2] = [srgba_fill(0xFF_93FF00), srgba_fill(0x7F_93FF00)];
        let hb: [_; 2] = [srgba_fill(0xFF_007FFF), srgba_fill(0x7F_007FFF)];

        let h = 4.0;
        let rect = Rect::from_size(width, h);
        ctx.draw_rect(rect.shift(x, y + h * 1.0), hr[0]);
        ctx.draw_rect(rect.shift(x, y + h * 2.0), hr[1]);

        ctx.draw_rect(rect.shift(x, y + h * 4.0), hg[0]);
        ctx.draw_rect(rect.shift(x, y + h * 5.0), hg[1]);

        ctx.draw_rect(rect.shift(x, y + h * 7.0), hb[0]);
        ctx.draw_rect(rect.shift(x, y + h * 8.0), hb[1]);
    }
}

fn draw_palette(ctx: &mut Canvas, offset: Offset) {
    let colors: &[u32] = &[
        0xFF_001F3F,
        0xFF_0074D9,
        0xFF_7FDBFF,
        0xFF_39CCCC,
        0xFF_3D9970,
        0xFF_2ECC40,
        0xFF_01FF70,
        0xFF_FFDC00,
        0xFF_FF851B,
        0xFF_FF4136,
        0xFF_85144B,
        0xFF_F012BE,
        0xFF_B10DC9,
        0xFF_111111,
        0xFF_AAAAAA,
        0xFF_DDDDDD,
        0xFF_FFFFFF,
    ];

    for (i, &color) in colors.iter().enumerate() {
        let color = Color::hex(color);
        let size = 10.0;

        let (x, y) = (i % 8, i / 8);

        let x = offset.x + 1.25 * size * x as f32;
        let y = offset.y + 1.25 * size * y as f32;
        ctx.draw_rect(Rect::from_ltwh(x, y, size, size), Paint::fill(color))
    }

    let grad: &[[u32; 2]] = &[
        [0xFF_FFFFFF, 0xFF_000000], // 111 000
        [0xFF_FF0000, 0xFF_00FF00], // 100 010
        [0xFF_00FF00, 0xFF_0000FF], // 010 001
        [0xFF_0000FF, 0xFF_FF0000], // 001 100
        [0xFF_FF00FF, 0xFF_FFFF00], // 101 110
        [0xFF_FFFF00, 0xFF_00FFFF], // 110 011
    ];

    for (i, &[inner_color, outer_color]) in grad.iter().enumerate() {
        let w = 128.0;
        let h = 5.0;
        let x = offset.x + 110.0;
        let y = offset.y + h * i as f32;

        let inner_color = Color::hex(inner_color);
        let outer_color = Color::hex(outer_color);

        let paint = Paint::linear_gradient([x, y], [x + w, y], inner_color, outer_color);
        ctx.draw_rect(Rect::from_ltwh(x, y, w, h), paint);
    }
}

pub fn draw_window(ctx: &mut Canvas, bounds: Rect, content: impl FnOnce(&mut Canvas, Rect)) {
    let header_bg = Color::hex(0xFC_3B3F41);
    let body_bg = Color::hex(0xFC_47484B);

    let btn_r = Color::hex(0xFF_FE5850);
    let btn_y = Color::hex(0xFF_FDBE34);
    let btn_g = Color::hex(0xFF_26C940);

    //let header_bg = Color::hex(0xFF_0D0D0D);
    //let body_bg = Color::hex(0xEE_0D0D0D);

    let [left, top, width, height] = bounds.to_xywh();

    let corner_radius = 4.0;
    let header_height = 20.0;

    {
        // body
        let rect = Rect::from_ltwh(left, top + header_height, width, height - header_height);
        let radius = Corners::bottom(corner_radius);
        ctx.draw_rrect(rect, radius, Paint::fill(body_bg));

        content(ctx, rect);
    }

    {
        // header
        let header_paint = Paint::fill(header_bg);

        let radius = Corners::top(corner_radius);
        let rect = Rect::from_ltwh(left, top, width, header_height);
        ctx.draw_rrect(rect, radius, header_paint);

        let radius = 5.0;
        let center = Offset::new(left, top) + Offset::new(header_height, header_height) / 2.0;
        let off = Offset::new(radius * 3.0, 0.0);
        ctx.draw_circle(center + off * 0.0, radius, Paint::fill(btn_r));
        ctx.draw_circle(center + off * 1.0, radius, Paint::fill(btn_y));
        ctx.draw_circle(center + off * 2.0, radius, Paint::fill(btn_g));
    }

    // Drop shadow
    {
        let mut path = Path::new();
        path.rect(bounds.inflate(20.0));
        path.rrect(bounds, Corners::all_same(corner_radius));
        path.solidity(Solidity::Hole);
        let shadow_paint = Paint::box_gradient(
            bounds,
            corner_radius,
            20.0,
            Color::hex(0xF8_FF00FF),
            Color::hex(0x00_000000),
        );
        ctx.draw_path(&path, shadow_paint);
    }
}

pub fn draw_search_box(ctx: &mut Canvas, rect: Rect) {
    let [left, top, width, height] = rect.to_xywh();

    let corner_radius = Corners::all_same(height / 2.0 - 1.0);

    let paint = Paint::box_gradient(
        Rect::from_ltwh(left, top + 1.5, width, height),
        height / 2.0,
        5.0,
        Color::hex(0x10_000000),
        Color::hex(0x60_000000),
    );
    ctx.draw_rrect(rect, corner_radius, paint);
}

pub fn draw_button(ctx: &mut Canvas, rr: Rect, col: u32) {
    let [x, y, w, h] = rr.to_xywh();

    let corner_radius = 4.0;

    let col = Color::hex(col);
    let alpha = if col == Color::TRANSPARENT { 16 } else { 32 };

    let radius = Corners::all_same(corner_radius - 1.0);
    let rrect = Rect::from_ltwh(x + 1.0, y + 1.0, w - 2.0, h - 2.0);
    if col != Color::TRANSPARENT {
        ctx.draw_rrect(rrect, radius, Paint::fill(col));
    }
    let rgb = 127;
    let inner_color = Color::new_srgba8(rgb, rgb, rgb, alpha);
    let outer_color = Color::new_srgba8(0, 0, 0, alpha);
    let paint = Paint::linear_gradient([x, y], [x, y + h], inner_color, outer_color);
    ctx.draw_rrect(rrect, radius, paint);

    let radius = Corners::all_same(corner_radius - 0.5);
    let rrect = Rect::from_ltwh(x + 0.5, y + 0.5, w - 1.0, h - 1.0);
    ctx.draw_rrect(rrect, radius, Paint::stroke(Color::hex(0x30_000000)));
}

pub fn draw_checkbox(ctx: &mut Canvas, rr: Rect) {
    let [x, y, _, h] = rr.to_xywh();

    let radius = Corners::all_same(3.0);
    let rect = Rect::from_ltwh(x + 1.0, y + (h * 0.5).floor() - 9.0, 18.0, 18.0);
    let paint = Paint::box_gradient(
        Rect::from_ltwh(x + 1.0, y + (h * 0.5).floor() - 9.0 + 1.0, 18.0, 18.0),
        3.0,
        3.0,
        Color::hex(0x20_000000),
        Color::hex(0x60_000000),
    );
    ctx.draw_rrect(rect, radius, paint);
}

pub fn draw_drop_down(ctx: &mut Canvas, bounds: Rect) {
    let [x, y, w, h] = bounds.to_xywh();

    let corner_radius = 4.0;

    let rect = Rect::from_ltwh(x + 1.0, y + 1.0, w - 2.0, h - 2.0);
    let radius = Corners::all_same(corner_radius - 1.0);
    let paint = Paint::linear_gradient(
        bounds.min.into(),
        [x, y + h],
        Color::hex(0x10_FFFFFF),
        Color::hex(0x10_000000),
    );
    ctx.draw_rrect(rect, radius, paint);

    let rect = Rect::from_ltwh(x + 0.5, y + 0.5, w - 1.0, h - 1.0);
    let radius = Corners::all_same(corner_radius - 0.5);
    ctx.draw_rrect(rect, radius, Paint::stroke(Color::hex(0x30_000000)));
}

pub fn draw_eyes(ctx: &mut Canvas, rr: Rect, mouse: Offset, time: f32) {
    let [x, y, w, h] = rr.to_xywh();

    let (mx, my) = mouse.into();

    let [ex, ey] = [w * 0.23, h * 0.5];
    let lx = x + ex;
    let ly = y + ey;
    let rx = x + w - ex;
    let ry = y + ey;
    let br = f32::min(ex, ey) * 0.5;
    let blink = 1.0 - (time * 0.5).sin().powf(200.0) * 0.8;

    let bg = Paint::linear_gradient(
        [x, y + h * 0.5],
        [x + w * 0.1, y + h],
        Color::hex(0x20_000000),
        Color::hex(0x10_000000),
    );
    ctx.draw_oval(Rect::from_oval(lx + 3.0, ly + 16.0, ex, ey), bg);
    ctx.draw_oval(Rect::from_oval(rx + 3.0, ry + 16.0, ex, ey), bg);

    let bg = Paint::linear_gradient(
        [x, y + h * 0.25],
        [x + w * 0.1, y + h],
        Color::hex(0xFF_DCDCDC),
        Color::hex(0xFF_808080),
    );
    ctx.draw_oval(Rect::from_oval(lx, ly, ex, ey), bg);
    ctx.draw_oval(Rect::from_oval(rx, ry, ex, ey), bg);

    let eye_paint = Paint::fill(Color::hex(0xFF_202020));

    let mut dx = (mx - rx) / (ex * 10.0);
    let mut dy = (my - ry) / (ey * 10.0);
    let dd = (dx * dx + dy * dy).sqrt();
    if dd > 1.0 {
        dx /= dd;
        dy /= dd;
    }
    dx *= ex * 0.4;
    dy *= ey * 0.5;
    ctx.draw_oval(
        Rect::from_oval(lx + dx, ly + dy + ey * 0.25 * (1.0 - blink), br, br * blink),
        eye_paint,
    );

    let mut dx = (mx - rx) / (ex * 10.0);
    let mut dy = (my - ry) / (ey * 10.0);
    let dd = (dx * dx + dy * dy).sqrt();
    if dd > 1.0 {
        dx /= dd;
        dy /= dd;
    }
    dx *= ex * 0.4;
    dy *= ey * 0.5;
    ctx.draw_oval(
        Rect::from_oval(rx + dx, ry + dy + ey * 0.25 * (1.0 - blink), br, br * blink),
        eye_paint,
    );

    let inr = ex * 0.1;
    let outr = ex * 0.75;
    let inner_color = Color::hex(0x80_FFFFFF);
    let outer_color = Color::hex(0x00_FFFFFF);

    let left = [lx - ex * 0.25, ly - ey * 0.5];
    let right = [rx - ex * 0.25, ry - ey * 0.5];

    let left_paint = Paint::radial_gradient(left, inr, outr, inner_color, outer_color);
    ctx.draw_oval(Rect::from_oval(lx, ly, ex, ey), left_paint);

    let right_paint = Paint::radial_gradient(right, inr, outr, inner_color, outer_color);
    ctx.draw_oval(Rect::from_oval(rx, ry, ex, ey), right_paint);
}

pub fn draw_graph(ctx: &mut Canvas, x: f32, y: f32, w: f32, h: f32, time: f32) {
    let samples = [
        (1.0 + (time * 1.2345 + (time * 0.33457).cos() * 0.44).sin()) * 0.5,
        (1.0 + (time * 0.68363 + (time * 1.3).cos() * 1.55).sin()) * 0.5,
        (1.0 + (time * 1.1642 + (time * 0.33457).cos() * 1.24).sin()) * 0.5,
        (1.0 + (time * 0.56345 + (time * 1.63).cos() * 0.14).sin()) * 0.5,
        (1.0 + (time * 1.6245 + (time * 0.254).cos() * 0.3).sin()) * 0.5,
        (1.0 + (time * 0.345 + (time * 0.03).cos() * 0.6).sin()) * 0.5,
    ];

    let dx = w / 5.0;

    let mut sx = [0f32; 6];
    let mut sy = [0f32; 6];
    for i in 0..6 {
        sx[i] = x + (i as f32) * dx;
        sy[i] = y + h * samples[i] * 0.8;
    }

    // Graph background
    let mut path = Path::new();
    path.move_to((sx[0], sy[0]).into());
    for i in 1..6 {
        path.cubic_to(
            (sx[i - 1] + dx * 0.5, sy[i - 1]).into(),
            (sx[i] - dx * 0.5, sy[i]).into(),
            (sx[i], sy[i]).into(),
        );
    }
    path.line_to((x + w, y + h).into());
    path.line_to((x, y + h).into());
    ctx.draw_path(
        &path,
        Paint::linear_gradient(
            [x, y],
            [x, y + h],
            Color::new_srgba8(0, 160, 192, 0),
            Color::new_srgba8(0, 160, 192, 64),
        ),
    );

    // Graph line
    path.clear();
    path.move_to((sx[0], sy[0] + 2.0).into());
    for i in 1..6 {
        path.cubic_to(
            (sx[i - 1] + dx * 0.5, sy[i - 1] + 2.0).into(),
            (sx[i] - dx * 0.5, sy[i] + 2.0).into(),
            (sx[i], sy[i] + 2.0).into(),
        );
    }
    ctx.draw_path(
        &path,
        Paint::stroke(Color::hex(0x20_000000)).stroke_width(3.0),
    );

    path.clear();
    path.move_to((sx[0], sy[0]).into());
    for i in 1..6 {
        path.cubic_to(
            (sx[i - 1] + dx * 0.5, sy[i - 1]).into(),
            (sx[i] - dx * 0.5, sy[i]).into(),
            (sx[i], sy[i]).into(),
        );
    }
    ctx.draw_path(
        &path,
        Paint::stroke(Color::hex(0xFF_00A0C0)).stroke_width(3.0),
    );

    // Graph sample pos
    for i in 0..6 {
        let [x, y] = [sx[i] - 10.0, sy[i] - 10.0 + 2.0];
        ctx.draw_rect(
            Rect::from_ltwh(x, y, 20.0, 20.0),
            Paint::radial_gradient(
                [sx[i], sy[i] + 2.0],
                3.0,
                8.0,
                Color::hex(0x20_000000),
                Color::hex(0x00_000000),
            ),
        );
    }

    for i in 0..6 {
        ctx.draw_circle(
            [sx[i], sy[i]].into(),
            4.0,
            Paint::fill(Color::hex(0xFF_00A0C0)),
        );
        ctx.draw_circle(
            [sx[i], sy[i]].into(),
            2.0,
            Paint::fill(Color::hex(0xFF_DCDCDC)),
        );
    }
}

pub fn draw_widths(ctx: &mut Canvas, x: f32, y: f32, width: f32) {
    let paint = Paint::stroke(Color::hex(0xFF_999999));

    let mut y = y;
    for i in 0..75 {
        let paint = paint.stroke_width((0.05 + i as f32) * 0.025);
        ctx.draw_line([x, y].into(), [x + width, y + width * 0.3].into(), paint);
        y += 4.0;
    }
}

pub fn draw_caps(ctx: &mut Canvas, x: f32, y: f32, width: f32) {
    let caps = [LineCap::Butt, LineCap::Round, LineCap::Square];
    let line_width = 8.0;

    ctx.draw_rect(
        Rect::from_ltwh(x - line_width / 2.0, y, width + line_width, 40.0),
        Paint::fill(Color::hex(0x20_FFFFFF)),
    );
    ctx.draw_rect(
        Rect::from_ltwh(x, y, width, 40.0),
        Paint::fill(Color::hex(0x20_FFFFFF)),
    );

    let paint = Paint::stroke(Color::BLACK).stroke_width(line_width);

    for (i, &cap) in caps.iter().enumerate() {
        let y = y + ((i * 10) as f32) + 5.0;
        ctx.draw_line([x, y].into(), [x + width, y].into(), paint.stroke_cap(cap))
    }
}

pub fn draw_lines(ctx: &mut Canvas, x: f32, y: f32, w: f32, _h: f32, t: f32) {
    let pad = 5.0;
    let size = w / 9.0 - pad * 2.0;

    let joins = [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel];
    let caps = [LineCap::Butt, LineCap::Round, LineCap::Square];

    let pts = [
        -size * 0.25 + (t * 0.3).cos() * size * 0.5,
        (t * 0.3).sin() * size * 0.5,
        -size * 0.25,
        0.0,
        size * 0.25,
        0.0,
        size * 0.25 + (-t * 0.3).cos() * size * 0.5,
        (-t * 0.3).sin() * size * 0.5,
    ];

    let mut path = Path::new();
    for (i, &cap) in caps.iter().enumerate() {
        for (j, &join) in joins.iter().enumerate() {
            let fx = x + size * 0.5 + (i as f32) / 6.0 * w + pad;
            let fy = y - size * 0.5 + (j as f32) * 40.0 + pad;

            path.clear();
            path.move_to((fx + pts[0], fy + pts[1]).into());
            path.line_to((fx + pts[2], fy + pts[3]).into());
            path.line_to((fx + pts[4], fy + pts[5]).into());
            path.line_to((fx + pts[6], fy + pts[7]).into());

            ctx.draw_path(
                &path,
                Paint::stroke(Color::hex(0xA0_000000))
                    .stroke_width(size * 0.3)
                    .stroke_cap(cap)
                    .stroke_join(join),
            );

            ctx.draw_path(
                &path,
                Paint::stroke(Color::hex(0xFF_00C0FF))
                    .stroke_width(1.0)
                    .stroke_cap(LineCap::Butt)
                    .stroke_join(LineJoin::Bevel),
            );
        }
    }
}

fn draw_edit_box_base(ctx: &mut Canvas, rr: Rect) {
    let [left, top, width, height] = rr.to_xywh();

    let bg = Paint::box_gradient(
        Rect::from_ltwh(left + 1.0, top + 1.0 + 1.5, width - 2.0, height - 2.0),
        3.0,
        4.0,
        Color::hex(0x20_FFFFFF),
        Color::hex(0x20_202020),
    );

    let rect = Rect::from_ltwh(left + 1.0, top + 1.0, width - 2.0, height - 2.0);
    let radius = Corners::all_same(4.0 - 1.0);
    ctx.draw_rrect(rect, radius, bg);

    let rect = Rect::from_ltwh(left + 0.5, top + 0.5, width - 1.0, height - 1.0);
    let radius = Corners::all_same(4.0 - 0.5);
    ctx.draw_rrect(rect, radius, Paint::stroke(Color::hex(0x30_000000)));
}

pub fn draw_edit_box(ctx: &mut Canvas, rr: Rect) {
    draw_edit_box_base(ctx, rr);
}

pub fn draw_edit_box_num(ctx: &mut Canvas, rr: Rect) {
    draw_edit_box_base(ctx, rr);
}

pub fn draw_slider(ctx: &mut Canvas, pos: f32, rect: Rect) {
    let [x, y, w, h] = rect.to_xywh();
    let cy = y + (h * 0.5).floor();
    let kr = (h * 0.25).floor();

    // vg.clear_state();

    // Slot
    ctx.draw_rrect(
        Rect::from_ltwh(x, cy - 2.0, w, 4.0),
        Corners::all_same(2.0),
        Paint::box_gradient(
            Rect::from_ltwh(x, cy - 2.0 + 1.0, w, 4.0),
            2.0,
            2.0,
            Color::hex(0x20_000000),
            Color::hex(0x80_000000),
        ),
    );

    // Knob Shadow
    let mut path = Path::new();
    path.rect(Rect::from_ltwh(
        x + (pos * w).floor() - kr - 5.0,
        cy - kr - 5.0,
        kr * 2.0 + 5.0 + 5.0,
        kr * 2.0 + 5.0 + 5.0 + 3.0,
    ));
    path.circle([x + (pos * w).floor(), cy].into(), kr);
    path.solidity(Solidity::Hole);
    ctx.draw_path(
        &path,
        Paint::radial_gradient(
            [x + (pos * w).floor(), cy + 1.0],
            kr - 3.0,
            kr + 3.0,
            Color::hex(0x40_000000),
            Color::hex(0x00_000000),
        ),
    );

    // Knob
    let knob = Paint::linear_gradient(
        [x, cy - kr],
        [x, cy + kr],
        Color::hex(0x10_FFFFFF),
        Color::hex(0x10_000000),
    );

    let center = [x + (pos * w).floor(), cy].into();
    ctx.draw_circle(center, kr - 1.0, Paint::fill(Color::hex(0xFF_282B30)));
    ctx.draw_circle(center, kr - 1.0, knob);
    ctx.draw_circle(center, kr - 0.5, Paint::stroke(Color::hex(0x5C_000000)));
}

pub fn draw_colorwheel(ctx: &mut Canvas, rr: Rect, time: f32) {
    let [x, y, w, h] = rr.to_xywh();
    let hue = (time * 0.12).sin();

    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    let r1 = if w < h { w } else { h } * 0.5 - 5.0;
    let r0 = r1 - 20.0;
    let aeps = 0.5 / r1; // half a pixel arc length in radians (2pi cancels out).

    let mut path = Path::new();
    for i in 0..6 {
        let a0 = (i as f32) / 6.0 * PI * 2.0 - aeps;
        let a1 = ((i as f32) + 1.0) / 6.0 * PI * 2.0 + aeps;
        let ax = cx + a0.cos() * (r0 + r1) * 0.5;
        let ay = cy + a0.sin() * (r0 + r1) * 0.5;
        let bx = cx + a1.cos() * (r0 + r1) * 0.5;
        let by = cy + a1.sin() * (r0 + r1) * 0.5;

        let center = Offset::new(cx, cy);

        path.clear();
        path.arc(center, r0, a0, a1, Solidity::Hole);
        path.arc(center, r1, a1, a0, Solidity::Solid);
        path.close();

        let inner_color = Color::hsla(a0 / (PI * 2.0), 1.0, 0.55, 1.0);
        let outer_color = Color::hsla(a1 / (PI * 2.0), 1.0, 0.55, 1.0);

        let paint = Paint::linear_gradient([ax, ay], [bx, by], inner_color, outer_color);
        ctx.draw_path(&path, paint);
    }

    path.clear();
    path.circle([cx, cy].into(), r0 - 0.5);
    path.circle([cx, cy].into(), r1 + 0.5);
    ctx.draw_path(&path, Paint::stroke(Color::hex(0x40_000000)));

    // Selector
    ctx.save();
    ctx.translate(cx, cy);
    ctx.rotate(hue * PI * 2.0);

    // Marker on
    let paint = Paint::stroke(Color::hex(0xC0_FFFFFF)).stroke_width(2.0);
    ctx.draw_rect(Rect::from_ltwh(r0 - 1.0, -3.0, r1 - r0 + 2.0, 6.0), paint);

    let paint = Paint::box_gradient(
        Rect::from_ltwh(r0 - 3.0, -5.0, r1 - r0 + 6.0, 10.0),
        2.0,
        4.0,
        Color::hex(0x80_000000),
        Color::hex(0x00_000000),
    );
    path.clear();
    path.rect(Rect::from_ltwh(
        r0 - 2.0 - 10.0,
        -4.0 - 10.0,
        r1 - r0 + 4.0 + 20.0,
        8.0 + 20.0,
    ));
    path.rect(Rect::from_ltwh(r0 - 2.0, -4.0, r1 - r0 + 4.0, 8.0));
    path.solidity(Solidity::Hole);
    ctx.draw_path(&path, paint);

    // Center triangle
    let radius = r0 - 6.0;
    let ax = (120.0 / 180.0 * PI).cos() * radius;
    let ay = (120.0 / 180.0 * PI).sin() * radius;
    let bx = (-120.0 / 180.0 * PI).cos() * radius;
    let by = (-120.0 / 180.0 * PI).sin() * radius;

    path.clear();
    path.move_to((radius, 0.0).into());
    path.line_to((ax, ay).into());
    path.line_to((bx, by).into());
    path.close();

    let inner_color = Color::hsla(hue, 1.0, 0.5, 1.0);
    let paint = Paint::linear_gradient([radius, 0.0], [ax, ay], inner_color, Color::WHITE);

    ctx.draw_path(&path, paint);

    let from = [(radius + ax) * 0.5, (0.0 + ay) * 0.5];
    let paint = Paint::linear_gradient(from, [bx, by], Color::TRANSPARENT, Color::BLACK);
    ctx.draw_path(&path, paint);

    let paint = Paint::stroke(Color::hex(0x40_000000)).stroke_width(2.0);
    ctx.draw_path(&path, paint);

    // Select circle on triangle
    let ax = (120.0 / 180.0 * PI).cos() * radius * 0.3;
    let ay = (120.0 / 180.0 * PI).sin() * radius * 0.4;
    let paint = Paint::stroke(Color::hex(0xC0_FFFFFF)).stroke_width(2.0);
    ctx.draw_circle([ax, ay].into(), 5.0, paint);

    let paint = Paint::radial_gradient(
        [ax, ay],
        7.0,
        9.0,
        Color::hex(0x40_000000),
        Color::hex(0x00_000000),
    );
    path.clear();
    path.rect(Rect::from_ltwh(ax - 20.0, ay - 20.0, 40.0, 40.0));
    path.circle([ax, ay].into(), 7.0);
    path.solidity(Solidity::Solid);
    ctx.draw_path(&path, paint);

    ctx.restore();
}
