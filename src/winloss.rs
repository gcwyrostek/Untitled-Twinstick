use crate::{GameState, components::Dead, player::Player};
use bevy::prelude::*;

pub struct WinLossPlugin;
impl Plugin for WinLossPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, check_for_win.run_if(in_state(GameState::Playing)))
            .add_systems(Update, check_for_lose.run_if(in_state(GameState::Playing)));
    }
}

fn check_for_win(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    player_loc: Query<
        (&Transform),
        (With<Player>, Without<Dead>),
    >,
) {
    if !player_loc.is_empty() {
        let mut flag = true;
        for trans in player_loc {
            let extract_range = (Vec3::new(2350., -3100., trans.translation.z) - trans.translation).length();
            if extract_range > 250. {
                flag = false;
            } 
        }
        if flag {
            next_state.set(GameState::Credits);
        }
    }
}

fn check_for_lose(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    livingplayers: Query<
        (Entity),
        (With<Player>, Without<Dead>),
    >,
) {
    if livingplayers.is_empty() {
        next_state.set(GameState::GameOver);
    }
}