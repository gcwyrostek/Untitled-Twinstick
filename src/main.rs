use bevy::{prelude::*, window::PresentMode};

// Game modules
mod player;
//mod enemy;
//mod bullet;
//mod reticle;
//mod ground_tiles;
//mod ammo_pickup;
//mod guns;
//mod revive_kit_pickup;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Untitled Survival Shooter".into(),
                resolution: (1280., 720.).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        // Core game systems
        .add_systems(Startup, player::setup_player)
        .add_systems(Update, player::player_movement)
        .run();
} 