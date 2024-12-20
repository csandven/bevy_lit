#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_lit::{
    types::LightOccluder2d,
    view_transformations::{frag_coord_to_ndc, position_ndc_to_world},
}

#if AVAILABLE_STORAGE_BUFFER_BINDINGS >= 6
    @group(0) @binding(1) var<storage> occluders: array<LightOccluder2d>;
#else
    const MAX_OCCLUDERS: u32 = 82u;

    @group(0) @binding(1) var<uniform> occluders: array<LightOccluder2d, MAX_OCCLUDERS>;
#endif

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let pos = position_ndc_to_world(frag_coord_to_ndc(in.position)).xy;

#if AVAILABLE_STORAGE_BUFFER_BINDINGS >= 6
    let occluder_count = arrayLength(&occluders);
#else
    let occluder_count = MAX_OCCLUDERS;
#endif

    var sdf = occluder_sd(pos, occluders[0]);
    for (var i = 1u; i < occluder_count; i++) {
        let occluder = occluders[i];

        // ignore occluders with half_size == (0.0, 0.0)
        if occluder.half_size.x == 0.0 && occluder.half_size.y == 0.0 {
            continue;
        }

        sdf = min(sdf, occluder_sd(pos, occluder));
    }

    return vec4(sdf, 0.0, 0.0, 1.0);
}

fn occluder_sd(p: vec2f, occluder: LightOccluder2d) -> f32 {
    let local_pos = occluder.center - p;
    let d = abs(local_pos) - occluder.half_size;

    return length(max(d, vec2f(0.))) + min(max(d.x, d.y), 0.);
}
