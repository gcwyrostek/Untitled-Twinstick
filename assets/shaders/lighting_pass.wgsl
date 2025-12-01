#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;
@group(2) @binding(2) var normals_buffer: texture_2d<f32>;
@group(2) @binding(3) var normals_sampler: sampler;

struct LightParams {
    view_size: vec2<f32>,
    light_pos: vec2<f32>,
}

@group(2) @binding(4) var<uniform> params: LightParams;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let screen_uv = vec2<f32>(
        (in.world_position.x + params.view_size.x * 0.5) / params.view_size.x,
        (params.view_size.y * 0.5 - in.world_position.y) / params.view_size.y
    );

    let normal_sample = textureSample(normals_buffer, normals_sampler, screen_uv);
    let has_geometry = normal_sample.a > 0.1;

    if (!has_geometry) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    let normal = normalize(normal_sample.rgb * 2.0 - 1.0);
    let albedo = textureSample(albedo_tex, albedo_sampler, screen_uv);

    let light_z = 100.0;
    let to_light = vec3<f32>(params.light_pos.x - in.world_position.x, params.light_pos.y - in.world_position.y, light_z);
    let dist = length(to_light);
    let light_dir = normalize(to_light);

    let diffuse = max(dot(normal, light_dir), 0.0);
    let atten = 1.0 / (1.0 + dist * dist * 0.0001);

    let ambient = 0.3;
    let light = ambient + diffuse * atten * 5.0;

    return vec4<f32>(albedo.rgb * light, 1.0);
}
