use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use crate::components::StaticCollider;
use crate::enemy::Enemy;
use crate::player::Player;
use crate::player_material::PlayerBaseMaterial;

// texture size for the shadow map
const SDF_TEXTURE_SIZE: u32 = 512;

// world size that the texture covers
const SDF_WORLD_SIZE: f32 = 5120.0;

// how far from player to check for shadows (600 units = flashlight range + some extra)
const SDF_CULLING_RADIUS: f32 = 600.0;

pub struct SdfShadowsPlugin;

impl Plugin for SdfShadowsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SdfTexture>()
            .add_systems(Update, generate_sdf)
            .add_systems(Update, update_material_sdf_textures);
    }
}

// holds the shadow texture
#[derive(Resource)]
pub struct SdfTexture {
    pub texture: Handle<Image>,
}

impl FromWorld for SdfTexture {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();

        let size = Extent3d {
            width: SDF_TEXTURE_SIZE,
            height: SDF_TEXTURE_SIZE,
            depth_or_array_layers: 1,
        };

        let mut image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0u8; 4],
            TextureFormat::R32Float,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );

        // fill with big numbers initially (no shadows)
        let data: Vec<f32> = vec![SDF_WORLD_SIZE; (SDF_TEXTURE_SIZE * SDF_TEXTURE_SIZE) as usize];
        image.data = Some(data.iter().flat_map(|f| f.to_le_bytes()).collect());

        let texture = images.add(image);

        SdfTexture { texture }
    }
}

// stuff that blocks light (walls, enemies, etc)
struct Occluder {
    position: Vec2,
    size: Vec2,
}

impl Occluder {
    // calculate distance from a point to this box
    fn distance(&self, point: Vec2) -> f32 {
        let local = point - self.position;
        let half_size = self.size * 0.5;
        let d = local.abs() - half_size;

        // if outside the box, get distance to edge
        // if inside, get negative distance
        let outside = d.max(Vec2::ZERO).length();
        let inside = d.x.max(d.y).min(0.0);

        outside + inside
    }
}

// this runs every frame and creates the shadow map
fn generate_sdf(
    mut images: ResMut<Assets<Image>>,
    sdf_texture: Res<SdfTexture>,
    enemies: Query<&Transform, With<Enemy>>,
    walls: Query<&Transform, With<StaticCollider>>,
    players: Query<&Transform, With<Player>>,
) {
    // get all player positions
    let player_positions: Vec<Vec2> = players.iter()
        .map(|t| t.translation.truncate())
        .collect();

    let mut occluders = Vec::new();

    // add enemies that are close to any player
    for transform in enemies.iter() {
        let pos = transform.translation.truncate();
        if player_positions.iter().any(|p| pos.distance(*p) <= SDF_CULLING_RADIUS) {
            occluders.push(Occluder {
                position: pos,
                size: transform.scale.truncate(),
            });
        }
    }

    // add walls that are close to any player
    for transform in walls.iter() {
        let pos = transform.translation.truncate();
        if player_positions.iter().any(|p| pos.distance(*p) <= SDF_CULLING_RADIUS) {
            // some walls have scale 1.0 but are actually 64 units big
            let wall_size = if transform.scale.x < 2.0 && transform.scale.y < 2.0 {
                Vec2::splat(64.0)
            } else {
                transform.scale.truncate()
            };

            occluders.push(Occluder {
                position: pos,
                size: wall_size,
            });
        }
    }

    let Some(image) = images.get_mut(&sdf_texture.texture) else {
        return;
    };

    let mut sdf_data = Vec::with_capacity((SDF_TEXTURE_SIZE * SDF_TEXTURE_SIZE) as usize);

    let texel_to_world = SDF_WORLD_SIZE / SDF_TEXTURE_SIZE as f32;
    let half_world = SDF_WORLD_SIZE * 0.5;

    // go through each pixel in the texture
    for y in 0..SDF_TEXTURE_SIZE {
        for x in 0..SDF_TEXTURE_SIZE {
            // convert pixel to world position
            let world_x = (x as f32 * texel_to_world) - half_world;
            let world_y = (y as f32 * texel_to_world) - half_world;
            let world_pos = Vec2::new(world_x, world_y);

            // find closest occluder
            let mut min_distance = SDF_WORLD_SIZE;
            for occluder in &occluders {
                let dist = occluder.distance(world_pos);
                min_distance = min_distance.min(dist);
            }

            sdf_data.push(min_distance);
        }
    }

    // convert floats to bytes and update texture
    image.data = Some(sdf_data.iter().flat_map(|f| f.to_le_bytes()).collect());
}

// copy shadow texture to all materials
fn update_material_sdf_textures(
    sdf_texture: Res<SdfTexture>,
    mut materials: ResMut<Assets<PlayerBaseMaterial>>,
) {
    for (_, material) in materials.iter_mut() {
        material.sdf_texture = Some(sdf_texture.texture.clone());
    }
}
