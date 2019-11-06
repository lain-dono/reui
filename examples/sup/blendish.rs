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
        outline: 0xFF_444444,
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

    let mut num = Rect::new(
        bounds.min() + Vector::new(10.0, 10.0),
        euclid::Size2D::new(160.0, 18.0),
    );

    draw_num(ctx, num, &num_theme, State::Normal, Gropped::StartVertical, "Normal");
    let num = num.translate(Vector::new(0.0, 18.5));
    draw_num(ctx, num, &num_theme, State::Hovered, Gropped::Middle, "Hovered");
    let num = num.translate(Vector::new(0.0, 18.5));
    draw_num(ctx, num, &num_theme, State::Active, Gropped::EndVertical, "Active");

    let num = num.translate(Vector::new(0.0, 40.0));

    let mut opt = Rect::new(
        num.min(),
        euclid::Size2D::new(13.0, 13.0),
    );
    draw_option(ctx, opt, &opt_theme, State::Normal, "Normal");
    let opt = opt.translate(Vector::new(0.0, 20.0));
    draw_option(ctx, opt, &opt_theme, State::Hovered, "Hovered");
    let opt = opt.translate(Vector::new(0.0, 20.0));
    draw_option(ctx, opt, &opt_theme, State::Active, "Hovered");
    let opt = opt.translate(Vector::new(0.0, 20.0));

    {
        use oni2d::math::transform::Transform;

        let pos = bounds.center().to_vector();
        let rect = Rect::from_size(euclid::size2(4.0, 4.0))
            .translate(pos);

        ctx.draw_rect(rect, Paint::fill(0xFF_CC0000));

        let tr = Transform::rotation(time);
        let pos = tr.apply([20.0, 0.0]);

        ctx.draw_rect(rect.translate(pos.into()), Paint::fill(0x99_CC0000));
    }
}

pub fn draw_window(ctx: &mut Canvas, bounds: Rect, theme: &WindowTheme) {
    ctx.draw_rect(bounds, Paint::fill(theme.background));

    let rect = bounds.deflate(3.0);
    let rrect = RRect::new(rect.min().into(), rect.size().into(), 2.5);

    let left_scroll = RRect {
        left: rrect.right - 5.0,
        bottom: rrect.bottom - 50.0,
        .. rrect
    };

    ctx.draw_rrect(left_scroll, Paint::fill(0xFF_676767));
    ctx.draw_rrect(left_scroll.add(1.0), Paint::stroke(0xFF_424242));
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
    let rrect = RRect::new(bounds.min().into(), bounds.size().into(), theme.radius);
    ctx.draw_rrect(rrect, Paint::fill(bg));
    let a = vec2(2.5, 6.0);
    let b = vec2(5.5, 9.0);
    let c = vec2(10.5, 3.5);

    if state == State::Active {
        ctx.draw_lines(&[
            (bounds.min() + a).into(),
            (bounds.min() + b).into(),
            (bounds.min() + c).into(),
        ], Paint::fill(0xFF_E6E6E6).stroke_width(2.0))
    }

    ctx.draw_rrect(rrect.add(1.0), Paint::stroke(theme.outline).stroke_width(0.5));

    let p = point2(bounds.min_x() + bounds.dx() * 1.375, bounds.min_y() + bounds.dy() / 2.0);

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

    let mut rrect = RRect::new(bounds.min().into(), bounds.size().into(), theme.radius);
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
    ctx.draw_rrect(rrect, Paint::fill(bg));

    if state == State::Hovered {
        let arr = 13.0;
        let left = RRect {
            right: rrect.left + arr,
            radius: CornerRadius {
                tr: 0.0,
                br: 0.0,
                .. rrect.radius
            },
            .. rrect
        };
        let right = RRect {
            left: rrect.right - arr,
            radius: CornerRadius {
                tl: 0.0,
                bl: 0.0,
                .. rrect.radius
            },
            .. rrect
        };

        let paint = Paint::fill(shade(theme.background, HOVER_SHADE));
        ctx.draw_rrect(left, paint);
        ctx.draw_rrect(right, paint);

        let left = left.rect().center();
        let right = right.rect().center();

        let paint = Paint::fill(0xFF_E6E6E6).stroke_width(1.0);

        let cc = vec2(1.5, 0.0);
        let aa = vec2(1.5, -3.0);
        let bb = vec2(1.5, 3.0);

        ctx.draw_lines(&[
            (left + aa).into(),
            (left - cc).into(),
            (left + bb).into(),
        ], paint);
        ctx.draw_lines(&[
            (right - aa).into(),
            (right + cc).into(),
            (right - bb).into(),
        ], paint);
    }

    ctx.draw_rrect(rrect.add(1.0), Paint::stroke(theme.outline).stroke_width(0.5));

    text_with_shadow( ctx, bounds.center(), theme.color, Align::CENTER|Align::MIDDLE, text);
}

fn text_with_shadow(
    ctx: &mut Canvas,
    position: Point,
    color: u32,
    text_align: Align,
    text: &str,
) {
    let text_style = TextStyle {
        font_size: 14.0,
        font_face: b"sans\0",
        font_blur: 0.0,
        color,
        text_align,
    };

    let text_style_back = TextStyle {
        font_size: 14.0,
        font_face: b"sans\0",
        font_blur: 1.0,
        color: 0xFF_000000,
        text_align,
    };

    ctx.text((position + vec2(0.0, 0.5)).into(), text, text_style_back);
    ctx.text(position.into(), text, text_style);
}