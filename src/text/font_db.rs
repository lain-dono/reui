/*!
`fontdb` is a simple, in-memory font database with CSS-like queries.

# Features

- The database can load fonts from files, directories and raw data (`Vec<u8>`).
- The database can match a font using CSS-like queries. See `Database::query`.
- The database can try to load system fonts.
  Currently, this is implemented by scanning predefined directories.
  The library does not interact with the system API.
- Provides a unique ID for each font face.

# Non-goals

- Advanced font properties querying.<br>
  The database provides only storage and matching capabilities.
  For font properties querying you can use [ttf-parser].

- A font fallback mechanism.<br>
  This library can be used to implement a font fallback mechanism, but it doesn't implement one.

- Application's global database.<br>
  The database doesn't use `static`, therefore it's up to the caller where it should be stored.

- Font types support other than TrueType.

# Font vs Face

A font is a collection of font faces. Therefore, a font face is a subset of a font.
A simple font (\*.ttf/\*.otf) usually contains a single font face,
but a font collection (\*.ttc) can contain multiple font faces.

`fontdb` stores and matches font faces, not fonts.
Therefore, after loading a font collection with 5 faces (for example), the database will be populated
with 5 `FaceInfo` objects, all of which will be pointing to the same file or binary data.

# Performance

The database performance is largely limited by the storage itself.
We are using [ttf-parser], so the parsing should not be a bottleneck.

On my machine with Samsung SSD 860 and Gentoo Linux, it takes ~20ms
to load 1906 font faces (most of them are from Google Noto collection)
with a hot disk cache and ~860ms with a cold one.

# Safety

The library relies on memory-mapped files, which is inherently unsafe.
But we do not keep such files open forever. Instead, we are memory-mapping files only when needed.

[ttf-parser]: https://github.com/RazrFalcon/ttf-parser
*/

//#![doc(html_root_url = "https://docs.rs/fontdb/0.5.4")]

//#![warn(missing_docs)]
//#![warn(missing_debug_implementations)]
//#![warn(missing_copy_implementations)]

use super::style::{Stretch, Style, Weight};
use log::warn;
use slotmap::SlotMap;
use std::{
    fmt, io,
    path::{Path, PathBuf},
    sync::Arc,
};

slotmap::new_key_type! {
    /// A unique per database face ID.
    ///
    /// Since `Database` is not global/unique, we cannot guarantee that a specific ID
    /// is actually from the same db instance. This is up to the caller.
    pub struct FaceId;
}

/// A list of possible font loading errors.
#[derive(Debug)]
enum LoadError {
    /// A malformed font.
    ///
    /// Typically means that [ttf-parser](https://github.com/RazrFalcon/ttf-parser)
    /// wasn't able to parse it.
    MalformedFont(ttf_parser::FaceParsingError),
    /// A valid TrueType font without a valid *Family Name*.
    UnnamedFont,
    /// A file IO related error.
    IoError(io::Error),
}

impl From<io::Error> for LoadError {
    #[inline]
    fn from(e: io::Error) -> Self {
        LoadError::IoError(e)
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::MalformedFont(_) => write!(f, "malformed font"),
            LoadError::UnnamedFont => write!(f, "font doesn't have a family name"),
            LoadError::IoError(ref e) => write!(f, "{}", e),
        }
    }
}

/// A font database.
#[derive(Clone, Debug, Default)]
pub struct Database {
    faces: SlotMap<FaceId, FaceInfo>,
}

impl Database {
    /// Create a new, empty `Database`.
    #[inline]
    pub fn new() -> Self {
        Self {
            faces: SlotMap::with_key(),
        }
    }

    /// Loads a font data into the `Database`.
    ///
    /// Will load all font faces in case of a font collection.
    ///
    /// # Panics
    pub fn load_font_data(&mut self, data: Vec<u8>) {
        let source = Arc::new(Source::Binary(data));

        // Borrow `source` data.
        let data = match &*source {
            Source::Binary(ref data) => data,
            Source::File(_) => unreachable!(),
        };

        let n = ttf_parser::fonts_in_collection(data).unwrap_or(1);
        for index in 0..n {
            if let Err(err) = parse_face_info(&mut self.faces, source.clone(), data, index) {
                warn!(
                    "Failed to load a font face {} from data cause {}.",
                    index, err
                );
            }
        }
    }

    /// Loads a font file into the `Database`.
    ///
    /// Will load all font faces in case of a font collection.
    ///
    /// # Errors
    ///
    /// # Panics
    pub fn load_font_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let source = Arc::new(Source::File(path.as_ref().into()));

        let file = std::fs::File::open(path.as_ref())?;
        let data = unsafe { &memmap2::MmapOptions::new().map(&file)? };

        let n = ttf_parser::fonts_in_collection(data).unwrap_or(1);
        for index in 0..n {
            if let Err(err) = parse_face_info(&mut self.faces, source.clone(), data, index) {
                warn!(
                    "Failed to load a font face {} from '{}' cause {}.",
                    index,
                    path.as_ref().display(),
                    err
                );
            }
        }

        Ok(())
    }

    /// Loads font files from the selected directory into the `Database`.
    ///
    /// This method will scan directories recursively.
    ///
    /// Will load `ttf`, `otf`, `ttc` and `otc` fonts.
    ///
    /// Unlike other `load_*` methods, this one doesn't return an error.
    /// It will simply skip malformed fonts and will print a warning into the log for each of them.
    ///
    /// # Panics
    pub fn load_fonts_dir<P: AsRef<Path>>(&mut self, dir: P) {
        let fonts_dir = match std::fs::read_dir(dir.as_ref()) {
            Ok(dir) => dir,
            Err(_) => return,
        };

        fonts_dir.for_each(|entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some("ttf" | "ttc" | "TTF" | "TTC" | "otf" | "otc" | "OTF" | "OTC") =
                        path.extension().and_then(std::ffi::OsStr::to_str)
                    {
                        if let Err(e) = self.load_font_file(&path) {
                            warn!("Failed to load '{}' cause {}.", path.display(), e);
                        }
                    }
                } else if path.is_dir() {
                    // TODO: ignore symlinks?
                    self.load_fonts_dir(path);
                }
            }
        });
    }

    /// Attempts to load system fonts.
    ///
    /// Supports Windows, Linux and macOS.
    ///
    /// System fonts loading is a surprisingly complicated task,
    /// mostly unsolvable without interacting with system libraries.
    /// And since `fontdb` tries to be small and portable, this method
    /// will simply scan some predefined directories.
    /// Which means that fonts that are not in those directories must
    /// be added manually.
    pub fn load_system_fonts(&mut self) {
        #[cfg(target_os = "windows")]
        {
            self.load_fonts_dir("C:\\Windows\\Fonts\\");
        }

        #[cfg(target_os = "macos")]
        {
            self.load_fonts_dir("/Library/Fonts");
            self.load_fonts_dir("/System/Library/Fonts");
            self.load_fonts_dir("/System/Library/AssetsV2/com_apple_MobileAsset_Font6");
            self.load_fonts_dir("/Network/Library/Fonts");

            if let Ok(ref home) = std::env::var("HOME") {
                let path = Path::new(home).join("Library/Fonts");
                self.load_fonts_dir(path);
            }
        }

        // Linux.
        #[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
        {
            self.load_fonts_dir("/usr/share/fonts/");
            self.load_fonts_dir("/usr/local/share/fonts/");

            if let Ok(ref home) = std::env::var("HOME") {
                let path = Path::new(home).join(".local/share/fonts");
                self.load_fonts_dir(path);
            }
        }
    }

    /// Removes a font face by `id` from the database.
    ///
    /// Returns `false` while attempting to remove a non-existing font face.
    ///
    /// Useful when you want to ignore some specific font face(s)
    /// after loading a large directory with fonts.
    /// Or a specific face from a font.
    pub fn remove_face(&mut self, id: FaceId) -> bool {
        self.faces.remove(id).is_some()
    }

    /// Returns `true` if the `Database` contains no font faces.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }

    /// Returns the number of font faces in the `Database`.
    ///
    /// Note that `Database` stores font faces, not fonts.
    /// For example, if a caller will try to load a font collection (`*.ttc`) that contains 5 faces,
    /// then the `Database` will load 5 font faces and this method will return 5, not 1.
    #[inline]
    pub fn len(&self) -> usize {
        self.faces.len()
    }

    /// Performs a CSS-like query and returns the best matched font face.
    pub fn query(&self, query: &Query) -> Option<FaceId> {
        for &family in query.families {
            let mut ids = Vec::new();
            let mut candidates = Vec::new();
            for (id, face) in self.faces.iter().filter(|(_, face)| face.family == family) {
                ids.push(id);
                candidates.push(FaceProperties {
                    style: face.style,
                    weight: face.weight,
                    stretch: face.stretch,
                });
            }

            if !candidates.is_empty() {
                if let Some(index) = find_best_match(&candidates, query) {
                    return Some(ids[index]);
                }
            }
        }

        None
    }

    /// Returns a reference to an internal storage.
    ///
    /// This can be used for manual font matching.
    #[inline]
    pub fn faces(&self) -> slotmap::basic::Iter<FaceId, FaceInfo> {
        self.faces.iter()
    }

    /// Selects a `FaceInfo` by `id`.
    ///
    /// Returns `None` if a face with such ID was already removed,
    /// or this ID belong to the other `Database`.
    pub fn face(&self, id: FaceId) -> Option<&FaceInfo> {
        self.faces.get(id)
    }

    /// Returns font face storage and the face index by `ID`.
    pub fn face_source(&self, id: FaceId) -> Option<(Arc<Source>, u32)> {
        self.face(id).map(|info| (info.source.clone(), info.index))
    }

    /// Executes a closure with a font's data.
    ///
    /// We can't return a reference to a font binary data because of lifetimes.
    /// So instead, you can use this method to process font's data.
    ///
    /// The closure accepts raw font data and font face index.
    ///
    /// In case of `Source::File`, the font file will be memory mapped.
    ///
    /// Returns `None` when font file loading failed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let is_variable = db.with_face_data(id, |font_data, face_index| {
    ///     let font = ttf_parser::Face::from_slice(font_data, face_index).unwrap();
    ///     font.is_variable()
    /// })?;
    /// ```
    pub fn with_face_data<T>(&self, id: FaceId, p: impl FnOnce(&[u8], u32) -> T) -> Option<T> {
        let (src, face_index) = self.face_source(id)?;
        match &*src {
            Source::File(ref path) => {
                let file = std::fs::File::open(path).ok()?;
                let data = unsafe { &memmap2::MmapOptions::new().map(&file).ok()? };
                Some(p(data, face_index))
            }
            Source::Binary(ref data) => Some(p(data, face_index)),
        }
    }
}

/// A single font face info.
///
/// A font can have multiple faces.
///
/// A single item of the `Database`.
#[derive(Clone, Debug)]
pub struct FaceInfo {
    /// An unique ID.
    //pub id: FaceId,

    /// A font source.
    ///
    /// We have to use `Rc`, because multiple `FaceInfo` objects can reference
    /// the same data in case of font collections.
    pub source: Arc<Source>,

    /// A face index in the `source`.
    pub index: u32,

    /// A family name.
    ///
    /// Corresponds to a *Font Family* (1) [name ID] in a TrueType font.
    ///
    /// [name ID]: https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-ids
    pub family: String,

    /// A PostScript name.
    ///
    /// Corresponds to a *PostScript name* (6) [name ID] in a TrueType font.
    ///
    /// [name ID]: https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-ids
    pub post_script_name: String,

    /// A font face style.
    pub style: Style,

    /// A font face weight.
    pub weight: Weight,

    /// A font face stretch.
    pub stretch: Stretch,

    /// Indicates that the font face is monospaced.
    pub monospaced: bool,
}

/// CSS-related face properties.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
struct FaceProperties {
    style: Style,
    weight: Weight,
    stretch: Stretch,
}

/// A font source.
///
/// Either a raw binary data or a file path.
///
/// Stores the whole font and not just a single face.
#[derive(Clone, Debug)]
pub enum Source {
    /// A font's raw data. Owned by the database.
    Binary(Vec<u8>),

    /// A font's path.
    File(PathBuf),
}

/// A database query.
///
/// Mainly used by `Database::query()`.
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, Hash)]
pub struct Query<'a> {
    /// A prioritized list of font family names or generic family names.
    ///
    /// [font-family](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#propdef-font-family) in CSS.
    pub families: &'a [Family<'a>],

    /// Specifies the weight of glyphs in the font, their degree of blackness or stroke thickness.
    ///
    /// [font-weight](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#font-weight-prop) in CSS.
    pub weight: Weight,

    /// Selects a normal, condensed, or expanded face from a font family.
    ///
    /// [font-stretch](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#font-stretch-prop) in CSS.
    pub stretch: Stretch,

    /// Allows italic or oblique faces to be selected.
    ///
    /// [font-style](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#font-style-prop) in CSS.
    pub style: Style,
}

// Descriptions are from the CSS spec.
/// A [font family](https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#propdef-font-family).
pub type Family<'a> = &'a str;

fn parse_face_info(
    faces: &mut SlotMap<FaceId, FaceInfo>,
    source: Arc<Source>,
    data: &[u8],
    index: u32,
) -> Result<FaceId, LoadError> {
    let face = ttf_parser::Face::from_slice(data, index).map_err(LoadError::MalformedFont)?;

    //let family = parse_name(ttf_parser::name_id::FAMILY, &face).ok_or(LoadError::UnnamedFont)?;
    let family = parse_name(ttf_parser::name_id::TYPOGRAPHIC_FAMILY, &face)
        .or_else(|| parse_name(ttf_parser::name_id::FAMILY, &face))
        .ok_or(LoadError::UnnamedFont)?;

    let post_script_name =
        parse_name(ttf_parser::name_id::POST_SCRIPT_NAME, &face).ok_or(LoadError::UnnamedFont)?;

    let style = if face.is_italic() {
        Style::Italic
    } else if face.is_oblique() {
        Style::Oblique
    } else {
        Style::Normal
    };

    Ok(faces.insert(FaceInfo {
        source,
        index,
        family,
        post_script_name,
        style,
        weight: Weight(face.weight().to_number()),
        stretch: face.width(),
        monospaced: face.is_monospaced(),
    }))
}

fn parse_name(name_id: u16, face: &ttf_parser::Face) -> Option<String> {
    trait NameExt {
        fn is_mac_roman(&self) -> bool;
        fn is_supported_encoding(&self) -> bool;
    }

    impl NameExt for ttf_parser::name::Name<'_> {
        #[inline]
        fn is_mac_roman(&self) -> bool {
            // https://docs.microsoft.com/en-us/typography/opentype/spec/name#macintosh-encoding-ids-script-manager-codes
            const MACINTOSH_ROMAN_ENCODING_ID: u16 = 0;

            self.platform_id == ttf_parser::PlatformId::Macintosh
                && self.encoding_id == MACINTOSH_ROMAN_ENCODING_ID
        }

        #[inline]
        fn is_supported_encoding(&self) -> bool {
            self.is_unicode() || self.is_mac_roman()
        }
    }

    let names = face.names();
    let mut names = (0..names.len()).map(|index| names.get(index).unwrap());
    let name_record = names.find(|name| name.name_id == name_id && name.is_supported_encoding())?;

    if name_record.is_unicode() {
        name_record.to_string()
    } else if name_record.is_mac_roman() {
        // We support only MacRoman encoding here, which should be enough in most cases.
        let mut raw_data = Vec::with_capacity(name_record.name.len());
        for b in name_record.name {
            raw_data.push(MAC_ROMAN[*b as usize]);
        }

        String::from_utf16(&raw_data).ok()
    } else {
        None
    }
}

// https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#font-style-matching
// Based on https://github.com/servo/font-kit
#[inline(never)]
fn find_best_match(candidates: &[FaceProperties], query: &Query) -> Option<usize> {
    debug_assert!(!candidates.is_empty());

    // Step 4.
    let mut matching_set: Vec<usize> = (0..candidates.len()).collect();

    // Step 4a (`font-stretch`).
    let stretch_matches = matching_set
        .iter()
        .any(|&index| candidates[index].stretch == query.stretch);

    let matching_stretch = if stretch_matches {
        // Exact match.
        query.stretch
    } else if query.stretch <= Stretch::Normal {
        // Closest stretch, first checking narrower values and then wider values.
        matching_set
            .iter()
            .filter(|&&index| candidates[index].stretch < query.stretch)
            .min_by_key(|&&index| query.stretch.to_number() - candidates[index].stretch.to_number())
            .or_else(|| {
                matching_set.iter().min_by_key(|&&index| {
                    candidates[index].stretch.to_number() - query.stretch.to_number()
                })
            })
            .map(|&matching_index| candidates[matching_index].stretch)?
    } else {
        // Closest stretch, first checking wider values and then narrower values.
        matching_set
            .iter()
            .filter(|&&index| candidates[index].stretch > query.stretch)
            .min_by_key(|&&index| candidates[index].stretch.to_number() - query.stretch.to_number())
            .or_else(|| {
                matching_set.iter().min_by_key(|&&index| {
                    query.stretch.to_number() - candidates[index].stretch.to_number()
                })
            })
            .map(|&matching_index| candidates[matching_index].stretch)?
    };
    matching_set.retain(|&index| candidates[index].stretch == matching_stretch);

    // Step 4b (`font-style`).
    let style_preference = match query.style {
        Style::Italic => [Style::Italic, Style::Oblique, Style::Normal],
        Style::Oblique => [Style::Oblique, Style::Italic, Style::Normal],
        Style::Normal => [Style::Normal, Style::Oblique, Style::Italic],
    };
    let matching_style = *style_preference.iter().find(|&query_style| {
        matching_set
            .iter()
            .any(|&index| candidates[index].style == *query_style)
    })?;

    matching_set.retain(|&index| candidates[index].style == matching_style);

    // Step 4c (`font-weight`).
    //
    // The spec doesn't say what to do if the weight is between 400 and 500 exclusive, so we
    // just use 450 as the cutoff.
    let weight = query.weight.0;
    let weight_matches = (400..450).contains(&weight)
        && matching_set
            .iter()
            .any(|&index| candidates[index].weight.0 == 500);

    let matching_weight = if weight_matches {
        // Check 500 first.
        Weight::MEDIUM
    } else if (450..=500).contains(&weight)
        && matching_set
            .iter()
            .any(|&index| candidates[index].weight.0 == 400)
    {
        // Check 400 first.
        Weight::NORMAL
    } else if weight <= 500 {
        // Closest weight, first checking thinner values and then fatter ones.
        matching_set
            .iter()
            .filter(|&&index| candidates[index].weight.0 <= weight)
            .min_by_key(|&&index| weight - candidates[index].weight.0)
            .or_else(|| {
                matching_set
                    .iter()
                    .min_by_key(|&&index| candidates[index].weight.0 - weight)
            })
            .map(|&matching_index| candidates[matching_index].weight)?
    } else {
        // Closest weight, first checking fatter values and then thinner ones.
        matching_set
            .iter()
            .filter(|&&index| candidates[index].weight.0 >= weight)
            .min_by_key(|&&index| candidates[index].weight.0 - weight)
            .or_else(|| {
                matching_set
                    .iter()
                    .min_by_key(|&&index| weight - candidates[index].weight.0)
            })
            .map(|&matching_index| candidates[matching_index].weight)?
    };
    matching_set.retain(|&index| candidates[index].weight == matching_weight);

    // Ignore step 4d (`font-size`).

    // Return the result.
    matching_set.into_iter().next()
}

/// Macintosh Roman to UTF-16 encoding table.
///
/// <https://en.wikipedia.org/wiki/Mac_OS_Roman>
const MAC_ROMAN: &[u16; 256] = &[
    0x0000, 0x0001, 0x0002, 0x0003, 0x0004, 0x0005, 0x0006, 0x0007, 0x0008, 0x0009, 0x000A, 0x000B,
    0x000C, 0x000D, 0x000E, 0x000F, 0x0010, 0x2318, 0x21E7, 0x2325, 0x2303, 0x0015, 0x0016, 0x0017,
    0x0018, 0x0019, 0x001A, 0x001B, 0x001C, 0x001D, 0x001E, 0x001F, 0x0020, 0x0021, 0x0022, 0x0023,
    0x0024, 0x0025, 0x0026, 0x0027, 0x0028, 0x0029, 0x002A, 0x002B, 0x002C, 0x002D, 0x002E, 0x002F,
    0x0030, 0x0031, 0x0032, 0x0033, 0x0034, 0x0035, 0x0036, 0x0037, 0x0038, 0x0039, 0x003A, 0x003B,
    0x003C, 0x003D, 0x003E, 0x003F, 0x0040, 0x0041, 0x0042, 0x0043, 0x0044, 0x0045, 0x0046, 0x0047,
    0x0048, 0x0049, 0x004A, 0x004B, 0x004C, 0x004D, 0x004E, 0x004F, 0x0050, 0x0051, 0x0052, 0x0053,
    0x0054, 0x0055, 0x0056, 0x0057, 0x0058, 0x0059, 0x005A, 0x005B, 0x005C, 0x005D, 0x005E, 0x005F,
    0x0060, 0x0061, 0x0062, 0x0063, 0x0064, 0x0065, 0x0066, 0x0067, 0x0068, 0x0069, 0x006A, 0x006B,
    0x006C, 0x006D, 0x006E, 0x006F, 0x0070, 0x0071, 0x0072, 0x0073, 0x0074, 0x0075, 0x0076, 0x0077,
    0x0078, 0x0079, 0x007A, 0x007B, 0x007C, 0x007D, 0x007E, 0x007F, 0x00C4, 0x00C5, 0x00C7, 0x00C9,
    0x00D1, 0x00D6, 0x00DC, 0x00E1, 0x00E0, 0x00E2, 0x00E4, 0x00E3, 0x00E5, 0x00E7, 0x00E9, 0x00E8,
    0x00EA, 0x00EB, 0x00ED, 0x00EC, 0x00EE, 0x00EF, 0x00F1, 0x00F3, 0x00F2, 0x00F4, 0x00F6, 0x00F5,
    0x00FA, 0x00F9, 0x00FB, 0x00FC, 0x2020, 0x00B0, 0x00A2, 0x00A3, 0x00A7, 0x2022, 0x00B6, 0x00DF,
    0x00AE, 0x00A9, 0x2122, 0x00B4, 0x00A8, 0x2260, 0x00C6, 0x00D8, 0x221E, 0x00B1, 0x2264, 0x2265,
    0x00A5, 0x00B5, 0x2202, 0x2211, 0x220F, 0x03C0, 0x222B, 0x00AA, 0x00BA, 0x03A9, 0x00E6, 0x00F8,
    0x00BF, 0x00A1, 0x00AC, 0x221A, 0x0192, 0x2248, 0x2206, 0x00AB, 0x00BB, 0x2026, 0x00A0, 0x00C0,
    0x00C3, 0x00D5, 0x0152, 0x0153, 0x2013, 0x2014, 0x201C, 0x201D, 0x2018, 0x2019, 0x00F7, 0x25CA,
    0x00FF, 0x0178, 0x2044, 0x20AC, 0x2039, 0x203A, 0xFB01, 0xFB02, 0x2021, 0x00B7, 0x201A, 0x201E,
    0x2030, 0x00C2, 0x00CA, 0x00C1, 0x00CB, 0x00C8, 0x00CD, 0x00CE, 0x00CF, 0x00CC, 0x00D3, 0x00D4,
    0xF8FF, 0x00D2, 0x00DA, 0x00DB, 0x00D9, 0x0131, 0x02C6, 0x02DC, 0x00AF, 0x02D8, 0x02D9, 0x02DA,
    0x00B8, 0x02DD, 0x02DB, 0x02C7,
];