use super::TextBaseline;
use crate::geom::Size;

/// Where to vertically align the placeholder relative to the surrounding text.
pub enum PlaceholderAlignment {
    /// Match the baseline of the placeholder with the baseline.
    Baseline,

    /// Align the bottom edge of the placeholder with the baseline such that the
    /// placeholder sits on top of the baseline.
    AboveBaseline,

    /// Align the top edge of the placeholder with the baseline specified in
    /// such that the placeholder hangs below the baseline.
    BelowBaseline,

    /// Align the top edge of the placeholder with the top edge of the font.
    /// When the placeholder is very tall, the extra space will hang from
    /// the top and extend through the bottom of the line.
    Top,

    /// Align the bottom edge of the placeholder with the top edge of the font.
    /// When the placeholder is very tall, the extra space will rise from
    /// the bottom and extend through the top of the line.
    Bottom,

    /// Align the middle of the placeholder with the middle of the text. When the
    /// placeholder is very tall, the extra space will grow equally from
    /// the top and bottom of the line.
    Middle,
}

/// Represents the metrics required to fully define a rect that will fit a placeholder.
///
/// LibTxt will leave an empty space in the layout of the text of the size
/// defined by this class. After layout, the framework will draw placeholders
/// into the reserved space.
pub struct Placeholder {
    pub size: Size,
    pub alignment: PlaceholderAlignment,
    pub baseline: TextBaseline,

    /// Distance from the top edge of the rect to the baseline position.
    /// This baseline will be aligned against the alphabetic baseline of the surrounding text.
    ///
    /// Positive values drop the baseline lower (positions the rect higher) and
    /// small or negative values will cause the rect to be positioned underneath
    /// the line. When baseline == height, the bottom edge of the rect will rest on
    /// the alphabetic baseline.
    pub baseline_offset: f32,
}
