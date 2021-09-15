use super::Error;
use crate::{Offset, Path, Size};
use fnv::FnvHashMap;
use generational_arena::{Arena, Index};
use std::{collections::hash_map::Entry, marker::PhantomPinned};
use ttf_parser::{FaceParsingError, GlyphId};

impl ttf_parser::OutlineBuilder for Path {
    fn move_to(&mut self, x: f32, y: f32) {
        self.move_to(Offset::new(x, -y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.line_to(Offset::new(x, -y));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.quad_to(Offset::new(x1, -y1), Offset::new(x, -y));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.cubic_to(
            Offset::new(x1, -y1),
            Offset::new(x2, -y2),
            Offset::new(x, -y),
        );
    }
    fn close(&mut self) {
        Path::close(self);
    }
}

#[derive(Clone)]
pub struct GlyphMetrics {
    pub size: Size,
    pub bearing: Offset,
}

impl GlyphMetrics {
    pub fn scale(self, scale: f32) -> Self {
        Self {
            size: self.size * scale,
            bearing: self.bearing * scale,
        }
    }
}

pub struct Glyph {
    pub path: Path,
    pub bbox: ttf_parser::Rect,
}

impl Glyph {
    pub fn metrics(&self) -> GlyphMetrics {
        GlyphMetrics {
            size: Size::new(self.bbox.width() as f32, self.bbox.height() as f32),
            bearing: Offset::new(self.bbox.x_min as f32, self.bbox.y_max as f32),
        }
    }
}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct Flags: u32 {
        const REGULAR = 0x01;
        const ITALIC = 0x02;
        const BOLD = 0x04;
        const OBLIQUE = 0x08;
        const VARIABLE = 0x10;
    }
}

/*
/// Information about a font face.
// TODO: underline, strikeout, subscript, superscript metrics
#[derive(Copy, Clone, Default, Debug)]
pub struct Metrics {
    /// The distance from the baseline to the top of the highest glyph
    pub ascender: f32,
    /// The distance from the baseline to the bottom of the lowest descenders on the glyphs
    pub descender: f32,

    pub height: f32,
    pub weight: u16,
    pub width: u16,
    pub flags: Flags,
}

impl Metrics {
    fn scale(mut self, scale: f32) -> Self {
        self.ascender *= scale;
        self.descender *= scale;
        self.height *= scale;
        self
    }

    pub fn ascender(&self) -> f32 {
        self.ascender
    }

    pub fn descender(&self) -> f32 {
        self.descender
    }

    pub fn round_height(&self) -> f32 {
        self.height.round()
    }

    pub fn regular(&self) -> bool {
        self.flags.contains(Flags::REGULAR)
    }

    pub fn italic(&self) -> bool {
        self.flags.contains(Flags::ITALIC)
    }

    pub fn bold(&self) -> bool {
        self.flags.contains(Flags::BOLD)
    }

    pub fn oblique(&self) -> bool {
        self.flags.contains(Flags::OBLIQUE)
    }

    pub fn variable(&self) -> bool {
        self.flags.contains(Flags::VARIABLE)
    }
}
*/

pub struct Face {
    data: Box<[u8]>,
    raw: rustybuzz::Face<'static>,
    cache: FnvHashMap<GlyphId, Glyph>,
    _pin: PhantomPinned,
}

impl Face {
    pub fn new(data: impl Into<Box<[u8]>>, index: u32) -> Result<Self, Error> {
        let data = data.into();
        let raw = unsafe {
            // 'static lifetime is a lie, this data is owned, it has pseudo-self lifetime.
            let data: &'static [u8] = core::slice::from_raw_parts(data.as_ptr(), data.len());
            let face = ttf_parser::Face::from_slice(data, index).map_err(Error::Parse)?;
            rustybuzz::Face::from_face(face).ok_or(Error::Parse(FaceParsingError::MalformedFont))?
        };

        Ok(Self {
            data,
            raw,
            cache: FnvHashMap::default(),
            _pin: PhantomPinned,
        })
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_ref()
    }

    pub fn ttf(&self) -> &ttf_parser::Face<'_> {
        self.raw.as_ref()
    }

    pub fn raw(&self) -> &rustybuzz::Face<'_> {
        &self.raw
    }

    pub fn ascender(&self, font_size: f32) -> f32 {
        self.scale(font_size) * self.raw.ascender() as f32
    }

    pub fn descender(&self, font_size: f32) -> f32 {
        self.scale(font_size) * self.raw.descender() as f32
    }

    pub fn height(&self, font_size: f32) -> f32 {
        self.scale(font_size) * self.raw.height() as f32
    }

    /*
    pub fn metrics(&self, scale: f32) -> Metrics {
        let face = &self.pin.ttf;

        let mut flags = Flags::empty();

        flags.set(Flags::REGULAR, face.is_regular());
        flags.set(Flags::ITALIC, face.is_italic());
        flags.set(Flags::BOLD, face.is_bold());
        flags.set(Flags::OBLIQUE, face.is_oblique());
        flags.set(Flags::VARIABLE, face.is_variable());

        Metrics {
            ascender: face.ascender() as f32,
            descender: face.descender() as f32,
            height: face.height() as f32,
            flags,

            weight: face.width().to_number(),
            width: face.weight().to_number(),
        }
        .scale(self.scale(scale))
    }
    */

    pub fn scale(&self, font_size: f32) -> f32 {
        font_size / self.raw.units_per_em() as f32
    }

    pub fn glyph(&mut self, id: GlyphId) -> Option<&Glyph> {
        self.glyph_mut(id).map(|g| &*g)
    }

    #[inline]
    fn glyph_mut(&mut self, id: GlyphId) -> Option<&mut Glyph> {
        match self.cache.entry(id) {
            Entry::Vacant(entry) => {
                let mut path = Path::new();
                self.raw
                    .outline_glyph(id, &mut path)
                    .map(|bbox| entry.insert(Glyph { path, bbox }))
            }
            Entry::Occupied(entry) => Some(entry.into_mut()),
        }
    }
}

/// A font face handle.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FaceId(Index);

#[derive(Default)]
pub struct FaceCollection {
    arena: Arena<Face>,
}

impl FaceCollection {
    pub fn get(&self, id: FaceId) -> Option<&Face> {
        self.arena.get(id.0)
    }

    pub fn get_mut(&mut self, id: FaceId) -> Option<&mut Face> {
        self.arena.get_mut(id.0)
    }

    pub fn add_mem(&mut self, data: impl Into<Box<[u8]>>, index: u32) -> Result<FaceId, Error> {
        Ok(FaceId(self.arena.insert(Face::new(data, index)?)))
    }

    pub fn find<T>(
        &mut self,
        _text: &str,
        ids: &[Option<FaceId>],
        mut callback: impl FnMut(FaceId, &mut Face) -> (bool, T),
    ) -> Option<T> {
        // Try each face font in the paint
        for maybe_id in ids {
            if let Some(id) = maybe_id {
                if let Some(face) = self.arena.get_mut(id.0) {
                    let (has_missing, result) = callback(*id, face);

                    if !has_missing {
                        return Some(result);
                    }
                }
            } else {
                break;
            }
        }

        // Try each registered font face
        // An optimisation here would be to skip font faces that were tried by the paint
        for (id, face) in &mut self.arena {
            let (has_missing, result) = callback(FaceId(id), face);

            if !has_missing {
                return Some(result);
            }
        }

        // Just return the first font face at this point and let it render .nodef glyphs
        self.arena
            .iter_mut()
            .next()
            .map(|(id, face)| callback(FaceId(id), face).1)
    }
}
