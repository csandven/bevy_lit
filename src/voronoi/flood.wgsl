#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var seed_texture: texture_2d<f32>;
@group(0) @binding(1) var sampler_obj: sampler;
@group(0) @binding(2) var<uniform> step: vec2<u32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let original_seed = textureSample(seed_texture, sampler_obj, in.uv);

    let dims = vec2<f32>(textureDimensions(seed_texture));

    var current_seed = original_seed.xy;
    var current_height = original_seed.z;
    var current_dist = select(
        9999999999.0,
        length(in.position.xy - current_seed),
        current_seed.x >= 0.0 && current_seed.y >= 0.0,
    );

    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let neighbour_coords = in.position.xy + vec2<f32>(vec2<i32>(x, y) * vec2<i32>(step));
            let neighbour_sample = textureSampleLevel(seed_texture, sampler_obj, neighbour_coords / dims, 0.0);
            let neighbour_seed = neighbour_sample.xy;
            let neighbour_dist = length(in.position.xy - neighbour_seed);

            if neighbour_seed.x >= 0. && neighbour_seed.y >= 0. && neighbour_dist < current_dist {
                current_seed = neighbour_seed;
                current_height = neighbour_sample.z;
                current_dist = neighbour_dist;
            }
        }
    }

    return vec4<f32>(current_seed, current_height, original_seed.w);
}
