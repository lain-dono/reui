use super::{Paragraph, ParagraphStyle, Placeholder, TextStyle};
use std::{collections::HashSet, ops::Range};

/// Constant with the unicode codepoint for the "Object replacement character".
/// Used as a stand-in character for Placeholder boxes.
const OBJ_REPLACEMENT_CHAR: char = 0xFFFC as char;

/// Constant with the unicode codepoint for the "Replacement character". This is
/// the character that commonly renders as a black diamond with a white question
/// mark. Used to replace non-placeholder instances of 0xFFFC in the text buffer.
const REPLACEMENT_CHAR: char = 0xFFFD as char;

/// This holds and handles the start/end positions of discrete chunks of text
/// that use different styles (a 'run').
#[derive(Debug)]
struct StyledRuns {
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
        if let Some(run) = self.runs.last_mut() {
            if run.1.is_empty() {
                let _ = self.runs.pop(); // The run is empty. We can skip it.
            } else {
                run.1.end = end;
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
}

pub struct ParagraphBuilder {
    text: String,

    // A vector of PlaceholderRuns, which detail the sizes, positioning and break
    // behavior of the empty spaces to leave. Each placeholder span corresponds to
    // a 0xFFFC (object replacement character) in text_, which indicates the
    // position in the text where the placeholder will occur. There should be an
    // equal number of 0xFFFC characters and elements in this vector.
    placeholders: Vec<Placeholder>,

    // The indexes of the obj replacement characters added through
    // ParagraphBuilder::addPlaceholder().
    replacement_chars: HashSet<usize>,

    style_stack: Vec<usize>,
    runs: StyledRuns,
    style: ParagraphStyle,
    style_index: usize,
}

impl ParagraphBuilder {
    pub fn new(style: ParagraphStyle) -> Self {
        Self {
            text: String::new(),
            placeholders: Vec::new(),
            replacement_chars: HashSet::default(),
            style_stack: Vec::new(),
            runs: StyledRuns::new(style.text_style()),
            style,
            style_index: 0,
        }
    }

    pub fn push_style(&mut self, style: TextStyle) {
        self.style_index = self.runs.add_style(style);
        self.style_stack.push(self.style_index);
        self.runs.start_run(self.style_index, self.text.len());
    }

    pub fn pop_style(&mut self) {
        if self.style_stack.pop().is_some() {
            self.runs.start_run(self.style_index(), self.text.len());
        }
    }

    pub fn peek_style(&self) -> &TextStyle {
        self.runs.get_style(self.style_index())
    }

    pub fn add_text(&mut self, text: &str) {
        self.text.push_str(text);
    }

    pub fn add_placeholder(&mut self, span: Placeholder) {
        self.replacement_chars.insert(self.text.len());
        self.runs.start_run(self.style_index(), self.text.len());
        self.text.push(OBJ_REPLACEMENT_CHAR);
        self.runs.start_run(self.style_index(), self.text.len());
        self.placeholders.push(span);
    }

    pub fn build(&mut self) -> Paragraph {
        self.runs.end_run_if_needed(self.text.len());

        let paragraph = Paragraph::default();

        paragraph.set_text(self.text, self.runs);
        paragraph.set_inline_placeholders(self.placeholders, self.replacement_chars);
        paragraph.set_paragraph_style(self.style);
        //paragraph.set_font_collection(font_collection);

        self.style_index = self.runs.add_style(self.style.text_style());
        self.runs.start_run(self.style_index, self.text.len());

        paragraph
    }

    #[inline]
    fn style_index(&self) -> usize {
        self.style_stack.last().copied().unwrap_or(self.style_index)
    }
}
