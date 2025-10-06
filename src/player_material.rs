use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{AlphaMode2d, Material2d, Material2dPlugin},
};

const PLAYER_MATERIAL_SHADER: &str = "shaders/player_base.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PlayerBaseMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
}

impl Material2d for PlayerBaseMaterial {
    fn fragment_shader() -> ShaderRef {
        PLAYER_MATERIAL_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
