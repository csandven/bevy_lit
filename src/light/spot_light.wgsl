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

struct SpotLight2d {
    color: vec4<f32>,
    inner_radius: f32,
    outer_radius: f32,
    radial_falloff: f32,
    inner_angle: f32,
    outer_angle: f32,
    angular_falloff: f32,
    cast_shadows: u32,
}

@group(1) @binding(0) var<uniform> light: SpotLight2d;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let pos = frag_to_world(in.clip_position / settings.scale, view).xy;
    let light_center = in.translation_rotation.xy;
    let light_direction = vec2<f32>(-in.translation_rotation.w, in.translation_rotation.z);

    let light_dist = distance(pos, light_center);
    let radial_attenuation = attenuation(
        light.inner_radius,
        light.outer_radius,
        light.radial_falloff,
        light_dist
    );

    if radial_attenuation == 0.0 {
        discard;
    }

    let fragment_direction = normalize(light_center - pos);
    let dot_product = dot(light_direction, fragment_direction);
    let cos_inner = cos(light.inner_angle);
    let cos_outer = cos(light.outer_angle);
    let cone_span = max(cos_inner - cos_outer, 0.0001);
    let cone_t = clamp((dot_product - cos_outer) / cone_span, 0.0, 1.0);
    let cone_t_sq = cone_t * cone_t;
    let cone_falloff = 1.0 - cone_t;
    let angular_attenuation = cone_t_sq / (1.0 + light.angular_falloff * cone_falloff * cone_falloff);

    if angular_attenuation == 0.0 {
        discard;
    }

    let sdf = get_sdf(pos);

    var light_contrib = light.color.rgb * radial_attenuation * angular_attenuation;

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
