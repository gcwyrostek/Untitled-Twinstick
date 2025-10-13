use crate::components::{Collectible, CollectibleKind, Health};
use crate::player::Player;
use bevy::prelude::*;

/// how close to pick up
const PICKUP_RADIUS: f32 = 32.0;

/// collecting ammo
#[derive(Event, Debug, Clone, Copy)]
pub struct AmmoPickupEvent {
    pub amount: i32,
}

/// collecting battery
#[derive(Event, Debug, Clone, Copy)]
pub struct BatteryPickupEvent {
    pub amount: i32,
}

/// collecting revive kit
#[derive(Event, Debug, Clone, Copy)]
pub struct ReviveKitPickupEvent;

/// Plugin
pub struct PickupPlugin;

impl Plugin for PickupPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AmmoPickupEvent>()
            .add_event::<BatteryPickupEvent>()
            .add_event::<ReviveKitPickupEvent>()
            .add_systems(Startup, spawn_revive_kit)
            .add_systems(Update, pickup_system);
    }
}

fn spawn_revive_kit(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite::from_image(asset_server.load("revive kit/Revive Kit_albedo.png")),
        Transform::from_xyz(200.0, 150.0, 0.0).with_scale(Vec3::new(0.5, 0.5, 0.5)),
        Collectible::revive(),
    ));
    //spawn a second revive kit
    commands.spawn((
        Sprite::from_image(asset_server.load("revive kit/Revive Kit_albedo.png")),
        Transform::from_xyz(-200.0, -150.0, 0.0).with_scale(Vec3::new(0.5, 0.5, 0.5)),
        Collectible::revive(),
    ));
}

/// detect collectibles near the player, apply effects, and despawn pickups.
fn pickup_system(
    mut commands: Commands,
    mut ammo_writer: EventWriter<AmmoPickupEvent>,
    mut battery_writer: EventWriter<BatteryPickupEvent>,
    mut revive_writer: EventWriter<ReviveKitPickupEvent>,
    mut player_q: Query<(&Transform, Option<Mut<Health>>), With<Player>>,
    collectibles_q: Query<(Entity, &Transform, &Collectible)>,
) {
    // single_mut is the non-deprecated call
    let (player_tf, mut player_health_opt) = match player_q.single_mut() {
        Ok(v) => v,
        Err(_) => return,
    };

    for (entity, item_tf, col) in collectibles_q.iter() {
        if player_tf.translation.distance(item_tf.translation) > PICKUP_RADIUS {
            continue;
        }

        match col.kind {
            CollectibleKind::Health => {
                // borrow the inner Health mutably WITHOUT moving it out of the Option
                if let Some(h) = player_health_opt.as_deref_mut() {
                    h.heal(col.amount.max(0));
                }
            }
            CollectibleKind::Ammo => {
                ammo_writer.write(AmmoPickupEvent {
                    amount: col.amount.max(0),
                });
            }
            CollectibleKind::Battery => {
                battery_writer.write(BatteryPickupEvent {
                    amount: col.amount.max(0),
                });
            }
            CollectibleKind::ReviveKit => {
                if let Some(h) = player_health_opt.as_deref_mut() {
                    h.current = h.max; //refill health
                }

                revive_writer.write(ReviveKitPickupEvent);
            }
        }

        commands.entity(entity).despawn();
    }
}
