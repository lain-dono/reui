pub struct Shader {
    pub vs: wgpu::ShaderModule,
    pub fs: wgpu::ShaderModule,
}

impl Shader {
    fn new(device: &wgpu::Device, vs: &[u8], fs: &[u8]) -> std::io::Result<Self> {
        use std::io::Cursor;

        let vs = wgpu::read_spirv(Cursor::new(&vs[..]))?;
        let vs = device.create_shader_module(&vs);

        let fs = wgpu::read_spirv(Cursor::new(&fs[..]))?;
        let fs = device.create_shader_module(&fs);

        Ok(Self { vs, fs })
    }

    pub fn base(device: &wgpu::Device) -> std::io::Result<Self> {
        let vs = include_bytes!("shader.vert.spv");
        let fs = include_bytes!("shader.frag.spv");
        Self::new(device, vs, fs)
    }

    pub fn stencil(device: &wgpu::Device) -> std::io::Result<Self> {
        let vs = include_bytes!("stencil.vert.spv");
        let fs = include_bytes!("stencil.frag.spv");
        Self::new(device, vs, fs)
    }
}
