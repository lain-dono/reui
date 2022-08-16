use crate::{Canvas, GpuBatch, ImageBind, Images, Pipeline, Recorder};
use wgpu::util::DeviceExt as _;

pub type Image = u32;

pub struct Renderer {
    pub recorder: Recorder<Image>,
    pub batch: GpuBatch,
    pub pipeline: Pipeline,
    pub images: Images<Image>,

    pub(crate) view_buffer: wgpu::Buffer,
    pub(crate) view_bind_group: wgpu::BindGroup,
    pub(crate) depth_stencil: wgpu::TextureView,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) scale: f32,

    image_index: Image,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, width: u32, height: u32, scale: f32) -> Self {
        let images = Images::new(device);
        let batch = GpuBatch::new(device);
        let pipeline = Pipeline::new(device, &images.layout);
        let recorder = Recorder::default();

        let view_buffer = create_buffer(device, width, height, scale);
        let view_bind_group = create_bind_group(device, &pipeline.view_layout, &view_buffer);

        let depth_stencil = create_depth_texture(device, width, height);

        Self {
            batch,
            pipeline,
            recorder,
            images,

            view_buffer,
            view_bind_group,
            depth_stencil,
            width,
            height,
            scale,

            image_index: 0,
        }
    }

    /// # Errors
    pub fn open_image(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<std::path::Path>,
    ) -> image::ImageResult<u32> {
        let (texture, size) = open_rgba(device, queue, path)?;
        let desc = wgpu::TextureViewDescriptor::default();
        let view = texture.create_view(&desc);
        let bind_group = self.images.bind(device, &view, None);
        let bind = ImageBind { bind_group, size };

        let idx = self.image_index;
        drop(self.images.insert(idx, bind));
        self.image_index += 1;

        Ok(idx)
    }

    pub fn start_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        scale: f32,
    ) -> Canvas<Image> {
        self.recorder.clear();

        if self.width != width || self.height != height {
            self.depth_stencil = create_depth_texture(device, width, height);
        }

        let viewport = convert_viewport(width, height, scale);
        queue.write_buffer(&self.view_buffer, 0, bytemuck::bytes_of(&viewport));

        self.width = width;
        self.height = height;
        self.scale = scale;

        Canvas::new(&mut self.recorder, &self.images, scale)
    }

    pub fn flush(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        staging_belt: &mut wgpu::util::StagingBelt,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        clear_color: Option<wgpu::Color>,
    ) {
        let bundle = self.recorder.finish(
            encoder,
            staging_belt,
            device,
            &mut self.batch,
            &self.pipeline,
            &self.view_bind_group,
            &self.images,
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

fn open_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: impl AsRef<std::path::Path>,
) -> image::ImageResult<(wgpu::Texture, wgpu::Extent3d)> {
    let path = path.as_ref();
    let m = image::open(path)?;

    let m = m.to_rgba8();
    let width = m.width();
    let height = m.height();
    let texels = m.into_raw();

    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: path.to_str(),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &texels,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(4 * width),
            rows_per_image: None,
        },
        size,
    );

    Ok((texture, size))
}

fn convert_viewport(width: u32, height: u32, scale: f32) -> [f32; 4] {
    let (width, height) = (width as f32, height as f32);
    [scale / width, scale / height, width.recip(), height.recip()]
}

fn create_buffer(device: &wgpu::Device, width: u32, height: u32, scale: f32) -> wgpu::Buffer {
    let contents = convert_viewport(width, height, scale);
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("reui::Viewport"),
        contents: bytemuck::bytes_of(&contents),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("reui::Viewport"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    })
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
