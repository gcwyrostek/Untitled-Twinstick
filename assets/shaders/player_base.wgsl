#import bevy_sprite::mesh2d_vertex_output::VertexOutput

const NUM_LIGHTS: i32 = 4;
@group(2) @binding(0) var<uniform> lights: array<Light, NUM_LIGHTS>; // <-- Put total number of lights there.
@group(2) @binding(1) var<uniform> material_color: vec4<f32>;
@group(2) @binding(2) var<uniform> lighting: Lighting;
@group(2) @binding(3) var base_color_texture: texture_2d<f32>;
@group(2) @binding(4) var base_color_sampler: sampler;
@group(2) @binding(5) var normal_texture: texture_2d<f32>;
@group(2) @binding(6) var normal_sampler: sampler;
@group(2) @binding(7) var<uniform> mesh_rotation: f32;

// Specifically for the phone lighting
// Information about how this object reflects light
struct Lighting {
    ambient_reflection_coefficient: f32,
    ambient_light_intensity: f32,
    diffuse_reflection_coefficient: f32,
    shininess: f32,
};

// Struct representing each individual light in the scene
struct Light {
    position: vec3<f32>,
    intensity: f32,
    range: f32,
    cone: i32,
    _padding: vec2<f32>, // unform information can only be sent to gpu in chunks that are multiples of 16 bytes (i think?? not totally sure)
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(base_color_texture, base_color_sampler, mesh.uv);

    let normal = textureSample(normal_texture, normal_sampler, mesh.uv).rgb;
    let normal_range = normalize(normal * 2.0 - 0.5);

    let cos_r = cos(mesh_rotation);
    let sin_r = sin(mesh_rotation);
    let rotated_normal = vec3<f32>(normal_range.x * cos_r - normal_range.y * sin_r, normal_range.x * sin_r + normal_range.y * cos_r, normal_range.z);

    let ambient = lighting.ambient_reflection_coefficient * lighting.ambient_light_intensity;

    var diffuse: f32 = 0.0;
        var specular: f32 = 0.0;
        let view_direction = vec3<f32>(0.0, 0.0, 1.0);
        for (var i: i32 = 0; i < NUM_LIGHTS; i += 1) {
            // is a coner light
            if (lights[i].cone != 0) {
                continue;
            }
            let pixel_to_light = normalize(lights[i].position - vec3<f32>(mesh.world_position.x, mesh.world_position.y, mesh.world_position.z));
            diffuse += lighting.diffuse_reflection_coefficient * lights[i].intensity * max(0.0, dot(rotated_normal, pixel_to_light));

            let reflection = reflect(-pixel_to_light, rotated_normal);
            specular += 1.0 * lights[i].intensity * pow(max(dot(view_direction, reflection), 0.0), lighting.shininess);
        }

    let final_color = vec4<f32>(base_color.rgb * (ambient + diffuse + specular), base_color.a);
    return final_color;
}
