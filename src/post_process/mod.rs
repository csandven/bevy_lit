use bevy::{
    asset::embedded_asset,
    prelude::*,
    render::{
        extract_component::UniformComponentPlugin, render_resource::SpecializedRenderPipelines,
        Render, RenderApp, RenderStartup, RenderSystems,
    },
};

use crate::post_process::render::{
    extract_lighting2d_settings, init_directional_shadow_pipeline,
    init_lighting2d_composite_pipeline, init_post_process_pipelines, prepare_composite_pipelines,
    ExtractedLighting2dSettings, Lighting2dCompositePipeline,
};

pub mod render;

pub struct Lighting2dSettingsPlugin;
impl Plugin for Lighting2dSettingsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "penetration.wgsl");
        embedded_asset!(app, "blur.wgsl");
        embedded_asset!(app, "composite.wgsl");
        embedded_asset!(app, "directional_shadow.wgsl");

        app.add_plugins(UniformComponentPlugin::<ExtractedLighting2dSettings>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedRenderPipelines<Lighting2dCompositePipeline>>()
            .add_systems(ExtractSchedule, extract_lighting2d_settings)
            .add_systems(
                RenderStartup,
                (
                    init_post_process_pipelines,
                    init_lighting2d_composite_pipeline,
                    init_directional_shadow_pipeline,
                ),
            )
            .add_systems(
                Render,
                prepare_composite_pipelines.in_set(RenderSystems::Prepare),
            );
    }
}
