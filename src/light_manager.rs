use crate::{
    components::LightSource, player_material::PlayerBaseMaterial,
};
use bevy::{prelude::*, render::render_resource::ShaderType};

// Set number of total lights here
const NUM_LIGHTS: i32 = 4;
#[derive(Resource, Default)]
pub struct Lights {
    pub num_lights: i32,
    pub lights: [Light; NUM_LIGHTS as usize],
}

pub struct LightSourcePlugin;

impl Plugin for LightSourcePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Lights>()
            .add_systems(Startup, setup_lights)
            .add_systems(Update, collect_lights_into_resource)
            .add_systems(Update, update_material_rotation);
    }
}

#[derive(Default, Copy, Clone, ShaderType, Debug)]
pub struct Light {
    pub position: Vec3,
    pub intensity: f32,
    pub range: f32,
    pub cone: i32,
    pub _padding: Vec2,
}

pub fn setup_lights(mut commands: Commands) {
    commands.spawn({
        let transform = Transform::from_xyz(0., 0., 0.);
        (
            transform,
            LightSource::new(transform.translation, 1.0, 15.0, 0),
        )
    });
    commands.spawn({
        let transform = Transform::from_xyz(0., 0., 0.);
        (
            transform,
            LightSource::new(transform.translation, 0.0, 10.0, 0),
        )
    });
    
    commands.spawn({
        let transform = Transform::from_xyz(0., 0., 0.);
        (
            transform,
            LightSource::new(transform.translation, 0.0, 15.0, 0),
        )
    });

    commands.spawn({
        let transform = Transform::from_xyz(0., 0., 0.);
        (
            transform,
            LightSource::new(transform.translation, 0.0, 15.0, 0),
        )
    });
}

// System to collect LightSource components into the Lights resource
pub fn collect_lights_into_resource(
    mut lights_resource: ResMut<Lights>,
    lights_query: Query<(&Transform, &LightSource)>,
) {
    // Initialize with empty lights
    let mut light_array = [Light {
        position: Vec3::ZERO,
        intensity: 0.0,
        range: 0.0,
        cone: 0,
        _padding: Vec2::ZERO,
    }; NUM_LIGHTS as usize];
    
    // Fill the array with actual lights
    for (i, (transform, light_source)) in lights_query.iter().take(NUM_LIGHTS as usize).enumerate() {
        light_array[i] = Light {
            position: transform.translation,
            intensity: light_source.intensity,
            range: light_source.range,
            cone: light_source.cone,
            _padding: Vec2::ZERO,
        };
    }
    
    lights_resource.lights = light_array;
    lights_resource.num_lights = NUM_LIGHTS;
}

fn update_material_rotation(
    mut materials: ResMut<Assets<PlayerBaseMaterial>>,
    query: Query<(&Transform, &MeshMaterial2d<PlayerBaseMaterial>), Changed<Transform>>,
) {
    for (transform, mesh_material) in query.iter() {
        if let Some(material) = materials.get_mut(&mesh_material.0) {
            let (_, _, z_rotation) = transform.rotation.to_euler(EulerRot::XYZ);
            material.mesh_rotation = z_rotation;
        }
    }
}