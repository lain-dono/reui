#[derive(Clone, Copy)]
pub enum BlendFactor {
    Zero                = 0,
    One                 = 1,
    SrcColor            = 2,
    OneMinusSrcColor    = 3,
    DstColor            = 4,
    OneMinusDstColor    = 5,
    SrcAlpha            = 6,
    OneMinusSrcAlpha    = 7,
    DstAlpha            = 8,
    OneMinusDstAlpha    = 9,
    SrcAlphaSaturate    = 10,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CompositeOp {
    SrcOver = 0,
    SrcIn   = 1,
    SrcOut  = 2,
    Atop    = 3,
    DstOver = 4,
    DstIn   = 5,
    DstOut  = 6,
    DstAtop = 7,
    Lighter = 8,
    Copy    = 9,
    Xor     = 10,
}

#[derive(Clone, Copy)]
pub struct CompositeState {
    pub src_color: BlendFactor,
    pub dst_color: BlendFactor,
    pub src_alpha: BlendFactor,
    pub dst_alpha: BlendFactor,
}

impl From<CompositeOp> for CompositeState {
    fn from(op: CompositeOp) -> Self {
        use BlendFactor::*;

        let (sfactor, dfactor) = match op {
            CompositeOp::SrcOver    => (One, OneMinusSrcAlpha),
            CompositeOp::SrcIn      => (DstAlpha, Zero),
            CompositeOp::SrcOut     => (OneMinusDstAlpha, Zero),
            CompositeOp::Atop       => (DstAlpha, OneMinusSrcAlpha),
            CompositeOp::DstOver    => (OneMinusDstAlpha, One),
            CompositeOp::DstIn      => (Zero, SrcAlpha),
            CompositeOp::DstOut     => (Zero, OneMinusSrcAlpha),
            CompositeOp::DstAtop    => (OneMinusDstAlpha, SrcAlpha),
            CompositeOp::Lighter    => (One, One),
            CompositeOp::Copy       => (One, Zero),
            CompositeOp::Xor        => (OneMinusDstAlpha, OneMinusSrcAlpha),
        };

        Self {
            src_color: sfactor,
            dst_color: dfactor,
            src_alpha: sfactor,
            dst_alpha: dfactor,
        }
    }
}