use crate::light_manager::Lights;
use crate::{
    GameState, components::FlowMap, components::Health, components::KinematicCollider,
    components::LightSource, components::StaticCollider, events::DamagePlayerEvent,
    local_control::LocalControl, net_control::NetControl, net_control::PlayerType,
    player_material::PlayerBaseMaterial,
};
use bevy::math::bounding::Aabb2d;
use bevy::math::bounding::IntersectsVolume;
use bevy::prelude::*;
use bevy::time::Timer;
use bevy::time::TimerMode;
use bevy::window::PrimaryWindow;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::f32::consts;

const WIN_W: f32 = 1280.;
const WIN_H: f32 = 720.;
const PLAYER_SPEED: f32 = 300.;
const PLAYER_SIZE: f32 = 32.;
const ACCEL_RATE: f32 = 3600.;
const MAX_HEALTH: i32 = 100;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_player)
            .add_systems(Update, player_movement.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                player_orientation.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, player_damage.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                player_calculate_flow.run_if(in_state(GameState::Playing)),
            );
    }
}

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

pub fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PlayerBaseMaterial>>,
    asset_server: Res<AssetServer>,
    // query: Query<Entity, With<Camera>>,
    query: Query<Entity, With<Camera>>,
    players: Query<Entity, Or<(With<NetControl>, With<LocalControl>)>>,
    lights: Res<Lights>,
) {
    // if query.is_empty() {
    //     commands.spawn(Camera2d);
    // }

    for i in players {
        commands.entity(i).insert((
            // For any entities that we want to have lighting,
            // add the following two components.
            Mesh2d(meshes.add(Rectangle::default())),
            MeshMaterial2d(materials.add(PlayerBaseMaterial {
                // Generally, only change what's inside the 'lighting' struct and the 'texture' and 'normal' parameters.
                color: LinearRgba::BLUE,
                texture: Some(asset_server.load("player/player_albedo.png")),
                lighting: crate::player_material::Lighting {
                    // 'ambient_reflection_coefficient' and 'ambient_light_intensity' do the same thing.
                    // Should be 0 for everything except the player.
                    // 'diffuse_reflection_coefficient' is how much not-shiny light is reflected back.
                    // 'shininess' is what it sounds like. Higher number = shinier.
                    ambient_reflection_coefficient: 0.1,
                    ambient_light_intensity: 0.1,
                    diffuse_reflection_coefficient: 1.0,
                    shininess: 40.0,
                },
                lights: lights.lights,
                normal: Some(asset_server.load("player/player_normal.png")),
                mesh_rotation: 0.0,
            })),
            Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(128.)), // Change size of player here: current size: 64. (makes player 64x larger)
            // you can have a smaller player with 32 and larger player with 128
            Velocity::new(),
            FireCooldown(Timer::from_seconds(0.2, TimerMode::Repeating)),
            Player,
            Health::new(MAX_HEALTH),
            //NetControl::new(PlayerType::Local, 0),
            KinematicCollider {
                shape: Aabb2d {
                    min: Vec2 { x: 0., y: 0. },
                    max: Vec2 { x: 64., y: 64. },
                },
            },
            FlowMap::default(),
        ));
    }
}

pub fn shape_collides_statics(
    collider_aabb: &Aabb2d,
    collider_pos: &Vec2,
    statics: Query<(&StaticCollider, &Transform), Without<KinematicCollider>>,
) -> bool {
    for (sc, st) in &statics {
        let mut transformed_kc_shape = collider_aabb.clone();
        transformed_kc_shape.min += collider_pos;
        transformed_kc_shape.max += collider_pos;

        let mut transformed_sc_shape = sc.shape.clone();
        transformed_sc_shape.min += st.translation.truncate();
        transformed_sc_shape.max += st.translation.truncate();

        let colliding = transformed_kc_shape.intersects(&transformed_sc_shape);
        if colliding {
            return true;
        }
    }

    return false;
}

pub fn player_calculate_flow(
    statics: Query<(&StaticCollider, &Transform), Without<KinematicCollider>>,
    player_flows: Query<(&Transform, &mut FlowMap), With<Player>>,
) {
    let h_shape: Aabb2d = Aabb2d {
        min: Vec2 { x: 0., y: 0. },
        max: Vec2 { x: 64., y: 64. },
    };
    for (p_transform, mut flow) in player_flows {
        let player_2d_pos = p_transform.translation.truncate();
        for x in -50..50 {
            for y in -50..50 {
                let h_col_pos = player_2d_pos + Vec2::new(x as f32 * 64.0, y as f32 * 64.0);
                if shape_collides_statics(&h_shape, &h_col_pos, statics) {
                    flow.map.insert(IVec2::new(x, y), 1000);
                    return;
                }
                flow.map.insert(IVec2::new(x, y), x + y);
            }
        }
    }
}

pub fn player_movement(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    player_net: Query<
        (&mut Transform, &mut Velocity, &mut NetControl),
        (With<Player>, With<NetControl>, Without<LocalControl>),
    >,
    player_local: Query<
        (&mut Transform, &mut Velocity, &mut LocalControl),
        (With<Player>, With<LocalControl>, Without<NetControl>),
    >,
) {
    //This is pretty ugly. If we could condense it, that would be great, but I couldn't figure it out at the time.
    if player_net.iter().count() > player_local.iter().count() {
        for (mut transform, mut velocity, mut control) in player_net {
            let mut dir = Vec2::ZERO;

            if control.get_type() == PlayerType::Local {
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
            } else {
                if control.pressed(KeyCode::KeyA) {
                    dir.x -= 1.;
                }

                if control.pressed(KeyCode::KeyD) {
                    dir.x += 1.;
                }

                if control.pressed(KeyCode::KeyW) {
                    dir.y += 1.;
                }

                if control.pressed(KeyCode::KeyS) {
                    dir.y -= 1.;
                }
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

            //keep player in bounds
            let max = Vec3::new(
                WIN_W * 2. / 2. - PLAYER_SIZE / 2.,
                WIN_H * 2. / 2. - PLAYER_SIZE / 2.,
                0.,
            );

            let min = max.clone() * -1.;

            let translate = (transform.translation + change.extend(0.)).clamp(min, max);
            transform.translation = translate;

            //Rounds position to integers
            transform.translation.x = transform.translation.x.round();
            transform.translation.y = transform.translation.y.round();
            //info!("{:?}", transform.translation);

            //Sets position in NetControl
            control.set_pos_x(transform.translation.x);
            control.set_pos_y(transform.translation.y);
        }
    } else {
        for (mut transform, mut velocity, mut control) in player_local {
            let mut dir = Vec2::ZERO;

            if control.get_type() == PlayerType::Local {
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

                //keep player in bounds
                let max = Vec3::new(
                    WIN_W * 2. / 2. - PLAYER_SIZE / 2.,
                    WIN_H * 2. / 2. - PLAYER_SIZE / 2.,
                    0.,
                );

                let min = max.clone() * -1.;

                let translate = (transform.translation + change.extend(0.)).clamp(min, max);
                transform.translation = translate;

                //Rounds position to integers
                transform.translation.x = transform.translation.x.round();
                transform.translation.y = transform.translation.y.round();
                //info!("{:?}", transform.translation);
            } else {
                transform.translation = control.get_p_pos();
            }
        }
    }
}

pub fn player_orientation(
    mut players: Query<
        (
            &mut MeshMaterial2d<PlayerBaseMaterial>,
            &mut Transform,
            &mut NetControl,
        ),
        With<Player>,
    >,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<PlayerBaseMaterial>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(cursor_world_position) =
            camera.viewport_to_world_2d(camera_transform, cursor_position)
        {
            for (mut material, mut player_transform, mut netcontrol) in players.iter_mut() {
                let mut rounded_rot_z = 0.;

                if netcontrol.player_type == PlayerType::Local {
                    let player_position = player_transform.translation.truncate();
                    let direction = cursor_world_position - player_position;

                    if direction.length() > 0.0 {
                        let rotation_z = direction.y.atan2(direction.x);
                        //Rounding is needed to prevent precision errors when networking
                        rounded_rot_z = (rotation_z * 10.).round() / 10.;
                        netcontrol.set_angle(rounded_rot_z);
                    }
                } else if netcontrol.player_type == PlayerType::Network {
                    rounded_rot_z = netcontrol.get_angle();
                }

                player_transform.rotation = Quat::from_rotation_z(rounded_rot_z - consts::PI / 2.);
            }
        }
    }
}

pub fn player_damage(
    mut next_state: ResMut<NextState<GameState>>,
    mut events: EventReader<DamagePlayerEvent>,
    mut players: Query<(Entity, &mut Health), With<Player>>,
    mut commands: Commands,
) {
    for damage_event in events.read() {
        for (player, mut player_health) in players.iter_mut() {
            if damage_event.target == player {
                player_health.damage(damage_event.amount);
                if player_health.is_dead() {
                    next_state.set(GameState::GameOver);
                    commands.entity(player).despawn();
                }
            }
        }
    }
}
