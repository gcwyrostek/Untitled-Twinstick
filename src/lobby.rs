use crate::GameState;
use crate::server::inFlag;
use bevy::prelude::*;

pub struct LobbyPlugin;
impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Lobby), display_lobby)
            // Clean up lobby camera when exiting to prevent double cameras from breaking rotation/shooting in Playing state
            .add_systems(OnExit(GameState::Lobby), cleanup_lobby)
            .add_systems(Update, wait_for_input.run_if(in_state(GameState::Lobby)));
    }
}

#[derive(Component)]
pub struct LobbyScreen;

// Tag component to identify the lobby camera so it can be cleaned up when leaving lobby state
#[derive(Component)]
pub struct LobbyCamera;

fn cleanup_lobby(mut commands: Commands, query: Query<Entity, (With<Camera>, With<LobbyCamera>)>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn display_lobby(
    mut commands: Commands,
    query: Query<Entity, (With<Camera>, With<LobbyCamera>)>,
) {
    // Spawn lobby camera if none exists - needed to display lobby UI and avoid grey screen
    if query.is_empty() {
        commands.spawn((Camera2d, LobbyCamera));
    }

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
            LobbyScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Lobby, press 'P' or connect"),
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
    lobbyscreen: Single<Entity, With<LobbyScreen>>,
    mut flag: ResMut<inFlag>,
) {
    if input.pressed(KeyCode::KeyP) || flag.ready {
        commands.entity(*lobbyscreen).despawn();
        next_state.set(GameState::Playing);
    }
}
