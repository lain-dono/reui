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
mod image;
mod paint;
mod path;
mod picture;
mod pipeline;
mod tessellator;
mod upload_buffer;

mod viewport;

#[cfg(feature = "bevy")]
pub mod integration;

//pub mod image;
pub mod text;

pub use self::{
    canvas::Canvas,
    color::Color,
    geom::{Offset, Rect, Rounding, Transform},
    image::Images,
    paint::{Gradient, LineCap, LineJoin, Paint, PaintingStyle},
    path::{FillRule, Path, PathIter, PathTransformIter, Solidity},
    picture::{Picture, Recorder},
    pipeline::{BatchUpload, Pipeline},
    tessellator::Tessellator,
    viewport::{Target, Viewport},
};

pub fn render_pictures<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    color_view: &'a wgpu::TextureView,
    depth_view: &'a wgpu::TextureView,
    render_bundles: impl IntoIterator<Item = &'a wgpu::RenderBundle>,
) {
    encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("reui"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                }),
            }),
        })
        .execute_bundles(render_bundles)
}
