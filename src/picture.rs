use crate::{
    paint::RawPaint,
    pipeline::{Batch, GpuBatch, Instance, Pipeline, Vertex},
    tessellator::{Draw, Tessellator},
    FillRule, Images, LineJoin, Paint, Path, Rect, Transform,
};

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum DrawCall<Key> {
    Convex {
        start: u32,
        end: u32,
        base_vertex: i32,
        instance: u32,
    },

    Stroke {
        start: u32,
        end: u32,
        base_vertex: i32,
        first: u32,
        second: u32,
    },

    FillStencil {
        start: u32,
        end: u32,
        base_vertex: i32,
        instance: u32,
    },

    FillFringes {
        start: u32,
        end: u32,
        base_vertex: i32,
        instance: u32,
        fill_rule: FillRule,
    },

    FillQuad {
        start: u32,
        end: u32,
        base_vertex: i32,
        instance: u32,
        fill_rule: FillRule,
    },

    Image {
        start: u32,
        end: u32,
        base_vertex: i32,
        instance: u32,
        image: Key,
    },
}

#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct Picture(pub(crate) wgpu::RenderBundle);

impl<'a> std::iter::IntoIterator for &'a Picture {
    type Item = &'a wgpu::RenderBundle;
    type IntoIter = std::iter::Once<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(&self.0)
    }
}

impl Picture {
    pub fn new<Key>(
        device: &wgpu::Device,
        viewport: &wgpu::BindGroup,
        offset: u32,
        pipeline: &Pipeline,
        batch: &GpuBatch,
        calls: &[DrawCall<Key>],
    ) -> Self {
        let mut rpass = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: Some("reui::Picture"),
            color_formats: &[Some(wgpu::TextureFormat::Bgra8UnormSrgb)],
            depth_stencil: Some(wgpu::RenderBundleDepthStencil {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_read_only: false,
                stencil_read_only: false,
            }),
            sample_count: 1,
            multiview: None,
        });

        rpass.set_bind_group(0, viewport, &[offset]);

        rpass.set_index_buffer(batch.indices.slice(..), wgpu::IndexFormat::Uint32);
        rpass.set_vertex_buffer(0, batch.vertices.slice(..));
        rpass.set_vertex_buffer(1, batch.instances.slice(..));

        for call in calls {
            match call {
                &DrawCall::Convex {
                    start,
                    end,
                    base_vertex,
                    instance,
                } => {
                    rpass.set_pipeline(&pipeline.convex);
                    rpass.draw_indexed(start..end, base_vertex, instance..instance + 1);
                }

                &DrawCall::Stroke {
                    start,
                    end,
                    base_vertex,
                    first,
                    second,
                } => {
                    // Fill the stroke base without overlap
                    rpass.set_pipeline(&pipeline.stroke_base);
                    rpass.draw_indexed(start..end, base_vertex, first..first + 1);

                    // FringesNonZero
                    rpass.set_pipeline(&pipeline.fringes_non_zero);
                    rpass.draw_indexed(start..end, base_vertex, second..second + 1);

                    // Clear stencil buffer
                    rpass.set_pipeline(&pipeline.stroke_stencil);
                    rpass.draw_indexed(start..end, base_vertex, second..second + 1);
                }

                &DrawCall::FillStencil {
                    start,
                    end,
                    base_vertex,
                    instance,
                } => {
                    rpass.set_pipeline(&pipeline.fill_stencil);
                    rpass.draw_indexed(start..end, base_vertex, instance..instance + 1);
                }

                &DrawCall::FillQuad {
                    start,
                    end,
                    base_vertex,
                    instance,
                    fill_rule,
                } => {
                    match fill_rule {
                        FillRule::NonZero => rpass.set_pipeline(&pipeline.fill_quad_non_zero),
                        FillRule::EvenOdd => rpass.set_pipeline(&pipeline.fill_quad_even_odd),
                    };
                    rpass.draw_indexed(start..end, base_vertex, instance..instance + 1);
                }

                // Draw anti-aliased pixels.
                &DrawCall::FillFringes {
                    start,
                    end,
                    base_vertex,
                    instance,
                    fill_rule,
                } => {
                    match fill_rule {
                        FillRule::NonZero => rpass.set_pipeline(&pipeline.fringes_non_zero),
                        FillRule::EvenOdd => rpass.set_pipeline(&pipeline.fringes_even_odd),
                    }
                    rpass.draw_indexed(start..end, base_vertex, instance..instance + 1);
                }

                DrawCall::Image { .. } => {
                    // nothing
                }
            }
        }

        Self(rpass.finish(&wgpu::RenderBundleDescriptor {
            label: Some("reui::Picture"),
        }))
    }
}

#[derive(Default)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct Recorder<Key> {
    pub(crate) calls: Vec<DrawCall<Key>>,
    pub(crate) batch: Batch,
    pub(crate) cache: Tessellator,
}

impl<Key> Recorder<Key> {
    pub fn new() -> Self
    where
        Key: Default,
    {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.calls.clear();
        self.batch.clear();
        self.cache.clear();
    }

    pub fn stroke_path(
        &mut self,
        path: &Path,
        paint: impl Into<Paint>,
        xform: Transform,
        scale: f32,
    ) {
        let paint = paint.into();

        let mut raw_paint = RawPaint::convert(&paint, xform);

        let average_scale = (xform.re * xform.re + xform.im * xform.im).sqrt();
        let mut stroke_width = (paint.width * average_scale).max(0.0);

        let inv_scale = scale.recip();

        let fringe_width = inv_scale;

        if stroke_width < fringe_width {
            // If the stroke width is less than pixel size, use alpha to emulate coverage.
            // Since coverage is area, scale by alpha*alpha.
            let alpha = (stroke_width / fringe_width).clamp(0.0, 1.0);
            let coverage = alpha * alpha;
            raw_paint.inner_color.alpha *= coverage;
            raw_paint.outer_color.alpha *= coverage;
            stroke_width = fringe_width;
        }

        let fringe_width = if paint.antialias { fringe_width } else { 0.0 };

        let commands = path.transform_iter(xform);
        let tess_tol = 0.25 * inv_scale;
        self.cache.flatten(commands, tess_tol, 0.01 * inv_scale);

        let base_vertex = self.batch.base_vertex();
        let indices = self.cache.expand_stroke(
            &mut self.batch,
            stroke_width * 0.5,
            fringe_width,
            paint.cap_start,
            paint.cap_end,
            paint.join,
            paint.miter,
            tess_tol,
        );

        let stroke_thr = 1.0 - 0.5 / 255.0;
        let first = raw_paint.to_instance(stroke_width, fringe_width, stroke_thr);
        let first = self.batch.instance(first);

        let second = raw_paint.to_instance(stroke_width, fringe_width, -1.0);
        let second = self.batch.instance(second);

        self.calls.push(DrawCall::Stroke {
            start: indices.start,
            end: indices.end,
            base_vertex,
            first,
            second,
        });
    }

    pub fn fill_path(
        &mut self,
        path: &Path,
        paint: impl Into<Paint>,
        xform: Transform,
        scale: f32,
        fill_rule: FillRule,
    ) {
        let paint = paint.into();

        let raw_paint = RawPaint::convert(&paint, xform);

        let inv_scale = scale.recip();

        let fringe_width = if paint.antialias { inv_scale } else { 0.0 };

        // Setup uniforms for draw calls
        let raw = raw_paint.to_instance(fringe_width, fringe_width, -1.0);
        let instance = self.batch.instance(raw);

        let commands = path.transform_iter(xform);
        let tess_tol = 0.25 * inv_scale;
        self.cache.flatten(commands, tess_tol, 0.01 * inv_scale);

        let draw = self
            .cache
            .expand_fill(&mut self.batch, fringe_width, LineJoin::Miter, 2.4);

        match draw {
            // Bounding box fill quad not needed for convex fill
            Draw::Convex {
                base_vertex,
                start,
                end,
            } => self.calls.push(DrawCall::Convex {
                start,
                end,
                base_vertex,
                instance,
            }),
            Draw::Concave {
                base_vertex,
                fill,
                stroke,
                quad,
            } => {
                self.calls.push(DrawCall::FillStencil {
                    start: fill.start,
                    end: fill.end,
                    base_vertex,
                    instance,
                });
                self.calls.push(DrawCall::FillFringes {
                    start: stroke.start,
                    end: stroke.end,
                    base_vertex,
                    instance,
                    fill_rule,
                });
                self.calls.push(DrawCall::FillQuad {
                    start: quad.start,
                    end: quad.end,
                    base_vertex,
                    instance,
                    fill_rule,
                });
            }
        }
    }

    pub fn draw_image(&mut self, rect: Rect, transform: Transform, image: Key, color: [u8; 4]) {
        let Rect { min, max } = rect;
        let base_vertex = self.batch.base_vertex();
        let indices = self.batch.push_strip(
            0,
            &[
                Vertex::new([max.x, max.y], [1.0, 1.0]).transform(transform),
                Vertex::new([max.x, min.y], [1.0, 0.0]).transform(transform),
                Vertex::new([min.x, max.y], [0.0, 1.0]).transform(transform),
                Vertex::new([min.x, min.y], [0.0, 0.0]).transform(transform),
            ],
        );

        let instance = self.batch.instance(Instance::image(color));

        self.calls.push(DrawCall::Image {
            start: indices.start,
            end: indices.end,
            base_vertex,
            instance,
            image,
        });
    }

    pub fn finish(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        staging_belt: &mut wgpu::util::StagingBelt,
        device: &wgpu::Device,
        batch: &mut GpuBatch,
        pipeline: &Pipeline,
        viewport: &wgpu::BindGroup,
        images: &Images<Key>,
    ) -> Picture
    where
        Key: Copy + Eq + std::hash::Hash,
    {
        batch.upload_staging(encoder, staging_belt, device, &self.batch);

        let mut rpass = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: Some("reui::Picture bundle encoder"),
            color_formats: &[Some(wgpu::TextureFormat::Bgra8UnormSrgb)],
            depth_stencil: Some(wgpu::RenderBundleDepthStencil {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_read_only: false,
                stencil_read_only: false,
            }),
            sample_count: 1,
            multiview: None,
        });

        rpass.set_bind_group(0, viewport, &[0]);

        batch.bind(&mut rpass);

        encode(&mut rpass, self.calls.iter().cloned(), pipeline, images);

        Picture(rpass.finish(&wgpu::RenderBundleDescriptor {
            label: Some("reui::Picture"),
        }))
    }
}

fn encode<'a, Key: Eq + std::hash::Hash>(
    rpass: &mut impl wgpu::util::RenderEncoder<'a>,
    calls: impl IntoIterator<Item = DrawCall<Key>>,
    pipeline: &'a Pipeline,
    images: &'a Images<Key>,
) {
    for call in calls {
        match call {
            DrawCall::Convex {
                start,
                end,
                base_vertex,
                instance,
            } => {
                rpass.set_pipeline(&pipeline.convex);
                rpass.draw_indexed(start..end, base_vertex, instance..instance + 1);
            }

            DrawCall::Stroke {
                start,
                end,
                base_vertex,
                first,
                second,
            } => {
                // Fill the stroke base without overlap
                rpass.set_pipeline(&pipeline.stroke_base);
                rpass.draw_indexed(start..end, base_vertex, first..first + 1);

                // FringesNonZero
                rpass.set_pipeline(&pipeline.fringes_non_zero);
                rpass.draw_indexed(start..end, base_vertex, second..second + 1);

                // Clear stencil buffer
                rpass.set_pipeline(&pipeline.stroke_stencil);
                rpass.draw_indexed(start..end, base_vertex, second..second + 1);
            }

            DrawCall::FillStencil {
                start,
                end,
                base_vertex,
                instance,
            } => {
                rpass.set_pipeline(&pipeline.fill_stencil);
                rpass.draw_indexed(start..end, base_vertex, instance..instance + 1);
            }

            DrawCall::FillQuad {
                start,
                end,
                base_vertex,
                instance,
                fill_rule,
            } => {
                match fill_rule {
                    FillRule::NonZero => rpass.set_pipeline(&pipeline.fill_quad_non_zero),
                    FillRule::EvenOdd => rpass.set_pipeline(&pipeline.fill_quad_even_odd),
                };
                rpass.draw_indexed(start..end, base_vertex, instance..instance + 1);
            }

            // Draw anti-aliased pixels.
            DrawCall::FillFringes {
                start,
                end,
                base_vertex,
                instance,
                fill_rule,
            } => {
                match fill_rule {
                    FillRule::NonZero => rpass.set_pipeline(&pipeline.fringes_non_zero),
                    FillRule::EvenOdd => rpass.set_pipeline(&pipeline.fringes_even_odd),
                }
                rpass.draw_indexed(start..end, base_vertex, instance..instance + 1);
            }

            DrawCall::Image {
                start,
                end,
                base_vertex,
                instance,
                image,
            } => {
                rpass.set_pipeline(&pipeline.image);
                rpass.set_bind_group(1, &images[image].bind_group, &[]);
                rpass.draw_indexed(start..end, base_vertex, instance..instance + 1);
            }
        }
    }
}
