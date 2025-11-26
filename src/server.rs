use crate::{
    AssignedType, GameState, LogicType, net_control::NetControl, net_control::PlayerType, net_control::Local, net_control::Network,
    player::Player, player::Velocity, player,
};
use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::thread;

const IP_CONST: &str = "127.0.0.1:2525";
const MAX_PLAYER: u8 = 4;

#[derive(Resource)]
pub struct SocketResource {
    socket: UdpSocket,
}

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Lobby),
            (server_init.before(server_start), server_start),
        )
        .add_systems(
            OnEnter(GameState::Playing),
            (send_players).run_if(type_equals_host),
        )
        .add_systems(Update, server_run.run_if(in_state(GameState::Lobby)))
        .add_systems(
            Update,
            server_run
                .run_if(in_state(GameState::Playing))
                .run_if(type_equals_host),
        )
        .add_systems(
            FixedUpdate,
            send_player_update
                .run_if(in_state(GameState::Playing))
                .run_if(type_equals_host),
        )
        //Debug only module
        .add_systems(
            FixedLast,
            connection_health
                .run_if(in_state(GameState::Playing))
                .run_if(type_equals_host),
        )
        .add_systems(
            OnExit(GameState::Playing),
            server_close.run_if(type_equals_host),
        );
    }
}

#[derive(Resource)]
pub struct RollbackDetection {
    pub is_rollback: bool,
}
impl Default for RollbackDetection {
    fn default() -> Self {
        Self {
            is_rollback: false,
        }
    }
}

fn type_equals_host(game_type: Res<LogicType>) -> bool {
    return game_type.l_type == AssignedType::Host;
}


fn server_init(mut commands: Commands) {
    commands.insert_resource(SocketResource {
        socket: UdpSocket::bind(IP_CONST).expect("ERROR"),
    });
    commands.insert_resource(ServerMetrics::default());
    commands.spawn((NetControl::new(true, PlayerType::Local, 0, Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2525,)),),
                    Local,
    ),
);
}

fn server_close(mut commands: Commands) {
    commands.remove_resource::<SocketResource>();
}

fn server_start(socket: ResMut<SocketResource>) {
    //This makes it so the game doesn't wait to receive a message, before going to the next frame
    socket.socket.set_nonblocking(true);
}

fn server_run(
    mut commands: Commands,
    socket: ResMut<'_, SocketResource>,
    mut player: Query<(&mut NetControl, &mut InputHistory), With<NetControl>>,
    mut sm: ResMut<ServerMetrics>,
    mut roll: ResMut<RollbackDetection>,
) {
    let mut buf = [0; 257];
    for l in 1..8 {
        match socket.socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                //Server Receives Join Packet
                if (buf[0] == 255) {
                    //info!("{:?} + {:?} + {:?}", src, amt, buf);
                    if sm.player_count < MAX_PLAYER {
                        //Creates NetControl for connecting player
                        let tempNet = commands
                            .spawn(
                                (NetControl::new(
                                    true,
                                    PlayerType::Network,
                                    sm.player_count,
                                    Some(src),
                                ),
                                Network,
                            ),
                            )
                            .id();
                        //Adds one to player count in ServerMetrics
                        sm.player_count += 1;

                        //Clock Sync
                        socket
                            .socket
                            .send_to(
                                &[2, sm.player_count],
                                src,
                            )
                            .expect("couldn't send data");


                    } else {
                        //Send Reject Message
                    }
                }

                //Input Byte
                for (mut a, mut history) in player.iter_mut() {
                    if a.get_addr().unwrap() == src {
                        //Normal Input Packet
                        if amt != 257 {
                            sm.packets_rcvd += 1;
                            a.net_input = buf[0];
                            //info!("{:?}", buf[1]);
                            a.net_angle = buf[1];

                            //ServerMetric updates for testing
                            sm.packets[a.player_id as usize] += 1;
                            //info!("Amount: {:?} -> {:?}", amt, buf);
                            sm.seq[a.player_id as usize] = buf[2];
                        }
                        //Input History packet
                        else {
                            //Set up InputHistory attached to player
                            //commands.insert_resource(InputHistory::new(a.player_id, buf, sm.last_conf_seq[a.player_id as usize], buf[256], sm.last_pos[a.player_id as usize]));
                            history.set_all(a.player_id, buf, sm.last_conf_seq[a.player_id as usize], buf[256], sm.last_pos[a.player_id as usize]);
                            
                            //Call Player from vec once (make sure you set the pos of that player)
                            
                            //Triggers player.rs event
                            roll.is_rollback = true;
                            //world.run_system_once(player::player_movement_from_history);

                            //a.rollback = false;
                            
                            //Remove resource

                        }
                    }
                }
            }
            Err(e) => {
                //info!("Nothing");
            }
        }
    }
}

fn send_players(
    socket: ResMut<'_, SocketResource>,
    mut p_net: Query<&mut NetControl, With<NetControl>>,
    mut sm: ResMut<ServerMetrics>,
) {
    for i in p_net {
        if i.get_type() == PlayerType::Network {
            //info!{"{:?}", i.get_addr()};
            socket
                .socket
                .send_to(
                    &[0, sm.player_count, i.player_id],
                    i.get_addr().unwrap(),
                )
                .expect("couldn't send data");
        }
    }
}

fn send_player_update(
    socket: ResMut<'_, SocketResource>,
    mut p_net: Query<&mut NetControl, With<NetControl>>,
    mut sm: ResMut<ServerMetrics>,
) {
    let mut roll_check: [bool; 4] = [false; 4];

    for i in p_net.iter() {
        if i.get_type() == PlayerType::Network {
            sm.packets_sent += 1;
            for j in p_net.iter() {

                // If a rollback is decided, when you send the packet to that player, send with OP code 3 instead 
                if j.rollback && i.player_id == j.player_id && sm.loss[j.player_id as usize] < 255 && false {
                    let out = j.get_out_packet(3, j.player_id);
                    socket
                        .socket
                        .send_to(&out, i.get_addr().unwrap())
                        .expect("couldn't send data");
                    //If the rollback flag is flipped mark it in the bool array
                    roll_check[j.player_id as usize] = true;
                }
                //If packet loss was longer than the history window, just hard roll back
                else if j.rollback && i.player_id == j.player_id {
                    let out = j.get_out_packet(1, j.player_id);
                    socket
                        .socket
                        .send_to(&out, i.get_addr().unwrap())
                        .expect("couldn't send data");
                    roll_check[j.player_id as usize] = true;
                }
                else {
                    let out = j.get_out_packet(1, j.player_id);
                    socket
                        .socket
                        .send_to(&out, i.get_addr().unwrap())
                        .expect("couldn't send data");
                }
            }
        }
    }
}

#[derive(Component)]
pub struct InputHistory {
    pub usable: bool,
    pub player: u8,
    pub complete_history: [u8;257],
    pub start: u8,
    pub end: u8,
    pub last_pos: Vec3,
}
impl Default for InputHistory {
    fn default() -> Self {
        Self {
            usable: false,
            player: 0,
            complete_history: [0;257],
            start: 0,
            end: 0,
            last_pos: Vec3::ZERO,
        }
    }
}
impl InputHistory {
    pub fn set_all(&mut self, pl: u8, history: [u8;257], st: u8, en: u8, lp: Vec3) {
        self.usable = true;
        self.player = pl;
        self.complete_history = history;
        self.start = st;
        self.end = en;
        self.last_pos = lp;
    }
}

#[derive(Resource)]
pub struct ServerMetrics {
    pub player_count: u8,
    pub packets_sent: u8,
    pub packets_rcvd: u8,
    pub packets: Vec<u8>,

    pub seq: Vec<u8>,
    pub last: Vec<u8>,

    pub last_conf_seq: Vec<u8>,
    pub last_pos: Vec<Vec3>,

    pub loss: Vec<u8>,
    
}
impl Default for ServerMetrics {
    fn default() -> Self {
        Self {
            player_count: 1,
            packets_sent: 0,
            packets_rcvd: 0,
            packets: vec![0; 4],

            seq: vec![0; 4],
            last: vec![0; 4],

            last_conf_seq: vec![0; 4],
            last_pos: vec![Vec3::ZERO; 4],

            loss: vec![0; 4],
        }
    }
}
impl ServerMetrics {

}

fn connection_health(
    mut sm: ResMut<ServerMetrics>,
    mut p_net: Query<(&mut NetControl, &mut Transform), With<NetControl>>,
) {
    //info!("[Packets Sent: {}] [Packets Rcvd: {}]", sm.packets_sent, sm.packets_rcvd);
    //info!("[P0: {}] [P1: {}] [P2: {}] [P3: {}]", sm.packets[0], sm.packets[1], sm.packets[2], sm.packets[3]);
    //info!("[P0: {}] [P1: {}] [P2: {}] [P3: {}]", sm.seq[0], sm.seq[1], sm.seq[2], sm.seq[3]);

    for i in 1..(sm.player_count as usize) {
        if sm.seq[i] == sm.last[i] && sm.packets[i] != 0 {
            info!("Dupe Packet for {}: Seq {}", i, sm.seq[i]);
        } else if sm.last[i] == 255 && sm.seq[i] == 0 { 

        } else if (sm.last[i] == 255 && sm.seq[i] != 0) || (sm.last[i] + 1 != sm.seq[i]) {
            let mut dif = 0;
            if sm.seq[i] < sm.last[i] {
                dif = ((sm.seq[i] as u16 + 256) - (sm.last[i] as u16)) as u8;
            } else
            {
                dif = (sm.seq[i] - sm.last[i]) as u8;
            }
            //If we got enough packets to cover the gap
            if sm.packets[i] >= dif as u8 {

            } else {
                info!("Lost Packet(s) for {}: Last {}, Seq {}", i, sm.last[i], sm.seq[i]);
            }
        } 

        if sm.seq[i] == sm.last[i] {
            if sm.loss[i] < 255 {
                sm.loss[i] += 1;
            }
        } else {
            sm.loss[i] = 0;
        }

        if sm.loss[i] >= 5 {
            info!("Player {} missed {} packets!", i, sm.loss[i]);
            for (mut control, mut trans) in &mut p_net {
                if control.player_id == i as u8 {
                    //info!("sm.loss setting {}.rollback = true", i);
                    sm.last_pos[i] = trans.translation;
                    sm.last_conf_seq[i] = sm.last[i];
                    control.rollback = true;
                }
            }
        }

        sm.last[i] = sm.seq[i];
    }

    //Reset
    sm.packets_sent = 0;
    sm.packets_rcvd = 0;
    sm.packets = vec![0; 4];
}
