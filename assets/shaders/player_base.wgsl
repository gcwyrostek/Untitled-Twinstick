#import bevy_sprite::mesh2d_vertex_output::VertexOutput

const NUM_LIGHTS: i32 = 4;
@group(2) @binding(0) var<uniform> lights: array<Light, NUM_LIGHTS>; // <-- Put total number of lights there.
@group(2) @binding(1) var<uniform> material_color: vec4<f32>;
@group(2) @binding(2) var<uniform> lighting: Lighting;
@group(2) @binding(3) var base_color_texture: texture_2d<f32>;
@group(2) @binding(4) var base_color_sampler: sampler;
@group(2) @binding(5) var normal_texture: texture_2d<f32>;
@group(2) @binding(6) var normal_sampler: sampler;

// Specifically for the phone lighting
// Information about how this object reflects light
struct Lighting {
    ambient_reflection_coefficient: f32,
    ambient_light_intensity: f32,
    diffuse_reflection_coefficient: f32,
    _padding: f32,
};

// Struct representing each individual light in the scene
struct Light {
    position: vec3<f32>,
    intensity: f32,
    range: f32,
    _padding: vec3<f32>, // unform information can only be sent to gpu in chunks that are multiples of 16 bytes (i think?? not totally sure)
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(base_color_texture, base_color_sampler, mesh.uv);



    return base_color;
}
