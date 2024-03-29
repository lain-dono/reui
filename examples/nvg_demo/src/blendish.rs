use reui::{Canvas, Color, Offset, Rect, Rounding, Stroke};

const HOVER_SHADE: i32 = 15;

fn offset_color(
    Color {
        red,
        green,
        blue,
        alpha,
    }: Color,
    delta: i32,
) -> Color {
    if delta != 0 {
        let offset = delta as f32 / 255.0;
        Color {
            red: (red + offset).max(0.0).min(1.0),
            green: (green + offset).max(0.0).min(1.0),
            blue: (blue + offset).max(0.0).min(1.0),
            alpha,
        }
    } else {
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }
}

fn shade(color: Color, shade: i32) -> Color {
    offset_color(color, shade)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Normal,
    Hovered,
    Active,
}

#[allow(dead_code)]
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

/*
pub struct PanelTheme {
    pub header: u32,
    pub background: u32,
    pub sub_background: u32,
}
*/

pub struct WindowTheme {
    pub background: u32,
}

pub fn run(ctx: &mut Canvas, bounds: Rect) {
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

    draw_window(ctx, bounds, &win_theme);

    let num = Rect::from_size(160.0, 18.5).translate(bounds.min + Offset::new(10.0, 10.0));

    draw_num(ctx, num, &num_theme, State::Normal, Gropped::StartVertical);
    let num = num.translate(Offset::new(0.0, 19.0));
    draw_num(ctx, num, &num_theme, State::Hovered, Gropped::Middle);
    let num = num.translate(Offset::new(0.0, 19.0));
    draw_num(ctx, num, &num_theme, State::Active, Gropped::EndVertical);

    let num = num.translate(Offset::new(0.0, 30.0));
    draw_num(ctx, num, &num_theme, State::Normal, Gropped::None);
    let num = num.translate(Offset::new(0.0, 30.0));

    let opt = Rect::from_size(13.0, 13.0).translate(num.min);

    draw_option(ctx, opt, &opt_theme, State::Normal);
    let opt = opt.translate(Offset::new(0.0, 20.0));
    draw_option(ctx, opt, &opt_theme, State::Hovered);
    let opt = opt.translate(Offset::new(0.0, 20.0));
    draw_option(ctx, opt, &opt_theme, State::Active);
}

pub fn draw_window(ctx: &mut Canvas, bounds: Rect, theme: &WindowTheme) {
    ctx.fill_rect(bounds, Color::bgra(theme.background));

    let rect = bounds.deflate(3.0);
    let radius = Rounding::same(2.5);

    let mut left_scroll = rect;
    left_scroll.min.x = rect.max.x - 5.0;
    left_scroll.max.y = rect.max.y - 50.0;

    ctx.fill_rrect(left_scroll, radius, Color::bgra(0xFF_676767));
    //ctx.draw_rrect(left_scroll.add(1.0), Paint::stroke(0xFF_424242));
    //ctx.draw_rrect(left_scroll, Paint::stroke(0xFF_373737).stroke_width(0.5));
}

pub fn draw_option(ctx: &mut Canvas, bounds: Rect, theme: &WidgetTheme, state: State) {
    let bg = match state {
        State::Normal => Color::bgra(theme.background),
        State::Hovered => shade(Color::bgra(theme.background), HOVER_SHADE),
        State::Active => Color::bgra(theme.active),
    };

    let radius = Rounding::same(theme.radius);
    let a = Offset::new(2.5, 6.0);
    let b = Offset::new(5.5, 9.0);
    let c = Offset::new(10.5, 3.5);

    ctx.fill_rrect(bounds, radius, bg);
    ctx.stroke_rrect(
        bounds.inflate(0.5),
        radius,
        Color::bgra(theme.outline),
        Stroke::width(1.0),
    );

    if state == State::Active {
        ctx.stroke_polyline(
            &[bounds.min + a, bounds.min + b, bounds.min + c],
            false,
            Color::bgra(0xFF_E6E6E6),
            Stroke::width(2.0),
        )
    }
}

pub fn draw_num(
    ctx: &mut Canvas,
    bounds: Rect,
    theme: &WidgetTheme,
    state: State,
    gropped: Gropped,
) {
    let bg = match state {
        State::Normal => Color::bgra(theme.background),
        State::Hovered => shade(Color::bgra(theme.background), HOVER_SHADE * 2),
        State::Active => Color::bgra(theme.active),
    };

    let mut radius = Rounding::same(theme.radius);
    match gropped {
        Gropped::None => (),
        Gropped::StartVertical => {
            radius.se = 0.0;
            radius.sw = 0.0;
        }
        Gropped::StartHorizontal => {
            radius.sw = 0.0;
            radius.ne = 0.0;
        }
        Gropped::Middle => {
            radius.se = 0.0;
            radius.sw = 0.0;
            radius.nw = 0.0;
            radius.ne = 0.0;
        }
        Gropped::EndVertical => {
            radius.nw = 0.0;
            radius.ne = 0.0;
        }
        Gropped::EndHorizontal => {
            radius.nw = 0.0;
            radius.se = 0.0;
        }
    };

    let paint = Color::bgra(theme.outline);
    ctx.stroke_rrect(bounds.inflate(0.1), radius, paint, Stroke::width(1.0));
    ctx.fill_rrect(bounds, radius, bg);

    let arr = 13.0;

    let (mut left, mut left_radius) = (bounds, radius);
    left.max.x = bounds.min.x + arr;
    left_radius.ne = 0.0;
    left_radius.sw = 0.0;

    let (mut right, mut right_radius) = (bounds, radius);
    right.min.x = bounds.max.x - arr;
    right_radius.nw = 0.0;
    right_radius.se = 0.0;

    if state == State::Hovered {
        let paint = shade(Color::bgra(theme.background), HOVER_SHADE);
        ctx.fill_rrect(left, left_radius, paint);
        ctx.fill_rrect(right, right_radius, paint);
    }

    let l_arrow = left.center();
    let r_arrow = right.center();

    let paint = Color::bgra(0xFF_E6E6E6);

    let a = Offset::new(1.5, -3.0);
    let b = Offset::new(1.5, 0.0);
    let c = Offset::new(1.5, 3.0);

    ctx.stroke_polyline(
        &[l_arrow + a, l_arrow - b, l_arrow + c],
        false,
        paint,
        Stroke::width(1.0),
    );
    ctx.stroke_polyline(
        &[r_arrow - a, r_arrow + b, r_arrow - c],
        false,
        paint,
        Stroke::width(1.0),
    );
}
