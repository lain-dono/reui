//pub mod stuff;
//pub mod textwrap;

pub mod text;

//mod atlas;
mod context;
mod error;
mod face;
mod paragraph;
mod shaper;
mod style;

pub use self::{
    context::TextContext,
    error::Error,
    paragraph::{Paragraph, ParagraphStyle},
    style::{Align, Baseline},
};
use self::{face::FaceId, shaper::ShapedGlyph};
use crate::geom::{Offset, Size};

pub use fontdb::{
    Database as FontDatabase,
    {Family as FontFamily, Stretch as FontStretch, Style as FontStyle, Weight as FontWeight},
};

pub struct Paint {
    pub color: crate::Color,
    pub fonts: [Option<FaceId>; 8],
    pub font_size: f32,
    pub line_width: f32,
    pub letter_spacing: f32,
    pub text_baseline: Baseline,
    pub text_align: Align,
}

/// Result of a shaping run.
#[derive(Clone, Default, Debug)]
pub struct TextLayout {
    pub glyphs: Vec<ShapedGlyph>,
    pub position: Offset,
    pub size: Size,
    pub final_byte_index: usize,
}

impl TextLayout {
    pub fn scale(&mut self, scale: f32) {
        self.position *= scale;
        self.size *= scale;

        for glyph in &mut self.glyphs {
            glyph.position *= scale;
            glyph.size *= scale;
        }
    }

    pub fn translate(&mut self, offset: Offset) {
        self.position += offset;

        for glyph in &mut self.glyphs {
            glyph.position += offset;
        }
    }
}
