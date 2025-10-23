use crate::GameState;
use crate::player::Player;
use bevy::prelude::*;

#[derive(Resource, Clone, Copy)]
pub struct MapBounds {
    pub width: f32,
    pub height: f32,
}

// Tag the main game camera
#[derive(Component)]
pub struct GameCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_game_camera)
            .add_systems(OnExit(GameState::Playing), cleanup_game_camera)
            .add_systems(Update, camera_follow.run_if(in_state(GameState::Playing)));
    }
}

fn setup_game_camera(mut commands: Commands) {
    commands.spawn((Camera2d, GameCamera));
}

fn cleanup_game_camera(mut commands: Commands, query: Query<Entity, With<GameCamera>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn camera_follow(
    windows: Query<&Window>,
    map: Res<MapBounds>,
    player_q: Query<&Transform, With<Player>>,
    mut cam_q: Query<&mut Transform, (With<Camera2d>, With<GameCamera>, Without<Player>)>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };

    let Ok(mut cam_tf) = cam_q.single_mut() else {
        return;
    };

    let window = match windows.iter().next() {
        Some(w) => w,
        None => return,
    };

    let half_w = window.width() * 0.5;
    let half_h = window.height() * 0.5;

    // Start with player position as target
    let mut target_x = player_tf.translation.x;
    let mut target_y = player_tf.translation.y;

    let map_half_w = map.width * 0.5;
    let map_half_h = map.height * 0.5;

    // Clamp
    if map.width <= half_w * 2.0 {
        target_x = 0.0;
    } else {
        let min_x = -map_half_w + half_w;
        let max_x = map_half_w - half_w;
        target_x = target_x.clamp(min_x, max_x);
    }

    if map.height <= half_h * 2.0 {
        target_y = 0.0;
    } else {
        let min_y = -map_half_h + half_h;
        let max_y = map_half_h - half_h;
        target_y = target_y.clamp(min_y, max_y);
    }

    // Update camera position
    cam_tf.translation.x = target_x;
    cam_tf.translation.y = target_y;
}
