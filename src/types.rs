#![expect(deprecated)]

use bevy::{
    math::Vec2,
    prelude::*,
    reflect::Reflect,
    render::{render_resource::ShaderType, sync_world::SyncToRenderWorld, view::Visibility},
    transform::components::Transform,
};

/// Represents ambient light in a 2D environment. This component belongs to a [`Camera2d`] entity.
#[derive(Component, Clone, Reflect)]
#[require(SyncToRenderWorld)]
pub struct AmbientLight2d {
    /// The color of the ambient light.
    pub color: Color,
    /// The brightness of the ambient light.
    pub brightness: f32,
}

impl Default for AmbientLight2d {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            brightness: 1.0,
        }
    }
}

/// Raymarch settings
#[derive(Reflect, Clone, ShaderType)]
pub struct RaymarchSettings {
    /// The maximum steps the raymarch loop can take to return a result
    pub max_steps: u32,
    /// Random number from 0.0 to 1.0. Maximizes the number of raymarching steps, improving approximation
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

/// Settings for 2D lighting. This component belongs to a [`Camera2d`] entity and is mandatory for
/// lighting effects
#[derive(Component, Clone, Reflect)]
#[require(SyncToRenderWorld)]
pub struct Lighting2dSettings {
    /// The blur coc (circle of confusion) dimension contributing to the softness of the shadows
    pub blur: f32,
    /// If true (default), the blur is constant, else it's calculated in relation to the viewport size
    pub fixed_resolution: bool,
    /// Raymarch settings
    pub raymarch: RaymarchSettings,
}

impl Default for Lighting2dSettings {
    fn default() -> Self {
        Self {
            blur: 0.0,
            fixed_resolution: true,
            raymarch: Default::default(),
        }
    }
}

/// Represents a point light in a 2D environment.
#[derive(Component, Clone, Reflect)]
#[require(SyncToRenderWorld, Transform, Visibility)]
pub struct PointLight2d {
    /// The color of the point light.
    pub color: Color,
    /// The intensity of the point light.
    pub intensity: f32,
    /// The radius of the point light's influence.
    pub radius: f32,
    /// The falloff rate of the point light.
    pub falloff: f32,
}

impl Default for PointLight2d {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
            radius: 64.0,
            falloff: 1.0,
        }
    }
}

/// A bundle of components representing a point light in a 2D environment.
#[deprecated(since = "0.4.0", note = "Use the `PointLight2d` component instead.")]
#[derive(Bundle, Default)]
pub struct PointLight2dBundle {
    /// The point light component.
    pub point_light: PointLight2d,
    /// The transform component.
    pub transform: Transform,
    /// The visibility component.
    pub visibility: Visibility,
}

/// Represents an occluder that blocks light in a 2D environment.
#[derive(Component, Default, Clone, Reflect)]
#[require(SyncToRenderWorld, Transform, Visibility)]
pub struct LightOccluder2d {
    /// Half the size of the occluder AABB rectangle.
    pub half_size: Vec2,
}

impl LightOccluder2d {
    pub fn new(half_size: Vec2) -> Self {
        Self { half_size }
    }
}

/// A bundle of components representing a light occluder in a 2D environment.
#[deprecated(since = "0.4.0", note = "Use the `LightOccluder2d` component instead.")]
#[derive(Bundle, Default)]
pub struct LightOccluder2dBundle {
    /// The light occluder component.
    pub light_occluder: LightOccluder2d,
    /// The transform component.
    pub transform: Transform,
    /// The visibility component.
    pub visibility: Visibility,
}
