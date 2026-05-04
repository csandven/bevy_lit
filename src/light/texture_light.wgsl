#import bevy_lit::{
    view_transformations::frag_to_world,
    light2d_common::{
        VertexOutput,
        settings,
        view,
        get_sdf,
        attenuation,
        raymarch
    },
}

struct TextureLight2d {
    color: vec4<f32>,
    cast_shadows: u32,
}

@group(1) @binding(0) var<uniform> light: TextureLight2d;
@group(1) @binding(1) var light_texture: texture_2d<f32>;
@group(1) @binding(2) var light_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_sample = textureSample(light_texture, light_sampler, in.uv);

    if light_sample.a <= 0.0 {
        discard;
    }

    var light_contrib = light.color.rgb * light_sample.rgb * light_sample.a;

    let pos = frag_to_world(in.clip_position / settings.scale, view).xy;
    let light_center = in.translation_rotation.xy;
    let sdf = get_sdf(pos);

    // inside occluder
    if sdf <= 0.0 {
        light_contrib *= select(0.0, 1.0, bool(settings.tint_occluders));
    } else {
        if bool(settings.shadows_enabled) && bool(light.cast_shadows) {
            light_contrib *= raymarch(pos, light_center);
        }
    }

    if settings.edge_intensity > 0.0 && sdf > 0.0 {
        let edge_intensity = 1.0 / sdf * settings.edge_intensity;
        light_contrib += light_contrib * edge_intensity * 1.0;
    }

    return vec4<f32>(light_contrib, 1.0);
}
