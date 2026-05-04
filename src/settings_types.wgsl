#define_import_path bevy_lit::settings_types

struct RaymarchSettings {
    max_steps: u32,
    jitter: f32,
    sharpness: f32,
    pad: u32
}

struct PenetrationSettings {
    max: f32,
    intensity: f32,
    falloff: f32,
    directions: u32,
    steps: u32,
}

struct Lighting2dSettings {
    raymarch: RaymarchSettings,
    penetration: PenetrationSettings,
    ambient_light: vec4<f32>,
    scale: f32,
    tint_occluders: u32,
    edge_intensity: f32,
    blur: i32,
    shadows_enabled: u32,
    @size(12)
    max_screen_occluders: u32,
}
