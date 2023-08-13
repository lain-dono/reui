use crate::{picture::Picture, plugin::ViewDepthStencilTexture};
use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::{
        render_graph::{NodeRunError, RenderGraphContext, ViewNode},
        renderer::RenderContext,
        view::ViewTarget,
    },
};

#[derive(Default)]
pub struct ReuiNode;

impl ViewNode for ReuiNode {
    type ViewQuery = (
        &'static Picture,
        &'static ViewTarget,
        &'static ViewDepthStencilTexture,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext<'_>,
        render_context: &mut RenderContext,
        (picture, target, depth): QueryItem<Self::ViewQuery>,
        _world: &World,
    ) -> Result<(), NodeRunError> {
        #[cfg(feature = "trace")]
        let _span = info_span!("reui_pass").entered();

        crate::render_pictures(
            render_context.command_encoder(),
            target.main_texture_view(),
            &depth.view,
            picture,
            None,
            true,
        );

        Ok(())
    }
}
