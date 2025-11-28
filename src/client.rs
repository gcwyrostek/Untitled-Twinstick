use crate::{
    AssignedType, GameState, LogicType, net_control::NetControl, net_control::PlayerType, net_control::Local, net_control::Network,
    player::Player,
    collectible::PlayerInventory,
};
use bevy::prelude::*;
use bevy::time::Stopwatch;
use std::time::Duration;
use std::net::UdpSocket;

const IP_CONST: &str = "127.0.0.1:";

#[derive(Resource)]
pub struct SocketResource {
    socket: UdpSocket,
}

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Joining),
            (
                client_init.before(client_start),
                client_start.before(client_connect),
                client_connect,
            ),
        )
        .add_systems(
            FixedUpdate,
            client_run
                .run_if(in_state(GameState::Joining))
                .run_if(type_equals_client),
        )
        .add_systems(
            FixedUpdate,
            client_run
                .run_if(in_state(GameState::Playing))
                .run_if(type_equals_client),
        )
        .add_systems(
            FixedUpdate,
            input_converter
                .run_if(in_state(GameState::Playing))
                .run_if(type_equals_client),
        )
        .add_systems(
            OnExit(GameState::Playing),
            client_close.run_if(type_equals_client),
        );
    }
}

fn type_equals_client(game_type: Res<LogicType>) -> bool {
    return game_type.l_type == AssignedType::Client;
}

fn client_init(mut commands: Commands) {
    info!("In client init");
    //The random port is so I can hard code it and run more than one client at a time.
    let newIP = IP_CONST.to_owned() + &rand::random_range(25000..25999).to_string();
    commands.insert_resource(SocketResource {
        socket: UdpSocket::bind(newIP).expect("ERROR"),
    });
    commands.insert_resource(ClientMetrics::default());
}

fn client_start(socket: ResMut<SocketResource>) {
    info!("In client start");
    //This makes it so the game doesn't wait to receive a message, before going to the next frame
    socket.socket.set_nonblocking(true);
}

fn client_close(mut commands: Commands) {
    commands.remove_resource::<SocketResource>();
}

fn client_connect(
    socket: ResMut<SocketResource>,
    mut cm: ResMut<ClientMetrics>,
) {
    info!("In client connect, Starting Timer");
    let mut buf = [0];
    cm.sw.tick(Duration::from_millis(1));
    cm.sw.reset();
    socket
        .socket
        .send_to(&[255], "127.0.0.1:2525")
        .expect("couldn't send data");
}

fn client_run(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    socket: ResMut<'_, SocketResource>,
    mut p_loc: Query<(&mut NetControl, &mut Transform), With<NetControl>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut cm: ResMut<ClientMetrics>,
) {
    let mut buf = [0; 11];
    //Fake packet loss option
    if !input.pressed(KeyCode::KeyP) {
        for l in 1..8 {
            match socket.socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    //info!("{:?} + {:?} + {:?}", amt, src, buf);
                    match buf[0] {
                        //Code 0 -> Game Started. Send player counts for NetControl initialization.
                        0 => {
                            for i in 0..buf[1] {
                                if i == buf[2] {
                                    commands.spawn(
                                        (NetControl::new(false, PlayerType::Local, i, None),
                                        Local,
                                    )
                                );
                                    info!("I am player: {}", i);
                                } else {
                                    commands.spawn((NetControl::new(
                                        false,
                                        PlayerType::Network,
                                        i,
                                        None,
                                    ),
                                    Network,
                                    )
                                );
                                    info!("Created net player: {}", i);
                                }
                            }
                            //Start the game
                            info!("PLAY STATE");
                            next_state.set(GameState::Playing);
                        }

                        //Code 1 -> Player position/angle update.
                        1 => {
                            cm.rtt = cm.sw.elapsed();
                            //info!("Ping: {:?}", cm.rtt);
                            for (mut control, mut trans) in p_loc.iter_mut() {
                                //The first check hard limits us to 4 players (pid 0 to 3) as I started packing shooting into the same byte.
                                //The second check prevents server from overwriting active player info. Will need to add 'else' to handle rollback system
                                if control.player_id == (buf[1] & 3) && control.player_type == PlayerType::Network {
                                    control.set_player_state(buf);
                                //Sends the updated info except angle, used for rollback
                                } else if control.player_id == (buf[1] & 3) && control.player_type == PlayerType::Local {
                                    //info!("{:08b}", buf[1]);
                                    control.set_player_state_limited(buf);
                                }
                            }
                        }
                        //Code 2 -> Clock Sync
                        2 => {
                            cm.rtt = cm.sw.elapsed();
                            //info!("Ping: {:?}", cm.rtt);
                        }

                        //Request input history
                        3 => {
                            cm.send_history = true;
                        }

                        _ => {
                            info!("{:?} + {:?} + {:?}", amt, src, buf);
                        }
                        
                    }
                }
                Err(e) => {
                    //info!("ERROR");
                }
            }
        }
    }
}

pub fn input_converter(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mouse_button_io: Res<ButtonInput<MouseButton>>,
    socket: ResMut<SocketResource>,
    mut pl_cont: Query<&mut NetControl, (With<NetControl>, With<Local>)>,
    mut inventory: ResMut<PlayerInventory>,
    mut cm: ResMut<ClientMetrics>,
)   {
        let mut input_result: u8 = 0;
        let mut player = pl_cont.single_mut().unwrap();
        //WASD00L0
        if input.pressed(KeyCode::KeyW) {
            input_result += 128;
        }

        if input.pressed(KeyCode::KeyA) {
            input_result += 64;
        }

        if input.pressed(KeyCode::KeyS) {
            input_result += 32;
        }

        if input.pressed(KeyCode::KeyD) {
            input_result += 16;
        }

        //Ideally we would still send the click signal always, but this is easier if we aren't sending ammo info
        if mouse_button_io.pressed(MouseButton::Left) && inventory.has_available_ammo() {
            input_result += 2;
        }

        cm.sw.reset();

        let seq = cm.seq_num;
        cm.input_history[seq as usize] = input_result;
        /*//Save Input
        cm.input_history[seq as usize] = [input_result, player.net_angle, up_seq, low_seq];
        if input.pressed(KeyCode::KeyO) {
            info!("{:?}", cm.input_history);
        }*/

        //Fake packet loss option
        if !input.pressed(KeyCode::KeyP) {

        if !cm.send_history {
            //Send Input
            socket
                .socket
                .send_to(&[input_result, player.net_angle, cm.seq_num], "127.0.0.1:2525")
                .expect("couldn't send data");

            } else {
            cm.send_history = false;
            cm.input_history[256] = seq;
            socket
                .socket
                .send_to(&cm.input_history, "127.0.0.1:2525")
                .expect("couldn't send data");
            }
        }
        //info!("seq = {}", cm.seq_num);
        if cm.seq_num == 255 {
            cm.seq_num = 0;
        } else {
            cm.seq_num += 1;
        }

    //info!("WASD");
    //info!("{:08b}", input_result);

    /*for i in input.get_pressed() {
        info!("(KEYBOARD) {:?} is pressed.", i);
    }

    for i in mouse_button_io.get_pressed(){
        info!("(MOUSE) {:?} is pressed.", i);
    }*/
}

#[derive(Resource)]
pub struct ClientMetrics {
    pub seq_num: u8,
    pub sw: Stopwatch,
    pub rtt: Duration,
    pub input_history: [u8;257],

    pub send_history: bool,
}
impl Default for ClientMetrics {
    fn default() -> Self {
        Self {
           seq_num: 0,
           sw: Stopwatch::new(),
           rtt: Duration::ZERO,
           input_history: [0;257],

           send_history: false,
        }
    }
}
impl ClientMetrics {

}
