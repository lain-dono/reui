use oni2d::{canvas::*, math::*};

const HOVER_SHADE: i32 = 15;

fn shade(c: u32, shade: i32) -> u32 {
    Color::new(c).offset(shade).to_bgra()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Normal,
    Hovered,
    Active,
}

pub enum Gropped {
    None,
    StartVertical,
    StartHorizontal,
    Middle,
    EndVertical,
    EndHorizontal,
}

pub struct WidgetTheme {
    pub outline: u32,

    pub background: u32,
    pub active: u32,

    pub color: u32,
    pub radius: f32,

    /*
    pub outline: u32,
    pub item: u32,
    pub inner: u32,
    pub inner_selected: u32,
    pub text: u32,
    pub text_selected: u32,
    */
}

pub struct PanelTheme {
    pub header: u32,
    pub background: u32,
    pub sub_background: u32,
}

pub struct WindowTheme {
    pub background: u32,
}

pub fn run(ctx: &mut Canvas, time: f32, bounds: Rect) {
    let win_theme = WindowTheme {
        background: 0xFF_424242,
    };
    let num_theme = WidgetTheme {
        outline: 0xFF_373737,
        background: 0xFF_595959,
        active: 0xFF_505050,
        color: 0xFF_FFFFFF,
        radius: 4.0,
    };

    let opt_theme = WidgetTheme {
        outline: 0xFF_373737,
        background: 0xFF_666666,
        active: 0xE6_5680C2,
        color: 0xFF_FFFFFF,
        radius: 3.0,
    };

    ctx.scissor(bounds);
    draw_window(ctx, bounds, &win_theme);

    let num = Rect::from_size(160.0, 18.5)
        .translate(bounds.min + Offset::new(10.0, 10.0));

    draw_num(ctx, num, &num_theme, State::Normal, Gropped::StartVertical, "Normal");
    let num = num.translate(Offset::new(0.0, 19.0));
    draw_num(ctx, num, &num_theme, State::Hovered, Gropped::Middle, "Hovered");
    let num = num.translate(Offset::new(0.0, 19.0));
    draw_num(ctx, num, &num_theme, State::Active, Gropped::EndVertical, "Active");

    let num = num.translate(Offset::new(0.0, 30.0));
    draw_num(ctx, num, &num_theme, State::Normal, Gropped::None, "Normal");
    let num = num.translate(Offset::new(0.0, 30.0));

    let opt = Rect::from_size(13.0, 13.0).translate(num.min);

    draw_option(ctx, opt, &opt_theme, State::Normal, "Normal");
    let opt = opt.translate(Offset::new(0.0, 20.0));
    draw_option(ctx, opt, &opt_theme, State::Hovered, "Hovered");
    let opt = opt.translate(Offset::new(0.0, 20.0));
    draw_option(ctx, opt, &opt_theme, State::Active, "Hovered");

    {
        let pos = bounds.center();
        let rect = Rect::from_size(4.0, 4.0).translate(pos);

        ctx.draw_rect(rect, Paint::fill(0xFF_CC0000));

        let tr = Transform::rotation(time);
        let pos = tr.apply([20.0, 0.0]);

        ctx.draw_rect(rect.translate(pos.into()), Paint::fill(0x99_CC0000));
    }
}

pub fn draw_window(ctx: &mut Canvas, bounds: Rect, theme: &WindowTheme) {
    ctx.draw_rect(bounds, Paint::fill(theme.background));

    let rect = bounds.deflate(3.0);
    let rrect = RRect::from_rect_and_radius(rect, 2.5);

    let mut left_scroll = rrect;
    left_scroll.rect.min.x = rrect.rect.max.x - 5.0;
    left_scroll.rect.max.y = rrect.rect.max.y - 50.0;

    ctx.draw_rrect(left_scroll, Paint::fill(0xFF_676767));
    //ctx.draw_rrect(left_scroll.add(1.0), Paint::stroke(0xFF_424242));
    //ctx.draw_rrect(left_scroll, Paint::stroke(0xFF_373737).stroke_width(0.5));
}

pub fn draw_option(
    ctx: &mut Canvas,
    bounds: Rect,
    theme: &WidgetTheme,
    state: State,
    label: &str,
) {
    let bg = match state {
        State::Normal => theme.background,
        State::Hovered => shade(theme.background, HOVER_SHADE),
        State::Active => theme.active,
    };
    let rrect = RRect::from_rect_and_radius(bounds, theme.radius);
    let a = vec2(2.5, 6.0);
    let b = vec2(5.5, 9.0);
    let c = vec2(10.5, 3.5);

    ctx.draw_rrect(rrect, Paint::fill(bg));
    ctx.draw_rrect(rrect.inflate(0.5), Paint::stroke(theme.outline).stroke_width(0.5));

    if state == State::Active {
        ctx.draw_lines(&[
            bounds.min + a,
            bounds.min + b,
            bounds.min + c,
        ], Paint::fill(0xFF_E6E6E6).stroke_width(2.0))
    }

    let p = point2(bounds.min.x + bounds.dx() * 1.375, bounds.min.y + bounds.dy() / 2.0);

    text_with_shadow(ctx, p, theme.color, Align::LEFT|Align::MIDDLE, label);
}

pub fn draw_num(
    ctx: &mut Canvas,
    bounds: Rect,
    theme: &WidgetTheme,
    state: State,
    gropped: Gropped,
    text: &str,
) {
    let bg = match state {
        State::Normal => theme.background,
        State::Hovered => shade(theme.background, HOVER_SHADE * 2),
        State::Active => theme.active,
    };

    let mut rrect = RRect::from_rect_and_radius(bounds, theme.radius);
    match gropped {
        Gropped::None => (),
        Gropped::StartVertical => {
            rrect.radius.bl = 0.0;
            rrect.radius.br = 0.0;
        },
        Gropped::StartHorizontal => {
            rrect.radius.br = 0.0;
            rrect.radius.tr = 0.0;
        },
        Gropped::Middle => {
            rrect.radius.bl = 0.0;
            rrect.radius.br = 0.0;
            rrect.radius.tl = 0.0;
            rrect.radius.tr = 0.0;
        },
        Gropped::EndVertical => {
            rrect.radius.tl = 0.0;
            rrect.radius.tr = 0.0;
        },
        Gropped::EndHorizontal => {
            rrect.radius.tl = 0.0;
            rrect.radius.bl = 0.0;
        },
    }

    ctx.draw_rrect(rrect.inflate(0.5), Paint::stroke(theme.outline).stroke_width(0.5));
    ctx.draw_rrect(rrect, Paint::fill(bg));

        let arr = 13.0;
        let mut left = rrect;
        left.rect.max.x = rrect.rect.min.x + arr;
        left.radius.tr = 0.0;
        left.radius.br = 0.0;
        let mut right = rrect;
        right.rect.min.x = rrect.rect.max.x - arr;
        right.radius.tl = 0.0;
        right.radius.bl = 0.0;

    if state == State::Hovered {
        let paint = Paint::fill(shade(theme.background, HOVER_SHADE));
        ctx.draw_rrect(left, paint);
        ctx.draw_rrect(right, paint);
    }

        let left = left.rect().center();
        let right = right.rect().center();

        let paint = Paint::fill(0xFF_E6E6E6).stroke_width(0.5);

        let cc = vec2(1.5, 0.0);
        let aa = vec2(1.5, -3.0);
        let bb = vec2(1.5, 3.0);

        ctx.draw_lines(&[
            left + aa,
            left - cc,
            left + bb,
        ], paint);
        ctx.draw_lines(&[
            right - aa,
            right + cc,
            right - bb,
        ], paint);

    text_with_shadow(ctx, bounds.center(), theme.color, Align::CENTER|Align::MIDDLE, text);
}

fn text_with_shadow(
    ctx: &mut Canvas,
    position: Offset,
    color: u32,
    text_align: Align,
    text: &str,
) {
    let text_style = TextStyle {
        font_size: 14.0,
        font_face: "sans",
        font_blur: 0.0,
        color,
        text_align,
    };

    let text_style_back = TextStyle {
        font_size: 14.0,
        font_face: "sans",
        font_blur: 1.0,
        color: 0xFF_000000,
        text_align,
    };

    ctx.text(position + vec2(0.0, 0.5), text, text_style_back);
    ctx.text(position, text, text_style);
}