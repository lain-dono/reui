pub mod builder;
pub mod placeholder;
pub mod style;

pub use self::{
    placeholder::{Placeholder, PlaceholderAlignment},
    style::{ParagraphStyle, TextBaseline, TextStyle},
};

pub struct ParagraphConstraints {
    pub width: f32,
}

#[derive(Default)]
pub struct Paragraph {}

/*
impl Paragraph {
    /// The distance from the top of the paragraph to the alphabetic baseline of the first line, in logical pixels.
    pub fn alphabetic_baseline(&self) -> f32 {
        todo!()
    }

    /// The distance from the top of the paragraph to the ideographic baseline of the first line, in logical pixels.
    pub fn ideographic_baseline(&self) -> f32 {
        todo!()
    }

    pub fn did_exceed_max_lines(&self) -> bool {
        todo!()
    }

    pub fn longest_line(&self) -> f32 {
        todo!()
    }

    pub fn width(&self) -> f32 {
        todo!()
    }

    pub fn height(&self) -> f32 {
        todo!()
    }

    pub fn min_intrinsic_width(&self) -> f32 {
        todo!()
    }

    pub fn max_intrinsic_width(&self) -> f32 {
        todo!()
    }
}

impl Paragraph {
    pub fn compute_line_metrics(&self) -> Vec<LineMetrics> {
        todo!()
    }

    pub fn boxes_for_placeholders(&self) -> Vec<TextBox> {
        todo!()
    }

    pub fn boxes_for_range(
        &self,
        start: usize,
        end: usize,
        width: BoxWidthStyle,
        height: BoxHeightStyle,
    ) -> Vec<TextBox> {
        todo!()
    }

    pub fn line_boundary(&self, position: TextPosition) -> TextRange {
        todo!()
    }
    pub fn position_for_offset(&self, offset: Offset) -> TextPosition {
        todo!()
    }
    pub fn word_boundary(&self, position: TextPosition) -> TextRange {
        todo!()
    }

    pub fn layout(&mut self, constraints: ParagraphConstraints) {
        todo!()
    }
}
*/
