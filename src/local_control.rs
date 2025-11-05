use crate::{GameState, net_control::PlayerType, player::Player};
use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

//LocalControl gives the application access to the information sent by the clients
#[derive(Component)]
pub struct LocalControl {
    pub player_type: PlayerType,
    pub p_pos: Vec3,
    pub p_angle: u8,
    pub player_id: u8,
    //player_addr: SocketAddr,
}
impl LocalControl {
    pub fn new(ptype: PlayerType, pid: u8) -> Self {
        Self {
            player_type: ptype,
            p_pos: Vec3::ZERO,
            p_angle: 0,
            player_id: pid,
        }
    }

    //Getter for player_type
    pub fn get_type(&self) -> PlayerType {
        return self.player_type;
    }

    //Setter for net_angle
    pub fn set_angle(&mut self, angle: f32) {
        //This assumes that you've already rounded the float to 1 decimal point
        let angle_as_i8 = (angle * 10.) as i8;
        self.p_angle = angle_as_i8 as u8;
    }

    //Getter for p_angle
    pub fn get_angle(&self) -> f32 {
        //This assumes that you've already rounded the float to 1 decimal point
        let angle_as_i8 = (self.p_angle as i8);
        return angle_as_i8 as f32;
    }

    pub fn set_p_pos(&mut self, pack: [u8; 10]) {
        let mut unpack_x: [u8; 4] = [0; 4];
        let mut unpack_y: [u8; 4] = [0; 4];
        unpack_x.copy_from_slice(&pack[2..6]);
        unpack_y.copy_from_slice(&pack[6..10]);
        let x = i32::from_ne_bytes(unpack_x);
        let y = i32::from_ne_bytes(unpack_y);
        self.p_pos = Vec3::new(x as f32, y as f32, 0.);
        //info!("Player {}'s Position: {:?}", self.player_id, self.p_pos);
    }

    pub fn get_p_pos(&self) -> Vec3 {
        return self.p_pos;
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////
}

pub struct LocalControlPlugin;
impl Plugin for LocalControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Lobby), local_control_init);
    }
}

fn local_control_init(mut commands: Commands) {
    /*commands.spawn((
        NetControl::new(PlayerType::Local, 0, SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2525)),
    ));*/
}
