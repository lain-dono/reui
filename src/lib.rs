#![feature(ptr_offset_from, decl_macro, const_fn, const_int_conversion, clamp)]
#![warn(clippy::all)]

mod vg;
mod backend;
mod font;
mod cache;
mod draw_api;
mod context;

pub mod math;
pub mod perf;
pub mod canvas;

pub use crate::{
    cache::{Winding, LineJoin, LineCap},
    backend::{BackendGL, NFlags, Image, ImageFlags, gl},
    vg::{Paint, utils},
    context::Context,
    font::{Align, TextRow, GlyphPosition},
};

pub mod backend_wgpu;