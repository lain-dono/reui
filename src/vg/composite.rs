use crate::context::Context;

#[repr(i32)]
#[derive(Clone, Copy)]
pub enum BlendFactor {
    Zero                  = 0,
    One                   = 1,
    SrcColor             = 2,
    OneMinusSrcColor   = 3,
    DstColor             = 4,
    OneMinusDstColor   = 5,
    SrcAlpha             = 6,
    OneMinusSrcAlpha   = 7,
    DstAlpha             = 8,
    OneMinusDstAlpha   = 9,
    SrcAlphaSaturate    = 10,
}

#[repr(i32)]
#[derive(Clone, Copy)]
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CompositeState {
    pub src_color: BlendFactor,
    pub dst_color: BlendFactor,
    pub src_alpha: BlendFactor,
    pub dst_alpha: BlendFactor,
}

impl Context {
    pub fn global_composite(&mut self, op: CompositeOp) {
        self.states.last_mut().composite = op.into();
    }

    pub fn global_blend(&mut self, sfactor: BlendFactor, dfactor: BlendFactor) {
        self.global_blend_separate(sfactor, dfactor, sfactor, dfactor);
    }

    pub fn global_blend_separate(
        &mut self,
        src_color: BlendFactor,
        dst_color: BlendFactor,
        src_alpha: BlendFactor,
        dst_alpha: BlendFactor,
    ) {
        self.states.last_mut().composite = CompositeState {
            src_color,
            dst_color,
            src_alpha,
            dst_alpha,
        };
    }
}

impl From<CompositeOp> for CompositeState {
    fn from(op: CompositeOp) -> Self {
        composite_operation_state(op as i32)
    }
}

pub fn composite_operation_state(op: i32) -> CompositeState  {
    use CompositeOp::*;

    let (sfactor, dfactor);

    if op == SrcOver as i32 {
        sfactor = BlendFactor::One;
        dfactor = BlendFactor::OneMinusSrcAlpha;
    } else if op == SrcIn as i32 {
        sfactor = BlendFactor::DstAlpha;
        dfactor = BlendFactor::Zero;
    } else if op == SrcOut as i32 {
        sfactor = BlendFactor::OneMinusDstAlpha;
        dfactor = BlendFactor::Zero;
    } else if op == Atop as i32 {
        sfactor = BlendFactor::DstAlpha;
        dfactor = BlendFactor::OneMinusSrcAlpha;
    } else if op == DstOver as i32 {
        sfactor = BlendFactor::OneMinusDstAlpha;
        dfactor = BlendFactor::One;
    } else if op == DstIn as i32 {
        sfactor = BlendFactor::Zero;
        dfactor = BlendFactor::SrcAlpha;
    } else if op == DstOut as i32 {
        sfactor = BlendFactor::Zero;
        dfactor = BlendFactor::OneMinusSrcAlpha;
    } else if op == DstAtop as i32 {
        sfactor = BlendFactor::OneMinusDstAlpha;
        dfactor = BlendFactor::SrcAlpha;
    } else if op == Lighter as i32 {
        sfactor = BlendFactor::One;
        dfactor = BlendFactor::One;
    } else if op == Copy as i32 {
        sfactor = BlendFactor::One;
        dfactor = BlendFactor::Zero;
    } else if op == Xor as i32 {
        sfactor = BlendFactor::OneMinusDstAlpha;
        dfactor = BlendFactor::OneMinusSrcAlpha;
    } else {
        sfactor = BlendFactor::One;
        dfactor = BlendFactor::Zero;
    }

    CompositeState {
        src_color: sfactor,
        dst_color: dfactor,
        src_alpha: sfactor,
        dst_alpha: dfactor,
    }
}
