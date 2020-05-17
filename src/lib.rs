#![warn(clippy::all)]
#![allow(unstable_name_collisions)] // TODO: clamp

pub mod backend;
pub mod cache;

mod canvas;
mod math;
mod state;

pub use self::{
    backend::Renderer,
    canvas::{Canvas, Gradient, Paint, PaintingStyle, Path, StrokeCap, StrokeJoin, Winding},
    math::{Color, Offset, RRect, Rect, Transform},
};

#[doc(hidden)]
pub use self::math::PartialClamp;
