#![warn(clippy::all)]
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
    geom::{Offset, Rect, Rounding, Size, Transform},
    paint::{Gradient, LineCap, LineJoin, Paint, PaintingStyle},
    path::{Path, PathIter, PathTransformIter, Solidity},
    picture::{Picture, Recorder},
    renderer::Renderer,
    tessellator::Tessellator,
    viewport::{Target, TargetDescriptor, Viewport},
};

use std::ops::{Add, Div, Mul};

pub trait Bezier32:
    Copy
    + Mul<f32, Output = Self>
    + Mul<Self, Output = Self>
    + Add<Self, Output = Self>
    + Div<Self, Output = Self>
{
    #[inline(always)]
    fn core2([a, b, c]: [Self; 3], t: f32) -> [Self; 3] {
        let h = 1.0 - t;
        let a = a * h * h;
        let b = b * h * t * 2.0;
        let c = c * t * t;
        [a, b, c]
    }

    #[inline(always)]
    fn core3(w: [Self; 4], t: f32) -> [Self; 4] {
        let h = 1.0 - t;

        let tt = t * t;
        let hh = h * h;

        let a = hh * h;
        let b = hh * t;
        let c = tt * h;
        let d = tt * t;

        let a = w[0] * a;
        let b = w[1] * b * 3.0;
        let c = w[2] * c * 3.0;
        let d = w[3] * d;

        [a, b, c, d]
    }

    #[inline]
    fn bezier2(w: [Self; 3], t: f32) -> Self {
        let [a, b, c] = Self::core2(w, t);
        a + b + c
    }

    #[inline]
    fn bezier3(w: [Self; 4], t: f32) -> Self {
        let [a, b, c, d] = Self::core3(w, t);
        a + b + c + d
    }

    #[inline]
    fn rational2(r: [Self; 3], w: [Self; 3], t: f32) -> Self {
        let [a, b, c] = Self::core2(w, t);
        (a * r[0] + b * r[1] + c * r[2]) / (a + b + c)
    }

    #[inline]
    fn rational3(r: [Self; 4], w: [Self; 4], t: f32) -> Self {
        let [a, b, c, d] = Self::core3(w, t);
        (a * r[0] + b * r[1] + c * r[2] + d * r[3]) / (a + b + c + d)
    }
}

impl<T> Bezier32 for T where
    T: Copy
        + Mul<f32, Output = Self>
        + Mul<Self, Output = Self>
        + Add<Self, Output = Self>
        + Div<Self, Output = Self>
{
}
