use crate::{
    backend::{Picture, Pipeline, Target},
    cache::PathCache,
    canvas::{Canvas, PictureRecorder},
    state::States,
};

pub struct Renderer {
    pub(crate) recorder: PictureRecorder,
    pub(crate) states: States,
    pub(crate) cache: PathCache,
    pub(crate) picture: Picture,

    pipeline: Pipeline,

    width: f32,
    height: f32,
    dpi: f32,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            states: States::with_capacity(256),
            cache: PathCache::new(),
            recorder: PictureRecorder::new(),
            picture: Picture::new(),
            pipeline: Pipeline::new(device, format),
            width: 1.0,
            height: 1.0,
            dpi: 1.0,
        }
    }

    pub fn begin_frame(&mut self, width: f32, height: f32, dpi: f32) -> Canvas {
        self.picture.clear();
        self.states.clear();
        self.cache.set_dpi(dpi);
        self.width = width;
        self.height = height;
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
            .draw_commands(&self.picture, encoder, device, target);
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
