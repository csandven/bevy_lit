use crate::prelude::*;
use bevy::render::sync_world::RenderEntity;
use bevy::{
    prelude::*,
    render::{Extract, render_resource::ShaderType, view::ViewVisibility},
};

#[derive(Component, Clone, ShaderType)]
pub struct ExtractedLighting2dSettings {
    pub blur: f32,
    pub fixed_resolution: u32,
    pub ambient_light: LinearRgba,
    pub raymarch: RaymarchSettings,
}

pub fn extract_lighting_settings(
    mut commands: Commands,
    ambient_light_query: Extract<
        Query<(RenderEntity, &Lighting2dSettings, Option<&AmbientLight2d>), With<Camera2d>>,
    >,
) {
    let values = ambient_light_query
        .iter()
        .map(|(e, settings, ambient_light)| {
            let ambient_light = ambient_light.unwrap_or(&AmbientLight2d {
                color: Color::WHITE,
                brightness: 1.0,
            });

            (e, ExtractedLighting2dSettings {
                blur: settings.blur,
                fixed_resolution: if settings.fixed_resolution { 1 } else { 0 },
                ambient_light: ambient_light.color.to_linear() * ambient_light.brightness,
                raymarch: settings.raymarch.clone(),
            })
        })
        .collect::<Vec<_>>();

    commands.insert_or_spawn_batch(values);
}

#[derive(Component, Default, Clone, ShaderType)]
pub struct ExtractedLightOccluder2d {
    pub center: Vec2,
    pub half_size: Vec2,
}

pub fn extract_light_occluders(
    mut commands: Commands,
    light_occluders_query: Extract<
        Query<(
            RenderEntity,
            &LightOccluder2d,
            &GlobalTransform,
            &ViewVisibility,
        )>,
    >,
) {
    for (render_entity, light_occluder, transform, view_visibility) in &light_occluders_query {
        if !view_visibility.get() {
            continue;
        }

        commands
            .entity(render_entity)
            .insert(ExtractedLightOccluder2d {
                half_size: light_occluder.half_size,
                center: transform.translation().xy(),
            });
    }
}

#[derive(Component, Default, Clone, ShaderType)]
pub struct ExtractedPointLight2d {
    pub center: Vec2,
    pub color: LinearRgba,
    pub falloff: f32,
    pub intensity: f32,
    pub radius: f32,
}

pub fn extract_point_lights(
    mut commands: Commands,
    point_lights_query: Extract<
        Query<(
            RenderEntity,
            &PointLight2d,
            &GlobalTransform,
            &ViewVisibility,
        )>,
    >,
) {
    for (render_entity, point_light, transform, visibility) in point_lights_query.iter() {
        if !visibility.get() {
            continue;
        }

        commands
            .entity(render_entity)
            .insert(ExtractedPointLight2d {
                color: point_light.color.to_linear(),
                center: transform.translation().xy(),
                radius: point_light.radius,
                intensity: point_light.intensity,
                falloff: point_light.falloff,
            });
    }
}
