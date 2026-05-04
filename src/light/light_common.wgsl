#define_import_path bevy_lit::light2d_common

#import bevy_render::view::{
    View,
    frag_coord_to_ndc,
    position_ndc_to_world,
    position_world_to_ndc,
    ndc_to_uv,
    uv_to_ndc,
}
#import bevy_lit::{
    settings_types::Lighting2dSettings,
    view_transformations::{world_to_uv, frag_to_world},
}

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<uniform> settings: Lighting2dSettings;
@group(0) @binding(2) var voronoi_texture: texture_2d<f32>;
@group(0) @binding(3) var voronoi_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(2) @interpolate(flat) translation_rotation: vec4<f32>,
}

fn attenuation(inner: f32, outer: f32, falloff: f32, diff: f32) -> f32 {
    if diff <= inner {
        return 1.0;
    }

    if diff >= outer {
        return 0.0;
    }

    let s = (diff - inner) / (outer - inner);
    let s2 = s * s;
    let inv = 1.0 - s2;

    return (inv * inv) / (1.0 + falloff * s2);
}

fn get_sdf(pos: vec2<f32>) -> f32 {
    let uv = world_to_uv(vec3(pos, 0.0), view);
    let samp = textureSampleLevel(voronoi_texture, voronoi_sampler, uv, 0.0);

    if samp.x < 0.0 || samp.y < 0.0 {
        return 9999999999.0;
    }

    let seed = frag_to_world(vec4<f32>(samp.xy, 0.0, 0.0) / settings.scale, view).xy;
    let dist = length(pos - seed);

    if dist <= 2.0 / settings.scale {
        return 0.0;
    }

    // Determine if the pixel is inside or outside the shape
    return select(dist, -dist, samp.w == 1.0);
}

// Implementation follows the demo in this article
// https://www.rykap.com/2020/09/23/distance-fields
fn raymarch(ray_origin: vec2<f32>, ray_target: vec2<f32>) -> f32 {
    let config = settings.raymarch;
    let max_steps = config.max_steps;
    let sharpness = config.sharpness;
    let jitter = config.jitter;

    let ray_direction = normalize(ray_target - ray_origin);
    let stop_at = distance(ray_origin, ray_target);

    var ray_progress = 0.0;
    var light_contrib = 1.0;

    for (var i = 0u; i < max_steps; i++) {
        // ray found target
        if ray_progress > stop_at {
            return light_contrib;
        }

        let sdf = get_sdf(ray_origin + ray_progress * ray_direction);

        // ray found occluder
        if sdf <= 0.0 {
            break;
        }

        light_contrib = min(light_contrib, sdf / ray_progress * sharpness);

        // Early exit: contribution is negligibly small, treat as fully occluded
        if light_contrib < 0.001 {
            return 0.0;
        }

        ray_progress += sdf * (1.0 - jitter) + jitter * fract(sdf * 43758.5453);
    }

    return 0.0;
}
