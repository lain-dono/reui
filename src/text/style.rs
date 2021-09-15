use super::font::Font;
use core::ops::Range;
pub use ttf_parser::Width as Stretch;

/// Specifies the weight of glyphs in the font, their degree of blackness or stroke thickness.
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Hash)]
pub struct Weight(pub u16);

impl Default for Weight {
    #[inline]
    fn default() -> Weight {
        Weight::NORMAL
    }
}

impl Weight {
    /// Thin weight (100), the thinnest value.
    pub const THIN: Weight = Weight(100);
    /// Extra light weight (200).
    pub const EXTRA_LIGHT: Weight = Weight(200);
    /// Light weight (300).
    pub const LIGHT: Weight = Weight(300);
    /// Normal (400).
    pub const NORMAL: Weight = Weight(400);
    /// Medium weight (500, higher than normal).
    pub const MEDIUM: Weight = Weight(500);
    /// Semibold weight (600).
    pub const SEMIBOLD: Weight = Weight(600);
    /// Bold weight (700).
    pub const BOLD: Weight = Weight(700);
    /// Extra-bold weight (800).
    pub const EXTRA_BOLD: Weight = Weight(800);
    /// Black weight (900), the thickest value.
    pub const BLACK: Weight = Weight(900);
}

/// Allows italic or oblique faces to be selected.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Style {
    /// A face that is neither italic not obliqued.
    Normal,
    /// A form that is generally cursive in nature.
    Italic,
    /// A typically-sloped version of the regular face.
    Oblique,
}

impl Default for Style {
    #[inline]
    fn default() -> Style {
        Style::Normal
    }
}

#[derive(Clone, Debug)]
pub struct TextStyle {
    pub color: crate::Color,
    pub font: Font,
    pub font_size: f32,
    pub decoration: TextDecoration,
    pub letter_spacing: f32,
    pub word_spacing: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct TextDecorationStyle {
    pub color: crate::Color,
}

#[derive(Clone, Default, Debug)]
pub struct TextDecoration {
    pub underline: Option<TextDecorationStyle>,
    pub overline: Option<TextDecorationStyle>,
    pub line_through: Option<TextDecorationStyle>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextAnchor {
    Start,
    Middle,
    End,
}

impl Default for TextAnchor {
    fn default() -> Self {
        Self::Start
    }
}

/// This holds and handles the start/end positions of discrete chunks of text
/// that use different styles (a 'run').
#[derive(Debug)]
pub struct StyledRuns {
    styles: Vec<TextStyle>,
    runs: Vec<(usize, Range<usize>)>,
}

impl StyledRuns {
    pub const fn empty() -> Self {
        Self {
            styles: vec![],
            runs: vec![],
        }
    }

    pub fn new(style: TextStyle) -> Self {
        Self {
            styles: vec![style],
            runs: vec![(0, 0..0)],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.runs.len()
    }

    pub fn add_style(&mut self, style: TextStyle) -> usize {
        let index = self.styles.len();
        self.styles.push(style);
        index
    }

    pub fn start_run(&mut self, style: usize, start: usize) {
        self.end_run_if_needed(start);
        self.runs.push((style, start..start));
    }

    pub fn end_run_if_needed(&mut self, end: usize) {
        if let Some((_, range)) = self.runs.last_mut() {
            if range.start == end {
                let _ = self.runs.pop(); // The run is empty. We can skip it.
            } else {
                range.end = end;
            }
        }
    }

    pub fn get_style(&self, index: usize) -> &TextStyle {
        &self.styles[index]
    }

    pub fn get_run(&self, index: usize) -> (&TextStyle, Range<usize>) {
        let run = &self.runs[index];
        (&self.styles[run.0], run.1.clone())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&TextStyle, Range<usize>)> + '_ {
        let styles = &self.styles;
        self.runs
            .iter()
            .map(move |(style, range)| (&styles[*style], range.clone()))
    }
}
