#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;
@group(2) @binding(2) var normal_map: texture_2d<f32>;
@group(2) @binding(3) var normal_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal_sample = textureSample(normal_map, normal_sampler, in.uv);
    let albedo = textureSample(albedo_tex, albedo_sampler, in.uv);
    let albedo_gray = (albedo.r + albedo.g + albedo.b) / 3.0;

    return vec4<f32>(normal_sample.rgb, albedo_gray);
}
