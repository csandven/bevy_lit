use bevy::{asset::AsAssetId, prelude::*, render::extract_component::ExtractComponent};

/// A light occluder component. Should be used alongside a Mesh2d
#[derive(Component, Clone, Debug, Reflect, ExtractComponent)]
pub struct LightOccluder2d {
    /// Any texture with a transparent background. The occluder will take it's shape.
    pub occluder_mask: Handle<Image>,
    /// Height of this occluder in world-space height units (meters).
    /// Used by [`DirectionalLight2d`](crate::directional_light::DirectionalLight2d) to calculate
    /// the projected shadow length. Defaults to `32.0`.
    pub height: f32,
}

impl Default for LightOccluder2d {
    fn default() -> Self {
        Self {
            occluder_mask: Handle::default(),
            height: 32.0,
        }
    }
}

impl LightOccluder2d {
    /// Creates a new [`LightOccluder2d`] with an occlusion mask
    pub fn new(occluder_mask: Handle<Image>) -> Self {
        Self {
            occluder_mask,
            ..Default::default()
        }
    }

    /// Creates a new [`LightOccluder2d`] with an occlusion mask and a height
    pub fn with_height(occluder_mask: Handle<Image>, height: f32) -> Self {
        Self { occluder_mask, height }
    }
}

impl From<LightOccluder2d> for AssetId<Image> {
    fn from(material: LightOccluder2d) -> Self {
        material.occluder_mask.id()
    }
}

impl From<&LightOccluder2d> for AssetId<Image> {
    fn from(material: &LightOccluder2d) -> Self {
        material.occluder_mask.id()
    }
}

impl AsAssetId for LightOccluder2d {
    type Asset = Image;

    fn as_asset_id(&self) -> AssetId<Self::Asset> {
        self.occluder_mask.id()
    }
}
