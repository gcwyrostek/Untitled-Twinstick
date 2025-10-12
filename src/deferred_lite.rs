use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::{TextureCache, CachedTexture};
use bevy::render::{
    render_graph::*,
    render_resource::*,
    renderer::RenderContext,
    RenderApp, Render, RenderSystems,
};
use bevy::window::{PrimaryWindow, Window};
use bevy::core_pipeline::core_2d::graph::{Core2d, Node2d};
use bevy::prelude::Color;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct DeferredLiteLabel;

pub struct DeferredLitePlugin;

impl Plugin for DeferredLitePlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<GBufferLite>()
            .add_systems(Render, prepare_normal_render_target.in_set(RenderSystems::Prepare));

        // 1) Construct the ViewNodeRunner in its own borrow of the world
        let node_runner = {
            let world = render_app.world_mut();
            ViewNodeRunner::<NormalsNode>::new(NormalsNode::default(), world)
        };

        // 2) Now borrow the world again to mutate the graph
        {
            let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
            let core2d = graph.get_sub_graph_mut(Core2d).unwrap();

            core2d.add_node(DeferredLiteLabel, node_runner);
            // add_node_edge returns (), so don't unwrap/chain
            core2d.add_node_edge(Node2d::EndMainPass, DeferredLiteLabel);
        }
    }
}

#[derive(Resource)]
struct GBufferLite {
    texture_view: Option<TextureView>,
    size: UVec2,
    format: TextureFormat
}

impl Default for GBufferLite {
    fn default() -> Self {
        Self{
            texture_view: None,
            size: UVec2::ZERO,
            format: TextureFormat::Rgba8Unorm,
        }
    }
}

// gBufRes -> our resource that will hold everything for out render targets such as normals
// textures -> Bevy's render
// This is for the prepare stage of the render world for Bevy rendering
fn prepare_normal_render_target(
    mut gbufRes: ResMut<GBufferLite>,
    mut textures: ResMut<TextureCache>,
    window: Query<&Window, With<PrimaryWindow>>,
    render_dev: Res<RenderDevice> 
) {
    // Update the size of the window if needed
    let primary_window: &Window = match window.single() {
        Ok(v) => v,
        Err(_) => return
    };

    gbufRes.size = primary_window.resolution.physical_size();
    gbufRes.format = TextureFormat::Rgba8Unorm;

    let texture_desc = TextureDescriptor {
        label: Some("deferred_lite_normals"),
        size: Extent3d { width: gbufRes.size.x, height: gbufRes.size.y, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: gbufRes.format,
        usage: TextureUsages::COPY_SRC | TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };

    let cached = textures.get(&render_dev, texture_desc);
    gbufRes.texture_view = Some(cached.default_view);

    println!("{}", primary_window.width());
}

#[derive(Default)]
struct NormalsNode;

impl ViewNode for NormalsNode {
    type ViewQuery = ();

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        _view: QueryItem<Self::ViewQuery>,
        world: &World, 
    ) -> Result<(), NodeRunError> {
        let gbuf = world.resource::<GBufferLite>();
        let Some(view) = &gbuf.texture_view else {return Ok(())};

        let mut pass = render_context.command_encoder().begin_render_pass(
            &RenderPassDescriptor {
                label: Some("DL Normals Clear"), 
                color_attachments: &[Some(RenderPassColorAttachment { 
                    view: view, 
                    depth_slice: None, 
                    resolve_target: None, 
                    ops: Operations {
                        load: LoadOp::Clear(wgpu_types::Color::BLACK),
                        store: StoreOp::Store,
                    },
                })], 
                depth_stencil_attachment: None, 
                timestamp_writes: None, 
                occlusion_query_set: None, }
        );
        drop(pass);
        Ok(())
    }


}