use bevy::prelude::*;

#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct Lighting2dSettings {
    pub shadow_softness: f32,
    /// if false the shadow softness is calculated in relation to the viewport size
    pub fixed_resolution: bool,
}

impl Default for Lighting2dSettings {
    fn default() -> Self {
        Self {
            shadow_softness: 0.0,
            fixed_resolution: true,
        }
    }
}
