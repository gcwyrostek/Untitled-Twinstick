use crate::{GameState, components::Health, events::DamagePlayerEvent, player::Player};
use bevy::prelude::*;

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_ui)
            .add_systems(OnExit(GameState::Playing), cleanup_ui)
            .add_systems(Update, player_damage.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
struct HealthUIRoot;

const HEALTH_BAR_W: f32 = 64.0;
const HEALTH_BAR_H: f32 = 216.0;

pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
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

fn cleanup_ui(mut commands: Commands, query: Query<Entity, With<HealthUIRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
