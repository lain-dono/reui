use crate::geom::Offset;

#[derive(Debug)]
pub struct ParagraphStyle {
    pub text_align: TextAlign,
    pub text_direction: TextDirection,
    pub max_lines: usize,

    pub font_family: String,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,

    pub height: f32,
    pub text_height_behavior: TextHeightBehavior,
    pub strut_style: StrutStyle,
    pub ellipsis: String,
    pub locale: Locale,
}

impl ParagraphStyle {
    pub fn text_style(&self) -> TextStyle {
        todo!()
    }
}

#[derive(Debug)]
pub struct LineMetrics {
    pub hard_break: bool,
    pub ascent: f32,
    pub descent: f32,
    pub unscaled_ascent: f32,
    pub height: f32,
    pub width: f32,
    pub left: f32,
    pub baseline: f32,
    pub line_number: usize,
}

#[derive(Debug)]
pub struct TextBox {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub direction: TextDirection,
}

#[derive(Debug)]
pub struct TextPosition {
    pub offset: usize,
    pub affinity: TextAffinity, // .downstream
}

#[derive(Debug)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub enum TextAffinity {
    Downstream,
    Upstream,
}

#[derive(Debug)]
pub enum TextDirection {
    Ltr,
    Rtl,
}

#[derive(Debug)]
pub enum BoxWidthStyle {
    Tight,
    Max,
}

#[derive(Debug)]
pub enum BoxHeightStyle {
    Tight,
    Max,
    Strut,
    IncludeLineSpacingTop,
    IncludeLineSpacingMiddle,
    IncludeLineSpacingBottom,
}

#[derive(Debug)]
pub enum TextAlign {
    Start,
    Center,
    Justify,
    End,
}

pub enum Align {
    Left,
    Center,
    Justify,
    Right,
}

impl TextAlign {
    pub fn effective_align(self, direction: TextDirection) -> Align {
        match (self, direction) {
            (Self::Start, TextDirection::Ltr) | (Self::End, TextDirection::Rtl) => Align::Left,
            (Self::Start, TextDirection::Rtl) | (Self::End, TextDirection::Ltr) => Align::Right,
            (Self::Center, _) => Align::Center,
            (Self::Justify, _) => Align::Justify,
        }
    }
}

#[derive(Debug)]
pub struct TextHeightBehavior {
    pub apply_to_first_ascent: bool,
    pub apply_to_last_descent: bool,
}

impl Default for TextHeightBehavior {
    fn default() -> Self {
        Self {
            apply_to_first_ascent: true,
            apply_to_last_descent: true,
        }
    }
}

#[derive(Debug)]
pub enum FontWeight {
    W100,
    W200,
    W300,
    W400,
    W500,
    W600,
    W700,
    W800,
    W900,
}

impl FontWeight {
    pub const fn normal() -> Self {
        Self::W400
    }

    pub const fn bold() -> Self {
        Self::W700
    }
}

#[derive(Debug)]
pub enum FontStyle {
    Normal,
    Italic,
}

#[derive(Debug)]
pub struct StrutStyle {
    pub font_family: String,
    pub font_family_fallback: Vec<String>,
    pub font_size: f32,
    pub height: f32,
    pub leading: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub force_strut_height: bool,
}

#[derive(Debug)]
pub struct Locale {
    pub language_code: String,
    pub country_code: String,
    pub script_code: String,
}

#[derive(Debug)]
pub enum TextBaseline {
    Alphabetic,
    Ideographic,
}

#[derive(Debug)]
pub enum PlaceholderAlignment {
    Baseline,
    AboveBaseline,
    BelowBaseline,
    Middle,
    Top,
}

#[derive(Debug, Default)]
pub struct TextStyle {
    /*
pub color: Color,

pub decoration: TextStyleDecoration,

pub font_weight: FontWeight,
pub font_style: FontStyle,
pub font_family: String,
pub font_family_fallback: Vec<String>,
pub font_size: f32,
pub font_features: Vec<FontFeature>,

pub text_baseline: TextBaseline,

pub letter_spacing: f32,
pub word_spacing: f32,
pub height: f32,

pub locale: Locale,
pub background: Paint,
pub foreground: Paint,
pub shadows: Vec<Shadow>,
*/}

#[derive(Debug)]
pub struct Color;

#[derive(Debug)]
pub struct Paint;

#[derive(Debug)]
pub struct TextStyleDecoration {
    pub decoration: TextDecoration,
    pub color: Color,
    pub style: TextDecorationStyle,
    pub thickness: f32,
}

bitflags::bitflags! {
    pub struct TextDecoration: u8 {
        const UNDERLINE = 1;
        const OVERLINE = 2;
        const LINE_THROUGH = 4;
    }
}

#[derive(Debug)]
pub enum TextDecorationStyle {
    Solid,
    Double,
    Dotted,
    Dashed,
    Wavy,
}

#[derive(Debug)]
pub struct Shadow {
    pub color: Color,
    pub offset: Offset,
    pub blur_radius: f32,
}

pub type FontFeature = ttf_parser::Variation;
