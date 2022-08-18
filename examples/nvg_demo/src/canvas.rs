use reui::{
    BoxGradient, Canvas, Color, FillRule, LineCap, LineJoin, LinearGradient, Offset, Path,
    RadialGradient, Rect, Rounding, Solidity, Stroke, Transform,
};
use std::f32::consts::TAU;

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

        draw_rrect(ctx, 400.0, 15.0);
        draw_fills(ctx, 450.0, 0.0);

        draw_blending(ctx, 260.0, 5.0, 85.0, 40.0);
        draw_palette(ctx, Offset::new(10.0, 10.0));

        {
            //let pos = bounds.center();
            let pos = Offset::new(570.0, 15.0);
            let rect = Rect::from_size(4.0, 4.0).translate(pos);

            ctx.fill_rect(rect, Color::bgra(0xFF_CC0000));

            let tr = Transform::rotation(time);
            let pos = tr.apply(Offset::new(10.0, 0.0));

            ctx.fill_rect(rect.translate(pos), Color::bgra(0x99_CC0000));
        }

        if blowup {
            ctx.push_rotate(((time * 0.3).sin() * 5.0).to_radians());
            ctx.push_scale(2.0);
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
        super::blendish::run(ctx, Rect::from_ltwh(380.0, 50.0, 200.0, 200.0));
        ctx.restore();
    }

    if false {
        // Canvas test

        ctx.fill_rect(Rect::from_ltwh(50.0, 50.0, 100.0, 100.0), Color::BLACK);
        let radius = Rounding::same(15.0);
        ctx.fill_rrect(
            Rect::new(Offset::new(50.0, 50.0), Offset::new(100.0, 100.0)),
            radius,
            Color::bgra(0xFF_CC0000),
        );

        ctx.stroke_line(
            [60.0, 60.0].into(),
            [140.0, 140.0].into(),
            Color::bgra(0xFF_00CCCC),
            Stroke::default(),
        );
    }
}

fn draw_blending(ctx: &mut Canvas, x: f32, y: f32, width: f32, height: f32) {
    // blending test

    let bg_paint = Color::bgra(0xFF_FFFFFF);
    ctx.fill_rect(Rect::from_ltwh(x, y, width, height), bg_paint);

    let vg = Color::bgra(0xFF_18EA22);
    let vy = Color::bgra(0xFF_EAE818);
    let vc = Color::bgra(0xFF_18EAC8);
    let vm = Color::bgra(0xFF_EE46A6);

    let w = 10.0;
    let rect = Rect::from_size(w, height);
    ctx.fill_rect(rect.shift(x + w * 0.5, y), vc);
    ctx.fill_rect(rect.shift(x + w * 2.5, y), vm);
    ctx.fill_rect(rect.shift(x + w * 4.5, y), vy);
    ctx.fill_rect(rect.shift(x + w * 6.5, y), vg);

    let hr: [_; 2] = [Color::bgra(0xFF_FF0000), Color::bgra(0x7F_FF0000)];
    let hg: [_; 2] = [Color::bgra(0xFF_93FF00), Color::bgra(0x7F_93FF00)];
    let hb: [_; 2] = [Color::bgra(0xFF_007FFF), Color::bgra(0x7F_007FFF)];

    let h = 4.0;
    let rect = Rect::from_size(width, h);
    ctx.fill_rect(rect.shift(x, y + h * 1.0), hr[0]);
    ctx.fill_rect(rect.shift(x, y + h * 2.0), hr[1]);

    ctx.fill_rect(rect.shift(x, y + h * 4.0), hg[0]);
    ctx.fill_rect(rect.shift(x, y + h * 5.0), hg[1]);

    ctx.fill_rect(rect.shift(x, y + h * 7.0), hb[0]);
    ctx.fill_rect(rect.shift(x, y + h * 8.0), hb[1]);
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
        let color = Color::bgra(color);
        let size = 10.0;

        let (x, y) = (i % 8, i / 8);

        let x = offset.x + 1.25 * size * x as f32;
        let y = offset.y + 1.25 * size * y as f32;
        ctx.fill_rect(Rect::from_ltwh(x, y, size, size), color)
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

        let inner_color = Color::bgra(inner_color);
        let outer_color = Color::bgra(outer_color);

        let paint = LinearGradient::new([x, y], [x + w, y], inner_color, outer_color);
        ctx.fill_rect(Rect::from_ltwh(x, y, w, h), paint);
    }
}

pub fn draw_window(ctx: &mut Canvas, bounds: Rect, content: impl FnOnce(&mut Canvas, Rect)) {
    let header_bg = Color::bgra(0xFC_3B3F41);
    let body_bg = Color::bgra(0xFC_47484B);

    let btn_r = Color::bgra(0xFF_FE5850);
    let btn_y = Color::bgra(0xFF_FDBE34);
    let btn_g = Color::bgra(0xFF_26C940);

    //let header_bg = Color::hex(0xFF_0D0D0D);
    //let body_bg = Color::hex(0xEE_0D0D0D);

    let [left, top, width, height] = bounds.to_xywh();

    let corner_radius = 4.0;
    let header_height = 20.0;

    {
        // body
        let rect = Rect::from_ltwh(left, top + header_height, width, height - header_height);
        let radius = Rounding::bottom(corner_radius);
        ctx.fill_rrect(rect, radius, body_bg);

        content(ctx, rect);
    }

    {
        // header
        let header_paint = header_bg;

        let radius = Rounding::top(corner_radius);
        let rect = Rect::from_ltwh(left, top, width, header_height);
        ctx.fill_rrect(rect, radius, header_paint);

        let radius = 5.0;
        let center = Offset::new(left, top) + Offset::new(header_height, header_height) / 2.0;
        let off = Offset::new(radius * 3.0, 0.0);
        ctx.fill_circle(center + off * 0.0, radius, btn_r);
        ctx.fill_circle(center + off * 1.0, radius, btn_y);
        ctx.fill_circle(center + off * 2.0, radius, btn_g);
    }

    // Drop shadow
    {
        let mut path = Path::new();
        path.rect(bounds.inflate(20.0));
        path.rrect(bounds, Rounding::same(corner_radius));
        path.solidity(Solidity::Hole);
        let shadow_paint = BoxGradient::new(
            bounds,
            corner_radius,
            20.0,
            Color::bgra(0xF8_FF00FF),
            Color::bgra(0x00_000000),
        );
        ctx.fill_path(&path, shadow_paint, FillRule::NonZero);
    }
}

pub fn draw_search_box(ctx: &mut Canvas, rect: Rect) {
    let [left, top, width, height] = rect.to_xywh();

    let corner_radius = Rounding::same(height / 2.0 - 1.0);

    let paint = BoxGradient::new(
        Rect::from_ltwh(left, top + 1.5, width, height),
        height / 2.0,
        5.0,
        Color::bgra(0x10_000000),
        Color::bgra(0x60_000000),
    );
    ctx.fill_rrect(rect, corner_radius, paint);
}

pub fn draw_button(ctx: &mut Canvas, rr: Rect, col: u32) {
    let [x, y, w, h] = rr.to_xywh();

    let corner_radius = 4.0;

    let col = Color::bgra(col);
    let alpha = if col == Color::TRANSPARENT { 16 } else { 32 };

    let radius = Rounding::same(corner_radius - 1.0);
    let rrect = Rect::from_ltwh(x + 1.0, y + 1.0, w - 2.0, h - 2.0);
    if col != Color::TRANSPARENT {
        ctx.fill_rrect(rrect, radius, col);
    }
    let rgb = 127;
    let inner_color = Color::new_srgba8(rgb, rgb, rgb, alpha);
    let outer_color = Color::new_srgba8(0, 0, 0, alpha);
    let paint = LinearGradient::new([x, y], [x, y + h], inner_color, outer_color);
    ctx.fill_rrect(rrect, radius, paint);

    let radius = Rounding::same(corner_radius - 0.5);
    let rrect = Rect::from_ltwh(x + 0.5, y + 0.5, w - 1.0, h - 1.0);
    ctx.stroke_rrect(rrect, radius, Color::bgra(0x30_000000), Stroke::default());
}

pub fn draw_checkbox(ctx: &mut Canvas, rr: Rect) {
    let [x, y, _, h] = rr.to_xywh();

    let radius = Rounding::same(3.0);
    let rect = Rect::from_ltwh(x + 1.0, y + (h * 0.5).floor() - 9.0, 18.0, 18.0);
    let paint = BoxGradient::new(
        Rect::from_ltwh(x + 1.0, y + (h * 0.5).floor() - 9.0 + 1.0, 18.0, 18.0),
        3.0,
        3.0,
        Color::bgra(0x20_000000),
        Color::bgra(0x60_000000),
    );
    ctx.fill_rrect(rect, radius, paint);
}

pub fn draw_drop_down(ctx: &mut Canvas, bounds: Rect) {
    let [x, y, w, h] = bounds.to_xywh();

    let corner_radius = 4.0;

    let rect = Rect::from_ltwh(x + 1.0, y + 1.0, w - 2.0, h - 2.0);
    let radius = Rounding::same(corner_radius - 1.0);
    let paint = LinearGradient::new(
        bounds.min.into(),
        [x, y + h],
        Color::bgra(0x10_FFFFFF),
        Color::bgra(0x10_000000),
    );
    ctx.fill_rrect(rect, radius, paint);

    let rect = Rect::from_ltwh(x + 0.5, y + 0.5, w - 1.0, h - 1.0);
    let radius = Rounding::same(corner_radius - 0.5);
    ctx.stroke_rrect(rect, radius, Color::bgra(0x30_000000), Stroke::default());
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

    let bg = LinearGradient::new(
        [x, y + h * 0.5],
        [x + w * 0.1, y + h],
        Color::bgra(0x20_000000),
        Color::bgra(0x10_000000),
    );
    ctx.fill_oval(Rect::from_oval(lx + 3.0, ly + 16.0, ex, ey), bg);
    ctx.fill_oval(Rect::from_oval(rx + 3.0, ry + 16.0, ex, ey), bg);

    let bg = LinearGradient::new(
        [x, y + h * 0.25],
        [x + w * 0.1, y + h],
        Color::bgra(0xFF_DCDCDC),
        Color::bgra(0xFF_808080),
    );
    ctx.fill_oval(Rect::from_oval(lx, ly, ex, ey), bg);
    ctx.fill_oval(Rect::from_oval(rx, ry, ex, ey), bg);

    let eye_paint = Color::bgra(0xFF_202020);

    let mut dx = (mx - rx) / (ex * 10.0);
    let mut dy = (my - ry) / (ey * 10.0);
    let dd = (dx * dx + dy * dy).sqrt();
    if dd > 1.0 {
        dx /= dd;
        dy /= dd;
    }
    dx *= ex * 0.4;
    dy *= ey * 0.5;
    ctx.fill_oval(
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
    ctx.fill_oval(
        Rect::from_oval(rx + dx, ry + dy + ey * 0.25 * (1.0 - blink), br, br * blink),
        eye_paint,
    );

    let inr = ex * 0.1;
    let outr = ex * 0.75;
    let inner_color = Color::bgra(0x80_FFFFFF);
    let outer_color = Color::bgra(0x00_FFFFFF);

    let left = [lx - ex * 0.25, ly - ey * 0.5];
    let right = [rx - ex * 0.25, ry - ey * 0.5];

    let left_paint = RadialGradient::new(left, inr, outr, inner_color, outer_color);
    ctx.fill_oval(Rect::from_oval(lx, ly, ex, ey), left_paint);

    let right_paint = RadialGradient::new(right, inr, outr, inner_color, outer_color);
    ctx.fill_oval(Rect::from_oval(rx, ry, ex, ey), right_paint);
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
    ctx.fill_path(
        &path,
        LinearGradient::new(
            [x, y],
            [x, y + h],
            Color::new_srgba8(0, 160, 192, 0),
            Color::new_srgba8(0, 160, 192, 64),
        ),
        FillRule::NonZero,
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
    ctx.stroke_path(&path, Color::bgra(0x20_000000), Stroke::width(3.0));

    path.clear();
    path.move_to((sx[0], sy[0]).into());
    for i in 1..6 {
        path.cubic_to(
            (sx[i - 1] + dx * 0.5, sy[i - 1]).into(),
            (sx[i] - dx * 0.5, sy[i]).into(),
            (sx[i], sy[i]).into(),
        );
    }
    ctx.stroke_path(&path, Color::bgra(0xFF_00A0C0), Stroke::width(3.0));

    // Graph sample pos
    for i in 0..6 {
        let [x, y] = [sx[i] - 10.0, sy[i] - 10.0 + 2.0];
        ctx.fill_rect(
            Rect::from_ltwh(x, y, 20.0, 20.0),
            RadialGradient::new(
                [sx[i], sy[i] + 2.0],
                3.0,
                8.0,
                Color::bgra(0x20_000000),
                Color::bgra(0x00_000000),
            ),
        );
    }

    for i in 0..6 {
        ctx.fill_circle([sx[i], sy[i]].into(), 4.0, Color::bgra(0xFF_00A0C0));
        ctx.fill_circle([sx[i], sy[i]].into(), 2.0, Color::bgra(0xFF_DCDCDC));
    }
}

pub fn draw_widths(ctx: &mut Canvas, x: f32, y: f32, width: f32) {
    let paint = Color::bgra(0xFF_999999);

    let mut y = y;
    for i in 0..75 {
        let stroke = Stroke::width((0.05 + i as f32) * 0.025);
        ctx.stroke_line(
            [x, y].into(),
            [x + width, y + width * 0.3].into(),
            paint,
            stroke,
        );
        y += 4.0;
    }
}

pub fn draw_caps(ctx: &mut Canvas, x: f32, y: f32, width: f32) {
    let caps = [LineCap::Butt, LineCap::Round, LineCap::Square];
    let line_width = 8.0;

    ctx.fill_rect(
        Rect::from_ltwh(x - line_width / 2.0, y, width + line_width, 40.0),
        Color::bgra(0x20_FFFFFF),
    );
    ctx.fill_rect(Rect::from_ltwh(x, y, width, 40.0), Color::bgra(0x20_FFFFFF));

    let paint = Color::BLACK;
    let stroke = Stroke::width(line_width);

    for (i, &cap) in caps.iter().enumerate() {
        let stroke = stroke.cap(cap);
        let y = y + ((i * 10) as f32) + 5.0;
        ctx.stroke_line([x, y].into(), [x + width, y].into(), paint, stroke)
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

            ctx.stroke_path(
                &path,
                Color::bgra(0xA0_000000),
                Stroke::width(size * 0.3).cap(cap).joint(join),
            );

            ctx.stroke_path(
                &path,
                Color::bgra(0xFF_00C0FF),
                Stroke::width(1.0).cap(LineCap::Butt).joint(LineJoin::Bevel),
            );
        }
    }
}

fn draw_edit_box_base(ctx: &mut Canvas, rr: Rect) {
    let [left, top, width, height] = rr.to_xywh();

    let bg = BoxGradient::new(
        Rect::from_ltwh(left + 1.0, top + 1.0 + 1.5, width - 2.0, height - 2.0),
        3.0,
        4.0,
        Color::bgra(0x20_FFFFFF),
        Color::bgra(0x20_202020),
    );

    let rect = Rect::from_ltwh(left + 1.0, top + 1.0, width - 2.0, height - 2.0);
    let radius = Rounding::same(4.0 - 1.0);
    ctx.fill_rrect(rect, radius, bg);

    let rect = Rect::from_ltwh(left + 0.5, top + 0.5, width - 1.0, height - 1.0);
    let radius = Rounding::same(4.0 - 0.5);
    ctx.stroke_rrect(rect, radius, Color::bgra(0x30_000000), Stroke::default());
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
    ctx.fill_rrect(
        Rect::from_ltwh(x, cy - 2.0, w, 4.0),
        Rounding::same(2.0),
        BoxGradient::new(
            Rect::from_ltwh(x, cy - 2.0 + 1.0, w, 4.0),
            2.0,
            2.0,
            Color::bgra(0x20_000000),
            Color::bgra(0x80_000000),
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
    ctx.fill_path(
        &path,
        RadialGradient::new(
            [x + (pos * w).floor(), cy + 1.0],
            kr - 3.0,
            kr + 3.0,
            Color::bgra(0x40_000000),
            Color::bgra(0x00_000000),
        ),
        FillRule::NonZero,
    );

    // Knob
    let knob = LinearGradient::new(
        [x, cy - kr],
        [x, cy + kr],
        Color::bgra(0x10_FFFFFF),
        Color::bgra(0x10_000000),
    );

    let center = [x + (pos * w).floor(), cy].into();
    ctx.fill_circle(center, kr - 1.0, Color::bgra(0xFF_282B30));
    ctx.fill_circle(center, kr - 1.0, knob);
    ctx.stroke_circle(
        center,
        kr - 0.5,
        Color::bgra(0x5C_000000),
        Stroke::default(),
    );
}

pub fn draw_colorwheel(ctx: &mut Canvas, rr: Rect, time: f32) {
    let [x, y, w, h] = rr.to_xywh();
    let hue = (time * 0.12).sin();

    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    let r1 = if w < h { w } else { h } * 0.5 - 5.0;
    let r0 = r1 - 20.0;
    let aeps = 0.5 / r1; // half a pixel arc length in radians (2pi cancels out).

    let count = 12;
    let mut path = Path::new();
    for i in 0..count {
        let a0 = (i as f32) * TAU / count as f32 - aeps;
        let a1 = ((i as f32) + 1.0) * TAU / count as f32 + aeps;
        let ax = cx + a0.cos() * (r0 + r1) * 0.5;
        let ay = cy + a0.sin() * (r0 + r1) * 0.5;
        let bx = cx + a1.cos() * (r0 + r1) * 0.5;
        let by = cy + a1.sin() * (r0 + r1) * 0.5;

        let center = Offset::new(cx, cy);

        path.clear();
        path.arc(center, r0, a0, a1, Solidity::Hole);
        path.arc(center, r1, a1, a0, Solidity::Solid);
        path.close();

        let inner_color = Color::hsla(a0.to_degrees(), 1.0, 0.55, 1.0);
        let outer_color = Color::hsla(a1.to_degrees(), 1.0, 0.55, 1.0);

        let paint = LinearGradient::new([ax, ay], [bx, by], inner_color, outer_color);
        ctx.fill_path(&path, paint, FillRule::NonZero);
    }

    path.clear();
    path.circle([cx, cy].into(), r0 - 0.5);
    path.circle([cx, cy].into(), r1 + 0.5);
    ctx.stroke_path(&path, Color::bgra(0x40_000000), Stroke::default());

    // Selector
    ctx.save();
    ctx.push_translate(cx, cy);
    ctx.push_rotate(hue * TAU);

    // Marker on
    let paint = Color::bgra(0xC0_FFFFFF);
    ctx.stroke_rect(
        Rect::from_ltwh(r0 - 1.0, -3.0, r1 - r0 + 2.0, 6.0),
        paint,
        Stroke::width(2.0),
    );

    let paint = BoxGradient::new(
        Rect::from_ltwh(r0 - 3.0, -5.0, r1 - r0 + 6.0, 10.0),
        2.0,
        4.0,
        Color::bgra(0x80_000000),
        Color::bgra(0x00_000000),
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
    ctx.fill_path(&path, paint, FillRule::NonZero);

    // Center triangle
    let radius = r0 - 6.0;
    let ax = f32::to_radians(120.0).cos() * radius;
    let ay = f32::to_radians(120.0).sin() * radius;
    let bx = f32::to_radians(-120.0).cos() * radius;
    let by = f32::to_radians(-120.0).sin() * radius;

    path.clear();
    path.polyline(
        &[
            Offset::new(radius, 0.0),
            Offset::new(ax, ay),
            Offset::new(bx, by),
        ],
        true,
    );

    let inner_hue = (hue * TAU).to_degrees();
    let inner_color = Color::hsla(inner_hue, 1.0, 0.5, 1.0);
    let paint = LinearGradient::new([radius, 0.0], [ax, ay], inner_color, Color::WHITE);

    ctx.fill_path(&path, paint, FillRule::NonZero);

    let from = [(radius + ax) * 0.5, (0.0 + ay) * 0.5];
    let paint = LinearGradient::new(from, [bx, by], Color::TRANSPARENT, Color::BLACK);
    ctx.fill_path(&path, paint, FillRule::NonZero);

    let paint = Color::bgra(0x40_000000);
    ctx.stroke_path(&path, paint, Stroke::width(2.0));

    // Select circle on triangle
    let ax = f32::to_radians(120.0).cos() * radius * 0.3;
    let ay = f32::to_radians(120.0).sin() * radius * 0.4;
    let paint = Color::bgra(0xC0_FFFFFF);
    ctx.stroke_circle([ax, ay].into(), 5.0, paint, Stroke::width(2.0));

    let paint = RadialGradient::new(
        [ax, ay],
        7.0,
        9.0,
        Color::bgra(0x40_000000),
        Color::bgra(0x00_000000),
    );
    path.clear();
    path.rect(Rect::from_ltwh(ax - 20.0, ay - 20.0, 40.0, 40.0));
    path.circle([ax, ay].into(), 7.0);
    path.solidity(Solidity::Solid);
    ctx.fill_path(&path, paint, FillRule::NonZero);

    ctx.restore();
}

fn draw_rrect(canvas: &mut Canvas, x: f32, y: f32) {
    canvas.save();
    canvas.push_translate(x, y);

    let paint = Color::bgra(0x78_DCDCDC);

    let r = 10.0;
    let size = 15.0;
    let mid = 20.0;

    // North-West (left top)
    let rect = Rect::from_center(Offset::new(0.0, 0.0), size, size);
    canvas.fill_rrect(rect, Rounding::nw(r), paint);

    // North-East (right top)
    let rect = Rect::from_center(Offset::new(mid, 0.0), size, size);
    canvas.fill_rrect(rect, Rounding::ne(r), paint);

    // South-West (left bottom)
    let rect = Rect::from_center(Offset::new(0.0, mid), size, size);
    canvas.fill_rrect(rect, Rounding::sw(r), paint);

    // South-East (right bottom)
    let rect = Rect::from_center(Offset::new(mid, mid), size, size);
    canvas.fill_rrect(rect, Rounding::se(r), paint);

    canvas.restore();
}

fn draw_fills(canvas: &mut Canvas, x: f32, y: f32) {
    canvas.save();
    canvas.push_translate(x, y);
    canvas.push_scale(0.5);

    let color = Color::bgra(0x78_DCDCDC);

    let mut path = Path::new();
    path.move_to(Offset::new(50.0, 0.0));
    path.line_to(Offset::new(21.0, 90.0));
    path.line_to(Offset::new(98.0, 35.0));
    path.line_to(Offset::new(2.0, 35.0));
    path.line_to(Offset::new(79.0, 90.0));
    path.close();

    canvas.fill_path(&path, color, FillRule::EvenOdd);

    canvas.push_translate(100.0, 0.0);

    let mut path = Path::new();
    path.move_to(Offset::new(50.0, 0.0));
    path.line_to(Offset::new(21.0, 90.0));
    path.line_to(Offset::new(98.0, 35.0));
    path.line_to(Offset::new(2.0, 35.0));
    path.line_to(Offset::new(79.0, 90.0));
    path.close();

    canvas.fill_path(&path, color, FillRule::NonZero);

    canvas.restore();
}
