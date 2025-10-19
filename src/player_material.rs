use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{AlphaMode2d, Material2d, Material2dPlugin},
};
use crate::{light_manager::Light};
use crate::{light_manager::Lights};

const PLAYER_MATERIAL_SHADER: &str = "shaders/player_base.wgsl";

#[derive(Clone, ShaderType, Debug)]
pub struct Lighting {
    pub ambient_reflection_coefficient: f32,
    pub ambient_light_intensity: f32,
    pub diffuse_reflection_coefficient: f32,
    pub _padding: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PlayerBaseMaterial {
    #[uniform(0)]
    pub lights: [Light; 4],

    #[uniform(1)]
    pub color: LinearRgba,

    #[uniform(2)]
    pub lighting: Lighting,

    #[texture(3)]
    #[sampler(4)]
    pub texture: Option<Handle<Image>>,

    #[texture(5)]
    #[sampler(6)]
    pub normal: Option<Handle<Image>>,
}

impl Material2d for PlayerBaseMaterial {
    fn fragment_shader() -> ShaderRef {
        PLAYER_MATERIAL_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
