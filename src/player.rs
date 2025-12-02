use crate::{
    GameState, components::FlowMap, components::Health, components::KinematicCollider,
    components::LightSource, components::StaticCollider, events::DamagePlayerEvent,
    net_control::NetControl, net_control::PlayerType, player_material::PlayerBaseMaterial,
    collisions::find_mtv, server::InputHistory, server::RollbackDetection, wall::Door,
    light_manager::Lights,
};
use bevy::math::bounding::Aabb2d;
use bevy::math::bounding::IntersectsVolume;
use bevy::prelude::*;
use bevy::time::Timer;
use bevy::time::TimerMode;
use bevy::window::PrimaryWindow;
use crate::collectible::PlayerInventory;
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
            .add_systems(FixedUpdate, player_movement.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                player_orientation.run_if(in_state(GameState::Playing)),
            )
            /* .add_systems(
                Update,
                drain_battery.run_if(in_state(GameState::Playing)),
                
            )*/
            .add_systems(
            Update,
            player_movement_from_history
                .run_if(in_state(GameState::Playing))
                .run_if(rollback_from_history),
        )
            .add_systems(Update, player_damage.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                player_calculate_flow.run_if(in_state(GameState::Playing)),
            );
    }
}

fn rollback_from_history(roll: Res<RollbackDetection>) -> bool {
    return roll.is_rollback;
}

#[derive(Component)]
pub struct Player {
    pub charge: i32,
    pub flashlight: Option<Entity>,
}

//Local player resource
#[derive(Resource)]
pub struct LocalPlayer {
    pub entity: Entity,
}

impl Player {
    pub fn charge_battery(&mut self, value: i32) {
        self.charge += value;
        if self.charge >= 500 {
            self.charge = 500;
        }
    }
}

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
    players: Query<(Entity, &mut NetControl), With<NetControl>>,
    lights: Res<Lights>,
) {
    // if query.is_empty() {
    //     commands.spawn(Camera2d);
    // }

    for i in players {
        let mut model_select;
        let mut start_pos_x = 0.;
        let mut start_pos_y = 0.;
        match i.1.player_id {
            0 => {
                model_select = "player/player_albedo_blue.png";
                start_pos_x = -3200. + (64. * 8.);
                start_pos_y = 3200. - (64. * 4.);
            }
            1 => {
                model_select = "player/player_albedo_purple.png";
                start_pos_x = -3200. + (64. * 12.);
                start_pos_y = 3200. - (64. * 4.);
            }
            2 => {
                model_select = "player/player_albedo_yellow.png";
                start_pos_x = -3200. + (64. * 8.);
                start_pos_y = 3200. - (64. * 8.);
            }
            3 => {
                model_select = "player/player_albedo_orange.png";
                start_pos_x = -3200. + (64. * 12.);
                start_pos_y = 3200. - (64. * 8.);
            }
            _ => model_select = "player/player_albedo.png",
        }
        commands.entity(i.0).insert((
            // For any entities that we want to have lighting,
            // add the following two components.
            Mesh2d(meshes.add(Rectangle::default())),
            MeshMaterial2d(materials.add(PlayerBaseMaterial {
                // Generally, only change what's inside the 'lighting' struct and the 'texture' and 'normal' parameters.
                color: LinearRgba::BLUE,
                texture: Some(asset_server.load(model_select)),
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
            Transform::from_xyz(start_pos_x, start_pos_y, 0.).with_scale(Vec3::splat(128.)), // Change size of player here: current size: 64. (makes player 64x larger)
            // you can have a smaller player with 32 and larger player with 128
            Velocity::new(),
            FireCooldown(Timer::from_seconds(0.2, TimerMode::Repeating)),
            Player { charge: 500, flashlight: None },
            Health::new(MAX_HEALTH),
            PlayerInventory::default(),
            KinematicCollider {
                shape: Aabb2d {
                    min: Vec2 { x: 0., y: 0. },
                    max: Vec2 { x: 64., y: 64. },
                },
            },
            FlowMap::default(),
            InputHistory::default(),
        ));

        // Identify the local player and store it
        if i.1.get_type() == PlayerType::Local {
            commands.insert_resource(LocalPlayer { entity: i.0 });
            info!("Registered LocalPlayer: {:?}", i.0);
        }

    }
}

pub fn shape_collides_statics(
    collider_aabb: &Aabb2d,
    collider_pos: &Vec2,
    statics: Query<(&StaticCollider, &Transform), (Without<KinematicCollider>, Without<Door>)>,
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
    statics: Query<(&StaticCollider, &Transform), (Without<KinematicCollider>, Without<Door>)>,
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
        (&mut Transform, &mut Velocity, &mut NetControl, &KinematicCollider, &mut InputHistory),
        (With<Player>, With<NetControl>),
    >,
    statics: Query<(&StaticCollider, &Transform), (Without<KinematicCollider>, Without<Door>)>,
) {
    for (mut transform, mut velocity, mut control, player_collider, hist) in player_net {
        let mut dir = Vec2::ZERO;
        
        if control.get_type() == PlayerType::Local && !control.host {
            //info!("Rollback = {:?}", control.rollback)
        }

        if control.get_type() == PlayerType::Local {//&& !control.rollback {
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

            //Debug for checking current pos
            if input.pressed(KeyCode::KeyO) {
                info!("Player {}'s Current Position -> {:?}", control.player_id, transform.translation);
            }

        //REMOTE PLAYER INPUTS ON HOST
        } else if control.host {
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

            //REMOTE PLAYER ANGLE ON HOST
            let rounded_rot_z = control.get_angle();
            //info!("Player {}: {:?}", control.player_id, rounded_rot_z);
            transform.rotation = Quat::from_rotation_z(rounded_rot_z - consts::PI / 2.);

        //REMOTE PLAYER ON REMOTE
        } else {

            if control.get_type() == PlayerType::Local {
                info!("Rollback {}: {:?}", control.player_id, control.get_p_pos());
            }
            
            transform.translation = control.get_p_pos();
            let rounded_rot_z = control.get_angle();
            transform.rotation = Quat::from_rotation_z(rounded_rot_z - consts::PI / 2.);
            control.rollback = false;
            continue;
        }

        let deltat = time.delta_secs();
        let accel = ACCEL_RATE * deltat;
        //info!("deltat = {:?}", deltat);

        **velocity = if dir.length() > 0. {
            (**velocity + (dir.normalize_or_zero() * accel)).clamp_length_max(PLAYER_SPEED)
        } else if velocity.length() > accel {
            **velocity + (velocity.normalize_or_zero() * -accel)
        } else {
            Vec2::ZERO
        };

        let change = **velocity * deltat;


        //If a rollback is detected, set the starting position to the server stated position
        if control.get_type() == PlayerType::Local && control.rollback {
            transform.translation = control.get_p_pos(); 
        }

        transform.translation += change.extend(0.);

        //keep player in bounds
        let max = Vec3::new(
            100. * 64. / 2. - PLAYER_SIZE / 2.,
            100. * 64. / 2. - PLAYER_SIZE / 2.,
            0.,
        );

        let min = max.clone() * -1.;

        let translate = (transform.translation + change.extend(0.)).clamp(min, max);

        transform.translation = translate;

        //Collision check
        for (sc, st) in &statics {
            let mut transformed_kc_shape = player_collider.shape.clone();
            transformed_kc_shape.min += transform.translation.truncate();
            transformed_kc_shape.max += transform.translation.truncate();

            let mut transformed_sc_shape = sc.shape.clone();
            transformed_sc_shape.min += st.translation.truncate();
            transformed_sc_shape.max += st.translation.truncate();

            let colliding = transformed_kc_shape.intersects(&transformed_sc_shape);
            if colliding {
                transform.translation = transform.translation
                    + find_mtv(&transformed_kc_shape, &transformed_sc_shape).extend(0.);
            }
        }

        //Rounds position to integers
        transform.translation.x = transform.translation.x.round();
        transform.translation.y = transform.translation.y.round();
        
        if control.get_type() == PlayerType::Local && !control.host && input.pressed(KeyCode::KeyP) {
            info!("Steps: {:?}", transform.translation);
        }

        //Sets position in NetControl
        control.set_pos_x(transform.translation.x);
        control.set_pos_y(transform.translation.y);
    }
}

pub fn player_movement_from_history(
    time: Res<Time>,
    player_net: Query<
        (&mut Transform, &mut Velocity, &mut NetControl, &KinematicCollider, &mut InputHistory),
        (With<Player>, With<NetControl>),
    >,
    statics: Query<(&StaticCollider, &Transform), Without<KinematicCollider>>,
    mut roll: ResMut<RollbackDetection>,
) {
    for (mut transform, mut velocity, mut control, player_collider, mut hist) in player_net {

        //Check if correct player for rollback
        if control.player_id == hist.player && hist.usable {
            hist.history_used();

            info!("hist.last_pos -> {:?}", hist.last_pos);

            //Reset player position before rollback
            let mut trans_temp = hist.last_pos;
            //transform.translation = hist.last_pos;

            let input_seq;

            if (hist.start > hist.end) {
                input_seq = (hist.start..=255).chain(0..=hist.end).into_iter();
            }
            else {
                //This is just hist.start..=hist.end, but built with a chain so they could be assigned to the same var for the for loop
                input_seq = (hist.start..=(hist.start+1)).chain((hist.start+2)..=hist.end);
            }

            let mut counter = 0;

            for i in input_seq {
                let mut dir = Vec2::ZERO;
                counter += 1;

                if NetControl::pressed_u8(KeyCode::KeyA, hist.complete_history[i as usize]) {
                    dir.x -= 1.;
                }

                if NetControl::pressed_u8(KeyCode::KeyD, hist.complete_history[i as usize]) {
                    dir.x += 1.;
                }

                if NetControl::pressed_u8(KeyCode::KeyW, hist.complete_history[i as usize]) {
                    dir.y += 1.;
                }

                if NetControl::pressed_u8(KeyCode::KeyS, hist.complete_history[i as usize]) {
                    dir.y -= 1.;
                }

                let deltat = 0.015625;//time.delta_secs();
                let accel = ACCEL_RATE * deltat;
                //info!("deltat = {:?}", deltat);

                **velocity = if dir.length() > 0. {
                    (**velocity + (dir.normalize_or_zero() * accel)).clamp_length_max(PLAYER_SPEED)
                } else if velocity.length() > accel {
                    **velocity + (velocity.normalize_or_zero() * -accel)
                } else {
                    Vec2::ZERO
                };

                let change = **velocity * deltat;


                //transform.translation += change.extend(0.);
                trans_temp += change.extend(0.);

                //keep player in bounds
                let max = Vec3::new(
                    100. * 64. / 2. - PLAYER_SIZE / 2.,
                    100. * 64. / 2. - PLAYER_SIZE / 2.,
                    0.,
                );

                let min = max.clone() * -1.;

                let translate = (transform.translation + change.extend(0.)).clamp(min, max);

                //HERE
                //transform.translation = translate;
                trans_temp = translate;

                //Collision check
                /*for (sc, st) in &statics {
                    let mut transformed_kc_shape = player_collider.shape.clone();
                    transformed_kc_shape.min += transform.translation.truncate();
                    transformed_kc_shape.max += transform.translation.truncate();

                    let mut transformed_sc_shape = sc.shape.clone();
                    transformed_sc_shape.min += st.translation.truncate();
                    transformed_sc_shape.max += st.translation.truncate();

                    let colliding = transformed_kc_shape.intersects(&transformed_sc_shape);
                    if colliding {
                        transform.translation = transform.translation
                        + find_mtv(&transformed_kc_shape, &transformed_sc_shape).extend(0.);
                    }
                }*/
                for (sc, st) in &statics {
                    let mut transformed_kc_shape = player_collider.shape.clone();
                    transformed_kc_shape.min += trans_temp.truncate();
                    transformed_kc_shape.max += trans_temp.truncate();

                    let mut transformed_sc_shape = sc.shape.clone();
                    transformed_sc_shape.min += st.translation.truncate();
                    transformed_sc_shape.max += st.translation.truncate();

                    let colliding = transformed_kc_shape.intersects(&transformed_sc_shape);
                    if colliding {
                        trans_temp = trans_temp
                        + find_mtv(&transformed_kc_shape, &transformed_sc_shape).extend(0.);
                    }
                }

                //Rounds position to integers
                trans_temp.x = trans_temp.x.round();
                trans_temp.y = trans_temp.y.round();
                info!("History Traceback Iter: {:?}, Seq: {:?} -> {:?}", counter, i, trans_temp);

                //Sets position in NetControl
                //control.set_pos_x(trans_temp.x);
                //control.set_pos_y(trans_temp.y);
            }

            //info!("Setting player position via REMOTE PLAYER ON REMOTE for Player {}: {:?}", control.player_id, transform.translation);

            control.set_pos_x(trans_temp.x);
            control.set_pos_y(trans_temp.y);

            roll.is_rollback = false;
            control.rollback = false;
        }
    }
}

pub fn player_orientation(
    mut player_net: Query<
        (
            &mut MeshMaterial2d<PlayerBaseMaterial>,
            &mut Transform,
            &mut NetControl,
        ),
        (With<Player>, With<NetControl>),
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
            for (mut material, mut player_transform, mut netcontrol) in player_net.iter_mut() {
                let mut rounded_rot_z = 0.;

                if netcontrol.player_type == PlayerType::Local {
                    let player_position = player_transform.translation.truncate();
                    let direction = cursor_world_position - player_position;

                    if direction.length() > 0.0 {
                        let rotation_z = direction.y.atan2(direction.x);
                        //Rounding is needed to prevent precision errors when networking
                        rounded_rot_z = (rotation_z * 10.).round() / 10.;
                        //info!("PL_ROT Player {}: {:?}", netcontrol.player_id, rounded_rot_z);
                        netcontrol.set_angle(rounded_rot_z);
                    }
                    player_transform.rotation =
                        Quat::from_rotation_z(rounded_rot_z - consts::PI / 2.);
                }
            }

            /*  else
            {
                for (mut material, mut player_transform, mut localcontrol) in player_local.iter_mut() {

                    let mut rounded_rot_z = 0.;

                    if localcontrol.player_type == PlayerType::Local {
                        let player_position = player_transform.translation.truncate();
                        let direction = cursor_world_position - player_position;

                        if direction.length() > 0.0 {
                            let rotation_z = direction.y.atan2(direction.x);
                            //Rounding is needed to prevent precision errors when networking
                            rounded_rot_z = (rotation_z * 10.).round()/10.;
                            localcontrol.set_angle(rounded_rot_z);
                        }
                    }
                    player_transform.rotation = Quat::from_rotation_z(rounded_rot_z - consts::PI / 2.);
                }
            }*/
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
        info!(
            "EMIT_DAMAGE: target={:?}, amount={}",
            damage_event.target,
            damage_event.amount
        );
        for (player_entity, mut player_health) in players.iter_mut() {
            if damage_event.target == player_entity {
                player_health.damage(damage_event.amount);
                if player_health.is_dead() {
                    // Mark as dead instead of despawning
                    player_health.current = 0;
                    commands.entity(player_entity).insert(crate::components::Dead);
                    info!("Player {:?} died", player_entity);
                }
            }
        }
    }
}

pub fn drain_battery(
    time: Res<Time>,
    mut players: Query<&mut Player>,
    mut lights: Query<&mut LightSource>,
) {
    let delta = time.delta_secs();
    let max_range = 500.0;
    let min_range = 50.0;

    for mut player in players.iter_mut() {
        // Drain battery
        let drain_rate = 0.5;
        if player.charge <= 0 {
            //instant_kill();
            // kill them
        }
        player.charge = (player.charge as f32 - drain_rate * delta).max(0.0) as i32;

        // Update flashlight range.
        if let Some(flashlight_entity) = player.flashlight {
            if let Ok(mut light) = lights.get_mut(flashlight_entity) {
                light.range = min_range + (max_range - min_range) * (player.charge as f32 / 500.0);
            }
        }
    }
}

