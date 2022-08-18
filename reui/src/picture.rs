use crate::{
    internals::{Batch, Draw, GpuBatch, Tessellator, Vertex},
    FillRule, Images, IntoPaint, LineJoin, Path, Pipeline, Rect, Stroke, Transform,
};

#[derive(Clone, Copy, Debug)]
pub struct DrawIndexed {
    pub start: u32,
    pub end: u32,
    pub base_vertex: i32,
    pub instance: u32,
}

impl DrawIndexed {
    #[inline]
    fn new(start: u32, end: u32, base_vertex: i32, instance: u32) -> Self {
        Self {
            start,
            end,
            base_vertex,
            instance,
        }
    }

    #[inline]
    fn call<'a>(
        &self,
        rpass: &mut wgpu::RenderBundleEncoder<'a>,
        pipeline: &'a wgpu::RenderPipeline,
    ) {
        let instances = self.instance..self.instance + 1;
        rpass.set_pipeline(pipeline);
        rpass.draw_indexed(self.start..self.end, self.base_vertex, instances);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DrawCall<Key> {
    Convex(DrawIndexed),
    ConvexSimple(DrawIndexed),
    Stencil(DrawIndexed),
    FringesNonZero(DrawIndexed),
    FringesEvenOdd(DrawIndexed),
    QuadNonZero(DrawIndexed),
    QuadEvenOdd(DrawIndexed),
    ImagePremultiplied(DrawIndexed),
    ImageUnmultiplied(DrawIndexed),
    ImageFont(DrawIndexed),

    BindImage(Key),
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
                depth_read_only: true,
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
                DrawCall::BindImage(image) => rpass.set_bind_group(1, &images[image].bind, &[]),

                DrawCall::Convex(draw) => draw.call(&mut rpass, &pipeline.convex),
                DrawCall::ConvexSimple(draw) => draw.call(&mut rpass, &pipeline.convex_simple),
                DrawCall::Stencil(draw) => draw.call(&mut rpass, &pipeline.fill_stencil),
                DrawCall::QuadNonZero(draw) => draw.call(&mut rpass, &pipeline.fill_quad_non_zero),
                DrawCall::QuadEvenOdd(draw) => draw.call(&mut rpass, &pipeline.fill_quad_even_odd),
                DrawCall::FringesNonZero(draw) => draw.call(&mut rpass, &pipeline.fringes_non_zero),
                DrawCall::FringesEvenOdd(draw) => draw.call(&mut rpass, &pipeline.fringes_even_odd),
                DrawCall::ImagePremultiplied(draw) => {
                    draw.call(&mut rpass, &pipeline.premultiplied)
                }
                DrawCall::ImageUnmultiplied(draw) => draw.call(&mut rpass, &pipeline.unmultiplied),
                DrawCall::ImageFont(draw) => draw.call(&mut rpass, &pipeline.font),

                &DrawCall::Stroke {
                    start,
                    end,
                    base_vertex,
                    instance,
                } => {
                    rpass.set_pipeline(&pipeline.stroke_base);
                    rpass.draw_indexed(start..end, base_vertex, instance..instance + 1);

                    rpass.set_pipeline(&pipeline.fringes_non_zero);
                    rpass.draw_indexed(start..end, base_vertex, instance + 1..instance + 2);

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
        paint: impl IntoPaint,
        mut stroke: Stroke,
        transform: Transform,
        antialias: bool,
    ) {
        let mut paint = paint.into_paint(transform);

        let average_scale = (transform.re * transform.re + transform.im * transform.im).sqrt();
        stroke.width = (stroke.width * average_scale).max(0.0);

        let fringe_width = 1.0;

        if stroke.width < fringe_width {
            // If the stroke width is less than pixel size, use alpha to emulate coverage.
            // Since coverage is area, scale by alpha*alpha.
            let alpha = (stroke.width / fringe_width).clamp(0.0, 1.0);
            let coverage = alpha * alpha;
            paint.inner_color.alpha *= coverage;
            paint.outer_color.alpha *= coverage;
            stroke.width = fringe_width;
        }

        let fringe_width = if antialias { fringe_width } else { 0.0 };

        let commands = path.transform_iter(transform);
        let tess_tol = 0.25;
        self.cache.flatten(commands, tess_tol, 0.01);

        stroke.width *= 0.5;

        let base_vertex = self.batch.base_vertex();
        let indices = self
            .cache
            .expand_stroke(&mut self.batch, stroke, fringe_width, tess_tol);

        let stroke_thr = 1.0 - 0.5 / 255.0;
        let first = paint.to_instance(stroke.width, fringe_width, stroke_thr);
        let instance = self.batch.instance(first);

        let second = paint.to_instance(stroke.width, fringe_width, -1.0);
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
        transform: Transform,
        fill_rule: FillRule,
        antialias: bool,
    ) {
        let paint = paint.into_paint(transform);

        let fringe_width = if antialias { 1.0 } else { 0.0 };

        // Setup uniforms for draw calls
        let raw = paint.to_instance(fringe_width, fringe_width, -1.0);
        let instance = self.batch.instance(raw);

        let commands = path.transform_iter(transform);
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
                let draw = DrawIndexed::new(start, end, base_vertex, instance);
                if paint.inner_color == paint.outer_color {
                    self.calls.push(DrawCall::ConvexSimple(draw));
                } else {
                    self.calls.push(DrawCall::Convex(draw));
                }
            }
            Draw::Concave {
                base_vertex,
                fill,
                stroke,
                quad,
            } => {
                let stenicl = DrawIndexed::new(fill.start, fill.end, base_vertex, instance);

                let stroke = DrawIndexed::new(stroke.start, stroke.end, base_vertex, instance);
                let quad = DrawIndexed::new(quad.start, quad.end, base_vertex, instance);

                self.calls.push(DrawCall::Stencil(stenicl));
                self.calls.push(match fill_rule {
                    FillRule::NonZero => DrawCall::FringesNonZero(stroke),
                    FillRule::EvenOdd => DrawCall::FringesEvenOdd(stroke),
                });
                self.calls.push(match fill_rule {
                    FillRule::NonZero => DrawCall::QuadNonZero(quad),
                    FillRule::EvenOdd => DrawCall::QuadEvenOdd(quad),
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
        let draw = DrawIndexed::new(indices.start, indices.end, base_vertex, 0);
        self.calls.push(DrawCall::BindImage(image));
        self.calls.push(DrawCall::ImagePremultiplied(draw));
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
        let draw = DrawIndexed::new(indices.start, indices.end, base_vertex, 0);
        self.calls.push(DrawCall::BindImage(image));
        self.calls.push(DrawCall::ImageUnmultiplied(draw));
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
        let draw = DrawIndexed::new(indices.start, indices.end, base_vertex, 0);
        self.calls.push(DrawCall::BindImage(image));
        self.calls.push(DrawCall::ImageFont(draw));
    }
}
