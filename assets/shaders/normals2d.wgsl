#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var normal_map: texture_2d<f32>;
@group(2) @binding(1) var normal_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let normal = textureSample(normal_map, normal_sampler, mesh.uv).rgb;

    let normal_range = normalize(normal * 2.0 - 1.0);

    let packed_normal = normal_range * 0.5 + 0.5;

    return vec4<f32>(packed_normal, 1.0);
}
