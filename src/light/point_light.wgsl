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

struct PointLight2d {
    color: vec4<f32>,
    inner_radius: f32,
    outer_radius: f32,
    falloff: f32,
    cast_shadows: u32,
}

@group(1) @binding(0) var<uniform> light: PointLight2d;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos = frag_to_world(in.clip_position / settings.scale, view).xy;
    let light_center = in.translation_rotation.xy;

    let light_dist = distance(pos, light_center);

    let radial_attenuation = attenuation(
        light.inner_radius,
        light.outer_radius,
        light.falloff,
        light_dist
    );

    if radial_attenuation == 0.0 {
        discard;
    }

    let sdf = get_sdf(pos);

    var light_contrib = light.color.rgb * radial_attenuation;

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
