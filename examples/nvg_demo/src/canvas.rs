use reui::{
    Canvas, Color, Offset, Paint, PaintingStyle, Path, RRect, Rect, StrokeCap, StrokeJoin, Winding,
};
use std::f32::consts::PI;

pub fn render_demo(ctx: &mut Canvas, mouse: Offset, wsize: Offset, time: f32, blowup: bool) {
    let (width, height) = wsize.into();

    {
        draw_eyes(
            ctx,
            Rect::from_ltwh(width - 250.0, 50.0, 150.0, 100.0),
            mouse,
            time,
        );
        draw_graph(ctx, 0.0, height / 2.0, width, height / 2.0, time);
        draw_colorwheel(
            ctx,
            Rect::from_ltwh(width - 300.0, height - 300.0, 250.0, 250.0),
            time,
        );

        // Line joints
        draw_lines(ctx, 120.0, height - 50.0, 600.0, 50.0, time);
        // Line caps
        draw_widths(ctx, 10.0, 50.0, 30.0);
        // Line caps
        draw_caps(ctx, 10.0, 300.0, 30.0);

        if blowup {
            ctx.rotate((time * 0.3).sin() * 5.0 / 180.0 * PI);
            ctx.scale(2.0);
        }

        // Widgets
        draw_window(ctx, Rect::from_ltwh(50.0, 50.0, 300.0, 400.0));

        let (x, mut y) = (60.0, 95.0);
        draw_search_box(ctx, Rect::from_ltwh(x, y, 280.0, 25.0));
        y += 40.0;
        draw_drop_down(ctx, Rect::from_ltwh(x, y, 280.0, 28.0));
        y += 45.0;

        // Form
        y += 25.0;
        draw_edit_box(ctx, Rect::from_ltwh(x, y, 280.0, 28.0));
        y += 35.0;
        draw_edit_box(ctx, Rect::from_ltwh(x, y, 280.0, 28.0));
        y += 38.0;
        draw_checkbox(ctx, Rect::from_ltwh(x, y, 140.0, 28.0));
        draw_button(ctx, Rect::from_ltwh(x + 138.0, y, 140.0, 28.0), 0xFF_006080);
        y += 45.0;

        // Slider
        y += 25.0;
        draw_edit_box_num(ctx, Rect::from_ltwh(x + 180.0, y, 100.0, 28.0));
        draw_slider(ctx, 0.4, x, y, 170.0, 28.0);
        y += 55.0;

        draw_button(ctx, Rect::from_ltwh(x, y, 160.0, 28.0), 0xFF_801008);
        draw_button(ctx, Rect::from_ltwh(x + 170.0, y, 110.0, 28.0), 0x00_000000);
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
        ctx.draw_rrect(
            RRect::new([50.0, 50.0].into(), [100.0, 100.0].into(), 15.0),
            Paint::fill(Color::hex(0xFF_CC0000)),
        );

        ctx.draw_line(
            [60.0, 60.0].into(),
            [140.0, 140.0].into(),
            Paint::stroke(Color::hex(0xFF_00CCCC)),
        );
    }

    {
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

            let x = 80.0 + (i * 32) as f32;
            ctx.draw_rect(
                Rect::from_ltwh(x, 10.0, 20.0, 20.0),
                Paint {
                    style: PaintingStyle::Fill,
                    color,
                    ..Paint::default()
                },
            )
        }

        let grad: &[[u32; 2]] = &[
            [0xFF_000000, 0xFF_FFFFFF], // 000 111
            [0xFF_FF0000, 0xFF_00FF00], // 100 010
            [0xFF_00FF00, 0xFF_0000FF], // 010 001
            [0xFF_0000FF, 0xFF_FF0000], // 001 100
            [0xFF_FF00FF, 0xFF_FFFF00], // 101 110
            [0xFF_FFFF00, 0xFF_00FFFF], // 110 011
        ];

        for (i, &[inner_color, outer_color]) in grad.iter().enumerate() {
            let w = 128.0;
            let h = 5.0;
            let x = 650.0;
            let y = 5.0 + h * i as f32;

            let inner_color = Color::hex(inner_color);
            let outer_color = Color::hex(outer_color);

            let paint = Paint::linear_gradient([x, y], [x + w, y], inner_color, outer_color);
            ctx.draw_rect(Rect::from_ltwh(x, y, w, h), paint);
        }
    }

    {
        // blending test

        fn srgba_fill(c: u32) -> Paint {
            Paint::fill(Color::hex(c))
        }

        let x = 600.0;
        let y = 40.0;
        let w = 85.0;
        let h = 70.0;

        let bg_paint = Paint::fill(Color::hex(0xFF_FFFFFF));
        ctx.draw_rect(Rect::from_ltwh(x, y, w, h), bg_paint);

        let vg = srgba_fill(0xFF_18EA22);
        let vy = srgba_fill(0xFF_EAE818);
        let vc = srgba_fill(0xFF_18EAC8);
        let vm = srgba_fill(0xFF_EE46A6);

        ctx.draw_rect(Rect::from_ltwh(x + 15.0 * 1.0, y, 10.0, h), vc);
        ctx.draw_rect(Rect::from_ltwh(x + 15.0 * 2.0, y, 10.0, h), vm);
        ctx.draw_rect(Rect::from_ltwh(x + 15.0 * 3.0, y, 10.0, h), vy);
        ctx.draw_rect(Rect::from_ltwh(x + 15.0 * 4.0, y, 10.0, h), vg);

        let hr: [_; 2] = [srgba_fill(0xFF_FF0000), srgba_fill(0x7F_FF0000)];
        let hg: [_; 2] = [srgba_fill(0xFF_93FF00), srgba_fill(0x7F_93FF00)];
        let hb: [_; 2] = [srgba_fill(0xFF_007FFF), srgba_fill(0x7F_007FFF)];

        ctx.draw_rect(Rect::from_ltwh(x, y + 15.0 * 1.0, w, 5.0), hr[0]);
        ctx.draw_rect(Rect::from_ltwh(x, y + 15.0 * 1.0 + 5.0, w, 5.0), hr[1]);

        ctx.draw_rect(Rect::from_ltwh(x, y + 15.0 * 2.0, w, 5.0), hg[0]);
        ctx.draw_rect(Rect::from_ltwh(x, y + 15.0 * 2.0 + 5.0, w, 5.0), hg[1]);

        ctx.draw_rect(Rect::from_ltwh(x, y + 15.0 * 3.0, w, 5.0), hb[0]);
        ctx.draw_rect(Rect::from_ltwh(x, y + 15.0 * 3.0 + 5.0, w, 5.0), hb[1]);
    }
}

pub fn draw_window(ctx: &mut Canvas, rr: Rect) {
    let [x, y, w, h] = rr.to_xywh();

    let corner_radius = 3.0;

    // Window
    let rrect = RRect::from_rect_and_radius(rr, corner_radius);
    ctx.draw_rrect(rrect, Paint::fill(Color::hex(0xC0_1C1E22)));

    // Drop shadow
    let mut path = Path::new();
    path.add_rect(Rect::from_ltwh(x - 10.0, y - 10.0, w + 20.0, h + 20.0));
    path.add_rrect(rrect);
    path.path_winding(Winding::CW);
    ctx.draw_path(
        &path,
        Paint::box_gradient(
            Rect::from_ltwh(x, y + 2.0, rr.dx(), rr.dy()),
            corner_radius * 2.0,
            10.0,
            Color::hex(0x80_000000),
            Color::hex(0x00_000000),
        ),
    );

    // Header
    let header_paint = Paint::linear_gradient(
        [x, y],
        [x, y + 15.0],
        Color::hex(0x08_FFFFFF),
        Color::hex(0x10_000000),
    );

    let rrect = RRect::new(
        [x + 1.0, y + 1.0].into(),
        [w - 2.0, 30.0].into(),
        corner_radius - 1.0,
    );
    ctx.draw_rrect(rrect, header_paint);

    ctx.draw_line(
        [x + 0.5, y + 0.5 + 30.0].into(),
        [x + 0.5 + w - 1.0, y + 0.5 + 30.0].into(),
        Paint::stroke(Color::hex(0x20_000000)),
    );
}

pub fn draw_search_box(ctx: &mut Canvas, rr: Rect) {
    let [x, y, _, h] = rr.to_xywh();

    let corner_radius = h / 2.0 - 1.0;

    // Edit
    let rrect = RRect::from_rect_and_radius(rr, corner_radius);
    ctx.draw_rrect(
        rrect,
        Paint::box_gradient(
            Rect::from_ltwh(x, y + 1.5, rr.dx(), rr.dy()),
            h / 2.0,
            5.0,
            Color::hex(0x10_000000),
            Color::hex(0x60_000000),
        ),
    );
}

pub fn draw_button(ctx: &mut Canvas, rr: Rect, col: u32) {
    let [x, y, w, h] = rr.to_xywh();

    let corner_radius = 4.0;

    let col = Color::hex(col);
    let alpha = if col == Color::TRANSPARENT { 16 } else { 32 };

    let rrect = RRect::new(
        [x + 1.0, y + 1.0].into(),
        [w - 2.0, h - 2.0].into(),
        corner_radius - 1.0,
    );
    if col != Color::TRANSPARENT {
        ctx.draw_rrect(rrect, Paint::fill(col));
    }
    let rgb = 127;
    let inner_color = Color::new_srgba8(rgb, rgb, rgb, alpha);
    let outer_color = Color::new_srgba8(0, 0, 0, alpha);
    ctx.draw_rrect(
        rrect,
        Paint::linear_gradient([x, y], [x, y + h], inner_color, outer_color),
    );

    let rrect = RRect::new(
        [x + 0.5, y + 0.5].into(),
        [w - 1.0, h - 1.0].into(),
        corner_radius - 0.5,
    );
    ctx.draw_rrect(rrect, Paint::stroke(Color::hex(0x30_000000)));
}

pub fn draw_checkbox(ctx: &mut Canvas, rr: Rect) {
    let [x, y, _, h] = rr.to_xywh();

    let rrect = RRect::new(
        [x + 1.0, y + (h * 0.5).floor() - 9.0].into(),
        [18.0, 18.0].into(),
        3.0,
    );
    ctx.draw_rrect(
        rrect,
        Paint::box_gradient(
            Rect::from_ltwh(x + 1.0, y + (h * 0.5).floor() - 9.0 + 1.0, 18.0, 18.0),
            3.0,
            3.0,
            Color::hex(0x20_000000),
            Color::hex(0x60_000000),
        ),
    );
}

pub fn draw_drop_down(ctx: &mut Canvas, bounds: Rect) {
    let [x, y, w, h] = bounds.to_xywh();

    let corner_radius = 4.0;

    let rrect = RRect::new(
        [x + 1.0, y + 1.0].into(),
        [w - 2.0, h - 2.0].into(),
        corner_radius - 1.0,
    );
    ctx.draw_rrect(
        rrect,
        Paint::linear_gradient(
            bounds.min.into(),
            [x, y + h],
            Color::hex(0x10_FFFFFF),
            Color::hex(0x10_000000),
        ),
    );

    let rrect = RRect::new(
        [x + 0.5, y + 0.5].into(),
        [w - 1.0, h - 1.0].into(),
        corner_radius - 0.5,
    );
    ctx.draw_rrect(rrect, Paint::stroke(Color::hex(0x30_000000)));
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

    ctx.draw_oval(
        Rect::from_oval(lx, ly, ex, ey),
        Paint::radial_gradient(left, inr, outr, inner_color, outer_color),
    );

    ctx.draw_oval(
        Rect::from_oval(rx, ry, ex, ey),
        Paint::radial_gradient(right, inr, outr, inner_color, outer_color),
    );
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
    let paint = Paint::stroke(Color::hex(0xFF_000000));

    let mut y = y;
    for i in 0..20 {
        let paint = paint.stroke_width(((i as f32) + 0.5) * 0.1);
        ctx.draw_line([x, y].into(), [x + width, y + width * 0.3].into(), paint);
        y += 10.0;
    }
}

pub fn draw_caps(ctx: &mut Canvas, x: f32, y: f32, width: f32) {
    let caps = [StrokeCap::Butt, StrokeCap::Round, StrokeCap::Square];
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

    let joins = [StrokeJoin::Miter, StrokeJoin::Round, StrokeJoin::Bevel];
    let caps = [StrokeCap::Butt, StrokeCap::Round, StrokeCap::Square];

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
            let fx = x + size * 0.5 + ((i * 3 + j) as f32) / 9.0 * w + pad;
            let fy = y - size * 0.5 + pad;

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
                    .stroke_cap(StrokeCap::Butt)
                    .stroke_join(StrokeJoin::Bevel),
            );
        }
    }
}

fn draw_edit_box_base(ctx: &mut Canvas, rr: Rect) {
    let [x, y, w, h] = rr.to_xywh();

    let bg = Paint::box_gradient(
        Rect::from_ltwh(x + 1.0, y + 1.0 + 1.5, w - 2.0, h - 2.0),
        3.0,
        4.0,
        Color::hex(0x20_FFFFFF),
        Color::hex(0x20_202020),
    );

    ctx.draw_rrect(
        RRect::new(
            [x + 1.0, y + 1.0].into(),
            [w - 2.0, h - 2.0].into(),
            4.0 - 1.0,
        ),
        bg,
    );
    ctx.draw_rrect(
        RRect::new(
            [x + 0.5, y + 0.5].into(),
            [w - 1.0, h - 1.0].into(),
            4.0 - 0.5,
        ),
        Paint::stroke(Color::hex(0x30_000000)),
    );
}

pub fn draw_edit_box(ctx: &mut Canvas, rr: Rect) {
    draw_edit_box_base(ctx, rr);
}

pub fn draw_edit_box_num(ctx: &mut Canvas, rr: Rect) {
    draw_edit_box_base(ctx, rr);
}

pub fn draw_slider(ctx: &mut Canvas, pos: f32, x: f32, y: f32, w: f32, h: f32) {
    let cy = y + (h * 0.5).floor();
    let kr = (h * 0.25).floor();

    // vg.clear_state();

    // Slot
    ctx.draw_rrect(
        RRect::new([x, cy - 2.0].into(), [w, 4.0].into(), 2.0),
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
    path.add_rect(Rect::from_ltwh(
        x + (pos * w).floor() - kr - 5.0,
        cy - kr - 5.0,
        kr * 2.0 + 5.0 + 5.0,
        kr * 2.0 + 5.0 + 5.0 + 3.0,
    ));
    path.add_circle([x + (pos * w).floor(), cy].into(), kr);
    path.path_winding(Winding::CW);
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

        path.clear();
        path.arc(cx, cy, r0, a0, a1, Winding::CW);
        path.arc(cx, cy, r1, a1, a0, Winding::CCW);
        path.close();

        let inner_color = Color::hsla(a0 / (PI * 2.0), 1.0, 0.55, 1.0);
        let outer_color = Color::hsla(a1 / (PI * 2.0), 1.0, 0.55, 1.0);

        let paint = Paint::linear_gradient([ax, ay], [bx, by], inner_color, outer_color);
        ctx.draw_path(&path, paint);
    }

    path.clear();
    path.add_circle([cx, cy].into(), r0 - 0.5);
    path.add_circle([cx, cy].into(), r1 + 0.5);
    ctx.draw_path(&path, Paint::stroke(Color::hex(0x40_000000)));

    // Selector
    ctx.save();
    ctx.translate(cx, cy);
    ctx.rotate(-hue * PI * 2.0);

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
    path.add_rect(Rect::from_ltwh(
        r0 - 2.0 - 10.0,
        -4.0 - 10.0,
        r1 - r0 + 4.0 + 20.0,
        8.0 + 20.0,
    ));
    path.add_rect(Rect::from_ltwh(r0 - 2.0, -4.0, r1 - r0 + 4.0, 8.0));
    path.path_winding(Winding::CW);
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
    path.add_rect(Rect::from_ltwh(ax - 20.0, ay - 20.0, 40.0, 40.0));
    path.add_circle([ax, ay].into(), 7.0);
    path.path_winding(Winding::CW);
    ctx.draw_path(&path, paint);

    ctx.restore();
}
