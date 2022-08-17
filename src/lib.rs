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

mod batch;
mod canvas;
mod color;
mod geom;
mod image;
mod paint;
mod path;
mod picture;
mod pipeline;
mod renderer;
mod tessellator;

#[cfg(feature = "bevy")]
#[cfg_attr(docsrs, doc(cfg(feature = "bevy")))]
pub mod plugin;

pub use crate::{
    canvas::Canvas,
    color::Color,
    geom::{Offset, Rect, Rounding, Transform},
    image::Images,
    paint::{BoxGradient, LineCap, LineJoin, LinearGradient, Paint, RadialGradient, Stroke},
    path::{Command, FillRule, Path, Solidity},
    picture::{Picture, Recorder},
    pipeline::Pipeline,
    renderer::{Image, Renderer},
};

pub mod internals {
    pub use crate::{
        batch::{Batch, GpuBatch},
        canvas::TransformStack,
        image::ImageBind,
        paint::{IntoPaint, RawPaint},
        path::{PathIter, PathTransformIter},
        picture::DrawCall,
        pipeline::{Instance, Vertex},
        tessellator::{Draw, Tessellator},
    };
}

pub fn render_pictures<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    color_view: &'a wgpu::TextureView,
    depth_view: &'a wgpu::TextureView,
    bundles: impl IntoIterator<Item = &'a wgpu::RenderBundle>,
    clear_color: Option<wgpu::Color>,
    clear: bool,
) {
    let cload = clear_color.map_or(wgpu::LoadOp::Load, wgpu::LoadOp::Clear);
    let dload = clear.then_some(wgpu::LoadOp::Load).unwrap_or_default();
    let sload = clear.then_some(wgpu::LoadOp::Load).unwrap_or_default();
    let store = true;

    let desc = wgpu::RenderPassDescriptor {
        label: Some("reui"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: color_view,
            resolve_target: None,
            ops: wgpu::Operations { load: cload, store },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations { load: dload, store }),
            stencil_ops: Some(wgpu::Operations { load: sload, store }),
        }),
    };

    encoder.begin_render_pass(&desc).execute_bundles(bundles)
}

pub fn combine_viewport(width: u32, height: u32) -> [f32; 4] {
    let w = f32::recip(width as f32);
    let h = f32::recip(height as f32);
    [w, h, w, h]
}
