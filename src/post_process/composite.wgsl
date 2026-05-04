#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_lit::settings_types::Lighting2dSettings

@group(0) @binding(0) var<uniform> settings: Lighting2dSettings;
@group(0) @binding(1) var view_texture: texture_2d<f32>;
@group(0) @binding(2) var lighting_texture: texture_2d<f32>;
@group(0) @binding(3) var texture_sampler: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let light_frag = textureSample(lighting_texture, texture_sampler, in.uv);
    let scene_frag = textureSample(view_texture, texture_sampler, in.uv);
    let ambient = settings.ambient_light * light_frag.a;

    return scene_frag * (vec4(light_frag.rgb, 1.0) + ambient);
}

