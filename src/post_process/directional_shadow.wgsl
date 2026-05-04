
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import bevy_lit::{
    settings_types::Lighting2dSettings,
    view_transformations::{frag_to_world, world_to_uv},
}

struct DirectionalLight2d {
    color: vec4<f32>,
    direction: vec2<f32>,
    tile_size: f32,
    shadow_length: f32,
    strength: f32,
}

const MAX_OCCLUDER_HEIGHT_TILES: f32 = 16.0;

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<uniform> settings: Lighting2dSettings;
@group(0) @binding(2) var<uniform> sun: DirectionalLight2d;
@group(0) @binding(3) var lighting_texture: texture_2d<f32>;
@group(0) @binding(4) var occluder_texture: texture_2d<f32>;
@group(0) @binding(5) var roof_texture: texture_2d<f32>;
@group(0) @binding(6) var sampler_obj: sampler;

struct OccluderInfo {
    covered: f32,
    height: f32,
};

fn get_occluder_info(pos: vec2<f32>) -> OccluderInfo {
    let uv = world_to_uv(vec3(pos, 0.0), view);
    if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
        return OccluderInfo(0.0, 0.0);
    }

    let samp = textureSampleLevel(occluder_texture, sampler_obj, uv, 0.0);
    return OccluderInfo(samp.a, samp.b);
}

fn directional_shadow_factor(pos: vec2<f32>) -> f32 {
    let dir = sun.direction; // toward the sun
    let max_length = sun.shadow_length * MAX_OCCLUDER_HEIGHT_TILES;
    let step_size = max(3.0 / settings.scale, sun.tile_size * 0.125);
    let tip_softness = max(sun.tile_size * 0.35, step_size * 2.0);

    var dist = step_size;

    for (var i = 0u; i < 96u; i++) {
        if dist >= max_length {
            break;
        }

        let sample_pos = pos + dir * dist;
        let info = get_occluder_info(sample_pos);

        if info.covered > 0.5 {
            let occluder_height_tiles = max(info.height, 0.0) / sun.tile_size;
            let max_shadow_length = occluder_height_tiles * sun.shadow_length;

            if dist <= max_shadow_length {
                let remaining = max_shadow_length - dist;
                let tip_fade = smoothstep(0.0, tip_softness, remaining);
                return 1.0 - tip_fade;
            }
        }

        dist += step_size;
    }

    return 1.0;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let lighting = textureSample(lighting_texture, sampler_obj, in.uv);

    if !bool(settings.shadows_enabled) {
        return vec4(lighting.rgb, 1.0);
    }

    // No sun contribution → pass lighting through unchanged.
    if sun.strength <= 0.0 {
        return vec4(lighting.rgb, 1.0);
    }

    // Under a roof → directional shadow is suppressed.
    let roof = textureSample(roof_texture, sampler_obj, in.uv).a;
    if roof > 0.5 {
        return vec4(lighting.rgb, 1.0);
    }

    let pos = frag_to_world(in.position / settings.scale, view).xy;
    let occluder = get_occluder_info(pos);

    if occluder.covered > 0.5 {
        return vec4(lighting.rgb, 1.0);
    }

    let shadow = directional_shadow_factor(pos);

    let shadow_mult = max(0.0, 1.0 - sun.strength * (1.0 - shadow) * 0.9);
    return vec4(lighting.rgb, shadow_mult);
}
