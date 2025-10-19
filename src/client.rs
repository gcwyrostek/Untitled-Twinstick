use crate::GameState;
use bevy::prelude::*;
use std::net::UdpSocket;

const IP_CONST: &str = "127.0.0.1:25125";

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
        .add_systems(FixedUpdate, client_run.run_if(in_state(GameState::Joining)))
        .add_systems(OnExit(GameState::Playing), client_close);
    }
}

fn client_init(mut commands: Commands) {
    info!("In client init");
    commands.insert_resource(SocketResource {
        socket: UdpSocket::bind(IP_CONST).expect("ERROR"),
    });
}

fn client_start(socket: ResMut<SocketResource>) {
    info!("In client start");
    //This makes it so the game doesn't wait to receive a message, before going to the next frame
    socket.socket.set_nonblocking(true);
}

fn client_close(mut commands: Commands) {
    commands.remove_resource::<SocketResource>();
}

fn client_connect(socket: ResMut<SocketResource>) {
    info!("In client connect");
    let mut buf = [0; 10];
    socket
        .socket
        .send_to(&[1; 10], "127.0.0.1:2525")
        .expect("couldn't send data");
    match socket.socket.recv_from(&mut buf) {
        Ok((amt, src)) => {
            info!("{:?} + {:?} + {:?}", amt, src, buf);
        }
        Err(e) => {
            info!("{:?}", e);
        }
    }
}

fn client_run(socket: ResMut<'_, SocketResource>) {
    //let mut buf = [0; 10];
    socket
        .socket
        .send_to(&[9; 10], "127.0.0.1:2525")
        .expect("couldn't send data");

    /*match socket.socket.recv_from(&mut buf)
    {
        Ok((amt, src)) => {
            info!("{:?} + {:?} + {:?}", amt, src, buf);
        }
        Err(e) => {
            //info!("ERROR");
        }
    }*/
}
