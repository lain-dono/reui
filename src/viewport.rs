use wgpu::util::DeviceExt as _;

#[derive(Clone, Copy)]
pub struct Target<'a> {
    pub color: &'a wgpu::TextureView,
    pub depth: &'a wgpu::TextureView,
    pub bind_group: &'a wgpu::BindGroup,
}

impl<'a> Target<'a> {
    pub fn rpass(
        self,
        encoder: &'a mut wgpu::CommandEncoder,
        clear_color: Option<wgpu::Color>,
        clear: bool,
        store: bool,
    ) -> wgpu::RenderPass {
        let (depth, stencil) = if clear {
            (wgpu::LoadOp::Clear(0.0), wgpu::LoadOp::Clear(0))
        } else {
            (wgpu::LoadOp::Load, wgpu::LoadOp::Load)
        };

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("reui::RenderPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: self.color,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: clear_color.map_or(wgpu::LoadOp::Load, wgpu::LoadOp::Clear),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: self.depth,
                depth_ops: Some(wgpu::Operations { load: depth, store }),
                stencil_ops: Some(wgpu::Operations {
                    load: stencil,
                    store,
                }),
            }),
        });

        rpass.set_bind_group(0, self.bind_group, &[]);
        rpass
    }

    pub fn bind<'rpass>(&'rpass self, rpass: &mut impl wgpu::util::RenderEncoder<'rpass>) {
        rpass.set_bind_group(0, self.bind_group, &[]);
    }
}

pub struct Viewport {
    depth_stencil: wgpu::TextureView,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
    scale: f32,
}

impl Viewport {
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        scale: f32,
    ) -> Self {
        let contents = Self::convert_viewport(width as f32, height as f32, scale);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("reui::Viewport.buffer"),
            contents: bytemuck::bytes_of(&contents),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("reui::Viewport.bind_group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        let depth_stencil = Self::create_depth_texture(device, width, height);

        Self {
            depth_stencil,
            buffer,
            bind_group,
            width,
            height,
            scale,
        }
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    #[inline]
    pub fn scale(&self) -> f32 {
        self.scale
    }

    #[inline]
    pub fn target<'a>(&'a self, attachment: &'a wgpu::TextureView) -> Target<'a> {
        Target {
            color: attachment,
            depth: &self.depth_stencil,
            bind_group: &self.bind_group,
        }
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        scale: f32,
    ) {
        if self.width != width || self.height != height {
            self.depth_stencil = Self::create_depth_texture(device, width, height);
        }

        if self.width != width || self.height != height || self.scale.to_bits() != scale.to_bits() {
            let viewport = Self::convert_viewport(width as f32, height as f32, scale);
            queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&viewport));
        }

        self.width = width;
        self.height = height;
        self.scale = scale;
    }

    fn convert_viewport(width: f32, height: f32, scale: f32) -> [f32; 4] {
        [scale / width, scale / height, width.recip(), height.recip()]
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
}
