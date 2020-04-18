#![feature(ptr_offset_from, decl_macro, const_fn, clamp)]
#![warn(clippy::all)]

mod context;
mod draw_api;

pub mod cache;
pub mod vg;

pub mod backend;
pub mod canvas;
pub mod math;

pub use crate::{
    cache::{LineCap, LineJoin, Winding},
    context::Context,
    vg::Paint,
};
