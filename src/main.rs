use bevy::{prelude::*, window::PresentMode};

// Game modules
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
        .add_systems(Startup, tiling::setup)
        .add_systems(Update, player::player_movement)
        .run();
} 