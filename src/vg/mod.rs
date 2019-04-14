mod composite;
mod paint;
mod color;
mod scissor;
mod counters;

pub mod utils;

pub use self::{
    composite::{CompositeState, CompositeOp, BlendFactor},
    paint::Paint,
    color::Color,
    scissor::Scissor,
    counters::Counters,
};

#[repr(transparent)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Image(pub(crate) u32);

impl From<u32> for Image {
    fn from(id: u32) -> Self {
        Image(id)
    }
}
