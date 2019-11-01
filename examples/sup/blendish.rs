use oni2d::canvas::*;

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

pub fn run(ctx: &mut Canvas, bounds: Rect) {
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
        bounds.origin + Vector::new(10.0, 10.0),
        euclid::Size2D::new(160.0, 18.0),
    );

    draw_num(ctx, num, &num_theme, State::Normal, "1234");
    num.origin += Vector::new(0.0, 20.0);
    draw_num(ctx, num, &num_theme, State::Hovered, "1234");
    num.origin += Vector::new(0.0, 20.0);
    draw_num(ctx, num, &num_theme, State::Active, "1234");
    num.origin += Vector::new(0.0, 20.0);
    num.origin += Vector::new(0.0, 20.0);

    let mut opt = Rect::new(
        num.origin,
        euclid::Size2D::new(13.0, 13.0),
    );
    draw_opt(ctx, opt, &opt_theme, State::Normal);
    opt.origin += Vector::new(0.0, 20.0);
    draw_opt(ctx, opt, &opt_theme, State::Hovered);
    opt.origin += Vector::new(0.0, 20.0);
    draw_opt(ctx, opt, &opt_theme, State::Active);
    opt.origin += Vector::new(0.0, 20.0);
}

pub fn draw_window(ctx: &mut Canvas, bounds: Rect, theme: &WindowTheme) {
    ctx.draw_rect(bounds, Paint::fill(theme.background));

    let rect = bounds.inner_rect(euclid::SideOffsets2D::new_all_same(3.0));
    let rrect = RRect::new(rect.origin.into(), rect.size.into(), 2.5);

    let left_scroll = RRect {
        left: rrect.right - 5.0,
        bottom: rrect.bottom - 50.0,
        .. rrect
    };

    ctx.draw_rrect(left_scroll, Paint::fill(0xFF_676767));
    ctx.draw_rrect(left_scroll.add(1.0), Paint::stroke(0xFF_424242));
}

pub fn draw_opt(
    ctx: &mut Canvas,
    bounds: Rect,
    theme: &WidgetTheme,
    state: State,
) {
    let bg = match state {
        State::Normal => theme.background,
        State::Hovered => shade(theme.background, HOVER_SHADE),
        State::Active => theme.active,
    };
    let rrect = RRect::new(bounds.origin.into(), bounds.size.into(), theme.radius);
    ctx.draw_rrect(rrect, Paint::fill(bg));
    let a = oni2d::vec2(2.5, 6.0);
    let b = oni2d::vec2(5.5, 9.0);
    let c = oni2d::vec2(10.5, 3.5);

    if state == State::Active {
        ctx.draw_lines(&[
            (bounds.origin + a).into(),
            (bounds.origin + b).into(),
            (bounds.origin + c).into(),
        ], Paint::fill(0xFF_E6E6E6).stroke_width(2.0))
    }

    ctx.draw_rrect(rrect.add(1.0), Paint::stroke(theme.outline).stroke_width(0.5));
}

pub fn draw_num(
    ctx: &mut Canvas,
    bounds: Rect,
    theme: &WidgetTheme,
    state: State,
    text: &str,
) {
    let bg = match state {
        State::Normal => theme.background,
        State::Hovered => shade(theme.background, HOVER_SHADE * 2),
        State::Active => theme.active,
    };

    let text_style = TextStyle {
        font_size: 14.0,
        font_face: b"sans\0",
        font_blur: 0.0,
        color: theme.color,
        text_align: Align::CENTER|Align::MIDDLE,
    };

    let text_style_back = TextStyle {
        font_size: 14.0,
        font_face: b"sans\0",
        font_blur: 1.0,
        color: 0xFF_000000,
        text_align: Align::CENTER|Align::MIDDLE,
    };

    let rrect = RRect::new(bounds.origin.into(), bounds.size.into(), theme.radius);
    ctx.draw_rrect(rrect, Paint::fill(bg));

    if state == State::Hovered {
        let arr = 13.0;
        let left = RRect {
            right: rrect.left + arr,
            top_right: 0.0,
            bottom_right: 0.0,
            .. rrect
        };
        let right = RRect {
            left: rrect.right - arr,
            top_left: 0.0,
            bottom_left: 0.0,
            .. rrect
        };

        let paint = Paint::fill(shade(theme.background, HOVER_SHADE));
        ctx.draw_rrect(left, paint);
        ctx.draw_rrect(right, paint);

        let left = left.rect().center();
        let right = right.rect().center();

        let paint = Paint::fill(0xFF_E6E6E6).stroke_width(1.0);

        use euclid::vec2;

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

    ctx.text(bounds.center().into(), text, text_style_back);
    ctx.text(bounds.center().into(), text, text_style);
}