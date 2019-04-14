use crate::context::Context;

#[repr(i32)]
#[derive(Clone, Copy)]
pub enum BlendFactor {
    ZERO                  = 0,
    ONE                   = 1,
    SRC_COLOR             = 2,
    ONE_MINUS_SRC_COLOR   = 3,
    DST_COLOR             = 4,
    ONE_MINUS_DST_COLOR   = 5,
    SRC_ALPHA             = 6,
    ONE_MINUS_SRC_ALPHA   = 7,
    DST_ALPHA             = 8,
    ONE_MINUS_DST_ALPHA   = 9,
    SRC_ALPHA_SATURATE    = 10,
}

#[repr(i32)]
#[derive(Clone, Copy)]
pub enum CompositeOp {
    SOURCE_OVER     = 0,
    SOURCE_IN       = 1,
    SOURCE_OUT      = 2,
    ATOP            = 3,
    DESTINATION_OVER= 4,
    DESTINATION_IN  = 5,
    DESTINATION_OUT = 6,
    DESTINATION_ATOP= 7,
    LIGHTER         = 8,
    COPY            = 9,
    XOR             = 10,
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

    if op == SOURCE_OVER as i32 {
        sfactor = BlendFactor::ONE;
        dfactor = BlendFactor::ONE_MINUS_SRC_ALPHA;
    } else if op == SOURCE_IN as i32 {
        sfactor = BlendFactor::DST_ALPHA;
        dfactor = BlendFactor::ZERO;
    } else if op == SOURCE_OUT as i32 {
        sfactor = BlendFactor::ONE_MINUS_DST_ALPHA;
        dfactor = BlendFactor::ZERO;
    } else if op == ATOP as i32 {
        sfactor = BlendFactor::DST_ALPHA;
        dfactor = BlendFactor::ONE_MINUS_SRC_ALPHA;
    } else if op == DESTINATION_OVER as i32 {
        sfactor = BlendFactor::ONE_MINUS_DST_ALPHA;
        dfactor = BlendFactor::ONE;
    } else if op == DESTINATION_IN as i32 {
        sfactor = BlendFactor::ZERO;
        dfactor = BlendFactor::SRC_ALPHA;
    } else if op == DESTINATION_OUT as i32 {
        sfactor = BlendFactor::ZERO;
        dfactor = BlendFactor::ONE_MINUS_SRC_ALPHA;
    } else if op == DESTINATION_ATOP as i32 {
        sfactor = BlendFactor::ONE_MINUS_DST_ALPHA;
        dfactor = BlendFactor::SRC_ALPHA;
    } else if op == LIGHTER as i32 {
        sfactor = BlendFactor::ONE;
        dfactor = BlendFactor::ONE;
    } else if op == COPY as i32 {
        sfactor = BlendFactor::ONE;
        dfactor = BlendFactor::ZERO;
    } else if op == XOR as i32 {
        sfactor = BlendFactor::ONE_MINUS_DST_ALPHA;
        dfactor = BlendFactor::ONE_MINUS_SRC_ALPHA;
    } else {
        sfactor = BlendFactor::ONE;
        dfactor = BlendFactor::ZERO;
    }

    CompositeState {
        src_color: sfactor,
        dst_color: dfactor,
        src_alpha: sfactor,
        dst_alpha: dfactor,
    }
}
