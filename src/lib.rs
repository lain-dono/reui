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
    clippy::too_many_arguments,
    clippy::cast_lossless
)]

pub use wgpu;

mod canvas;
mod color;
mod geom;
mod paint;
mod path;
mod picture;
mod pipeline;
mod renderer;
mod tessellator;
mod upload_buffer;

mod viewport;

pub mod image;
pub mod text;

pub use self::{
    canvas::Canvas,
    color::Color,
    geom::{Corners, Offset, RRect, Rect, Size, Transform},
    paint::{Gradient, LineCap, LineJoin, Paint, PaintingStyle},
    path::{Path, PathIter, PathTransformIter, Solidity},
    picture::{Picture, Recorder},
    renderer::Renderer,
    tessellator::Tessellator,
    viewport::{Target, TargetDescriptor, Viewport},
};
