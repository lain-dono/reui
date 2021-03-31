use crate::{
    math::{Offset, Rect, Transform},
    paint::{LineJoin, Paint, RawPaint},
    path::PathIter,
    tessellator::Contour,
    Tessellator,
};
use std::{mem::size_of, ops::Range, slice::from_raw_parts};

pub(crate) fn cast_slice<T: Sized>(slice: &[T]) -> &[u8] {
    unsafe { from_raw_parts(slice.as_ptr().cast(), slice.len() * size_of::<T>()) }
}

#[derive(Clone)]
pub enum DrawCall {
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
    FillQuad {
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

    Fringes {
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

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [u16; 2],
}

impl Vertex {
    #[inline]
    pub fn new(pos: impl Into<[f32; 2]>, uv: [f32; 2]) -> Self {
        let pos = pos.into();
        let uv = [(uv[0] * 65535.0) as u16, (uv[1] * 65535.0) as u16];
        Self { pos, uv }
    }

    #[inline]
    pub fn transform(self, transform: Transform) -> Self {
        let pos = transform.apply(Offset::from(self.pos)).into();
        Self { pos, ..self }
    }
}

#[repr(C, align(4))]
#[derive(Clone, Copy, Default)]
pub struct Instance {
    pub paint_mat: [f32; 4],
    pub inner_color: [u8; 4],
    pub outer_color: [u8; 4],

    pub extent: [f32; 2],
    pub radius: f32,
    pub feather: f32,

    pub stroke_mul: f32, // scale
    pub stroke_thr: f32, // threshold
}

impl Instance {
    pub fn image(color: [u8; 4]) -> Self {
        Self {
            inner_color: color,
            ..Self::default()
        }
    }
}

pub struct Picture {
    pub(crate) indices: wgpu::Buffer,
    pub(crate) vertices: wgpu::Buffer,
    pub(crate) instances: wgpu::Buffer,
}

#[derive(Default)]
pub struct Recorder {
    pub(crate) calls: Vec<DrawCall>,
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

    pub fn call(&mut self, call: DrawCall) {
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

        let mut stroke_width = (paint.width * xform.average_scale()).clamp(0.0, 200.0);

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

        let tess_tol = 0.25 / scale;
        self.cache
            .flatten(commands.transform(xform), tess_tol, 0.01 / scale);
        self.cache.expand_stroke(
            stroke_width * 0.5,
            fringe_width,
            paint.cap_start,
            paint.cap_end,
            paint.join,
            paint.miter,
            tess_tol,
        );

        let base_vertex = self.batch.base_vertex();
        let indices = self
            .batch
            .stroke(0, self.cache.contours(), self.cache.vertices());

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
        self.call(DrawCall::Fringes {
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
    ) {
        let raw_paint = RawPaint::convert(paint, xform);

        let fringe_width = if paint.antialias { 1.0 / scale } else { 0.0 };
        self.cache
            .flatten(commands.transform(xform), 0.25 / scale, 0.01 / scale);
        self.cache.expand_fill(fringe_width, LineJoin::Miter, 2.4);

        let base_vertex = self.batch.base_vertex();
        let fill = self
            .batch
            .fill(0, self.cache.contours(), self.cache.vertices());
        let offset = (self.batch.base_vertex() - base_vertex) as u16;
        let stroke = self
            .batch
            .stroke(offset, self.cache.contours(), self.cache.vertices());

        // Setup uniforms for draw calls
        let instance = self
            .batch
            .instance(raw_paint.to_instance(fringe_width, fringe_width, -1.0));
        if self.cache.is_convex() {
            // Bounding box fill quad not needed for convex fill
            self.call(DrawCall::Convex {
                indices: fill.start..stroke.end,
                base_vertex,
                instance,
            });
        } else {
            let Rect { min, max } = self.cache.bounds();

            let quad = self.batch.push_strip(
                (self.batch.base_vertex() - base_vertex) as u16,
                &[
                    Vertex::new([max.x, max.y], [0.5, 1.0]),
                    Vertex::new([max.x, min.y], [0.5, 1.0]),
                    Vertex::new([min.x, max.y], [0.5, 1.0]),
                    Vertex::new([min.x, min.y], [0.5, 1.0]),
                ],
            );

            self.call(DrawCall::FillStencil {
                indices: fill,
                base_vertex,
                instance,
            });
            self.call(DrawCall::Fringes {
                indices: stroke,
                base_vertex,
                instance,
            });
            self.call(DrawCall::FillQuad {
                indices: quad,
                base_vertex,
                instance,
            });
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

    pub fn build(&mut self, device: &wgpu::Device) -> Picture {
        use wgpu::util::DeviceExt;

        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: cast_slice(self.batch.indices.as_ref()),
            usage: wgpu::BufferUsage::INDEX,
        });

        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: cast_slice(self.batch.vertices.as_ref()),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let instances = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: cast_slice(self.batch.instances.as_ref()),
            usage: wgpu::BufferUsage::VERTEX,
        });

        Picture {
            indices,
            vertices,
            instances,
        }
    }
}

#[derive(Default)]
struct Batch {
    instances: Vec<Instance>,
    indices: Vec<u16>,
    vertices: Vec<Vertex>,
}

#[allow(clippy::cast_possible_wrap)]
impl Batch {
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.instances.clear();
    }

    pub fn instance(&mut self, instance: Instance) -> u32 {
        self.instances.push(instance);
        self.instances.len() as u32 - 1
    }

    fn base_vertex(&self) -> i32 {
        self.vertices.len() as i32
    }

    fn base_index(&self) -> u32 {
        self.indices.len() as u32
    }

    fn fill(&mut self, mut offset: u16, contours: &[Contour], vertices: &[Vertex]) -> Range<u32> {
        let start = self.base_index();
        for contour in contours {
            if !contour.fill.is_empty() {
                let vertices = &vertices[contour.fill.clone()];
                self.vertices.extend_from_slice(vertices);
                self.fan(offset, vertices.len());
                offset += vertices.len() as u16;
            }
        }
        start..self.base_index()
    }

    fn stroke(&mut self, mut offset: u16, contours: &[Contour], vertices: &[Vertex]) -> Range<u32> {
        let start = self.base_index();
        for contour in contours {
            if !contour.stroke.is_empty() {
                let vertices = &vertices[contour.stroke.clone()];

                self.vertices.extend_from_slice(vertices);
                self.strip(offset, vertices.len());
                offset += vertices.len() as u16;
            }
        }
        start..self.base_index()
    }

    fn push_strip(&mut self, offset: u16, vertices: &[Vertex]) -> Range<u32> {
        self.vertices.extend_from_slice(vertices);

        let start = self.base_index();
        self.strip(offset, vertices.len());
        start..self.base_index()
    }

    fn strip(&mut self, offset: u16, num_vertices: usize) {
        for i in 0..num_vertices.saturating_sub(2) as u16 {
            let (a, b) = if 0 == i % 2 { (1, 2) } else { (2, 1) };
            self.indices.push(offset + i);
            self.indices.push(offset + i + a);
            self.indices.push(offset + i + b);
        }
    }

    fn fan(&mut self, offset: u16, num_vertices: usize) {
        for i in 0..num_vertices.saturating_sub(2) as u16 {
            self.indices.push(offset);
            self.indices.push(offset + i + 1);
            self.indices.push(offset + i + 2);
        }
    }
}
