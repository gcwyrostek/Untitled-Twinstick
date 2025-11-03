use crate::{components::LightSource, player_material::PlayerBaseMaterial};
use bevy::{prelude::*, render::render_resource::ShaderType};

// Set number of total lights here, as well as in player_base.wgsl. Current limit of 4 light sources
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
    pub angle: f32,
    pub _padding: f32,
}

pub fn setup_lights(mut commands: Commands) {
    commands.spawn({
        let transform = Transform::from_xyz(0., 0., 0.);
        (
            transform,
            // Here's how this works:
            // If you want point light...'cone' = 0.
            // For cone lights, 'cone' = angle of the cone.
            // 'angle' is only for cone lights
            // range does nothing for now. all lights have infinite range.
            LightSource::new(transform.translation, 1.0, 15.0, 80, 45.0),
        )
    });
    commands.spawn({
        let transform = Transform::from_xyz(0., 0., 0.);
        (
            transform,
            LightSource::new(transform.translation, 0.0, 10.0, 0, 0.0),
        )
    });

    commands.spawn({
        let transform = Transform::from_xyz(0., 0., 0.);
        (
            transform,
            LightSource::new(transform.translation, 0.0, 15.0, 0, 0.0),
        )
    });

    commands.spawn({
        let transform = Transform::from_xyz(0., 0., 0.);
        (
            transform,
            LightSource::new(transform.translation, 0.0, 15.0, 0, 0.0),
        )
    });
}

// System to collect LightSource components into the Lights resource
//...this resource is used by PlayerBaseMaterial in player_material.rs!
pub fn collect_lights_into_resource(
    mut lights_resource: ResMut<Lights>,
    lights_query: Query<(&Transform, &LightSource)>,
) {
    let mut light_array = [Light {
        position: Vec3::ZERO,
        intensity: 0.0,
        range: 0.0,
        cone: 0,
        angle: 0.0,
        _padding: 0.0,
    }; NUM_LIGHTS as usize];

    for (i, (transform, light_source)) in lights_query.iter().take(NUM_LIGHTS as usize).enumerate()
    {
        light_array[i] = Light {
            position: transform.translation,
            intensity: light_source.intensity,
            range: light_source.range,
            cone: light_source.cone,
            angle: light_source.angle,
            _padding: 0.0,
        };
    }

    lights_resource.lights = light_array;
    lights_resource.num_lights = NUM_LIGHTS;
}

// For the shader, mesh_rotation must be updated to match the actual rotation of the object.
// Otherwise, shading will not change when rotating.
fn update_material_rotation(
    mut materials: ResMut<Assets<PlayerBaseMaterial>>,
    query: Query<(&Transform, &MeshMaterial2d<PlayerBaseMaterial>), With<Transform>>,
) {
    for (transform, mesh_material) in query.iter() {
        if let Some(material) = materials.get_mut(&mesh_material.0) {
            let (_, _, z_rotation) = transform.rotation.to_euler(EulerRot::XYZ);
            material.mesh_rotation = z_rotation;
        }
    }
}
