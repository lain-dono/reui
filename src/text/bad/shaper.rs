use {
    crate::geom::{Offset, Size},
    crate::text::{
        face::{FaceCollection, FaceId},
        {Align, Baseline, Error, Paint, TextLayout},
    },
    fnv::{FnvBuildHasher, FnvHasher},
    lru::LruCache,
    std::hash::{Hash, Hasher},
    ttf_parser::GlyphId,
    unicode_bidi::BidiInfo,
    unicode_segmentation::UnicodeSegmentation,
};

#[derive(Copy, Clone, Debug)]
pub struct ShapedGlyph {
    pub position: Offset,

    pub c: char,
    pub byte_index: usize,
    pub face_id: FaceId,
    pub glyph: u32,

    pub size: Size,
    pub advance: Offset,
    pub offset: Offset,
    pub bearing: Offset,
}

#[derive(Clone, Debug, Default)]
pub struct ShapedWord {
    pub glyphs: Vec<ShapedGlyph>,
    pub width: f32,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ShapingId {
    size: u32,
    word_hash: u64,
    font_ids: [Option<FaceId>; 8],
}

impl ShapingId {
    pub fn new(paint: &Paint, word: &str, max_width: Option<f32>) -> Self {
        let mut hasher = FnvHasher::default();
        word.hash(&mut hasher);
        max_width.map(|w| w.trunc() as i32).hash(&mut hasher);

        Self {
            size: (paint.font_size * 10.0).trunc() as u32,
            word_hash: hasher.finish(),
            font_ids: paint.fonts,
        }
    }
}

pub struct Shaper {
    // shaping run cache
    pub run_cache: LruCache<ShapingId, TextLayout, FnvBuildHasher>,
    // shaped words cache
    pub words_cache: LruCache<ShapingId, Result<ShapedWord, Error>, FnvBuildHasher>,
}

impl Default for Shaper {
    fn default() -> Self {
        Self {
            run_cache: LruCache::with_hasher(Self::LRU_CACHE_CAPACITY, FnvBuildHasher::default()),
            words_cache: LruCache::with_hasher(Self::LRU_CACHE_CAPACITY, FnvBuildHasher::default()),
        }
    }
}

impl Shaper {
    const LRU_CACHE_CAPACITY: usize = 1000;

    /// # Errors
    pub fn shape(
        &mut self,
        paint: &Paint,
        text: &str,
        max_width: Option<f32>,
        fonts: &mut FaceCollection,
    ) -> Result<TextLayout, Error> {
        let id = ShapingId::new(paint, text, max_width);

        if !self.run_cache.contains(&id) {
            let layout = self.shape_run(paint, text, max_width, fonts);
            self.run_cache.put(id, layout);
        }

        self.run_cache
            .get(&id)
            .cloned()
            .ok_or(Error::Unknown)
            .and_then(|layout| Self::layout(paint, layout, fonts))
    }

    fn shape_run(
        &mut self,
        paint: &Paint,
        text: &str,
        max_width: Option<f32>,
        fonts: &mut FaceCollection,
    ) -> TextLayout {
        let mut text_layout = TextLayout {
            glyphs: Vec::with_capacity(text.len()),
            ..TextLayout::default()
        };

        let bidi_info = BidiInfo::new(text, Some(unicode_bidi::Level::ltr()));

        for (_, paragraph) in bidi_info.paragraphs.iter().enumerate() {
            let line = paragraph.range.clone();

            let (levels, runs) = bidi_info.visual_runs(paragraph, line);

            for run in runs {
                let sub_text = &text[run.clone()];

                if sub_text.is_empty() {
                    continue;
                }

                let hb_direction = if levels[run.start].is_rtl() {
                    rustybuzz::Direction::RightToLeft
                } else {
                    rustybuzz::Direction::LeftToRight
                };

                let mut words = Vec::new();
                let mut word_break_reached = false;
                let mut byte_index = run.start;

                for word in sub_text.split_word_bounds() {
                    let id = ShapingId::new(paint, word, max_width);

                    if !self.words_cache.contains(&id) {
                        let word = Self::shape_word(word, hb_direction, paint, fonts);
                        self.words_cache.put(id, word);
                    }

                    if let Some(Ok(word)) = self.words_cache.get(&id) {
                        let mut word = word.clone();

                        if let Some(max_width) = max_width {
                            if text_layout.size.x + word.width >= max_width {
                                word_break_reached = true;
                                break;
                            }
                        }

                        text_layout.size.x += word.width;

                        for glyph in &mut word.glyphs {
                            glyph.byte_index += byte_index;
                            debug_assert!(text.get(glyph.byte_index..).is_some());
                        }

                        words.push(word);
                    }

                    byte_index += word.len();
                }

                if levels[run.start].is_rtl() {
                    words.reverse();
                }

                for word in words {
                    text_layout.glyphs.extend_from_slice(&word.glyphs);
                }

                text_layout.final_byte_index = byte_index;

                if word_break_reached {
                    break;
                }
            }
        }

        text_layout
    }

    fn shape_word(
        word: &str,
        hb_direction: rustybuzz::Direction,
        paint: &Paint,
        fonts: &mut FaceCollection,
    ) -> Result<ShapedWord, Error> {
        // find_face will call the closure with each font face matching the provided style
        // until a font face capable of shaping the word is found
        fonts
            .find(word, &paint.fonts, |face_id, face| {
                // Call harfbuzz
                let output = {
                    let mut buffer = rustybuzz::UnicodeBuffer::new();
                    buffer.push_str(word);
                    buffer.set_direction(hb_direction);
                    rustybuzz::shape(face.raw(), &[], buffer)
                };

                let positions = output.glyph_positions();

                positions
                    .iter()
                    .zip(output.glyph_infos().iter().zip(word.chars()))
                    .fold(
                        (
                            false,
                            ShapedWord {
                                glyphs: Vec::with_capacity(positions.len()),
                                width: 0.0,
                            },
                        ),
                        |(has_missing, mut shaped_word), (position, (info, c))| {
                            let scale = face.scale(paint.font_size);

                            let mut g = ShapedGlyph {
                                position: Offset::zero(),
                                c,
                                byte_index: info.cluster as usize,
                                face_id,
                                glyph: info.glyph_id,

                                advance: Offset::new(
                                    position.x_advance as f32 * scale,
                                    position.y_advance as f32 * scale,
                                ),
                                offset: Offset::new(
                                    position.x_offset as f32 * scale,
                                    position.y_offset as f32 * scale,
                                ),

                                size: Size::zero(),
                                bearing: Offset::zero(),
                            };

                            if let Some(glyph) = face.glyph(GlyphId(info.glyph_id as u16)) {
                                let metrics = glyph.metrics().scale(scale);
                                g.size = metrics.size;
                                g.bearing = metrics.bearing;
                            }

                            shaped_word.width += g.advance.x + paint.letter_spacing;
                            shaped_word.glyphs.push(g);

                            (has_missing || info.glyph_id == 0, shaped_word)
                        },
                    )
            })
            .ok_or(Error::NotFound)
    }

    /// Calculates the x,y coordinates for each glyph based on their advances.
    /// Calculates total width and height of the shaped text run
    fn layout(
        paint: &Paint,
        mut layout: TextLayout,
        fonts: &mut FaceCollection,
    ) -> Result<TextLayout, Error> {
        let mut position = Offset::zero();

        // Horizontal alignment
        match paint.text_align {
            Align::Left => (),
            Align::Center => position.x -= layout.size.x / 2.0,
            Align::Right => position.x -= layout.size.x,
        }

        layout.position.x = position.x;

        let mut range = position.y..position.y;

        for glyph in &mut layout.glyphs {
            let face = fonts.get(glyph.face_id).ok_or(Error::NotFound)?;

            // Baseline alignment
            let scale = paint.font_size;

            let alignment_offset_y = match paint.text_baseline {
                Baseline::Top => face.ascender(scale),
                Baseline::Middle => (face.ascender(scale) + face.descender(scale)) / 2.0,
                Baseline::Alphabetic => 0.0,
                Baseline::Bottom => face.descender(scale),
            };

            glyph.position.x = position.x + glyph.offset.x + glyph.bearing.x;
            glyph.position.y =
                (position.y + alignment_offset_y).round() + glyph.offset.y - glyph.bearing.y;

            range.start = range.start.min(glyph.position.y);
            range.end = range.end.max(glyph.position.y + glyph.size.y);

            position.x += glyph.advance.x + paint.letter_spacing;
            position.y += glyph.advance.y;
        }

        layout.position.y = range.start;
        layout.size.y = range.end - range.start;

        Ok(layout)
    }
}
