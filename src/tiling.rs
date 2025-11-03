use crate::camera::MapBounds;
use crate::{GameState, WIN_H, WIN_W};
use bevy::prelude::*;

const TILE_SIZE: u32 = 128;

pub struct TilingPlugin;
impl Plugin for TilingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_tiling);
    }
}

#[derive(Component)]
pub struct Tile;

pub fn setup_tiling(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let ground_handle = asset_server.load("textures/ground.png");
    let ground_layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), 3, 1, None, None);
    let ground_layout_len = ground_layout.textures.len();
    let ground_layout_handle = texture_atlases.add(ground_layout);

    // Map is 2x the window size
    let map_width = WIN_W * 2.0;
    let map_height = WIN_H * 2.0;

    //We currently only tile over the 1280x720 window, we can adjust this to instead work with level size
    let x_bound = map_width / 2.0 - (TILE_SIZE as f32) / 2.0;
    let y_bound = map_height / 2.0 - (TILE_SIZE as f32) / 2.0;

    let mut x = 0;
    let mut y = 0;
    let mut t = Vec3::new(-x_bound, -y_bound, -10.);

    // Tile the entire map area
    while (y as f32) * (TILE_SIZE as f32) < map_height {
        while (x as f32) * (TILE_SIZE as f32) < map_width {
            commands.spawn((
                Sprite::from_atlas_image(
                    ground_handle.clone(),
                    TextureAtlas {
                        layout: ground_layout_handle.clone(),

                        //This will randomly select a tile from the map, to add variety
                        index: rand::random_range(0..ground_layout_len),
                    },
                ),
                Transform {
                    translation: t,
                    ..default()
                },
                Tile,
            ));

            x += 1;
            t += Vec3::new(TILE_SIZE as f32, 0., 0.);
        }
        x = 0;
        t = Vec3::new(-x_bound, -y_bound, -10.);

        y += 1;
        t += Vec3::new(0., (y as f32) * (TILE_SIZE as f32), 0.);
    }

    // Insert map bounds resource
    commands.insert_resource(MapBounds {
        width: map_width,
        height: map_height,
    });

    info!(
        "Map bounds set to width: {}, height: {}",
        map_width, map_height
    );
    info!("Total tiles spawned: {}", x * y);
}
