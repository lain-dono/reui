#![feature(ptr_offset_from, decl_macro, const_fn, const_int_conversion, clamp)]

#![warn(clippy::all)]

#![feature(extern_types)]

//mod api;
pub mod perf;
mod vg;
mod backend;
mod font;
mod cache;
mod draw_api;
mod context;
mod images;

mod picture;

pub mod canvas;

pub use euclid;


//mod wgpu;

pub use crate::{
    cache::{Winding, LineJoin, LineCap},
    backend::{BackendGL, NFlags, Image, ImageFlags, gl},
    vg::{Paint, Color, utils},
    context::Context,
    font::{Align, TextRow, GlyphPosition},
};

pub use euclid::{
    rect, vec2, point2, size2,
};

use euclid::UnknownUnit;

pub type Transform = euclid::Transform2D<f32, UnknownUnit, UnknownUnit>;
pub type Point = euclid::Point2D<f32, UnknownUnit>;
pub type Vector = euclid::Vector2D<f32, UnknownUnit>;
pub type Size = euclid::Size2D<f32, UnknownUnit>;
pub type Rect = euclid::Rect<f32, UnknownUnit>;
pub type Bounds = euclid::Box2D<f32, UnknownUnit>;

pub type SideOffset = euclid::SideOffsets2D<f32, UnknownUnit>;

#[inline(always)]
pub fn transform_pt(pt: &mut [f32], t: &Transform) {
    let p = t.transform_point(point2(pt[0], pt[1]));
    pt[0] = p.x;
    pt[1] = p.y;
}

#[inline(always)]
pub fn transform_point(t: &Transform, x: f32, y: f32) -> [f32; 2] {
    t.transform_point(point2(x, y)).to_array()
}

impl Context {
    pub fn transform(&mut self, m: [f32; 6]) {
        self.pre_transform(Transform::from_row_major_array(m))
    }

    pub fn current_transform(&self) -> &Transform {
        &self.states.last().xform
    }
    pub fn pre_transform(&mut self, m: Transform) {
        let t = &mut self.states.last_mut().xform;
        *t = t.pre_transform(&m);
    }
    pub fn post_transform(&mut self, m: Transform) {
        let t = &mut self.states.last_mut().xform;
        *t = t.post_transform(&m);
    }
    pub fn reset_transform(&mut self) {
        self.states.last_mut().xform = Transform::identity();
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.pre_transform(Transform::create_translation(x, y));
    }
    pub fn rotate(&mut self, angle: f32) {
        let angle = euclid::Angle::radians(angle);
        self.pre_transform(Transform::create_rotation(angle));
    }
    pub fn scale(&mut self, x: f32, y: f32) {
        self.pre_transform(Transform::create_scale(x, y));
    }
}