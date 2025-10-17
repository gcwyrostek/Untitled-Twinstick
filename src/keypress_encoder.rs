use bevy::prelude::*;
use bevy::input::mouse::MouseButton;
use crate::GameState;

pub struct KeyEncodePlugin;
impl Plugin for KeyEncodePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(FixedUpdate, input_converter.run_if(in_state(GameState::Playing)));
    }
}

pub fn input_converter(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mouse_button_io: Res<ButtonInput<MouseButton>>,
) {
    let mut out: u8 = 0;
    //WASDL
    if input.pressed(KeyCode::KeyW) {
        out += 128;
    }
    
    if input.pressed(KeyCode::KeyA) {
        out += 64;
    }
    
    if input.pressed(KeyCode::KeyS) {
        out += 32;
    }
    
    if input.pressed(KeyCode::KeyD) {
        out += 16;
    }
    
    if mouse_button_io.pressed(MouseButton::Left) {
        out += 2;
    }

    //info!("WASD");
    //info!("{:08b}", out);
    
    /*for i in input.get_pressed() {
        info!("(KEYBOARD) {:?} is pressed.", i);
    }

    for i in mouse_button_io.get_pressed(){
        info!("(MOUSE) {:?} is pressed.", i);
    }*/
}
