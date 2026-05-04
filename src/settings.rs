use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponent, render_resource::ShaderType,
        sync_world::SyncToRenderWorld,
    },
};

/// Represents ambient light in a 2D environment. This component belongs to a [`Camera2d`] entity.
#[derive(Component, Clone, Reflect)]
#[require(SyncToRenderWorld)]
pub struct AmbientLight2d {
    /// The color of the ambient light.
    pub color: Color,
    /// The intensity of the ambient light.
    pub intensity: f32,
}

impl Default for AmbientLight2d {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
        }
    }
}

/// Raymarch settings
#[derive(Reflect, Clone, ShaderType)]
pub struct RaymarchSettings {
    /// The maximum steps the raymarch loop can take to return a result
    pub max_steps: u32,
    /// Random number from 0.0 to 1.0. Minimizes the number of raymarching steps while reducing
    /// noise
    pub jitter_contrib: f32,
    /// How sharp should the shadow projections be
    pub sharpness: f32,
}

impl Default for RaymarchSettings {
    fn default() -> Self {
        Self {
            max_steps: 32,
            jitter_contrib: 0.5,
            sharpness: 5.0,
        }
    }
}

/// Penetration settings
#[derive(Clone, ShaderType, Reflect)]
pub struct PenetrationSettings {
    /// This defines the effective "thickness" of the light bleed.
    pub max: f32,
    /// Intensity multiplier for the final penetration color.
    pub intensity: f32,
    /// Controls how quickly light fades as it penetrates.
    pub falloff: f32,
    /// Number of radial directions to sample around the occluder.
    pub sample_directions: u32,
    /// Number of samples along each direction inside the occluder.
    pub sample_steps: u32,
}

impl Default for PenetrationSettings {
    fn default() -> Self {
        Self {
            max: 0.0,
            intensity: 0.0,
            falloff: 0.0,
            sample_directions: 8,
            sample_steps: 8,
        }
    }
}

/// Settings for 2D lighting. This component belongs to a [`Camera2d`] entity and is mandatory for
/// lighting effects
#[derive(Component, Clone, Reflect, ExtractComponent)]
#[require(SyncToRenderWorld, AmbientLight2d)]
pub struct Lighting2dSettings {
    /// Raymarch settings
    pub raymarch: RaymarchSettings,
    /// Controls how much light can penetrate into occluders and how it falls off
    pub penetration: PenetrationSettings,
    /// Whether light occlusion areas should be tinted by light sources
    pub tint_occluders: bool,
    /// Enables down sampling for the light map textures. Defaults to 0.5
    pub scale: f32,
    /// Controls the intensity of light in the egdes of occlusion areas
    pub edge_intensity: f32,
    /// The blur radius to be applied to the light map. Defaults to 0
    pub blur: u32,
    /// Maximum number of visible occluders to include in the shared shadow/SDF
    /// field for this camera.
    ///
    /// `0` means unlimited. 
    pub max_screen_occluders: u32,
}

impl Default for Lighting2dSettings {
    fn default() -> Self {
        Self {
            raymarch: Default::default(),
            penetration: Default::default(),
            tint_occluders: Default::default(),
            scale: 0.5,
            edge_intensity: 0.0,
            blur: 0,
            max_screen_occluders: 0,
        }
    }
}
