use crate::{
    internals::{DrawCall, GpuBatch},
    Picture, Pipeline,
};
use bevy::{
    prelude::*,
    render::{
        render_graph::{RenderGraph, RenderGraphError, RunGraphOnViewNode, SlotInfo, SlotType},
        renderer::{RenderDevice, RenderQueue},
        Extract, RenderApp, RenderStage,
    },
};

mod node;
mod viewport;

pub use self::node::ReuiPassNode;
pub use self::viewport::{UniformOffset, ViewDepthStencilTexture};

pub type Recorder = crate::Recorder<Handle<Image>>;
pub type Images = crate::Images<Handle<Image>>;

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

        render_app
            .init_resource::<Images>()
            .init_resource::<GpuBatch>()
            .init_resource::<Pipeline>()
            .init_resource::<viewport::Uniforms>()
            .add_system_to_stage(RenderStage::Extract, extract_recorder)
            .add_system_to_stage(RenderStage::Prepare, self::viewport::prepare_textures)
            .add_system_to_stage(RenderStage::Prepare, self::viewport::prepare_uniforms)
            .add_system_to_stage(RenderStage::Queue, queue_pictures);

        let reui_graph_2d = get_graph(render_app).unwrap();
        let reui_graph_3d = get_graph(render_app).unwrap();

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

fn get_graph(render_app: &mut App) -> Result<RenderGraph, RenderGraphError> {
    let mut graph = RenderGraph::default();

    graph.add_node(
        draw_reui_graph::node::REUI_PASS,
        ReuiPassNode::new(&mut render_app.world),
    );

    let input_node_id = graph.set_input(vec![SlotInfo::new(
        draw_reui_graph::input::VIEW_ENTITY,
        SlotType::Entity,
    )]);

    graph.add_slot_edge(
        input_node_id,
        draw_reui_graph::input::VIEW_ENTITY,
        draw_reui_graph::node::REUI_PASS,
        ReuiPassNode::IN_VIEW,
    )?;

    Ok(graph)
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        Self::new(device.wgpu_device())
    }
}

impl FromWorld for Pipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let images = world.resource::<Images>();
        Self::new(device.wgpu_device(), &images.layout)
    }
}

impl FromWorld for GpuBatch {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        Self::new(device.wgpu_device())
    }
}

#[derive(Component)]
struct ExtractedRecorder {
    calls: Vec<DrawCall<Handle<Image>>>,
}

fn extract_recorder(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut batch: ResMut<GpuBatch>,
    query: Extract<Query<(Entity, &Recorder)>>,
) {
    let device = render_device.wgpu_device();
    for (entity, recorder) in query.iter() {
        batch.queue(&render_queue, device, &recorder.batch);

        let calls = recorder.calls.clone();
        let component = ExtractedRecorder { calls };
        commands.get_or_spawn(entity).insert(component);
    }
}

fn queue_pictures(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline: Res<Pipeline>,
    batch: Res<GpuBatch>,
    uniforms: Res<viewport::Uniforms>,
    images: Res<Images>,
    query: Query<(Entity, &ExtractedRecorder, &UniformOffset)>,
) {
    let device = render_device.wgpu_device();

    if let Some(resource) = uniforms.binding() {
        let binding = 0;
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("reui::Viewport"),
            layout: &pipeline.view_layout,
            entries: &[wgpu::BindGroupEntry { resource, binding }],
        });

        for (entity, cmd, offset) in query.iter() {
            let picture = Picture::new(
                device,
                &bind_group,
                offset.offset,
                &pipeline,
                &batch,
                &images,
                &cmd.calls,
            );
            commands.get_or_spawn(entity).insert(picture);
        }
    }
}
