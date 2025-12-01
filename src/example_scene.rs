use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::*;
use bevy::render::view::RenderLayers;
use bevy::sprite::{Material2d, Material2dPlugin};
use bevy::image::ImageSampler;

pub struct ExampleScenePlugin;

impl Plugin for ExampleScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
                Material2dPlugin::<NormalsMaterial>::default(),
                Material2dPlugin::<DeferredLightMaterial>::default(),
            ))
            .add_systems(Update, toggle_scene)
            .add_systems(OnEnter(crate::GameState::ExampleScene), setup_deferred)
            .add_systems(OnExit(crate::GameState::ExampleScene), cleanup_deferred)
            .add_systems(Update, animate_light.run_if(in_state(crate::GameState::ExampleScene)));
    }
}

const LAYER_MAIN: usize = 5;
const LAYER_NORMALS: usize = 6;

#[derive(Component)]
struct ExampleEntity;

#[derive(Component)]
struct LightMarker;

#[derive(Resource)]
struct NormalsTexture(Handle<Image>);

#[derive(Resource)]
struct LightingMaterial(Handle<DeferredLightMaterial>);

#[derive(AsBindGroup, Asset, TypePath, Clone)]
struct NormalsMaterial {
    #[texture(0)]
    #[sampler(1)]
    albedo: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    normal_map: Handle<Image>,
}

impl Material2d for NormalsMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/normals_pass.wgsl".into()
    }
}

#[derive(AsBindGroup, Asset, TypePath, Clone)]
struct DeferredLightMaterial {
    #[texture(0)]
    #[sampler(1)]
    albedo: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    normals_buffer: Handle<Image>,
    #[uniform(4)]
    view_size: Vec2,
    #[uniform(4)]
    light_pos: Vec2,
}

impl Material2d for DeferredLightMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/lighting_pass.wgsl".into()
    }
}

fn toggle_scene(
    kbd: Res<ButtonInput<KeyCode>>,
    state: Res<State<crate::GameState>>,
    mut next: ResMut<NextState<crate::GameState>>,
) {
    if kbd.just_pressed(KeyCode::KeyG) {
        match state.get() {
            crate::GameState::Playing => next.set(crate::GameState::ExampleScene),
            crate::GameState::ExampleScene => next.set(crate::GameState::Playing),
            _ => {}
        }
    }
}

fn setup_deferred(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut normals_materials: ResMut<Assets<NormalsMaterial>>,
    mut light_materials: ResMut<Assets<DeferredLightMaterial>>,
    assets: Res<AssetServer>,
    mut main_cameras: Query<&mut Camera, Without<ExampleEntity>>,
    window: Single<&Window>,
) {
    for mut cam in main_cameras.iter_mut() {
        cam.is_active = false;
    }

    let window = window.into_inner();
    let size = UVec2::new(window.physical_width(), window.physical_height());

    let mut normals_img = Image::new_fill(
        Extent3d { width: size.x, height: size.y, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[128, 128, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    );
    normals_img.sampler = ImageSampler::nearest();
    normals_img.texture_descriptor.usage = TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;

    let normals_handle = images.add(normals_img);
    commands.insert_resource(NormalsTexture(normals_handle.clone()));

    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            target: RenderTarget::Image(bevy::render::camera::ImageRenderTarget {
                handle: normals_handle.clone(),
                scale_factor: bevy::math::FloatOrd(1.0),
            }),
            ..default()
        },
        RenderLayers::layer(LAYER_NORMALS),
        Name::new("NormalsCamera"),
        ExampleEntity,
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: 100,
            clear_color: bevy::render::camera::ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        RenderLayers::layer(LAYER_MAIN),
        Name::new("MainCamera"),
        ExampleEntity,
    ));

    let albedo_tex = assets.load("enemy/enemy_standard_albedo.png");
    let normal_tex = assets.load("enemy/enemy_standard_normal.png");

    let normals_mat = normals_materials.add(NormalsMaterial {
        albedo: albedo_tex.clone(),
        normal_map: normal_tex,
    });

    let mesh = meshes.add(Rectangle::new(64.0, 64.0));

    let positions = [
        Vec2::new(0.0, 0.0),
        Vec2::new(-150.0, -100.0),
        Vec2::new(150.0, -100.0),
    ];

    for &pos in positions.iter() {
        commands.spawn((
            Mesh2d(mesh.clone()),
            MeshMaterial2d(normals_mat.clone()),
            Transform::from_xyz(pos.x, pos.y, 0.0).with_scale(Vec3::splat(3.0)),
            RenderLayers::layer(LAYER_NORMALS),
            ExampleEntity,
        ));
    }

    let fs_mesh = meshes.add(Rectangle::new(window.width(), window.height()));

    let light_mat = light_materials.add(DeferredLightMaterial {
        albedo: albedo_tex,
        normals_buffer: normals_handle.clone(),
        view_size: Vec2::new(window.width(), window.height()),
        light_pos: Vec2::new(0.0, -200.0),
    });

    commands.insert_resource(LightingMaterial(light_mat.clone()));

    commands.spawn((
        Mesh2d(fs_mesh),
        MeshMaterial2d(light_mat),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RenderLayers::layer(LAYER_MAIN),
        Name::new("LightingQuad"),
        ExampleEntity,
    ));

    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 0.0),
            custom_size: Some(Vec2::splat(20.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -200.0, 5.0),
        RenderLayers::layer(LAYER_MAIN),
        ExampleEntity,
        LightMarker,
    ));
}

fn cleanup_deferred(
    mut commands: Commands,
    entities: Query<Entity, With<ExampleEntity>>,
    mut main_cameras: Query<&mut Camera, Without<ExampleEntity>>,
) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<NormalsTexture>();
    commands.remove_resource::<LightingMaterial>();

    for mut cam in main_cameras.iter_mut() {
        cam.is_active = true;
    }
}

fn animate_light(
    time: Res<Time>,
    mut materials: ResMut<Assets<DeferredLightMaterial>>,
    light_mat: Res<LightingMaterial>,
    mut marker: Query<&mut Transform, With<LightMarker>>,
) {
    let t = time.elapsed_secs();
    let radius = 200.0;
    let x = t.sin() * radius;
    let y = -t.cos() * radius;

    if let Some(mat) = materials.get_mut(&light_mat.0) {
        mat.light_pos = Vec2::new(x, y);
    }

    if let Ok(mut transform) = marker.get_single_mut() {
        transform.translation.x = x;
        transform.translation.y = y;
    }
}
