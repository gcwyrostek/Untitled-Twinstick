use crate::{
    GameState, components::StaticCollider, components::ZoneBox
};
use bevy::{prelude::*, render::render_resource::DownlevelFlags};
use std::f32::consts;

pub struct WallPlugin;
impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_walls);
    }
}

pub fn setup_walls(mut commands: Commands, asset_server: Res<AssetServer>) {
    for i in 0..=9 {
        commands.spawn((
            Sprite::from_image(asset_server.load("textures/wall.png")),
            Transform::from_xyz(-100., (i * 64) as f32, 0.),
            StaticCollider{
                shape: ZoneBox{
                    nw_corner: Vec2 { x: 0.0, y: 0.0 },
                    se_corner: Vec2 { x: 64.0, y: 64.0},
                },
            },
        ));
    }
}

