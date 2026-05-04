use bevy::prelude::*;

/// Represents a directional light (like a sun) in a 2D environment.
#[derive(Component, Clone, Debug, Reflect)]
pub struct DirectionalLight2d {
    /// The color of the directional light.
    pub color: Color,
    /// The intensity of the directional light.
    ///
    /// * `1.0` is default noon-ish light, shadow length equals the occluder height in tiles.
    /// * Higher values simulate a higher sun (shorter shadows).
    /// * Lower values simulate a lower sun near the horizon (longer shadows).
    /// * Must be greater than `0.0` for shadows to be rendered.
    pub strength: f32,
    /// The 2D direction from which the light comes (i.e. the light source direction).
    /// Should be a unit vector. Shadows are cast in the opposite direction.
    ///
    /// For example `Vec2::new(0.0, 1.0)` means the sun is directly "north" (above in
    /// screen space) and shadows point south.
    pub direction: Vec2,
    /// World units per height unit used for shadow length calculation.
    ///
    /// Set this to match your tile size so that `occluder.height = 1.0` (one meter)
    /// produces a shadow that is exactly one tile long at `strength = 1.0`.
    /// Defaults to `64.0`.
    pub tile_size: f32,
}

impl Default for DirectionalLight2d {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            strength: 1.0,
            direction: Vec2::new(0.0, 1.0),
            tile_size: 64.0,
        }
    }
}
