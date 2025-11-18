use crate::GameState;
use crate::collectible::Collectible;
use crate::enemy::Enemy;
use crate::player::Player;
use crate::tiling::Tile;
use crate::ui::HealthBar;
use bevy::prelude::*;

pub struct GameOverPlugin;
impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), display_game_over)
            .add_systems(Update, wait_for_input.run_if(in_state(GameState::GameOver)));
    }
}

#[derive(Component)]
pub struct GameOverScreen;

pub fn display_game_over(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            GameOverScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 96.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.0, 0.0)), //red
            ));
        });
}

fn wait_for_input(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    input: Res<ButtonInput<KeyCode>>,
    mouse_button_io: Res<ButtonInput<MouseButton>>,
    query_player: Query<Entity, With<Player>>,
    query_ui: Query<Entity, With<HealthBar>>,
    query_tiles: Query<Entity, With<Tile>>,
    query_collectible: Query<Entity, With<Collectible>>,
    query_enemy: Query<Entity, With<Enemy>>,
    query_gameover: Query<Entity, With<GameOverScreen>>,
) {
    if mouse_button_io.pressed(MouseButton::Left) {
        for entity in query_player.iter() {
            commands.entity(entity).despawn();
        }
        for entity in query_ui.iter() {
            commands.entity(entity).despawn();
        }
        for entity in query_tiles.iter() {
            commands.entity(entity).despawn();
        }
        for entity in query_collectible.iter() {
            commands.entity(entity).despawn();
        }
        for entity in query_enemy.iter() {
            commands.entity(entity).despawn();
        }
        for entity in query_gameover.iter() {
            commands.entity(entity).despawn();
        }
        next_state.set(GameState::Menu);
    }
}
