use crate::{integration::ViewDepthStencilTexture, picture::Picture};
use bevy::{
    prelude::*,
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        renderer::RenderContext,
        view::{ExtractedView, ViewTarget},
    },
};

pub struct ReuiPassNode {
    query: QueryState<
        (
            &'static Picture,
            &'static ViewTarget,
            &'static ViewDepthStencilTexture,
        ),
        With<ExtractedView>,
    >,
}

impl ReuiPassNode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: world.query_filtered(),
        }
    }
}

impl Node for ReuiPassNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(Self::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext<'_>,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let (picture, target, depth) = match self.query.get_manual(world, view_entity) {
            Ok(query) => query,
            Err(_) => return Ok(()), // No window
        };

        #[cfg(feature = "trace")]
        let _span = info_span!("reui_pass").entered();

        crate::render_pictures(
            &mut render_context.command_encoder,
            &target.view,
            &depth.view,
            picture,
        );

        Ok(())
    }
}
