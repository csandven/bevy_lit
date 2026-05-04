#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var mask_texture: texture_2d<f32>;
@group(0) @binding(1) var sampler_obj: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let screen_size = vec2<f32>(textureDimensions(mask_texture));
    let sample = textureSample(mask_texture, sampler_obj, in.uv);
    let height = sample.z;
    let mask = sample.a;
    let null_seed = vec2(-1.0);

    if mask != 1.0 {
        return vec4(null_seed, height, mask);
    }

    let offsets = array<vec2<f32>, 4>(
        vec2<f32>(1.0, 0.0), vec2<f32>(-1.0, 0.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(0.0, -1.0)
    );

    for (var i = 0; i < 4; i++) {
        let neighbor_uv = (in.position.xy + offsets[i]) / screen_size;
        let neighbor_mask = textureSampleLevel(mask_texture, sampler_obj, neighbor_uv, 0.0).a;

        if neighbor_mask != mask {
            return vec4(in.position.xy, height, mask);
        }
    }

    return vec4(null_seed, height, mask);
}
