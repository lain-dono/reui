use crate::{
    image::Images,
    picture::{Picture, Recorder},
    pipeline::{BatchUpload, Pipeline},
};
use bevy::{
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_graph::{RenderGraph, RunGraphOnViewNode, SlotInfo, SlotType},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        texture::TextureCache,
        Extract, RenderApp, RenderStage,
    },
    utils::HashMap,
};
use wgpu::util::DeviceExt;

mod node;

pub use self::node::ReuiPassNode;

pub const REUI_PASS_DRIVER: &str = "reui_pass_driver";

pub mod draw_reui_graph {
    pub const NAME: &str = "draw_reui";
    pub mod input {
        pub const VIEW_ENTITY: &str = "view_entity";
    }
    pub mod node {
        pub const REUI_PASS: &str = "reui_pass";
    }
}

pub struct ReuiPlugin;

impl bevy::app::Plugin for ReuiPlugin {
    fn build(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        render_app.init_resource::<Images>();
        render_app.init_resource::<BatchUpload>();
        render_app.init_resource::<Pipeline>();

        render_app.add_system_to_stage(RenderStage::Prepare, prepare_textures);
        render_app.add_system_to_stage(RenderStage::Extract, extract_pictures);

        let reui_graph_2d = get_graph(render_app);
        let reui_graph_3d = get_graph(render_app);

        let mut graph = render_app.world.resource_mut::<RenderGraph>();

        if let Some(graph) = graph.get_sub_graph_mut(bevy::core_pipeline::core_2d::graph::NAME) {
            graph.add_sub_graph(draw_reui_graph::NAME, reui_graph_2d);
            graph.add_node(
                draw_reui_graph::node::REUI_PASS,
                RunGraphOnViewNode::new(draw_reui_graph::NAME),
            );
            graph
                .add_node_edge(
                    bevy::core_pipeline::core_2d::graph::node::MAIN_PASS,
                    draw_reui_graph::node::REUI_PASS,
                )
                .unwrap();
            graph
                .add_slot_edge(
                    graph.input_node().unwrap().id,
                    bevy::core_pipeline::core_2d::graph::input::VIEW_ENTITY,
                    draw_reui_graph::node::REUI_PASS,
                    RunGraphOnViewNode::IN_VIEW,
                )
                .unwrap();
        }

        if let Some(graph) = graph.get_sub_graph_mut(bevy::core_pipeline::core_3d::graph::NAME) {
            graph.add_sub_graph(draw_reui_graph::NAME, reui_graph_3d);
            graph.add_node(
                draw_reui_graph::node::REUI_PASS,
                RunGraphOnViewNode::new(draw_reui_graph::NAME),
            );
            graph
                .add_node_edge(
                    bevy::core_pipeline::core_3d::graph::node::MAIN_PASS,
                    draw_reui_graph::node::REUI_PASS,
                )
                .unwrap();
            graph
                .add_slot_edge(
                    graph.input_node().unwrap().id,
                    bevy::core_pipeline::core_3d::graph::input::VIEW_ENTITY,
                    draw_reui_graph::node::REUI_PASS,
                    RunGraphOnViewNode::IN_VIEW,
                )
                .unwrap();
        }
    }
}

fn get_graph(render_app: &mut App) -> RenderGraph {
    let ui_pass_node = ReuiPassNode::new(&mut render_app.world);
    let mut ui_graph = RenderGraph::default();

    ui_graph.add_node(draw_reui_graph::node::REUI_PASS, ui_pass_node);

    let input_node_id = ui_graph.set_input(vec![SlotInfo::new(
        draw_reui_graph::input::VIEW_ENTITY,
        SlotType::Entity,
    )]);

    ui_graph
        .add_slot_edge(
            input_node_id,
            draw_reui_graph::input::VIEW_ENTITY,
            draw_reui_graph::node::REUI_PASS,
            ReuiPassNode::IN_VIEW,
        )
        .unwrap();
    ui_graph
}

impl FromWorld for crate::image::Images {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        Self::new(device.wgpu_device())
    }
}

impl FromWorld for crate::pipeline::Pipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let images = world.resource::<Images>();
        Self::new(device.wgpu_device(), images)
    }
}

impl FromWorld for crate::pipeline::BatchUpload {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        Self::new(device.wgpu_device())
    }
}

fn extract_pictures(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut batch: ResMut<BatchUpload>,
    pipeline: Res<Pipeline>,
    query: Extract<Query<(Entity, &Recorder, &Camera)>>,
) {
    let device = render_device.wgpu_device();

    for (entity, recorder, camera) in query.iter() {
        let has_target = camera
            .physical_target_size()
            .map_or(false, |s| s.x != 0 && s.y != 0);

        if !camera.is_active || !has_target {
            continue;
        }

        let (width, height) = match camera.physical_viewport_size() {
            Some(size) => (size.x, size.y),
            None => continue,
        };

        let viewport = ViewportMeta::new(device, &pipeline.view_layout, width, height, 2.0);

        batch.upload_queue(&render_queue, device, &recorder.batch);

        let picture = Picture::new(
            device,
            &viewport.bind_group,
            &pipeline,
            &batch,
            &recorder.calls,
        );

        commands.get_or_spawn(entity).insert(picture);
    }
}

pub struct ViewportMeta {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl ViewportMeta {
    fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        scale: f32,
    ) -> Self {
        let (width, height) = (width as f32, height as f32);
        let contents = [scale / width, scale / height, width.recip(), height.recip()];
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("reui::Viewport"),
            contents: bytemuck::bytes_of(&contents),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("reui::Viewport"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self { buffer, bind_group }
    }
}

#[derive(Component)]
pub struct ViewDepthStencilTexture {
    pub texture: Texture,
    pub view: TextureView,
}

pub fn prepare_textures(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedCamera)>,
) {
    let mut textures = HashMap::default();
    for (entity, camera) in &views {
        if let Some(physical_target_size) = camera.physical_target_size {
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
                            usage: TextureUsages::RENDER_ATTACHMENT
                                | TextureUsages::TEXTURE_BINDING,
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
}
