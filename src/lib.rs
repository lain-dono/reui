#![warn(clippy::all)]

pub mod backend;
pub mod cache;
pub mod context;
pub mod math;

mod canvas;
mod state;

pub use self::{
    canvas::{Canvas, Gradient, Paint, PaintingStyle, Path, Winding, StrokeCap, StrokeJoin},
    math::{rect, Color, Offset, RRect, Rect, Transform},
};
