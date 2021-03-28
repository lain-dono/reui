#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::must_use_candidate,
    clippy::range_plus_one,
    clippy::module_name_repetitions,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::items_after_statements,
    clippy::too_many_lines,
    clippy::too_many_arguments
)]

pub use wgpu;

#[cfg(feature = "standalone")]
pub mod app;

mod canvas;
mod math;
mod paint;
mod path;
mod picture;
mod renderer;
mod tessellator;
mod valloc;

pub use self::{
    canvas::Canvas,
    math::{Color, Corners, Offset, RRect, Rect, Transform},
    paint::{Gradient, LineCap, LineJoin, Paint, PaintingStyle, Stroke},
    path::{Path, Solidity, Winding},
    picture::{PictureBundle, PictureRecorder},
    renderer::Renderer,
};
