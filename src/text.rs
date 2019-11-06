#[derive(Default)]
pub struct Locale {
    pub language_code: String,
    pub script_code: String,
    pub country_code: String,
}

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

pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
    Start,
    End,
}

pub enum TextDirection {
    Ltr,
    Rtl,
}

pub enum FontWeight {
    W100,
    W200,
    W300,
    W400, // normal
    W500,
    W600,
    W700, // bold
    W800,
    W900,
}

pub enum FontStyle {
    Normal,
    Italic,
}

pub struct Paragraph {
}

pub struct ParagraphStyle {
    pub text_align: TextAlign,
    pub text_direction: TextDirection,
    pub max_lines: usize,
    pub font_family: String,
    pub font_size: f32,
    pub height: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub strut_style StrutStyle,
    pub ellipsis: String,
    pub locale: Locale,
}

pub struct ParagraphBuilder {
}

impl ParagraphBuilder {
    pub fn push_style(&mut self, text_style: TextStyle) {
        unimplemented!("push style")
    }

    pub fn pop_style(&mut self) {
        unimplemented!("pop style")
    }


    pub fn add_text(&mut self, text: &str) {
        unimplemented!("add text")
    }

    pub fn build(self) -> Paragraph {
    }
}