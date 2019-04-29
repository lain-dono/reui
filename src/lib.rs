#![feature(ptr_offset_from, decl_macro, const_fn, const_int_conversion)]

#![allow(improper_ctypes)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

#![warn(clippy::all)]

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