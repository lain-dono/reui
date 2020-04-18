#![feature(ptr_offset_from, decl_macro, const_fn, clamp)]
#![warn(clippy::all)]

mod backend;
mod cache;
mod context;
mod draw_api;
mod vg;

pub mod canvas;
pub mod math;

pub use crate::{
    backend::{gl, BackendGL},
    cache::{LineCap, LineJoin, Winding},
    context::Context,
    vg::Paint,
};
