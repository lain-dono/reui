/*
void glnvg__fill(GLNVGcontext* gl, GLNVGcall* call)
{
    GLNVGpath* paths = &gl->paths[call->pathOffset];
    int i, npaths = call->pathCount;

    // Draw shapes
    glEnable(GL_STENCIL_TEST);
    glnvg__stencilMask(gl, 0xff);
    glnvg__stencilFunc(gl, GL_ALWAYS, 0, 0xff);
    glColorMask(GL_FALSE, GL_FALSE, GL_FALSE, GL_FALSE);

    // set bindpoint for solid loc
    glnvg__setUniforms(gl, call->uniformOffset, 0);
    glnvg__checkError(gl, "fill simple");

    glStencilOpSeparate(GL_FRONT, GL_KEEP, GL_KEEP, GL_INCR_WRAP);
    glStencilOpSeparate(GL_BACK, GL_KEEP, GL_KEEP, GL_DECR_WRAP);
    glDisable(GL_CULL_FACE);
    for (i = 0; i < npaths; i++)
            glDrawArrays(GL_TRIANGLE_STRIP, paths[i].fillOffset, paths[i].fillCount); // XXX
    glEnable(GL_CULL_FACE);

    // Draw anti-aliased pixels
    glColorMask(GL_TRUE, GL_TRUE, GL_TRUE, GL_TRUE);

    glnvg__setUniforms(gl, call->uniformOffset + gl->fragSize, call->image);
    glnvg__checkError(gl, "fill fill");

    if (gl->flags & NVG_ANTIALIAS) {
            glnvg__stencilFunc(gl, GL_EQUAL, 0x00, 0xff);
            glStencilOp(GL_KEEP, GL_KEEP, GL_KEEP);
            // Draw fringes
            for (i = 0; i < npaths; i++)
                    glDrawArrays(GL_TRIANGLE_STRIP, paths[i].strokeOffset, paths[i].strokeCount);
    }

    // Draw fill
    glnvg__stencilFunc(gl, GL_NOTEQUAL, 0x0, 0xff);
    glStencilOp(GL_ZERO, GL_ZERO, GL_ZERO);
    glDrawArrays(GL_TRIANGLE_STRIP, call->triangleOffset, call->triangleCount);

    glDisable(GL_STENCIL_TEST);
}
*/

pub enum Pass {
    Stencil,
    Fringes,
    Fill,
}

fn stencil(format: wgpu::TextureFormat, pass: Pass) -> wgpu::DepthStencilStateDescriptor {
    let compare = match pass {
        Pass::Stencil => wgpu::CompareFuntion::Always,
        Pass::Fringes => wgpu::StencilOperation::Equal,
        Pass::Fill => wgpu::StencilOperation::NotEqual,
    };
    let op = match pass {
        Pass::Stencil => wgpu::StencilOperation::Keep,
        Pass::Fringes => wgpu::StencilOperation::Keep,
        Pass::Fill => wgpu::StencilOperation::Zero,
    };

    wgpu::DepthStencilStateDescriptor {
        format,
        depth_write_enabled: false,
        depth_compare: CompareFunction::Always,

        stencil_front: StencilStateFaceDescriptor {
            compare,
            fail_op: op,
            depth_fail_op: op,
            pass_op: match pass {
                Pass::Stencil => wgpu::StencilOperation::IncrementWrap,
                Pass::Fringes => wgpu::StencilOperation::Keep,
                Pass::Fill => wgpu::StencilOperation::Zero,
            },
        },
        stencil_back: StencilStateFaceDescriptor {
            compare,
            fail_op: op,
            depth_fail_op: op,
            pass_op: match pass {
                Pass::Stencil => wgpu::StencilOperation::DecrementWrap,
                Pass::Fringes => wgpu::StencilOperation::Keep,
                Pass::Fill => wgpu::StencilOperation::Zero,
            },
        },

        stencil_read_mask: 0xFF,
        stencil_write_mask: 0xFF,
    }
}

struct Shader<'vs, 'fs> {
    vertex_stage: wgpu::PipelineStageDescriptor<'vs>,
    fragment_stage: wgpu::PipelineStageDescriptor<'fs>,
}

fn fill_stencil(shader: Shader) {
    let Shader { vertex_stage, fragment_stage } = shader;
    wgpu::RenderPipelineDescriptor {
        layout: &layout,
        vertex_stage,
        fragment_stage,
        rasterization_state: wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        },
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[],
        depth_stencil_state: Some(),
        index_format: wgpu::IndexFormat::Uint16,
        //vertex_buffers: &[VERTEX_BUFFER_DESCRIPTOR],
        vertex_buffers: &[],
        sample_count: 1,
    }
}

fn fill_aa() {
    wgpu::RenderPipelineDescriptor {
        layout: &layout,
        vertex_stage: wgpu::PipelineStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: wgpu::PipelineStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        },
        rasterization_state: wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        },
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[],
        depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
            format: TextureFormat,
            depth_write_enabled: false,
            depth_compare: CompareFunction,

            stencil_front: StencilStateFaceDescriptor {
                compare: wgpu::CompareFunction::Always,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::IncrementWrap,
            },
            stencil_back: StencilStateFaceDescriptor {
                compare: wgpu::CompareFunction::Always,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::DecrementWrap,
            },

            stencil_read_mask: 0xFF,
            stencil_write_mask: 0xFF,
        }),
        index_format: wgpu::IndexFormat::Uint16,
        //vertex_buffers: &[VERTEX_BUFFER_DESCRIPTOR],
        vertex_buffers: &[],
        sample_count: 1,
    }
}

fn fill_aa() {
    wgpu::RenderPipelineDescriptor {
        layout: &layout,
        vertex_stage: wgpu::PipelineStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: wgpu::PipelineStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        },
        rasterization_state: wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        },
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[],
        depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
            format: TextureFormat,
            depth_write_enabled: false,
            depth_compare: CompareFunction,

            stencil_front: StencilStateFaceDescriptor {
                compare: wgpu::CompareFunction::Always,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::IncrementWrap,
            },
            stencil_back: StencilStateFaceDescriptor {
                compare: wgpu::CompareFunction::Always,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::DecrementWrap,
            },

            stencil_read_mask: 0xFF,
            stencil_write_mask: 0xFF,
        }),
        index_format: wgpu::IndexFormat::Uint16,
        //vertex_buffers: &[VERTEX_BUFFER_DESCRIPTOR],
        vertex_buffers: &[],
        sample_count: 1,
    }
}
