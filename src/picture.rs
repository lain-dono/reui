use crate::{
    math::{Rect, Transform},
    paint::{LineJoin, Paint, RawPaint},
    path::{FillRule, PathIter},
    pipeline::{Instance, Vertex},
    renderer::Renderer,
    tessellator::{Draw, Tessellator},
};
use std::{mem::size_of, ops::Range, slice::from_raw_parts};

pub(crate) fn cast_slice<T: Sized>(slice: &[T]) -> &[u8] {
    unsafe { from_raw_parts(slice.as_ptr().cast(), slice.len() * size_of::<T>()) }
}

#[derive(Clone)]
enum DrawCall {
    Convex {
        indices: Range<u32>,
        base_vertex: i32,
        instance: u32,
    },

    FillStencil {
        indices: Range<u32>,
        base_vertex: i32,
        instance: u32,
    },
    FillQuadNonZero {
        indices: Range<u32>,
        base_vertex: i32,
        instance: u32,
    },
    FillQuadEvenOdd {
        indices: Range<u32>,
        base_vertex: i32,
        instance: u32,
    },

    StrokeBase {
        indices: Range<u32>,
        base_vertex: i32,
        instance: u32,
    },
    StrokeStencil {
        indices: Range<u32>,
        base_vertex: i32,
        instance: u32,
    },

    FringesNonZero {
        indices: Range<u32>,
        base_vertex: i32,
        instance: u32,
    },
    FringesEvenOdd {
        indices: Range<u32>,
        base_vertex: i32,
        instance: u32,
    },

    SelectImage {
        image: u32,
    },

    Image {
        indices: Range<u32>,
        base_vertex: i32,
        instance: u32,
    },
}

pub struct Picture {
    bundle: wgpu::RenderBundle,
}

impl<'a> std::iter::IntoIterator for &'a Picture {
    type Item = &'a wgpu::RenderBundle;
    type IntoIter = std::iter::Once<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(&self.bundle)
    }
}

#[derive(Default)]
pub struct Recorder {
    calls: Vec<DrawCall>,
    batch: Batch,
    cache: Tessellator,
}

impl Recorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.calls.clear();
        self.batch.clear();
        self.cache.clear();
    }

    #[inline]
    fn call(&mut self, call: DrawCall) {
        self.calls.push(call);
    }

    pub(crate) fn stroke_path(
        &mut self,
        commands: PathIter,
        paint: &Paint,
        xform: Transform,
        scale: f32,
    ) {
        let mut raw_paint = RawPaint::convert(paint, xform);

        let mut stroke_width = (paint.width * xform.average_scale()).max(0.0);

        let fringe_width = 1.0 / scale;

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

        let commands = commands.transform(xform);
        let tess_tol = 0.25 / scale;
        self.cache.flatten(commands, tess_tol, 0.01 / scale);

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

        let first = self.batch.instance(raw_paint.to_instance(
            stroke_width,
            fringe_width,
            1.0 - 0.5 / 255.0,
        ));
        let second = self
            .batch
            .instance(raw_paint.to_instance(stroke_width, fringe_width, -1.0));

        self.call(DrawCall::StrokeBase {
            indices: indices.clone(),
            base_vertex,
            instance: first,
        });
        self.call(DrawCall::FringesNonZero {
            indices: indices.clone(),
            base_vertex,
            instance: second,
        });
        self.call(DrawCall::StrokeStencil {
            indices,
            base_vertex,
            instance: second,
        });
    }

    pub(crate) fn fill_path(
        &mut self,
        commands: PathIter,
        paint: &Paint,
        xform: Transform,
        scale: f32,
        fill_rule: FillRule,
    ) {
        let raw_paint = RawPaint::convert(paint, xform);

        let fringe_width = if paint.antialias { 1.0 / scale } else { 0.0 };

        // Setup uniforms for draw calls
        let instance = self
            .batch
            .instance(raw_paint.to_instance(fringe_width, fringe_width, -1.0));

        self.cache
            .flatten(commands.transform(xform), 0.25 / scale, 0.01 / scale);
        let draw = self
            .cache
            .expand_fill(&mut self.batch, fringe_width, LineJoin::Miter, 2.4);

        match draw {
            // Bounding box fill quad not needed for convex fill
            Draw::Convex {
                base_vertex,
                indices,
            } => self.call(DrawCall::Convex {
                indices,
                base_vertex,
                instance,
            }),
            Draw::Concave {
                base_vertex,
                fill,
                stroke,
                quad,
            } => {
                self.call(DrawCall::FillStencil {
                    indices: fill,
                    base_vertex,
                    instance,
                });
                match fill_rule {
                    FillRule::NonZero => {
                        self.call(DrawCall::FringesNonZero {
                            indices: stroke,
                            base_vertex,
                            instance,
                        });
                        self.call(DrawCall::FillQuadNonZero {
                            indices: quad,
                            base_vertex,
                            instance,
                        });
                    }
                    FillRule::EvenOdd => {
                        self.call(DrawCall::FringesEvenOdd {
                            indices: stroke,
                            base_vertex,
                            instance,
                        });
                        self.call(DrawCall::FillQuadEvenOdd {
                            indices: quad,
                            base_vertex,
                            instance,
                        });
                    }
                }
            }
        }
    }

    pub fn push_image(&mut self, rect: Rect, transform: Transform, image: u32, color: [u8; 4]) {
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

        self.call(DrawCall::SelectImage { image });
        self.call(DrawCall::Image {
            indices,
            base_vertex,
            instance,
        });
    }

    pub fn finish(&mut self, device: &wgpu::Device, renderer: &Renderer) -> Picture {
        use wgpu::util::DeviceExt;

        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("reui index buffer"),
            contents: cast_slice(self.batch.indices.as_ref()),
            usage: wgpu::BufferUsage::INDEX,
        });

        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("reui vertex buffer"),
            contents: cast_slice(self.batch.vertices.as_ref()),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let instances = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("reui instance buffer"),
            contents: cast_slice(self.batch.instances.as_ref()),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let label = Some("reui picture");

        let mut rpass = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label,
            color_formats: &[wgpu::TextureFormat::Bgra8UnormSrgb],
            depth_stencil_format: Some(wgpu::TextureFormat::Depth24PlusStencil8),
            sample_count: 1,
        });

        //rpass.set_stencil_reference(0);
        rpass.set_bind_group(0, &renderer.viewport.bind_group, &[]);
        rpass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint16);
        rpass.set_vertex_buffer(0, vertices.slice(..));
        rpass.set_vertex_buffer(1, instances.slice(..));

        let pipeline = &renderer.pipeline;
        for call in self.calls.iter().cloned() {
            match call {
                DrawCall::Convex {
                    indices,
                    base_vertex,
                    instance,
                } => {
                    rpass.set_pipeline(&pipeline.convex);
                    rpass.draw_indexed(indices, base_vertex, instance..instance + 1);
                }

                DrawCall::FillStencil {
                    indices,
                    base_vertex,
                    instance,
                } => {
                    rpass.set_pipeline(&pipeline.fill_stencil);
                    rpass.draw_indexed(indices, base_vertex, instance..instance + 1);
                }
                DrawCall::FillQuadNonZero {
                    indices,
                    base_vertex,
                    instance,
                } => {
                    rpass.set_pipeline(&pipeline.fill_quad_non_zero);
                    rpass.draw_indexed(indices, base_vertex, instance..instance + 1);
                }

                DrawCall::FillQuadEvenOdd {
                    indices,
                    base_vertex,
                    instance,
                } => {
                    rpass.set_pipeline(&pipeline.fill_quad_even_odd);
                    rpass.draw_indexed(indices, base_vertex, instance..instance + 1);
                }

                // Fill the stroke base without overlap
                DrawCall::StrokeBase {
                    indices,
                    base_vertex,
                    instance,
                } => {
                    rpass.set_pipeline(&pipeline.stroke_base);
                    rpass.draw_indexed(indices, base_vertex, instance..instance + 1);
                }
                // Clear stencil buffer
                DrawCall::StrokeStencil {
                    indices,
                    base_vertex,
                    instance,
                } => {
                    rpass.set_pipeline(&pipeline.stroke_stencil);
                    rpass.draw_indexed(indices, base_vertex, instance..instance + 1);
                }

                // Draw anti-aliased pixels.
                DrawCall::FringesNonZero {
                    indices,
                    base_vertex,
                    instance,
                } => {
                    rpass.set_pipeline(&pipeline.fringes_non_zero);
                    rpass.draw_indexed(indices, base_vertex, instance..instance + 1);
                }
                DrawCall::FringesEvenOdd {
                    indices,
                    base_vertex,
                    instance,
                } => {
                    rpass.set_pipeline(&pipeline.fringes_even_odd);
                    rpass.draw_indexed(indices, base_vertex, instance..instance + 1);
                }

                DrawCall::SelectImage { image } => {
                    rpass.set_bind_group(1, &renderer.images[image].bind_group, &[]);
                }

                DrawCall::Image {
                    indices,
                    base_vertex,
                    instance,
                } => {
                    rpass.set_pipeline(&pipeline.image);
                    rpass.draw_indexed(indices, base_vertex, instance..instance + 1);
                }
            }
        }

        let bundle = rpass.finish(&wgpu::RenderBundleDescriptor { label });
        Picture { bundle }
    }
}

#[derive(Default)]
pub(crate) struct Batch {
    instances: Vec<Instance>,
    indices: Vec<u16>,
    vertices: Vec<Vertex>,
}

impl std::ops::Index<i32> for Batch {
    type Output = Vertex;
    #[inline]
    fn index(&self, index: i32) -> &Self::Output {
        &self.vertices[index as usize]
    }
}

impl std::ops::IndexMut<i32> for Batch {
    #[inline]
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        &mut self.vertices[index as usize]
    }
}

#[allow(clippy::cast_possible_wrap)]
impl Batch {
    #[inline]
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.instances.clear();
    }

    #[inline]
    pub fn push(&mut self, vertex: Vertex) {
        self.vertices.push(vertex)
    }

    #[inline]
    pub(crate) fn instance(&mut self, instance: Instance) -> u32 {
        self.instances.push(instance);
        self.instances.len() as u32 - 1
    }

    #[inline]
    pub(crate) fn base_vertex(&self) -> i32 {
        self.vertices.len() as i32
    }

    #[inline]
    pub(crate) fn base_index(&self) -> u32 {
        self.indices.len() as u32
    }

    #[inline]
    pub(crate) fn push_strip(&mut self, offset: u16, vertices: &[Vertex]) -> Range<u32> {
        let start = self.base_index();
        self.vertices.extend_from_slice(vertices);
        self.strip(offset, vertices.len() as i32);
        start..self.base_index()
    }

    #[inline]
    pub(crate) fn strip(&mut self, offset: u16, num_vertices: i32) {
        for i in 0..num_vertices.saturating_sub(2) as u16 {
            let (a, b) = if 0 == i % 2 { (1, 2) } else { (2, 1) };
            self.indices.push(offset + i);
            self.indices.push(offset + i + a);
            self.indices.push(offset + i + b);
        }
    }

    #[inline]
    pub(crate) fn fan(&mut self, offset: u16, num_vertices: i32) {
        for i in 0..num_vertices.saturating_sub(2) as u16 {
            self.indices.push(offset);
            self.indices.push(offset + i + 1);
            self.indices.push(offset + i + 2);
        }
    }
}
