/// Text baseline vertical alignment:
/// `Top`, `Middle`, `Alphabetic` (default), `Bottom`.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Baseline {
    /// The text baseline is the top of the em square.
    Top,
    /// The text baseline is the middle of the em square.
    Middle,
    /// The text baseline is the normal alphabetic baseline. Default value.
    Alphabetic,
    // The text baseline is the bottom of the bounding box.
    Bottom,
}

impl Default for Baseline {
    fn default() -> Self {
        Self::Alphabetic
    }
}

/// Text horizontal alignment: `Left` (default), `Center`, `Right`.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Align {
    /// The text is left-aligned.
    Left,
    /// The text is centered.
    Center,
    /// The text is right-aligned.
    Right,
}

impl Default for Align {
    fn default() -> Self {
        Self::Left
    }
}
