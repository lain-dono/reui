use crate::{
    internals::{Batch, Draw, GpuBatch, IntoPaint, Tessellator, Vertex},
    FillRule, Images, LineJoin, Paint, Path, Pipeline, Rect, Transform,
};

#[derive(Clone, Copy, Debug)]
pub enum DrawCall<Key> {
    Indexed {
        start: u32,
        end: u32,
        base_vertex: i32,
        instance: u32,
    },
    BindImage(Key),

    Convex,
    ConvexSimple,
    FillStencil,
    FillFringesNonZero,
    FillFringesEvenOdd,
    FillQuadNonZero,
    FillQuadEvenOdd,
    ImagePremultiplied,
    ImageUnmultiplied,
    ImageFont,

    Stroke {
        start: u32,
        end: u32,
        base_vertex: i32,
        instance: u32,
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
    pub fn new<Key: Eq + std::hash::Hash>(
        device: &wgpu::Device,
        viewport: &wgpu::BindGroup,
        offset: u32,
        pipeline: &Pipeline,
        batch: &GpuBatch,
        images: &Images<Key>,
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
                &DrawCall::Indexed {
                    start,
                    end,
                    base_vertex,
                    instance,
                } => {
                    rpass.draw_indexed(start..end, base_vertex, instance..instance + 1);
                }

                DrawCall::BindImage(image) => rpass.set_bind_group(1, &images[image].bind, &[]),

                DrawCall::Convex => rpass.set_pipeline(&pipeline.convex),
                DrawCall::ConvexSimple => rpass.set_pipeline(&pipeline.convex_simple),

                DrawCall::FillStencil => rpass.set_pipeline(&pipeline.fill_stencil),
                DrawCall::FillQuadNonZero => rpass.set_pipeline(&pipeline.fill_quad_non_zero),
                DrawCall::FillQuadEvenOdd => rpass.set_pipeline(&pipeline.fill_quad_even_odd),
                DrawCall::FillFringesNonZero => rpass.set_pipeline(&pipeline.fringes_non_zero),
                DrawCall::FillFringesEvenOdd => rpass.set_pipeline(&pipeline.fringes_even_odd),
                DrawCall::ImagePremultiplied => rpass.set_pipeline(&pipeline.premultiplied),
                DrawCall::ImageUnmultiplied => rpass.set_pipeline(&pipeline.unmultiplied),
                DrawCall::ImageFont => rpass.set_pipeline(&pipeline.font),

                &DrawCall::Stroke {
                    start,
                    end,
                    base_vertex,
                    instance,
                } => {
                    // Fill the stroke base without overlap
                    rpass.set_pipeline(&pipeline.stroke_base);
                    rpass.draw_indexed(start..end, base_vertex, instance..instance + 1);

                    // FringesNonZero
                    rpass.set_pipeline(&pipeline.fringes_non_zero);
                    rpass.draw_indexed(start..end, base_vertex, instance + 1..instance + 2);

                    // Clear stencil buffer
                    rpass.set_pipeline(&pipeline.stroke_stencil);
                    rpass.draw_indexed(start..end, base_vertex, 0..1);
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
    pub fn clear(&mut self) {
        self.calls.clear();
        self.batch.clear();
        self.cache.clear();
    }

    pub fn stroke(
        &mut self,
        path: &Path,
        paint: impl Into<Paint>,
        xform: Transform,
        antialias: bool,
    ) {
        let paint = paint.into();
        let mut stroke = paint.stroke;

        let mut raw_paint = paint.into_paint(xform);

        let average_scale = (xform.re * xform.re + xform.im * xform.im).sqrt();
        stroke.width = (stroke.width * average_scale).max(0.0);

        let fringe_width = 1.0;

        if stroke.width < fringe_width {
            // If the stroke width is less than pixel size, use alpha to emulate coverage.
            // Since coverage is area, scale by alpha*alpha.
            let alpha = (stroke.width / fringe_width).clamp(0.0, 1.0);
            let coverage = alpha * alpha;
            raw_paint.inner_color.alpha *= coverage;
            raw_paint.outer_color.alpha *= coverage;
            stroke.width = fringe_width;
        }

        let fringe_width = if antialias { fringe_width } else { 0.0 };

        let commands = path.transform_iter(xform);
        let tess_tol = 0.25;
        self.cache.flatten(commands, tess_tol, 0.01);

        stroke.width *= 0.5;

        let base_vertex = self.batch.base_vertex();
        let indices = self
            .cache
            .expand_stroke(&mut self.batch, stroke, fringe_width, tess_tol);

        let stroke_thr = 1.0 - 0.5 / 255.0;
        let first = raw_paint.to_instance(stroke.width, fringe_width, stroke_thr);
        let instance = self.batch.instance(first);

        let second = raw_paint.to_instance(stroke.width, fringe_width, -1.0);
        let _ = self.batch.instance(second);

        self.calls.push(DrawCall::Stroke {
            start: indices.start,
            end: indices.end,
            base_vertex,
            instance,
        });
    }

    pub fn fill(
        &mut self,
        path: &Path,
        paint: impl IntoPaint,
        xform: Transform,
        fill_rule: FillRule,
        antialias: bool,
    ) {
        let paint = paint.into_paint(xform);

        let fringe_width = if antialias { 1.0 } else { 0.0 };

        // Setup uniforms for draw calls
        let raw = paint.to_instance(fringe_width, fringe_width, -1.0);
        let instance = self.batch.instance(raw);

        let commands = path.transform_iter(xform);
        self.cache.flatten(commands, 0.25, 0.01);

        let draw = self
            .cache
            .expand_fill(&mut self.batch, fringe_width, LineJoin::Miter, 2.4);

        match draw {
            // Bounding box fill quad not needed for convex fill
            Draw::Convex {
                base_vertex,
                start,
                end,
            } => {
                if paint.inner_color == paint.outer_color {
                    if !matches!(self.calls.last(), Some(DrawCall::ConvexSimple)) {
                        self.calls.push(DrawCall::ConvexSimple);
                    }
                } else if !matches!(self.calls.last(), Some(DrawCall::Convex)) {
                    self.calls.push(DrawCall::Convex);
                }

                self.calls.push(DrawCall::Indexed {
                    start,
                    end,
                    base_vertex,
                    instance,
                });
            }
            Draw::Concave {
                base_vertex,
                fill,
                stroke,
                quad,
            } => {
                self.calls.push(DrawCall::FillStencil);
                self.calls.push(DrawCall::Indexed {
                    start: fill.start,
                    end: fill.end,
                    base_vertex,
                    instance: 0,
                });

                self.calls.push(match fill_rule {
                    FillRule::NonZero => DrawCall::FillFringesNonZero,
                    FillRule::EvenOdd => DrawCall::FillFringesEvenOdd,
                });

                self.calls.push(DrawCall::Indexed {
                    start: stroke.start,
                    end: stroke.end,
                    base_vertex,
                    instance,
                });

                self.calls.push(match fill_rule {
                    FillRule::NonZero => DrawCall::FillQuadNonZero,
                    FillRule::EvenOdd => DrawCall::FillQuadEvenOdd,
                });
                self.calls.push(DrawCall::Indexed {
                    start: quad.start,
                    end: quad.end,
                    base_vertex,
                    instance,
                });
            }
        }
    }

    pub fn blit_premultiplied(&mut self, rect: Rect, transform: Transform, image: Key) {
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
        self.calls.push(DrawCall::BindImage(image));
        self.calls.push(DrawCall::ImagePremultiplied);
        self.calls.push(DrawCall::Indexed {
            start: indices.start,
            end: indices.end,
            base_vertex,
            instance: 0,
        });
    }

    pub fn blit_unmultiplied(&mut self, rect: Rect, transform: Transform, image: Key) {
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
        self.calls.push(DrawCall::BindImage(image));
        self.calls.push(DrawCall::ImageUnmultiplied);
        self.calls.push(DrawCall::Indexed {
            start: indices.start,
            end: indices.end,
            base_vertex,
            instance: 0,
        });
    }

    pub fn blit_font(&mut self, rect: Rect, transform: Transform, image: Key) {
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
        self.calls.push(DrawCall::BindImage(image));
        self.calls.push(DrawCall::ImageFont);
        self.calls.push(DrawCall::Indexed {
            start: indices.start,
            end: indices.end,
            base_vertex,
            instance: 0,
        });
    }
}
