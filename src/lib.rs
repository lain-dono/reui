#![warn(clippy::all)]
#![allow(unstable_name_collisions)] // TODO: clamp

mod backend;
mod cache;
mod canvas;
mod math;
mod paint;
mod path;
mod picture;
mod pipeline;
mod recorder;
mod shader;
mod state;

pub use self::{
    backend::Renderer,
    canvas::{Canvas, Winding},
    math::{Color, Offset, RRect, Rect, Transform},
    paint::{Gradient, Paint, PaintingStyle, StrokeCap, StrokeJoin},
    path::Path,
    pipeline::Target,
};

#[doc(hidden)]
pub use self::math::PartialClamp;
