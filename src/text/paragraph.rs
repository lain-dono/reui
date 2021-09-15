use super::font_db::Database;
use super::shaper::{shape_text, GlyphClusters, OutlinedCluster};
use super::style::{StyledRuns, TextAnchor, TextStyle};
use std::ops::Range;

/// Spans do not overlap.
#[derive(Clone)]
struct Span {
    range: Range<usize>,
    style: TextStyle,
}

/// A text chunk.
///
/// Text alignment and BIDI reordering can be done only inside a text chunk.
pub struct Paragraph {
    text: String,
    anchor: TextAnchor,

    style_stack: Vec<usize>,
    runs: StyledRuns,
    style_index: usize,
}

impl Paragraph {
    pub fn new(anchor: TextAnchor, style: TextStyle) -> Self {
        Self {
            text: String::new(),
            anchor,
            //placeholders: Vec::new(),
            //replacement_chars: HashSet::default(),
            style_stack: vec![],
            runs: StyledRuns::new(style),
            style_index: 0,
        }
    }

    pub fn push_style(&mut self, style: TextStyle) {
        let style_index = self.runs.add_style(style);
        self.style_stack.push(style_index);
        self.runs.start_run(style_index, self.text.len());
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

    #[inline]
    fn style_index(&self) -> usize {
        self.style_stack.last().copied().unwrap_or(self.style_index)
    }

    /// # Panics
    pub fn draw(&mut self, db: &Database, width: f32, canvas: &mut crate::Canvas) {
        self.runs.end_run_if_needed(self.text.len());

        let mut clusters = self.outline(db);

        self.apply_letter_spacing(&mut clusters);
        self.apply_word_spacing(&mut clusters);

        let mut paint = crate::Paint::fill_even_odd(crate::Color::BLACK);

        let text_width = Self::text_width(&clusters);
        /*
        let mut x = match self.anchor {
            TextAnchor::Start => 0.0, // Nothing.
            TextAnchor::Middle => -text_width / 2.0,
            TextAnchor::End => -text_width,
        };
        */

        let mut tx = 0.0;
        let mut ty = 0.0;

        let mut breaks = unicode_linebreak::linebreaks(&self.text);
        let mut last_break = breaks.next().unwrap();

        let mut spans = self.runs.iter();
        let mut span = spans.next().unwrap();

        for cluster in &clusters {
            if !span.1.contains(&cluster.start) {
                span = spans.next().unwrap();
            }

            //println!("{:?}", &self.text[cluster.start..cluster.end]);

            let transform = crate::Transform::translation(tx, ty);
            paint.color = span.0.color;
            canvas.draw_path(cluster.path.transform_iter(transform), paint);
            tx += cluster.advance;

            if last_break.0 <= cluster.end {
                match last_break.1 {
                    unicode_linebreak::BreakOpportunity::Allowed => {
                        if tx + cluster.advance >= width {
                            tx = 0.0;
                            ty += cluster.height();
                        }
                    }
                    unicode_linebreak::BreakOpportunity::Mandatory => {
                        tx = 0.0;
                        ty += cluster.height();
                    }
                }
                //println!("{:?}", last_break);
                last_break = match breaks.next() {
                    Some(br) => br,
                    None => break, // eof
                };
            }
        }
    }

    fn span_at(&self, offset: usize) -> Option<&TextStyle> {
        self.runs.iter().find_map(|(style, range)| {
            if range.contains(&offset) {
                Some(style)
            } else {
                None
            }
        })
    }

    /// Converts a text chunk into a list of outlined clusters.
    ///
    /// This function will do the BIDI reordering, text shaping and glyphs outlining,
    /// but not the text layouting. So all clusters are in the 0x0 position.
    fn outline(&self, db: &Database) -> Vec<OutlinedCluster> {
        let mut glyphs = Vec::new();
        for (style, range) in self.runs.iter() {
            let tmp_glyphs = shape_text(&self.text, style.font, db);

            // Do nothing with the first run.
            if glyphs.is_empty() {
                glyphs = tmp_glyphs;
                continue;
            }

            // We assume, that shaping with an any font will produce the same amount of glyphs.
            // Otherwise an error.
            if glyphs.len() != tmp_glyphs.len() {
                log::warn!("Text layouting failed.");
                return Vec::new();
            }

            // Copy span's glyphs.
            for (i, glyph) in tmp_glyphs.iter().enumerate() {
                if range.contains(&glyph.offset) {
                    glyphs[i] = glyph.clone();
                }
            }
        }

        // Convert glyphs to clusters.
        let mut clusters = Vec::new();
        for (range, byte_idx) in GlyphClusters::new(&glyphs) {
            if let Some(style) = self.span_at(byte_idx) {
                clusters.push(OutlinedCluster::outline(
                    &glyphs[range],
                    &self.text,
                    style.font_size,
                    db,
                ));
            }
        }

        clusters
    }

    fn text_width(clusters: &[OutlinedCluster]) -> f32 {
        clusters.iter().fold(0.0, |w, cluster| w + cluster.advance)
    }

    /// Applies the `letter-spacing` property to a text chunk clusters.
    ///
    /// [In the CSS spec](https://www.w3.org/TR/css-text-3/#letter-spacing-property).
    fn apply_letter_spacing(&self, clusters: &mut [OutlinedCluster]) {
        use unicode_script::UnicodeScript;

        /// Checks that selected script supports letter spacing.
        ///
        /// [In the CSS spec](https://www.w3.org/TR/css-text-3/#cursive-tracking).
        ///
        /// The list itself is from: <https://github.com/harfbuzz/harfbuzz/issues/64>
        fn script_supports_letter_spacing(script: unicode_script::Script) -> bool {
            use unicode_script::Script;

            !matches!(
                script,
                Script::Arabic
                    | Script::Syriac
                    | Script::Nko
                    | Script::Manichaean
                    | Script::Psalter_Pahlavi
                    | Script::Mandaic
                    | Script::Mongolian
                    | Script::Phags_Pa
                    | Script::Devanagari
                    | Script::Bengali
                    | Script::Gurmukhi
                    | Script::Modi
                    | Script::Sharada
                    | Script::Syloti_Nagri
                    | Script::Tirhuta
                    | Script::Ogham
            )
        }

        // At least one span should have a non-zero spacing.
        if !self
            .runs
            .iter()
            .any(|(style, _)| approx::ulps_ne!(style.letter_spacing, 0.0))
        {
            return;
        }

        let num_clusters = clusters.len();
        for (i, cluster) in clusters.iter_mut().enumerate() {
            // Spacing must be applied only to characters that belongs to the script
            // that supports spacing.
            // We are checking only the first code point, since it should be enough.
            let script = cluster.codepoint.script();
            if script_supports_letter_spacing(script) {
                if let Some(style) = self.span_at(cluster.start) {
                    // A space after the last cluster should be ignored,
                    // since it affects the bbox and text alignment.
                    if i != num_clusters - 1 {
                        cluster.advance += style.letter_spacing;
                    }

                    // If the cluster advance became negative - clear it.
                    // This is an UB so we can do whatever we want, and we mimic Chrome's behavior.
                    if cluster.advance < 0.0 {
                        cluster.width = 0.0;
                        cluster.advance = 0.0;
                        cluster.path.clear();
                    }
                }
            }
        }
    }

    /// Applies the `word-spacing` property to a text chunk clusters.
    ///
    /// [In the CSS spec](https://www.w3.org/TR/css-text-3/#propdef-word-spacing).
    fn apply_word_spacing(&self, clusters: &mut [OutlinedCluster]) {
        /// Checks that the selected character is a word separator.
        ///
        /// According to: <https://www.w3.org/TR/css-text-3/#word-separator>
        fn is_word_separator_characters(c: char) -> bool {
            matches!(
                c as u32,
                0x0020 | 0x00A0 | 0x1361 | 0x01_0100 | 0x01_0101 | 0x01_039F | 0x01_091F
            )
        }

        // At least one span should have a non-zero spacing.
        if !self
            .runs
            .iter()
            .any(|(style, _)| approx::ulps_ne!(style.word_spacing, 0.0))
        {
            return;
        }

        for cluster in clusters {
            if is_word_separator_characters(cluster.codepoint) {
                if let Some(style) = self.span_at(cluster.start) {
                    // Technically, word spacing 'should be applied half on each
                    // side of the character', but it doesn't affect us in any way,
                    // so we are ignoring this.
                    cluster.advance += style.word_spacing;

                    // After word spacing, `advance` can be negative.
                }
            }
        }
    }
}
