use bevy::{
    prelude::*,
    render::{
        extract_component::ComponentUniforms,
        render_resource::{
            BindGroup, BindGroupEntries, CachedRenderPipelineId, GpuArrayBuffer, PipelineCache,
            SamplerDescriptor, SpecializedRenderPipelines, TextureDescriptor, TextureDimension,
            TextureFormat, TextureUsages,
        },
        renderer::RenderDevice,
        texture::{CachedTexture, TextureCache},
        view::{ExtractedView, ViewTarget, ViewUniforms},
    },
};

use crate::{
    extract::{ExtractedLightOccluder2d, ExtractedLighting2dSettings, ExtractedPointLight2d},
    pipeline::{Lighting2dPipelineKey, Lighting2dPrepassPipelines, PostProcessPipeline},
};

fn create_aux_texture(
    view_target: &ViewTarget,
    texture_cache: &mut TextureCache,
    render_device: &RenderDevice,
    label: &'static str,
) -> CachedTexture {
    texture_cache.get(render_device, TextureDescriptor {
        label: Some(label),
        size: view_target.main_texture().size(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba16Float,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

#[derive(Component)]
pub struct Lighting2dAuxiliaryTextures {
    pub sdf: CachedTexture,
    pub lighting: CachedTexture,
    pub blur: Option<CachedTexture>,
}

pub fn prepare_lighting_auxiliary_textures(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
    view_targets: Query<(Entity, &ViewTarget, &ExtractedLighting2dSettings)>,
) {
    for (entity, view_target, settings) in &view_targets {
        commands.entity(entity).insert(Lighting2dAuxiliaryTextures {
            sdf: create_aux_texture(view_target, &mut texture_cache, &render_device, "sdf"),
            lighting: create_aux_texture(
                view_target,
                &mut texture_cache,
                &render_device,
                "lighting",
            ),
            blur: if settings.blur > 0.0 {
                Some(create_aux_texture(
                    view_target,
                    &mut texture_cache,
                    &render_device,
                    "blur",
                ))
            } else {
                None
            },
        });
    }
}

#[derive(Component)]
pub struct Lighting2dPostProcessPipelineId(pub CachedRenderPipelineId);

pub fn prepare_post_process_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut post_process_pipelines: ResMut<SpecializedRenderPipelines<PostProcessPipeline>>,
    post_process_pipeline: Res<PostProcessPipeline>,
    views_query: Query<(Entity, &ExtractedView), With<ExtractedLighting2dSettings>>,
) {
    for (entity, view) in &views_query {
        commands
            .entity(entity)
            .insert(Lighting2dPostProcessPipelineId(
                post_process_pipelines.specialize(
                    &pipeline_cache,
                    &post_process_pipeline,
                    Lighting2dPipelineKey { hdr: view.hdr },
                ),
            ));
    }
}

#[derive(Component)]
pub struct Lighting2dSurfaceBindGroups {
    pub sdf: BindGroup,
    pub lighting: BindGroup,
    pub blur: BindGroup,
}

pub fn prepare_lighting_bind_groups(
    mut commands: Commands,
    prepass_pipelines: Res<Lighting2dPrepassPipelines>,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
    light_settings: Res<ComponentUniforms<ExtractedLighting2dSettings>>,
    point_lights: Res<GpuArrayBuffer<ExtractedPointLight2d>>,
    light_occluders: Res<GpuArrayBuffer<ExtractedLightOccluder2d>>,
    views_query: Query<(Entity, &Lighting2dAuxiliaryTextures), With<ExtractedLighting2dSettings>>,
) {
    let (Some(view_uniform), Some(lighting_settings), Some(light_occluders), Some(point_lights)) = (
        view_uniforms.uniforms.binding(),
        light_settings.binding(),
        light_occluders.binding(),
        point_lights.binding(),
    ) else {
        return;
    };

    let sampler = render_device.create_sampler(&SamplerDescriptor::default());

    for (entity, aux_textures) in &views_query {
        commands.entity(entity).insert(Lighting2dSurfaceBindGroups {
            sdf: render_device.create_bind_group(
                "sdf_bind_group",
                &prepass_pipelines.sdf_layout,
                &BindGroupEntries::sequential((view_uniform.clone(), light_occluders.clone())),
            ),
            lighting: render_device.create_bind_group(
                "lighting2d_bind_group",
                &prepass_pipelines.lighting_layout,
                &BindGroupEntries::sequential((
                    view_uniform.clone(),
                    lighting_settings.clone(),
                    point_lights.clone(),
                    &aux_textures.sdf.default_view,
                    &sampler,
                )),
            ),
            blur: render_device.create_bind_group(
                "blur_bind_group",
                &prepass_pipelines.blur_layout,
                &BindGroupEntries::sequential((
                    view_uniform.clone(),
                    lighting_settings.clone(),
                    &aux_textures.lighting.default_view,
                    &sampler,
                )),
            ),
        });
    }
}
