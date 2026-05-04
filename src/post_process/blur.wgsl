#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_lit::settings_types::Lighting2dSettings

@group(0) @binding(0) var<uniform> settings: Lighting2dSettings;
@group(0) @binding(1) var<uniform> direction: vec2<i32>;
@group(0) @binding(2) var texture: texture_2d<f32>;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    return gaussian_blur(in.position.xy, direction, settings.blur);
}

fn gaussian_blur(frag_pos: vec2<f32>, direction: vec2<i32>, radius: i32) -> vec4<f32> {
    let effective_radius = clamp(radius, 1, 3);
    let texel_pos = vec2<i32>(frag_pos);
    let tex_max = vec2<i32>(textureDimensions(texture)) - vec2<i32>(1);

    var color = vec4<f32>(0.0);
    var total_weight: f32 = 0.0;

    for (var i = -effective_radius; i <= effective_radius; i++) {
        let w = f32(effective_radius + 1 - abs(i));
        let sample_pos = clamp(texel_pos + direction * i, vec2<i32>(0), tex_max);
        color += textureLoad(texture, sample_pos, 0) * w;
        total_weight += w;
    }

    return color / total_weight;
}
