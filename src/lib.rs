#![feature(ptr_offset_from, decl_macro, const_fn, const_int_conversion)]

#![warn(clippy::all)]

#![feature(extern_types)]

mod api;
pub mod perf;
mod vg;
mod backend;
mod fons;
mod font;
mod cache;
mod draw_api;
mod transform;
mod context;

mod fff;

//mod wgpu;

pub use crate::{
    cache::{Winding, LineJoin, LineCap},
    backend::{BackendGL, NFlags, Image, ImageFlags, gl},
    vg::{
        Paint,
        Color,
        utils,
    },
    context::{Context, Align, TextRow, GlyphPosition},
};
