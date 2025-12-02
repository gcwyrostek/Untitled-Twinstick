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
@group(2) @binding(8) var sdf_texture: texture_2d<f32>;
@group(2) @binding(9) var sdf_sampler: sampler;

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
    angle: f32,
    _padding: f32, // unform information can only be sent to gpu in chunks that are multiples of 16 bytes
}

// shadow map settings
const SDF_WORLD_SIZE: f32 = 5120.0;
const SDF_TEXTURE_SIZE: f32 = 512.0;
const MAX_RAYMARCH_STEPS: i32 = 64;
const SHADOW_THRESHOLD: f32 = 5.0;

// convert world coordinates to texture UV coordinates
fn world_to_sdf_uv(world_pos: vec2<f32>) -> vec2<f32> {
    let half_world = SDF_WORLD_SIZE * 0.5;
    let normalized = (world_pos + vec2<f32>(half_world, half_world)) / SDF_WORLD_SIZE;
    return normalized;
}

// check if light can reach this pixel or if something blocks it
// returns 0.0 if fully shadowed, 1.0 if lit
fn check_shadow(pixel_world_pos: vec2<f32>, light_pos: vec3<f32>) -> f32 {
    let light_pos_2d = light_pos.xy;
    let ray_dir = normalize(light_pos_2d - pixel_world_pos);
    let total_distance = distance(pixel_world_pos, light_pos_2d);

    var traveled: f32 = 2.0;
    var current_pos = pixel_world_pos + ray_dir * 2.0;

    // skip past our own object to avoid shadowing ourselves
    let start_uv = world_to_sdf_uv(current_pos);
    if (start_uv.x >= 0.0 && start_uv.x <= 1.0 && start_uv.y >= 0.0 && start_uv.y <= 1.0) {
        let start_sdf = textureSample(sdf_texture, sdf_sampler, start_uv).r;
        if (start_sdf < SHADOW_THRESHOLD) {
            // skip 45 units to get past even big enemies
            traveled += 45.0;
            current_pos += ray_dir * 45.0;
        }
    }

    // track how close we get to objects for softer shadows
    var min_shadow_factor: f32 = 1.0;
    let shadow_softness: f32 = 32.0;

    // walk towards the light checking for obstacles
    for (var i: i32 = 0; i < MAX_RAYMARCH_STEPS; i += 1) {
        if (traveled >= total_distance) {
            return min_shadow_factor;
        }

        let uv = world_to_sdf_uv(current_pos);

        if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
            return min_shadow_factor;
        }

        let sdf_dist = textureSample(sdf_texture, sdf_sampler, uv).r;

        // if we hit something, fully shadowed
        if (sdf_dist < SHADOW_THRESHOLD) {
            return 0.0;
        }

        // make shadows softer based on distance
        let shadow_factor = clamp(sdf_dist / shadow_softness, 0.0, 1.0);
        let darkened_shadow = pow(shadow_factor, 0.5);
        min_shadow_factor = min(min_shadow_factor, darkened_shadow);

        // move forward
        let step_size = max(sdf_dist * 0.5, 1.0);
        traveled += step_size;
        current_pos += ray_dir * step_size;
    }

    return min_shadow_factor;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(base_color_texture, base_color_sampler, mesh.uv);

    let normal = textureSample(normal_texture, normal_sampler, mesh.uv).rgb;
    let normal_range = normalize(normal * 2.0 - 0.5);

    let cos_r = cos(mesh_rotation);
    let sin_r = sin(mesh_rotation);
    let x = normal_range.x * cos_r - normal_range.y * sin_r;
    let y = normal_range.x * sin_r + normal_range.y * cos_r;
    let rotated_normal = vec3<f32>(x, y, normal_range.z);

    let ambient = lighting.ambient_reflection_coefficient * lighting.ambient_light_intensity;

    var diffuse: f32 = 0.0;
    var specular: f32 = 0.0;
    let view_direction = vec3<f32>(0.0, 0.0, 1.0);
    for (var i: i32 = 0; i < NUM_LIGHTS; i += 1) {
        let pixel_to_light_vec = lights[i].position - vec3<f32>(mesh.world_position.x, mesh.world_position.y, mesh.world_position.z);
        let distance = length(pixel_to_light_vec);
        let pixel_to_light = normalize(pixel_to_light_vec);
        let attenuation = clamp(1.0 - distance / lights[i].range, 0.0, 1.0);

        // is a coner light
        if (lights[i].cone != 0) {
            var cone_angle_half = f32(lights[i].cone) / 2.0 * 3.14159265359 / 180.0;
            var angle_radian = (lights[i].angle) * 3.14159265359 / 180.0;
            var light_vector = normalize(vec3<f32>(cos(angle_radian), sin(angle_radian), 0.0));
            var angle = acos(dot(-pixel_to_light, light_vector));

            if (angle > cone_angle_half) {
                continue;
            }
        }

        // check if this pixel is in shadow
        let shadow = check_shadow(vec2<f32>(mesh.world_position.x, mesh.world_position.y), lights[i].position);

        diffuse += shadow * lighting.diffuse_reflection_coefficient * lights[i].intensity * max(0.0, dot(rotated_normal, pixel_to_light)) * attenuation;

        let reflection = reflect(-pixel_to_light, rotated_normal);
        specular += shadow * 1.0 * lights[i].intensity * pow(max(dot(view_direction, reflection), 0.0), lighting.shininess) * attenuation;
    }

    let final_color = vec4<f32>(base_color.rgb * (ambient + diffuse + specular), base_color.a);
    return final_color;
}
