use bevy::prelude::*;
use bevy::image::ImageSampler;
use bevy::render::render_resource::*;
use bevy::render::view::RenderLayers;
use bevy::render::camera::RenderTarget;
use bevy::sprite::{Material2d, Material2dPlugin, AlphaMode2d};
use bevy::math::primitives::Rectangle;
use bevy::render::render_resource::ShaderRef;
use crate::player::Player;
use crate::GameState;

pub struct SimpleDeferredLitePlugin;

impl Plugin for SimpleDeferredLitePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
                Material2dPlugin::<NormalsMaterial>::default(),
                Material2dPlugin::<LightingMaterial>::default(),
            ))
            .init_resource::<NormalsTarget>()
            .init_resource::<LightingHandle>()
            .add_systems(Startup, (
                setup_normals_target,
                setup_normals_camera.after(setup_normals_target),
                setup_lighting_fullscreen.after(setup_normals_target),
            ))
            .add_systems(Update, (
                spawn_normals_proxy_for_added,
                handle_resize,
                update_light_from_player.run_if(in_state(crate::GameState::Playing)),
            ));
    }
}

#[derive(Component)]
pub struct DeferredLit2D;

#[derive(Resource, Default)]
pub struct LightingHandle(pub Option<Handle<LightingMaterial>>);

const LAYER_ALBEDO:  usize = 0;
const LAYER_NORMALS: usize = 1;

#[derive(Resource, Default)]
struct NormalsTarget {
    handle: Option<Handle<Image>>,
    size:   UVec2,
}

#[derive(Component)]
struct NormalsProxy;

#[derive(AsBindGroup, TypePath, Asset, Clone, Default)]
pub struct NormalsMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub normal_map: Option<Handle<Image>>,
}

impl Material2d for NormalsMaterial {
    fn fragment_shader() -> ShaderRef { "shaders/normals2d.wgsl".into() }
}

#[derive(Clone, Copy, Default, ShaderType)]
pub struct LightingParams {
    pub view_size: Vec2,        //
    pub _pad0: Vec2,            //
    pub light_pos_radius: Vec4, // xyz = position, w = radius
    pub light_color_int:  Vec4, // r,g,b,intensity
    pub ambient:          Vec4, // r,g,b,_
    pub cone_angle_dir:   Vec4, // x = cone angle (degrees), y = direction angle (radians), z,w = padding
}

#[derive(AsBindGroup, TypePath, Asset, Clone)]
pub struct LightingMaterial {
    #[uniform(0)]
    pub params: LightingParams,
    #[texture(1)]
    #[sampler(2)]
    pub normals_tex: Handle<Image>,
}

impl Material2d for LightingMaterial {
    fn fragment_shader() -> ShaderRef { "shaders/lighting2d.wgsl".into() }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}


fn setup_normals_target(
    mut rt: ResMut<NormalsTarget>,
    mut images: ResMut<Assets<Image>>,
    windows: Single<&Window>,
) {
    let size = UVec2::new(windows.physical_width(), windows.physical_height());

    // Flat +Z; need a default for now
    let mut img = Image::new_fill(
        Extent3d { width: size.x, height: size.y, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[0, 0, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    );
    img.sampler = ImageSampler::nearest();
    img.texture_descriptor.usage =
        TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC;

    rt.handle = Some(images.add(img));
    rt.size = size;
}

// Note: We don't create a main camera here because the existing CameraPlugin
// already handles that. The existing camera will render layer 0 (LAYER_ALBEDO).

fn setup_normals_camera(mut commands: Commands, rt: Res<NormalsTarget>) {
    // Camera that renders normals (layer 1) to the offscreen render target
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 0, // rendering early
            target: RenderTarget::Image(bevy::render::camera::ImageRenderTarget {
                handle: rt.handle.as_ref().unwrap().clone(),
                scale_factor: bevy::math::FloatOrd(1.0),
            }),
            ..default()
        },
        RenderLayers::layer(LAYER_NORMALS),
        Name::new("CameraNormals"),
    ));
}

fn setup_lighting_fullscreen(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<LightingMaterial>>,
    mut lh: ResMut<LightingHandle>,
    rt: Res<NormalsTarget>,
    windows: Single<&Window>,
) {
    let win = windows.into_inner();

    let fs_rect = Rectangle {
        half_size: Vec2::new(win.width() * 0.5, win.height() * 0.5),
        ..Default::default()
    };
    let fs_mesh = meshes.add(Mesh::from(fs_rect));

    let mat = mats.add(LightingMaterial {
        params: LightingParams {
            view_size: Vec2::new(win.width(), win.height()),
            _pad0: Vec2::ZERO,
            light_pos_radius: Vec4::new(0.0, 0.0, 0.0, 800.0), // origin, 800px radius (larger range)
            light_color_int:  Vec4::new(1.0, 0.95, 0.8, 1.5),   // warm light, intensity 1.5
            ambient:          Vec4::new(0.0, 0.0, 0.0, 0.0),    // Completely dark ambient for strong flashlight effect
            cone_angle_dir:   Vec4::new(80.0, 0.0, 0.0, 0.0),   // 80 degree cone, facing right initially
        },
        normals_tex: rt.handle.as_ref().unwrap().clone(),
    });

    lh.0 = Some(mat.clone());

    commands.spawn((
        Mesh2d(fs_mesh),
        MeshMaterial2d(mat),
        Transform::from_xyz(0.0, 0.0, 999.0), // High z to render on top
        RenderLayers::layer(LAYER_ALBEDO),
        Name::new("LightingFSQuad"),
    ));
}

fn spawn_normals_proxy_for_added(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut normals_assets: ResMut<Assets<NormalsMaterial>>,
    assets: Res<AssetServer>,
    q_added: Query<(Entity, &Sprite, &GlobalTransform), Added<DeferredLit2D>>,
) {
    for (e, sprite, _gt) in &q_added {
        let size = sprite.custom_size.unwrap_or(Vec2::splat(64.0));
        let rect = Rectangle { half_size: size * 0.5, ..Default::default() };
        let mesh = meshes.add(Mesh::from(rect));

        let mat = normals_assets.add(NormalsMaterial {
            normal_map: Some(assets.load("enemy/enemy_standard_normal.png")),
        });

        let child = commands
            .spawn((
                Mesh2d(mesh),
                MeshMaterial2d(mat),
                Transform::default(),
                RenderLayers::layer(LAYER_NORMALS),
                NormalsProxy,
                Name::new("NormalsProxy"),
            ))
            .id();

        commands.entity(e).add_child(child);
    }
}

fn handle_resize(
    mut ev: EventReader<bevy::window::WindowResized>,
    mut images: ResMut<Assets<Image>>,
    mut rt: ResMut<NormalsTarget>,
    mut mats: ResMut<Assets<LightingMaterial>>,
    lh: Res<LightingHandle>,
) {
    for e in ev.read() {
        let new = UVec2::new(e.width as u32, e.height as u32);

        if let Some(handle) = &rt.handle {
            if let Some(img) = images.get_mut(handle) {
                img.resize(Extent3d { width: new.x, height: new.y, depth_or_array_layers: 1 });
            }
        }
        rt.size = new;

        if let Some(h) = &lh.0 {
            if let Some(mat) = mats.get_mut(h) {
                mat.params.view_size = Vec2::new(e.width, e.height);
            }
        }
    }
}

fn update_light_from_player(
    player_query: Query<&Transform, With<Player>>,
    mut mats: ResMut<Assets<LightingMaterial>>,
    lh: Res<LightingHandle>,
) {
    if let Some(player_transform) = player_query.iter().next() {
        if let Some(h) = &lh.0 {
            if let Some(mat) = mats.get_mut(h) {
                mat.params.light_pos_radius.x = player_transform.translation.x;
                mat.params.light_pos_radius.y = player_transform.translation.y;
                mat.params.light_pos_radius.z = player_transform.translation.z;

                let (_, _, z_rotation) = player_transform.rotation.to_euler(EulerRot::XYZ);
                mat.params.cone_angle_dir.y = z_rotation + std::f32::consts::PI / 2.0;
            }
        }
    }
}
