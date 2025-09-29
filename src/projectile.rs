use crate::player::Player;
use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::prelude::*;

const PROJECTILE_SPEED: f32 = 600.;

#[derive(Component)]
pub struct Projectile;

#[derive(Component, Deref, DerefMut)]
pub struct Velocity {
    velocity: Vec2,
}

impl Velocity {
    fn new() -> Self {
        Self {
            velocity: Vec2::ZERO,
        }
    }
}

pub fn projectile_inputs(
    mut commands: Commands,
    mouse_button_io: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    player_q: Query<&Transform, With<Player>>,
    asset_server: Res<AssetServer>,
) {
    let not_shooting = !mouse_button_io.pressed(MouseButton::Left);
    if not_shooting {
        return;
    }

    let window = match windows.single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let (camera, camera_transform) = match camera_q.single() {
        Ok(v) => v,
        Err(_) => return,
    };
    let Some(cursor_screen_pos) = window.cursor_position() else {
        return;
    };
    let Ok(cursor_world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_screen_pos)
    else {
        return;
    };
    let player_transform = match player_q.single() {
        Ok(t) => t,
        Err(_) => return,
    };
    let projectile_pos = player_transform.translation;
    let dir = (cursor_world_pos - projectile_pos.truncate()).normalize();

    commands.spawn((
        Sprite::from_image(asset_server.load("textures/bullet.png")),
        Transform::from_scale(Vec3::splat(0.5)).with_translation(projectile_pos),
        Velocity {
            velocity: dir * PROJECTILE_SPEED,
        },
        Projectile,
    ));
}

pub fn projectile_movement(
    time: Res<Time>,
    mut projectiles: Query<(&mut Transform, &Velocity), With<Projectile>>,
) {
    for (mut transform, velocity) in &mut projectiles {
        let delta_t = time.delta_secs();
        let delta_d = **velocity * delta_t;
        transform.translation += delta_d.extend(0.);
    }
}
