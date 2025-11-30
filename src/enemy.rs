use crate::{
    GameState, components::Health, components::KinematicCollider, events::DamagePlayerEvent,
    light_manager::Lights, player::Player, player_material::PlayerBaseMaterial,
    projectile::Projectile, server::type_equals_host,
};
use bevy::math::bounding::Aabb2d;
use bevy::{prelude::*, render::render_resource::DownlevelFlags};
use std::f32::consts;

// Stats for different enemy types!
const NORMAL_SPEED: f32 = 100.;
const STRONG_SPEED: f32 = 100.;
const FAST_SPEED: f32 = 600.;

const NORMAL_HEALTH: i32 = 100;
const STRONG_HEALTH: i32 = 500;
const FAST_HEALTH: i32 = 50;

const RADIUS: f32 = 50.;
const ATTACK_RADIUS: f32 = 100.;

const ACCEL_RATE: f32 = 10000.;

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_enemy)
            .add_systems(
                Update,
                enemy_chase_velocity.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                enemy_cram_velocity.run_if(in_state(GameState::Playing)).run_if(type_equals_host),
            )
            .add_systems(
                Update,
                enemy_velocity_apply.run_if(in_state(GameState::Playing)).run_if(type_equals_host),
            )
            .add_systems(Update, enemy_damage.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                all_enemies_defeated.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, enemy_attack.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct Enemy {
    pub enemy_id: u8,
    enemy_type: EnemyType,
    enemy_speed: f32,
    pub awake: bool,
}

impl Enemy {
    fn new(id: u8, enemy_type: EnemyType) -> Enemy {
        let enemy_speed = match enemy_type {
            EnemyType::Normal => NORMAL_SPEED,
            EnemyType::Strong => STRONG_SPEED,
            EnemyType::Fast => FAST_SPEED,
        };
        Enemy {
            enemy_id: id,
            enemy_type,
            enemy_speed,
            awake: false,
        }
    }
}

enum EnemyType {
    Normal,
    Strong,
    Fast,
}

#[derive(Component)]
pub struct Awake {
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

pub fn setup_enemy(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<PlayerBaseMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    lights: Res<Lights>,
) {
    for i in 1..=16 {
        commands.spawn((
            // See player.rs for more info about the phong-lit material.
            Mesh2d(meshes.add(Rectangle::default())),
            MeshMaterial2d(materials.add(PlayerBaseMaterial {
                color: LinearRgba::BLUE,
                texture: Some(asset_server.load("enemy/enemy_standard_albedo.png")),
                lighting: crate::player_material::Lighting {
                    ambient_reflection_coefficient: 0.0,
                    ambient_light_intensity: 0.0,
                    diffuse_reflection_coefficient: 1.0,
                    shininess: 40.0,
                },
                lights: lights.lights,
                normal: Some(asset_server.load("enemy/enemy_standard_normal.png")),
                mesh_rotation: 0.0,
            })),
            Transform::from_xyz(-3000.0 + (i as f32 * 100.0), 2400., 5.0).with_scale(Vec3::splat(64.)),
            Velocity::new(),
            Enemy::new(i, EnemyType::Normal),
            KinematicCollider {
                shape: Aabb2d {
                    min: Vec2 { x: 0., y: 0. },
                    max: Vec2 { x: 64., y: 64. },
                },
            },
            Health::new(NORMAL_HEALTH),
        ));
    }
    // for i in 0..=3 {
    //     commands.spawn((
    //         Sprite::from_image(asset_server.load("enemy/enemy_strong_albedo.png")),
    //         Transform::from_xyz(-1000., (i * 300) as f32, 10.)
    //             .with_scale(Vec3::new(1.25, 1.25, 1.25)),
    //         Velocity::new(),
    //         Enemy::new(EnemyType::Strong),
    //         Health::new(STRONG_HEALTH),
    //     ));
    // }
    // for i in 0..=12 {
    //     commands.spawn((
    //         Sprite::from_image(asset_server.load("enemy/enemy_strong_albedo.png")),
    //         Transform::from_xyz((i * 1000) as f32, 15000., 10.)
    //             .with_scale(Vec3::new(0.75, 0.75, 0.75)),
    //         Velocity::new(),
    //         Enemy::new(EnemyType::Fast),
    //         Health::new(FAST_HEALTH),
    //     ));
    // }
}

pub fn enemy_chase_velocity(
    time: Res<Time>,
    mut enemy: Query<(&Enemy, &mut Transform, &mut Velocity), With<Enemy>>,
    mut player: Query<&mut Transform, (With<Player>, Without<Enemy>)>,
) {
    let mut min_dist = 10000.;
    let deltat = time.delta_secs();
    let accel = ACCEL_RATE * deltat;

    for (enemy, mut enemy_transform, mut velocity) in enemy.iter_mut() {
        let mut player_transform = enemy_transform.clone();
        for mut player_trans in player.iter_mut() {
            let new_dist = (enemy_transform.translation - player_trans.translation).length();
            if new_dist < min_dist {
               player_transform = *player_trans;
               min_dist = new_dist;
            }
    }
        // Create a vector FROM the enemy TO the player target.
        let mut dir = Vec2::ZERO;
        dir.x = player_transform.translation.x - enemy_transform.translation.x;
        dir.y = player_transform.translation.y - enemy_transform.translation.y;

        **velocity = if dir.length() > 0. {
            (**velocity + (dir.normalize_or_zero() * accel)).clamp_length_max(enemy.enemy_speed)
        } else if velocity.length() > accel {
            **velocity + (velocity.normalize_or_zero() * -accel)
        } else {
            Vec2::ZERO
        };

        let rotation_z = dir.y.atan2(dir.x);
        enemy_transform.rotation = Quat::from_rotation_z(rotation_z - consts::PI / 2.);
    }
}

pub fn enemy_cram_velocity(
    time: Res<Time>,
    mut enemy_tuples: Query<(&Enemy, &mut Transform, &mut Velocity), With<Enemy>>,
) {
    let mut other_tvs: Vec<(Mut<'_, Transform>, Mut<'_, Velocity>)> = Vec::new();
    for (e, t, mut v) in enemy_tuples.iter_mut() {
        for (other_t, other_v) in other_tvs.iter_mut() {
            let distance = t.translation.distance(other_t.translation);
            let overlap = RADIUS - (distance / 2.0);
            if overlap <= 0.0 {
                continue;
            }

            let mut repel_dir = Vec2::ZERO;
            repel_dir.x = other_t.translation.x - t.translation.x;
            repel_dir.y = other_t.translation.y - t.translation.y;
            repel_dir = repel_dir.normalize();
            repel_dir *= -1.0;
            let own_repel_velocity = repel_dir * overlap * e.enemy_speed / 100.0;
            v.velocity += own_repel_velocity;
            let other_repel_velocity = -1.0 * own_repel_velocity;
            other_v.velocity += other_repel_velocity;
        }
        other_tvs.push((t, v));
    }
}

pub fn enemy_velocity_apply(
    mut commands: Commands,
    time: Res<Time>,
    mut enemy_tuples: Query<(Entity, &mut Transform, &mut Velocity, &mut Enemy), With<Enemy>>,
    player: Query<&mut Transform, (With<Player>, Without<Enemy>)>,
) {
    let deltat = time.delta_secs();
    for (mut this, mut transform, velocity, mut enemy) in enemy_tuples.iter_mut() {
        //Awakens enemies if players are close enough
        if !enemy.awake {
            for player_trans in &player {
                let dist = (transform.translation - player_trans.translation).length();
                if dist < 640. {
                    enemy.awake = true;
                    commands.entity(this).insert((Awake {}));
                    continue;
                }
            }
        } 
        if enemy.awake {
            let change = **velocity * deltat;
            transform.translation += change.extend(0.);
            transform.translation.x = transform.translation.x.round();
            transform.translation.y = transform.translation.y.round();
        }
    }
}

pub fn enemy_attack(
    mut enemies: Query<&Transform, With<Enemy>>,
    player: Query<(Entity, &Transform), With<Player>>,
    mut event: EventWriter<DamagePlayerEvent>,
) {
    for (player_entity, player_transform) in player.iter() {
        for enemy_transform in enemies.iter_mut() {
            let distance = (enemy_transform.translation - player_transform.translation).length();
            if distance < ATTACK_RADIUS {
                event.write(DamagePlayerEvent::new(player_entity, 1));
            }
        }
    }
}

pub fn enemy_damage(
    mut enemies: Query<(Entity, &Transform, &mut Health), With<Enemy>>,
    projectiles: Query<&Transform, With<Projectile>>,
    mut commands: Commands,
) {
    for (enemy, enemy_transform, mut enemy_health) in enemies.iter_mut() {
        for projectile_transform in projectiles.iter() {
            let distance =
                (enemy_transform.translation - projectile_transform.translation).length();
            if distance > RADIUS {
                continue;
            }
            // Damage, then check if enemy is dead...
            if enemy_health.damage(10) {
                commands.entity(enemy).despawn();
            }
        }
    }
}

pub fn all_enemies_defeated(
    all_enemies: Query<&Health, With<Enemy>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    //
    // AAAAAAAAA!!!!!!!!!!!!!!
    // UNCOMMENT THIS!!! TEMPORARILY COMMENTED OUT FOR DEMO!!!!
    // AAAAAAAAAAAAAA!!!!!!!!!!!!!!!!!!!!!
    //
    //
    // let mut all_enemies_dead = true;
    // for enemy in all_enemies.iter() {
    //     if enemy.is_dead() == false {
    //         all_enemies_dead = false;
    //         break;
    //     }
    // }
    // if all_enemies_dead {
    //     next_state.set(GameState::GameOver);
    // }
}
