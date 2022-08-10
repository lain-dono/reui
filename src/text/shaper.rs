// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use {
    super::{
        font::Font,
        font_db::{Database, FaceId},
    },
    crate::Path,
    ttf_parser::GlyphId,
};

/// Converts byte position into a character.
fn char_from(start: usize, text: &str) -> char {
    text[start..].chars().next().unwrap()
}

/// A glyph.
///
/// Basically, a glyph ID and it's metrics.
#[derive(Clone)]
pub struct Glyph {
    /// The glyph ID in the font.
    pub id: GlyphId,

    /// Position in bytes in the original string.
    ///
    /// We use it to match a glyph with a character in the text chunk and therefore with the style.
    pub offset: usize,
    pub len: usize,

    /// The glyph offset in font units.
    pub dx: i32,

    /// The glyph offset in font units.
    pub dy: i32,

    /// The glyph width / X-advance in font units.
    pub width: i32,

    /// Reference to the source font.
    ///
    /// Each glyph can have it's own source font.
    pub font: Font,
}

impl Glyph {
    fn is_missing(&self) -> bool {
        self.id.0 == 0
    }
}

/// An outlined cluster.
///
/// Cluster/grapheme is a single, unbroken, renderable character.
/// It can be positioned, rotated, spaced, etc.
///
/// Let's say we have `й` which is *CYRILLIC SMALL LETTER I* and *COMBINING BREVE*.
/// It consists of two code points, will be shaped (via harfbuzz) as two glyphs into one cluster,
/// and then will be combined into the one `OutlinedCluster`.
#[derive(Clone)]
pub struct OutlinedCluster {
    /// Position in bytes in the original string.
    ///
    /// We use it to match a cluster with a character in the text chunk and therefore with the style.
    pub start: usize,
    pub end: usize,

    /// Cluster's original codepoint.
    ///
    /// Technically, a cluster can contain multiple codepoints,
    /// but we are storing only the first one.
    pub codepoint: char,

    /// Cluster's width.
    ///
    /// It's different from advance in that it's not affected by letter spacing and word spacing.
    pub width: f32,

    /// An advance along the X axis.
    ///
    /// Can be negative.
    pub advance: f32,

    /// An ascent in SVG coordinates.
    pub ascent: f32,

    /// A descent in SVG coordinates.
    pub descent: f32,

    /// A x-height in SVG coordinates.
    pub x_height: f32,

    /// An actual outline.
    pub path: Path,

    /// A cluster's transform that contains it's position, rotation, etc.
    pub transform: crate::Transform,

    /// Indicates that this cluster was affected by the relative shift (via dx/dy attributes)
    /// during the text layouting. Which breaks the `text-decoration` line.
    ///
    /// Used during the `text-decoration` processing.
    pub has_relative_shift: bool,
}

impl OutlinedCluster {
    pub fn height(&self) -> f32 {
        self.ascent - self.descent
    }

    /// Outlines a glyph cluster.
    ///
    /// Uses one or more `Glyph`s to construct an `OutlinedCluster`.
    pub fn outline(glyphs: &[Glyph], text: &str, font_size: f32, db: &Database) -> Self {
        debug_assert!(!glyphs.is_empty());

        let mut path = Path::new();
        let mut width = 0.0;
        let mut x = 0.0;

        const fn zero_rect() -> ttf_parser::Rect {
            ttf_parser::Rect {
                x_min: 0,
                y_min: 0,
                x_max: 0,
                y_max: 0,
            }
        }

        for glyph in glyphs {
            let (mut outline, _bounds) = db
                .outline(glyph.font.id(), glyph.id)
                .unwrap_or_else(|| (Path::new(), zero_rect()));

            let sx = glyph.font.scale(font_size);

            if !outline.is_empty() {
                // Scale to font-size.
                let scale = sx;

                // Apply offset.
                //
                // The first glyph in the cluster will have an offset from 0x0,
                // but the later one will have an offset from the "current position".
                // So we have to keep an advance.
                // TODO: should be done only inside a single text span
                outline.transform_inplace(x + glyph.dx as f32, glyph.dy as f32, scale);

                path.extend_with_path(&outline);
            }

            x += glyph.width as f32;

            let glyph_width = glyph.width as f32 * sx;
            if glyph_width > width {
                width = glyph_width;
            }
        }

        let start = glyphs[0].offset;
        let end = glyphs.last().map(|g| g.offset + g.len).unwrap();
        let font = glyphs[0].font;
        Self {
            start,
            end,
            codepoint: char_from(start, text),
            width,
            advance: width,
            ascent: font.ascent(font_size),
            descent: font.descent(font_size),
            x_height: font.x_height(font_size),
            has_relative_shift: false,
            path,
            transform: crate::Transform::default(),
        }
    }
}

/// An iterator over glyph clusters.
///
/// Input:  0 2 2 2 3 4 4 5 5
/// Result: 0 1     4 5   7
pub struct GlyphClusters<'a> {
    data: &'a [Glyph],
    idx: usize,
}

impl<'a> GlyphClusters<'a> {
    pub fn new(data: &'a [Glyph]) -> Self {
        Self { data, idx: 0 }
    }
}

impl<'a> Iterator for GlyphClusters<'a> {
    type Item = (std::ops::Range<usize>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.data.len() {
            return None;
        }

        let start = self.idx;
        let cluster = self.data[self.idx].offset;
        for g in &self.data[self.idx..] {
            if g.offset != cluster {
                break;
            }

            self.idx += 1;
        }

        Some((start..self.idx, cluster))
    }
}

/// Text shaping with font fallback.
pub fn shape_text(text: &str, font: Font, db: &Database) -> Vec<Glyph> {
    /// Finds a font with a specified char.
    ///
    /// This is a rudimentary font fallback algorithm.
    fn find_font_for_char(c: char, exclude_fonts: &[FaceId], db: &Database) -> Option<Font> {
        let base_font_id = exclude_fonts[0];

        use std::cell::RefCell;
        use std::collections::HashMap;
        thread_local!(static CACHE: RefCell<HashMap<char, FaceId>> = RefCell::new(HashMap::default()));

        if let Some(id) = CACHE.with(|cache| cache.borrow().get(&c).copied()) {
            return db.load_font(id);
        }

        // Iterate over fonts and check if any of them support the specified char.
        for (id, face) in db.faces() {
            // Ignore fonts, that were used for shaping already.
            if exclude_fonts.contains(&id) {
                continue;
            }

            // Check that the new face has the same style.
            let base_face = db.face(base_font_id)?;
            if base_face.style != face.style
                && base_face.weight != face.weight
                && base_face.stretch != face.stretch
            {
                continue;
            }

            if !db.has_char(id, c) {
                continue;
            }

            //log::warn!("Fallback from {} to {}.", base_face.family, face.family);
            let font = db.load_font(id);
            if font.is_some() {
                CACHE.with(|cache| cache.borrow_mut().insert(c, id));
            }
            return font;
        }

        None
    }

    let mut buffer = Some(rustybuzz::UnicodeBuffer::new());

    let mut glyphs = shape_text_with_font(text, font, db, &mut buffer).unwrap_or_default();

    // Remember all fonts used for shaping.
    let mut used_fonts = vec![font.id()];

    // Loop until all glyphs become resolved or until no more fonts are left.
    'outer: loop {
        let missing = glyphs.iter().find_map(|glyph| {
            if glyph.is_missing() {
                Some(char_from(glyph.offset, text))
            } else {
                None
            }
        });

        if let Some(c) = missing {
            let fallback_font = match find_font_for_char(c, &used_fonts, db) {
                Some(v) => v,
                None => break 'outer,
            };

            // Shape again, using a new font.
            let fallback_glyphs =
                shape_text_with_font(text, fallback_font, db, &mut buffer).unwrap_or_default();

            let all_matched = fallback_glyphs.iter().all(|g| !g.is_missing());
            if all_matched {
                // Replace all glyphs when all of them were matched.
                glyphs = fallback_glyphs;
                break 'outer;
            }

            // We assume, that shaping with an any font will produce the same amount of glyphs.
            // This is incorrect, but good enough for now.
            if glyphs.len() != fallback_glyphs.len() {
                break 'outer;
            }

            // TODO: Replace clusters and not glyphs. This should be more accurate.

            // Copy new glyphs.
            for i in 0..glyphs.len() {
                if glyphs[i].is_missing() && !fallback_glyphs[i].is_missing() {
                    glyphs[i] = fallback_glyphs[i].clone();
                }
            }

            // Remember this font.
            used_fonts.push(fallback_font.id());
        } else {
            break 'outer;
        }
    }

    // Warn about missing glyphs.
    for glyph in &glyphs {
        if glyph.is_missing() {
            let c = char_from(glyph.offset, text);
            // TODO: print a full grapheme
            log::warn!(
                "No fonts with a {}/U+{:X} character were found.",
                c,
                c as u32
            );
        }
    }

    glyphs
}

/// Converts a text into a list of glyph IDs.
///
/// This function will do the BIDI reordering and text shaping.
fn shape_text_with_font(
    text: &str,
    font: Font,
    db: &Database,
    buffer_reuse: &mut Option<rustybuzz::UnicodeBuffer>,
) -> Option<Vec<Glyph>> {
    db.with_face_data(font.id(), |font_data, face_index| -> Option<Vec<Glyph>> {
        let rb_font = rustybuzz::Face::from_slice(font_data, face_index)?;

        let mut buffer = buffer_reuse
            .take()
            .unwrap_or_else(rustybuzz::UnicodeBuffer::new);

        let mut glyphs = Vec::new();

        let bidi_info = unicode_bidi::BidiInfo::new(text, Some(unicode_bidi::Level::ltr()));
        for paragraph in &bidi_info.paragraphs {
            let line = paragraph.range.clone();

            let (levels, runs) = bidi_info.visual_runs(paragraph, line);

            for run in &runs {
                let sub_text = &text[run.clone()];
                if sub_text.is_empty() {
                    continue;
                }

                let hb_direction = if levels[run.start].is_rtl() {
                    rustybuzz::Direction::RightToLeft
                } else {
                    rustybuzz::Direction::LeftToRight
                };

                buffer.push_str(sub_text);
                buffer.set_direction(hb_direction);

                let output = rustybuzz::shape(&rb_font, &[], buffer);

                let positions = output.glyph_positions();
                let infos = output.glyph_infos();

                for (pos, info) in positions.iter().zip(infos) {
                    let idx = run.start + info.cluster as usize;
                    debug_assert!(text.get(idx..).is_some());

                    let len = text[idx..].chars().next().map(char::len_utf8).unwrap();

                    glyphs.push(Glyph {
                        offset: idx,
                        len,
                        id: GlyphId(info.glyph_id as u16),
                        dx: pos.x_offset,
                        dy: pos.y_offset,
                        width: pos.x_advance,
                        font,
                    });
                }

                buffer = output.clear();
            }
        }

        *buffer_reuse = Some(buffer);

        Some(glyphs)
    })?
}

/*
/// Rotates clusters according to
/// [Unicode Vertical Orientation Property](https://www.unicode.org/reports/tr50/tr50-19.html).
pub fn apply_writing_mode(writing_mode: WritingMode, clusters: &mut [OutlinedCluster]) {
    use unicode_vo::{char_orientation, Orientation as CharOrientation};

    if writing_mode != WritingMode::TopToBottom {
        return;
    }

    for cluster in clusters {
        let orientation = char_orientation(cluster.codepoint);
        if orientation == CharOrientation::Upright {
            // Additional offset. Not sure why.
            let dy = cluster.width - cluster.height();

            // Rotate a cluster 90deg counter clockwise by the center.
            let mut ts = tree::Transform::default();
            ts.translate(cluster.width / 2.0, 0.0);
            ts.rotate(-90.0);
            ts.translate(-cluster.width / 2.0, -dy);
            cluster.path.transform(ts);

            // Move "baseline" to the middle and make height equal to width.
            cluster.ascent = cluster.width / 2.0;
            cluster.descent = -cluster.width / 2.0;
        } else {
            // Could not find a spec that explains this,
            // but this is how other applications are shifting the "rotated" characters
            // in the top-to-bottom mode.
            cluster.transform.translate(0.0, cluster.x_height / 2.0);
        }
    }
}

*/
