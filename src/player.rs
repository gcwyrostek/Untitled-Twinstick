use bevy::prelude::*;
use bevy::time::Timer;
use bevy::time::TimerMode;

const PLAYER_SPEED: f32 = 300.;
const ACCEL_RATE: f32 = 3600.;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct FireCooldown(Timer);

impl FireCooldown {
    pub fn tick(&mut self, delta: std::time::Duration) -> bool {
        self.0.tick(delta).finished()
    }

    pub fn reset(&mut self) {
        self.0.reset();
    }
}

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

pub fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    
    commands.spawn((
        Sprite::from_image(asset_server.load("player/blueberryman.png")),
        Transform::from_xyz(-300., 0., 10.), // Scale down the image if needed, 1.0 means original size, 2.0 means double size, etc.
        Velocity::new(),
        FireCooldown(Timer::from_seconds(0.2, TimerMode::Repeating)),
        Player,
    ));
}

pub fn player_movement(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut Transform, &mut Velocity), With<Player>>,
) {
    let (mut transform, mut velocity) = player.into_inner();

    let mut dir = Vec2::ZERO;

    if input.pressed(KeyCode::KeyA) {
        dir.x -= 1.;
    }

    if input.pressed(KeyCode::KeyD) {
        dir.x += 1.;
    }

    if input.pressed(KeyCode::KeyW) {
        dir.y += 1.;
    }

    if input.pressed(KeyCode::KeyS) {
        dir.y -= 1.;
    }

    let deltat = time.delta_secs();
    let accel = ACCEL_RATE * deltat;

    **velocity = if dir.length() > 0. {
        (**velocity + (dir.normalize_or_zero() * accel)).clamp_length_max(PLAYER_SPEED)
    } else if velocity.length() > accel {
        **velocity + (velocity.normalize_or_zero() * -accel)
    } else {
        Vec2::ZERO
    };
    let change = **velocity * deltat;

    transform.translation += change.extend(0.);
} 