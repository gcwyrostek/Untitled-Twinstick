#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct LightingParams {
    view_size: vec2<f32>,
    _pad0: vec2<f32>,
    light_pos_radius: vec4<f32>,  // xyz = position, w = radius
    light_color_int: vec4<f32>,   // rgb = color, a = intensity
    ambient: vec4<f32>,           // rgb = ambient color
    cone_angle_dir: vec4<f32>,    // x = cone angle (degrees), y = direction (radians), zw = padding
}

@group(2) @binding(0) var<uniform> params: LightingParams;
@group(2) @binding(1) var normals_tex: texture_2d<f32>;
@group(2) @binding(2) var normals_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let screen_uv = vec2<f32>(
        (mesh.world_position.x + params.view_size.x * 0.5) / params.view_size.x,
        (params.view_size.y * 0.5 - mesh.world_position.y) / params.view_size.y
    );

    // Sample the normal from the normals render target
    let packed_normal = textureSample(normals_tex, normals_sampler, screen_uv).rgb;

    let normal = normalize(packed_normal * 2.0 - 1.0);

    let pixel_to_light = params.light_pos_radius.xyz - mesh.world_position;
    let distance = length(pixel_to_light);
    let light_dir = normalize(pixel_to_light);

    var cone_attenuation = 1.0;
    if (params.cone_angle_dir.x > 0.0) {
        let light_forward = vec3<f32>(cos(params.cone_angle_dir.y), sin(params.cone_angle_dir.y), 0.0);

        let angle_to_pixel = acos(dot(-light_dir, light_forward));
        let cone_angle_half = params.cone_angle_dir.x / 2.0 * 3.14159265359 / 180.0;

        if (angle_to_pixel > cone_angle_half) {
            cone_attenuation = 0.0;
        } else {
            cone_attenuation = smoothstep(cone_angle_half, cone_angle_half * 0.8, angle_to_pixel);
        }
    }

    let distance_attenuation = 1.0 - smoothstep(0.0, params.light_pos_radius.w, distance);

    let diffuse = max(dot(normal, light_dir), 0.0);

    let total_attenuation = distance_attenuation * cone_attenuation;

    let light_contribution = params.light_color_int.rgb * params.light_color_int.a * diffuse * total_attenuation;

    let light_intensity = (light_contribution.r + light_contribution.g + light_contribution.b) / 3.0;
    return vec4<f32>(light_contribution * 5.0, light_intensity * 2.0); // Boost intensity for visibility
}
