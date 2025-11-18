use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use crate::components::StaticCollider;
use crate::enemy::Enemy;
use crate::player_material::PlayerBaseMaterial;

/// Size of the SDF texture (512x512 balancing quality and performance)
const SDF_TEXTURE_SIZE: u32 = 512;

/// World space area covered by the SDF (centered on origin)
/// This defines a 5120x5120 world unit area
const SDF_WORLD_SIZE: f32 = 5120.0;

pub struct SdfShadowsPlugin;

impl Plugin for SdfShadowsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SdfTexture>()
            .add_systems(Update, generate_sdf)
            .add_systems(Update, update_material_sdf_textures);
    }
}

/// Resource that holds the SDF texture
#[derive(Resource)]
pub struct SdfTexture {
    pub texture: Handle<Image>,
}

impl FromWorld for SdfTexture {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();

        // Create an empty R32Float texture
        let size = Extent3d {
            width: SDF_TEXTURE_SIZE,
            height: SDF_TEXTURE_SIZE,
            depth_or_array_layers: 1,
        };

        let mut image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0u8; 4], // Will be filled with f32 data
            TextureFormat::R32Float,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );

        // Initialize with large distances
        let data: Vec<f32> = vec![SDF_WORLD_SIZE; (SDF_TEXTURE_SIZE * SDF_TEXTURE_SIZE) as usize];
        image.data = Some(data.iter().flat_map(|f| f.to_le_bytes()).collect());

        // TODO: Add linear filtering to reduce flickering - needs correct ImageSampler type for this Bevy version

        let texture = images.add(image);

        SdfTexture { texture }
    }
}

/// Represents an occluder in the scene (enemy or wall)
struct Occluder {
    position: Vec2,
    size: Vec2,
}

impl Occluder {
    /// Calculate signed distance from a point to this rectangular occluder
    /// Returns negative if inside, positive if outside
    fn distance(&self, point: Vec2) -> f32 {
        // Convert to local space (centered on occluder)
        let local = point - self.position;

        // Half extents
        let half_size = self.size * 0.5;

        // Distance to edge of rectangle
        let d = local.abs() - half_size;

        // Outside distance + inside distance
        let outside = d.max(Vec2::ZERO).length();
        let inside = d.x.max(d.y).min(0.0);

        outside + inside
    }
}

/// System that generates the SDF texture each frame
fn generate_sdf(
    mut images: ResMut<Assets<Image>>,
    sdf_texture: Res<SdfTexture>,
    enemies: Query<&Transform, With<Enemy>>,
    walls: Query<&Transform, With<StaticCollider>>,
) {
    // Collect all occluders
    let mut occluders = Vec::new();

    // Add enemies as occluders
    for transform in enemies.iter() {
        occluders.push(Occluder {
            position: transform.translation.truncate(),
            // Scale already contains the final size (Rectangle mesh is 1x1, scale makes it 64x64)
            size: transform.scale.truncate(),
        });
    }

    // Add walls as occluders
    for transform in walls.iter() {
        // Walls are Sprites with default scale of 1.0, but texture is 64x64
        // If scale is ~1.0, treat as 64x64 sprite, otherwise use scale
        let wall_size = if transform.scale.x < 2.0 && transform.scale.y < 2.0 {
            Vec2::splat(64.0) // Default wall sprite size
        } else {
            transform.scale.truncate() // Use actual scale if it's been modified
        };

        occluders.push(Occluder {
            position: transform.translation.truncate(),
            size: wall_size,
        });
    }

    // Get the image
    let Some(image) = images.get_mut(&sdf_texture.texture) else {
        return;
    };

    // Generate SDF data
    let mut sdf_data = Vec::with_capacity((SDF_TEXTURE_SIZE * SDF_TEXTURE_SIZE) as usize);

    let texel_to_world = SDF_WORLD_SIZE / SDF_TEXTURE_SIZE as f32;
    let half_world = SDF_WORLD_SIZE * 0.5;

    for y in 0..SDF_TEXTURE_SIZE {
        for x in 0..SDF_TEXTURE_SIZE {
            // Convert texture coordinate to world position
            let world_x = (x as f32 * texel_to_world) - half_world;
            let world_y = (y as f32 * texel_to_world) - half_world;
            let world_pos = Vec2::new(world_x, world_y);

            // Find minimum distance to any occluder
            let mut min_distance = SDF_WORLD_SIZE;
            for occluder in &occluders {
                let dist = occluder.distance(world_pos);
                min_distance = min_distance.min(dist);
            }

            sdf_data.push(min_distance);
        }
    }

    // Convert to bytes
    image.data = Some(sdf_data.iter().flat_map(|f| f.to_le_bytes()).collect());
}

/// System that updates all PlayerBaseMaterial instances with the SDF texture
fn update_material_sdf_textures(
    sdf_texture: Res<SdfTexture>,
    mut materials: ResMut<Assets<PlayerBaseMaterial>>,
) {
    // Update all materials with the SDF texture
    for (_, material) in materials.iter_mut() {
        material.sdf_texture = Some(sdf_texture.texture.clone());
    }
}
