use bevy::{prelude::*, input::mouse::MouseButton, input::ButtonInput, asset::LoadState};
use crate::{GameState};

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnEnter(GameState::Menu), setup_menu)
        .add_systems(Update, update_menu.run_if(in_state(GameState::Menu)));
    }
}

fn setup_menu(

) {

}

fn update_menu(
    mouse_button_io: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if (mouse_button_io.pressed(MouseButton::Left)) {
        next_state.set(GameState::Playing);
    }
}