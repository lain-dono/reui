pub struct Shader {
    pub vs: wgpu::ShaderModule,
    pub fs: Option<wgpu::ShaderModule>,
}

impl Shader {
    pub fn vertex_stage(&self) -> wgpu::ProgrammableStageDescriptor {
        wgpu::ProgrammableStageDescriptor {
            module: &self.vs,
            entry_point: "main",
        }
    }

    pub fn fragment_stage(&self) -> Option<wgpu::ProgrammableStageDescriptor> {
        self.fs
            .as_ref()
            .map(|module| wgpu::ProgrammableStageDescriptor {
                module,
                entry_point: "main",
            })
    }

    pub fn base(device: &wgpu::Device) -> Self {
        Self {
            vs: device.create_shader_module(wgpu::include_spirv!("shader.vert.spv")),
            fs: Some(device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"))),
        }
    }

    pub fn stencil(device: &wgpu::Device) -> Self {
        let vs = device.create_shader_module(wgpu::include_spirv!("stencil.vert.spv"));
        Self { vs, fs: None }
    }

    pub fn image(device: &wgpu::Device) -> Self {
        Self {
            vs: device.create_shader_module(wgpu::include_spirv!("image.vert.spv")),
            fs: Some(device.create_shader_module(wgpu::include_spirv!("image.frag.spv"))),
        }
    }
}
