#![feature(ptr_offset_from, decl_macro, const_fn, const_int_conversion, clamp)]

#![warn(clippy::all)]

#![feature(extern_types)]

//mod api;
mod vg;
mod backend;
mod font;
mod cache;
mod draw_api;
mod context;
mod images;

pub mod math;
pub mod perf;
pub mod canvas;

pub use crate::{
    cache::{Winding, LineJoin, LineCap},
    backend::{BackendGL, NFlags, Image, ImageFlags, gl},
    vg::{Paint, Color, utils},
    context::Context,
    font::{Align, TextRow, GlyphPosition},
};