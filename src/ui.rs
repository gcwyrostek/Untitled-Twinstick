use crate::{GameState, player::Player};
use crate::inventory_ui::{setup_revive_ui, update_revive_ui};
use crate::player::{use_revive_kit};
use crate::{
    collectible::PlayerInventory, components::Health, events::DamagePlayerEvent,
};
use bevy::prelude::*;

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_ui)
            .add_systems(OnExit(GameState::Playing), cleanup_ui)
            .add_systems(Update, player_damage.run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::Playing), setup_revive_ui)
            .add_systems(Update, update_revive_ui.run_if(in_state(GameState::Playing)))
            .add_systems(Update, use_revive_kit.run_if(in_state(GameState::Playing)))
            .add_systems(Update, (player_damage, update_ammo_ui).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
struct HealthUIRoot;

#[derive(Component)]
struct AmmoUIRoot;

#[derive(Component)]
struct AmmoText;

const HEALTH_BAR_W: f32 = 64.0;
const HEALTH_BAR_H: f32 = 216.0;

pub fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    inventory: Res<PlayerInventory>,
) {
    // Root UI node positioned at top-left of the camera
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(8.0),
                top: Val::Px(8.0),
                width: Val::Px(HEALTH_BAR_W),
                height: Val::Px(HEALTH_BAR_H),
                ..default()
            },
            HealthUIRoot,
        ))
        .with_children(|parent| {
            // Health bar background
            parent.spawn((
                ImageNode::new(asset_server.load("textures/rust.png")),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
            ));

            // Health meter that grows upward
            parent.spawn((
                ImageNode::new(asset_server.load("textures/health.png")),
                Node {
                    width: Val::Percent(80.0),
                    height: Val::Percent(90.0),
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    ..default()
                },
                HealthBar,
            ));
        });

    // Ammo text in the bottom-right corner
    let ammo_string = format!("{}/{}", inventory.magazine, inventory.reserve);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(24.0),
                bottom: Val::Px(24.0),
                ..default()
            },
            AmmoUIRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(ammo_string),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                AmmoText,
            ));
        });
}

pub fn player_damage(
    mut events: EventReader<DamagePlayerEvent>,
    mut health_bar_q: Query<&mut Node, With<HealthBar>>,
    players: Query<(Entity, &Health), With<Player>>,
) {
    let Ok(mut node) = health_bar_q.single_mut() else {
        return;
    };

    for damage_event in events.read() {
        for (player_entity, player_health) in players.iter() {
            if damage_event.target == player_entity {
                let fraction =
                    (player_health.current as f32 / player_health.max as f32).clamp(0.0, 1.0);
                node.height = Val::Px(HEALTH_BAR_H * fraction);
            }
        }
    }
}

fn cleanup_ui(
    mut commands: Commands,
    health_query: Query<Entity, With<HealthUIRoot>>,
    ammo_query: Query<Entity, With<AmmoUIRoot>>,
) {
    for entity in &health_query {
        commands.entity(entity).despawn();
    }

    for entity in &ammo_query {
        commands.entity(entity).despawn();
    }
}

fn update_ammo_ui(
    inventory: Res<PlayerInventory>,
    mut query: Query<&mut Text, With<AmmoText>>,
) {
    if !inventory.is_changed() {
        return;
    }

    if let Ok(mut text) = query.get_single_mut() {
        *text = Text::new(format!("{}/{}", inventory.magazine, inventory.reserve));
    }
}
