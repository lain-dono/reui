use crate::{internals::GpuBatch, Canvas, Images, Picture, Pipeline, Recorder};
use wgpu::util::DeviceExt as _;

pub type Image = u32;

pub struct Renderer {
    pub recorder: Recorder<Image>,
    pub batch: GpuBatch,
    pub pipeline: Pipeline,
    pub images: Images<Image>,

    pub(crate) view_buffer: wgpu::Buffer,
    pub(crate) view_binding: wgpu::BindGroup,
    pub(crate) depth_stencil: wgpu::TextureView,
    pub(crate) width: u32,
    pub(crate) height: u32,

    image_index: Image,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let images = Images::new(device);
        let batch = GpuBatch::new(device);
        let pipeline = Pipeline::new(device, &images.layout);
        let recorder = Recorder::default();

        let contents = crate::combine_viewport(width, height);
        let view_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("reui::Viewport"),
            contents: bytemuck::bytes_of(&contents),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let view_binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("reui::Viewport"),
            layout: &pipeline.view_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_buffer.as_entire_binding(),
            }],
        });

        let depth_stencil = create_depth_texture(device, width, height);

        Self {
            batch,
            pipeline,
            recorder,
            images,

            view_buffer,
            view_binding,
            depth_stencil,
            width,
            height,

            image_index: 0,
        }
    }

    pub fn upload_image(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: Option<&str>,
        width: u32,
        height: u32,
        data: &[u8],
        sampler: Option<&wgpu::Sampler>,
    ) -> Image {
        let image_key = self.image_index;
        drop(self.images.upload(
            device, queue, image_key, label, width, height, data, sampler,
        ));
        self.image_index += 1;
        image_key
    }

    pub fn start(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> Canvas<Image> {
        self.recorder.clear();

        if self.width != width || self.height != height {
            self.depth_stencil = create_depth_texture(device, width, height);
        }

        let viewport = crate::combine_viewport(width, height);
        queue.write_buffer(&self.view_buffer, 0, bytemuck::bytes_of(&viewport));

        self.width = width;
        self.height = height;

        Canvas::new(&mut self.recorder, &self.images)
    }

    pub fn flush(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        staging_belt: &mut wgpu::util::StagingBelt,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        clear_color: Option<wgpu::Color>,
    ) {
        self.batch
            .staging(encoder, staging_belt, device, &self.recorder.batch);

        let bundle = Picture::new(
            device,
            &self.view_binding,
            0,
            &self.pipeline,
            &self.batch,
            &self.images,
            &self.recorder.calls,
        );

        crate::render_pictures(
            encoder,
            view,
            &self.depth_stencil,
            &bundle,
            clear_color,
            true,
        )
    }
}

fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("reui::DepthStencil"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}
