use bevy::{
    prelude::*,
    render::{render_resource::ShaderType, view::ViewVisibility, Extract},
};

use crate::{components::LightOccluder2d, prelude::*, resources::Lighting2dSettings};

#[derive(Component, Clone, ShaderType)]
pub struct ExtractedLighting2dSettings {
    pub blur_coc: f32,
    pub fixed_resolution: u32,
    pub ambient_light: LinearRgba,
}

pub fn extract_lighting2d_settings(
    mut commands: Commands,
    lighting_settings: Extract<Res<Lighting2dSettings>>,
    ambient_light_query: Extract<Query<(Entity, &AmbientLight2d), With<Camera2d>>>,
    views_query: Extract<Query<Entity, (With<Camera2d>, Without<AmbientLight2d>)>>,
) {
    let values = ambient_light_query
        .iter()
        .map(|(e, ambient_light)| {
            (
                e,
                ExtractedLighting2dSettings {
                    blur_coc: lighting_settings.shadow_softness,
                    fixed_resolution: if lighting_settings.fixed_resolution {
                        1
                    } else {
                        0
                    },
                    ambient_light: ambient_light.color.to_linear() * ambient_light.brightness,
                },
            )
        })
        .collect::<Vec<_>>();

    commands.insert_or_spawn_batch(values);

    for entity in &views_query {
        commands.entity(entity).insert(ExtractedLighting2dSettings {
            blur_coc: lighting_settings.shadow_softness,
            fixed_resolution: if lighting_settings.fixed_resolution {
                1
            } else {
                0
            },
            ambient_light: Default::default(),
        });
    }
}

#[derive(Component, Default, Clone, ShaderType)]
pub struct ExtractedLightOccluder2d {
    pub center: Vec2,
    pub half_size: Vec2,
}

pub fn extract_light_occluders(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    light_occluders_query: Extract<
        Query<(Entity, &LightOccluder2d, &GlobalTransform, &ViewVisibility)>,
    >,
) {
    commands.spawn(ExtractedLightOccluder2d::default());

    let mut values = Vec::with_capacity(*previous_len);

    for (entity, light_occluder, transform, view_visibility) in &light_occluders_query {
        if !view_visibility.get() {
            continue;
        }

        values.push((
            entity,
            ExtractedLightOccluder2d {
                half_size: light_occluder.half_size,
                center: transform.translation().xy(),
            },
        ));
    }

    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
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
    mut previous_len: Local<usize>,
    point_lights_query: Extract<Query<(Entity, &PointLight2d, &GlobalTransform, &ViewVisibility)>>,
) {
    commands.spawn(ExtractedPointLight2d::default());

    let mut values = Vec::with_capacity(*previous_len);

    for (entity, point_light, transform, visibility) in point_lights_query.iter() {
        if !visibility.get() {
            continue;
        }

        values.push((
            entity,
            ExtractedPointLight2d {
                color: point_light.color.to_linear(),
                center: transform.translation().xy(),
                radius: point_light.radius,
                intensity: point_light.intensity,
                falloff: point_light.falloff,
            },
        ));
    }

    *previous_len = values.len();
    commands.insert_or_spawn_batch(values);
}
