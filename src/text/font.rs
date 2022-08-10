use super::font_db::{Database, FaceId};
use crate::Path;
use std::convert::TryFrom;
use std::num::NonZeroU16;
use ttf_parser::GlyphId;

impl Database {
    /// # Panics
    pub fn load_font(&self, id: FaceId) -> Option<Font> {
        self.with_face_data(id, |data, face_index| -> Option<Font> {
            let font = ttf_parser::Face::from_slice(data, face_index).ok()?;

            let units_per_em = NonZeroU16::new(font.units_per_em())?;

            let ascent = font.ascender();
            let descent = font.descender();

            let x_height = font
                .x_height()
                .and_then(|x| u16::try_from(x).ok())
                .and_then(NonZeroU16::new);
            let x_height = match x_height {
                Some(height) => height,
                None => {
                    // If not set - fallback to height * 45%.
                    // 45% is what Firefox uses.
                    u16::try_from((f32::from(ascent - descent) * 0.45) as i32)
                        .ok()
                        .and_then(NonZeroU16::new)?
                }
            };

            let line_through = font.strikeout_metrics();
            let line_through_position = match line_through {
                Some(metrics) => metrics.position,
                None => (x_height.get() / 2) as i16,
            };

            let (underline_position, underline_thickness) = match font.underline_metrics() {
                Some(metrics) => {
                    let thickness = u16::try_from(metrics.thickness)
                        .ok()
                        .and_then(NonZeroU16::new)
                        // `ttf_parser` guarantees that units_per_em is >= 16
                        .unwrap_or_else(|| NonZeroU16::new(units_per_em.get() / 12).unwrap());

                    (metrics.position, thickness)
                }
                None => (
                    -((units_per_em.get() / 9) as i16),
                    NonZeroU16::new(units_per_em.get() / 12).unwrap(),
                ),
            };

            // 0.2 and 0.4 are generic offsets used by some applications (Inkscape/librsvg).
            let subscript_offset = font.subscript_metrics().map_or_else(
                || (units_per_em.get() as f32 / 0.2).round() as i16,
                |metrics| metrics.y_offset,
            );
            let superscript_offset = font.superscript_metrics().map_or_else(
                || (units_per_em.get() as f32 / 0.4).round() as i16,
                |metrics| metrics.y_offset,
            );

            Some(Font {
                id,
                units_per_em,
                ascent,
                descent,
                x_height,
                underline_position,
                underline_thickness,
                line_through_position,
                subscript_offset,
                superscript_offset,
            })
        })?
    }

    pub fn outline(&self, id: FaceId, glyph_id: GlyphId) -> Option<(Path, ttf_parser::Rect)> {
        self.with_face_data(id, |data, face_index| -> Option<(Path, ttf_parser::Rect)> {
            let font = ttf_parser::Face::from_slice(data, face_index).ok()?;

            //let mut builder = PathBuilder(tree::PathData::with_capacity(16));
            let mut path = crate::Path::new();
            let bounds = font.outline_glyph(glyph_id, &mut path)?;

            Some((path, bounds))
        })?
    }

    pub fn has_char(&self, id: FaceId, c: char) -> bool {
        let res = self.with_face_data(id, |font_data, face_index| -> Option<bool> {
            let font = ttf_parser::Face::from_slice(font_data, face_index).ok()?;
            font.glyph_index(c)?;
            Some(true)
        });

        res == Some(Some(true))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Font {
    id: FaceId,

    units_per_em: NonZeroU16,

    // All values below are in font units.
    ascent: i16,
    descent: i16,
    x_height: NonZeroU16,

    underline_position: i16,
    underline_thickness: NonZeroU16,

    // line-through thickness should be the the same as underline thickness
    // according to the TrueType spec:
    // https://docs.microsoft.com/en-us/typography/opentype/spec/os2#ystrikeoutsize
    line_through_position: i16,

    subscript_offset: i16,
    superscript_offset: i16,
}

impl Font {
    #[inline]
    pub fn id(&self) -> FaceId {
        self.id
    }

    #[inline]
    pub fn scale(&self, font_size: f32) -> f32 {
        font_size / self.units_per_em.get() as f32
    }

    #[inline]
    pub fn ascent(&self, font_size: f32) -> f32 {
        self.ascent as f32 * self.scale(font_size)
    }

    #[inline]
    pub fn descent(&self, font_size: f32) -> f32 {
        self.descent as f32 * self.scale(font_size)
    }

    #[inline]
    pub fn height(&self, font_size: f32) -> f32 {
        self.ascent(font_size) - self.descent(font_size)
    }

    #[inline]
    pub fn x_height(&self, font_size: f32) -> f32 {
        self.x_height.get() as f32 * self.scale(font_size)
    }

    #[inline]
    pub fn underline_position(&self, font_size: f32) -> f32 {
        self.underline_position as f32 * self.scale(font_size)
    }

    #[inline]
    pub fn underline_thickness(&self, font_size: f32) -> f32 {
        self.underline_thickness.get() as f32 * self.scale(font_size)
    }

    #[inline]
    pub fn line_through_position(&self, font_size: f32) -> f32 {
        self.line_through_position as f32 * self.scale(font_size)
    }

    #[inline]
    pub fn subscript_offset(&self, font_size: f32) -> f32 {
        self.subscript_offset as f32 * self.scale(font_size)
    }

    #[inline]
    pub fn superscript_offset(&self, font_size: f32) -> f32 {
        self.superscript_offset as f32 * self.scale(font_size)
    }
}
