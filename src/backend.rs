use crate::{
    cache::PathCache,
    canvas::Canvas,
    picture::Picture,
    pipeline::{Pipeline, Target},
    recorder::PictureRecorder,
    state::TransformStack,
};

#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [u16; 2],
}

impl Vertex {
    #[inline(always)]
    pub fn new(pos: [f32; 2], uv: [f32; 2]) -> Self {
        let uv = [(uv[0] * 65535.0) as u16, (uv[1] * 65535.0) as u16];
        Self { pos, uv }
    }
}

pub struct Renderer {
    pub(crate) recorder: PictureRecorder,
    pub(crate) states: TransformStack,
    pub(crate) cache: PathCache,
    pub(crate) picture: Picture,

    pipeline: Pipeline,

    dpi: f32,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            states: TransformStack::with_capacity(256),
            cache: PathCache::new(),
            recorder: PictureRecorder::new(),
            picture: Picture::new(),
            pipeline: Pipeline::new(device, format),
            dpi: 1.0,
        }
    }

    pub fn begin_frame(&mut self, dpi: f32) -> Canvas {
        self.picture.clear();
        self.states.clear();
        self.cache.set_dpi(dpi);
        self.dpi = dpi;
        Canvas::new(self)
    }

    pub fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        target: Target,
    ) {
        self.pipeline
            .draw_picture(&self.picture, encoder, device, target);
    }

    /*
    pub fn _begin_path(&mut self) {
        self.picture.commands.clear();
        self.cache.clear();
    }

    pub fn _close_path(&mut self) {
        self.picture.close_path();
    }

    pub fn _bezier_to(&mut self, p1: Offset, p2: Offset, p3: Offset) {
        self.picture.xform = self.states.last().xform;
        self.picture.bezier_to(p1, p2, p3);
    }

    pub fn _quad_to(&mut self, p1: Offset, p2: Offset) {
        self.picture.xform = self.states.last().xform;
        self.picture.quad_to(p1, p2);
    }

    pub fn _arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture
            .arc_to(point2(x1, y1), point2(x2, y2), radius, self.cache.dist_tol);
    }

    pub fn _arc(&mut self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, dir: Winding) {
        self.picture.xform = self.states.last().xform;
        self.picture.arc(point2(cx, cy), r, a0, a1, dir);
    }
    */
}
