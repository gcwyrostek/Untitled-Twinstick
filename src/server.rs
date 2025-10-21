use crate::GameState;
use bevy::prelude::*;
use std::net::UdpSocket;

const IP_CONST: &str = "127.0.0.1:2525";

#[derive(Resource)]
pub struct SocketResource {
    socket: UdpSocket,
}

#[derive(Resource)]
pub struct inFlag {
    pub ready: bool,
}

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Lobby),
            (server_init.before(server_start), server_start),
        )
        .add_systems(FixedUpdate, server_run.run_if(in_state(GameState::Lobby)))
        .add_systems(FixedUpdate, server_run.run_if(in_state(GameState::Playing)))
        .add_systems(OnExit(GameState::Playing), server_close);
    }
}

fn server_init(mut commands: Commands) {
    commands.insert_resource(SocketResource {
        socket: UdpSocket::bind(IP_CONST).expect("ERROR"),
    });
    commands.insert_resource(inFlag {
        ready: false,
    });
}

fn server_close(mut commands: Commands) {
    commands.remove_resource::<SocketResource>();
}

fn server_start(socket: ResMut<SocketResource>) {
    //This makes it so the game doesn't wait to receive a message, before going to the next frame
    socket.socket.set_nonblocking(true);
}

fn server_run(
    socket: ResMut<'_, SocketResource>,
    mut flag: ResMut<inFlag>,
) {
    let mut buf = [0; 10];

    //This might only work for one client at a time, so we may need to adjust this when we get further
    match socket.socket.recv_from(&mut buf) {
        Ok((amt, src)) => {
            //info!("{:?} + {:?} + {:?}", amt, src, buf);
            if (buf[0] == 144) {
                //info!("it happened");
                flag.ready = true;
            }
            socket
                .socket
                .send_to(&[1; 10], src)
                .expect("couldn't send data");
        }
        Err(e) => {
            //info!("Nothing");
        }
    }
}
