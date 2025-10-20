use crate::{GameState, components::StaticCollider};
use bevy::{math::bounding::Aabb2d, prelude::*, render::render_resource::DownlevelFlags};
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
            StaticCollider {
                shape: Aabb2d {
                    min: Vec2 { x: 0., y: 0. },
                    max: Vec2 { x: 64., y: 64. },
                },
            },
        ));
    }
}
