#![warn(clippy::all)]
#![allow(unstable_name_collisions)] // TODO: clamp

pub use wgpu;

#[cfg(feature = "standalone")]
pub mod app;

mod canvas;
mod math;
mod paint;
mod path;
mod picture;
mod renderer;
mod shader;
mod tessellator;
mod valloc;

pub use self::{
    canvas::Canvas,
    math::{Color, Corners, Offset, RRect, Rect, Transform},
    paint::{Gradient, LineCap, LineJoin, Paint, PaintingStyle, Stroke},
    path::{Path, Winding},
    renderer::{Renderer, Target},
    picture::Picture,
};
