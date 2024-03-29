use bevy::{
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        texture::TextureCache,
        view::ExtractedView,
    },
    utils::HashMap,
};

#[derive(Component)]
pub struct ViewDepthStencilTexture {
    pub texture: Texture,
    pub view: TextureView,
}

#[derive(Component)]
pub struct UniformOffset {
    pub offset: u32,
}

#[derive(Resource, Default)]
pub struct Uniforms {
    pub buffer: DynamicUniformBuffer<Uniform>,
}

#[derive(Clone, ShaderType)]
pub struct Uniform {
    pub data: Vec4,
}

pub fn prepare_uniforms(
    mut commands: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut uniforms: ResMut<Uniforms>,
    views: Query<(Entity, &ExtractedView)>,
) {
    uniforms.buffer.clear();

    for (entity, camera) in &views {
        let width = camera.viewport.z;
        let height = camera.viewport.w;
        let data = crate::combine_viewport(width, height).into();
        let offset = uniforms.buffer.push(Uniform { data });
        let offset = UniformOffset { offset };
        commands.get_or_spawn(entity).insert(offset);
    }

    uniforms.buffer.write_buffer(&device, &queue);
}

pub fn prepare_textures(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedCamera)>,
) {
    let mut textures = HashMap::default();
    for (entity, camera) in &views {
        let Some(physical_target_size) = camera.physical_target_size else {
            continue;
        };

        let cached_texture = textures
            .entry(camera.target.clone())
            .or_insert_with(|| {
                texture_cache.get(
                    &render_device,
                    TextureDescriptor {
                        label: Some("depth_stenicl_texture"),
                        size: Extent3d {
                            depth_or_array_layers: 1,
                            width: physical_target_size.x,
                            height: physical_target_size.y,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: TextureDimension::D2,
                        format: TextureFormat::Depth24PlusStencil8,
                        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    },
                )
            })
            .clone();

        commands.entity(entity).insert(ViewDepthStencilTexture {
            texture: cached_texture.texture,
            view: cached_texture.default_view,
        });
    }
}
