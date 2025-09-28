use bevy::{
    prelude::*,
    window::PresentMode,
    winit::cursor::{CursorIcon, CustomCursor, CustomCursorImage},
};

// Game modules
mod enemy;
mod player;
mod tiling;
//mod enemy;
//mod bullet;
//mod reticle;
//mod ground_tiles;
//mod ammo_pickup;
//mod guns;
//mod revive_kit_pickup;

const WIN_W: f32 = 1280.;
const WIN_H: f32 = 720.;

fn setup_cursor_icon(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    window: Single<Entity, With<Window>>,
) {
    let layout =
        TextureAtlasLayout::from_grid(UVec2::splat(64), 20, 10, Some(UVec2::splat(5)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands.entity(*window).insert(
        (CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
            handle: asset_server.load("textures/reticle.png"),
            texture_atlas: None,
            flip_x: false,
            flip_y: false,
            rect: None,
            hotspot: (1, 1),
        }))),
    );
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Untitled Survival Shooter".into(),
                resolution: (WIN_W, WIN_H).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        // Core game systems
        .add_systems(Startup, player::setup_player)
        .add_systems(Startup, enemy::setup_enemy)
        .add_systems(Startup, tiling::setup)
        .add_systems(Startup, setup_cursor_icon)
        .add_systems(Update, player::player_movement)
        .add_systems(Update, enemy::enemy_movement)
        .run();
}
