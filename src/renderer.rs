use crate::{
    canvas::Canvas,
    picture::Recorder,
    pipeline::{BatchUpload, Pipeline},
    viewport::Viewport,
};
use std::collections::HashMap;

pub(crate) struct ImageBind {
    pub bind_group: wgpu::BindGroup,
    pub size: wgpu::Extent3d,
}

struct HostImage {
    texture: wgpu::Texture,
    size: wgpu::Extent3d,
}

impl HostImage {
    fn open_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<std::path::Path>,
    ) -> image::ImageResult<Self> {
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
            usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * width),
                rows_per_image: None,
            },
            size,
        );

        Ok(Self { texture, size })
    }
}

#[derive(Default)]
pub(crate) struct Images {
    pub(crate) images: HashMap<u32, ImageBind>,
    idx: u32,
}

impl std::ops::Index<u32> for Images {
    type Output = ImageBind;
    fn index(&self, index: u32) -> &Self::Output {
        &self.images[&index]
    }
}

impl Images {
    pub fn get(&self, image: u32) -> Option<&ImageBind> {
        self.images.get(&image)
    }

    pub fn create(&mut self, bind: ImageBind) -> u32 {
        let idx = self.idx;
        drop(self.images.insert(idx, bind));
        self.idx += 1;
        idx
    }
}

pub struct Renderer {
    pub(crate) images: Images,
    pub pipeline: Pipeline,
    pub batch: BatchUpload,
}

impl Renderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let pipeline = Pipeline::new(device);

        Self {
            images: Images::default(),
            pipeline,
            batch: BatchUpload::new(device),
        }
    }

    /// # Errors
    pub fn open_image(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<std::path::Path>,
    ) -> image::ImageResult<u32> {
        let image = HostImage::open_rgba(device, queue, path)?;

        let desc = wgpu::TextureViewDescriptor::default();
        let view = image.texture.create_view(&desc);
        let bind_group = self.pipeline.bind_texture_view(device, &view);

        let size = image.size;

        Ok(self.images.create(ImageBind { bind_group, size }))
    }

    pub fn begin_frame<'a>(
        &'a mut self,
        viewport: &Viewport,
        recorder: &'a mut Recorder,
    ) -> Canvas<'a> {
        Canvas::new(self, recorder, viewport.scale())
    }
}
