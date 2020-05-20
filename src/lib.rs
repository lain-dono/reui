#![warn(clippy::all)]
#![allow(unstable_name_collisions)] // TODO: clamp

mod cache;
mod canvas;
mod math;
mod paint;
mod path;
mod picture;
mod shader;
mod valloc;

pub use self::{
    canvas::Canvas,
    math::{Color, Offset, RRect, Rect, Transform},
    paint::{Gradient, Paint, PaintingStyle, StrokeCap, StrokeJoin},
    path::{Path, Winding},
    picture::{Renderer, Target},
};

#[doc(hidden)]
pub use self::math::PartialClamp;
