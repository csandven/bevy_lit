#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::view::View
#import bevy_lit::{
    settings_types::Lighting2dSettings,
    view_transformations::{frag_to_world, world_to_uv},
}

@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<uniform> settings: Lighting2dSettings;
@group(0) @binding(2) var lighting_texture: texture_2d<f32>;
@group(0) @binding(3) var voronoi_texture: texture_2d<f32>;
@group(0) @binding(4) var sampler_obj: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let pos = frag_to_world(in.position / settings.scale, view).xy;
    let current = textureSample(lighting_texture, sampler_obj, in.uv);
    let sdf = get_sdf(pos);
    let p = settings.penetration;

    // Skip if outside occluder or penetration range
    if sdf > 0.0 || sdf < -p.max {
        return vec4(current.rgb, 1.0);
    }

    var penetration_color = vec3(0.0);
    var total_weight = 0.0;

    // Sampling configuration
    let directions = min(p.directions, 6u);
    let steps = min(p.steps, 6u);
    if directions == 0u || steps == 0u {
        return vec4(current.rgb, 1.0);
    }

    let two_pi = 6.2831853;
    let angle_step = two_pi / f32(directions);

    for (var dir_index = 0u; dir_index < directions; dir_index++) {
        let angle = f32(dir_index) * angle_step;
        let direction = vec2(cos(angle), sin(angle));

        for (var i = 0u; i < steps; i++) {
            let t = (f32(i) + 0.5) / f32(steps); // [0.0833 ... 0.9167] with 6 steps
            let distance = t * p.max;
            let offset = direction * distance;
            let sample_pos = pos + offset;
            let uv = world_to_uv(vec3(sample_pos, 0.0), view);
            let sample = textureSampleLevel(lighting_texture, sampler_obj, uv, 0.0);

            // Smooth falloff weight
            let weight = pow(1.0 - t, p.falloff);
            penetration_color += sample.rgb * weight;
            total_weight += weight;
        }
    }

    // Normalize and apply intensity
    if total_weight > 0.0 {
        penetration_color = (penetration_color / total_weight) * p.intensity;
    }

    return vec4(penetration_color, sdf);
}

fn get_sdf(pos: vec2<f32>) -> f32 {
    let uv = world_to_uv(vec3(pos, 0.0), view);
    let samp = textureSampleLevel(voronoi_texture, sampler_obj, uv, 0.0);

    // Original seed
    if samp.z == 1.0 {
        return 0.0;
    }

    let seed = frag_to_world(samp / settings.scale, view).xy;
    let dist = length(pos - seed);

    // Determine if the pixel is inside or outside the shape
    return select(dist, -dist, samp.w == 1.0);
}
