use std::ops::Range;

use bevy::{
    ecs::{
        system::{lifetimeless::SRes, SystemChangeTick, SystemParamItem},
    },
    math::FloatOrd,
    prelude::*,
    render::{
        batching::no_gpu_preprocessing::batch_and_prepare_sorted_render_phase,
        mesh::RenderMesh,
        render_asset::{prepare_assets, RenderAssets},
        render_phase::{
            AddRenderCommand, CachedRenderPipelinePhaseItem, DrawFunctionId, DrawFunctions,
            PhaseItem, PhaseItemExtraIndex, RenderCommand, RenderCommandResult, SetItemPipeline,
            SortedPhaseItem, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{
            BindGroup, BindGroupEntries, CachedRenderPipelineId, PipelineCache,
            SamplerDescriptor, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        sync_world::{MainEntity, MainEntityHashMap},
        texture::FallbackImage,
        view::{ExtractedView, RenderVisibleEntities},
        Extract, Render, RenderApp, RenderSystems,
    },
    sprite_render::{
        DrawMesh2d, EntitiesNeedingSpecialization, EntitySpecializationTickPair, Mesh2dPipeline,
        Mesh2dPipelineKey, RenderMesh2dInstances, SetMesh2dBindGroup, SetMesh2dViewBindGroup,
        SpecializedMaterial2dPipelineCache, ViewKeyCache,
    },
    utils::Parallel,
};

use crate::{
    render::extract_light2d_phases,
    voronoi::MaskPipeline,
};

/// Marks an entity whose [`Mesh2d`] defines an area under a roof where
/// directional (sun) shadows should not be rendered.
///
/// Works exactly like [`LightOccluder2d`](crate::occlusion::LightOccluder2d)
/// from a rendering standpoint but writes to a separate roof-mask texture
/// instead of the Voronoi/SDF texture.
///
/// # Example
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_lit::prelude::RoofMask2d;
/// 
/// fn setup(mut commands: Commands) {
///     commands.spawn((
///         Transform::from_xyz(256.0, 256.0, 5.0),
///         RoofMask2d::new(Vec2::new(128.0, 96.0)),
///     ));
/// }
/// ```
#[derive(Component, Clone, Debug, Reflect)]
pub struct RoofMask2d {
    /// Size of the rectangular roof-mask area in world units.
    pub size: Vec2,
}

impl Default for RoofMask2d {
    fn default() -> Self {
        Self {
            size: Vec2::splat(64.0),
        }
    }
}

impl RoofMask2d {
    /// Creates a new rectangular roof mask with the provided size.
    pub fn new(size: Vec2) -> Self {
        Self { size }
    }
}

#[derive(Component)]
struct RoofMaskRenderChild;

#[derive(Component, Clone, Copy)]
struct RoofMaskOwner(Entity);

pub struct RoofMaskPhase {
    pub sort_key: FloatOrd,
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    pub entity: (Entity, MainEntity),
    pub batch_range: Range<u32>,
    pub extra_index: PhaseItemExtraIndex,
    pub indexed: bool,
}

impl PhaseItem for RoofMaskPhase {
    #[inline]
    fn entity(&self) -> Entity {
        self.entity.0
    }

    #[inline]
    fn main_entity(&self) -> MainEntity {
        self.entity.1
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }

    #[inline]
    fn batch_range(&self) -> &Range<u32> {
        &self.batch_range
    }

    #[inline]
    fn batch_range_mut(&mut self) -> &mut Range<u32> {
        &mut self.batch_range
    }

    #[inline]
    fn extra_index(&self) -> PhaseItemExtraIndex {
        self.extra_index.clone()
    }

    #[inline]
    fn batch_range_and_extra_index_mut(&mut self) -> (&mut Range<u32>, &mut PhaseItemExtraIndex) {
        (&mut self.batch_range, &mut self.extra_index)
    }
}

impl SortedPhaseItem for RoofMaskPhase {
    type SortKey = FloatOrd;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        self.sort_key
    }

    fn indexed(&self) -> bool {
        self.indexed
    }
}

impl CachedRenderPipelinePhaseItem for RoofMaskPhase {
    #[inline]
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct RenderRoofMaterials(MainEntityHashMap<()>);

fn extract_roof_materials(
    mut render_instances: ResMut<RenderRoofMaterials>,
    query: Extract<Query<(Entity, &ViewVisibility, &RoofMask2d), With<Mesh2d>>>,
) {
    render_instances.clear();
    for (entity, view_visibility, material) in &query {
        if view_visibility.get() {
            let _ = material;
            render_instances.insert(entity.into(), ());
        }
    }
}

fn sync_roof_mask_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    dirty_owners: Query<
        (Entity, &RoofMask2d),
        (
            Without<RoofMaskRenderChild>,
            Or<(Added<RoofMask2d>, Changed<RoofMask2d>)>,
        ),
    >,
    owners: Query<(), (With<RoofMask2d>, Without<RoofMaskRenderChild>)>,
    children: Query<(Entity, &RoofMaskOwner), With<RoofMaskRenderChild>>,
) {
    let child_by_owner = children
        .iter()
        .map(|(child, owner)| (owner.0, child))
        .collect::<std::collections::HashMap<_, _>>();

    for (owner, roof_mask) in &dirty_owners {
        let mesh = Mesh2d(meshes.add(Rectangle::from_size(roof_mask.size)));

        if let Some(child) = child_by_owner.get(&owner) {
            commands.entity(*child).insert((mesh, roof_mask.clone()));
        } else {
            commands.entity(owner).with_children(|parent| {
                parent.spawn((
                    mesh,
                    Transform::default(),
                    Visibility::default(),
                    roof_mask.clone(),
                    RoofMaskRenderChild,
                    RoofMaskOwner(owner),
                ));
            });
        }
    }

    for (child, owner) in &children {
        if owners.get(owner.0).is_err() {
            commands.entity(child).despawn();
        }
    }
}

fn check_roof_materials_needing_specialization(
    needs_specialization: Query<
        Entity,
        (
            Or<(
                Changed<Mesh2d>,
                AssetChanged<Mesh2d>,
                Changed<RoofMask2d>,
                Changed<GlobalTransform>,
            )>,
            With<RoofMask2d>,
        ),
    >,
    mut par_local: Local<Parallel<Vec<Entity>>>,
    mut entities_needing_specialization: ResMut<EntitiesNeedingSpecialization<RoofMask2d>>,
) {
    entities_needing_specialization.clear();
    needs_specialization
        .par_iter()
        .for_each(|entity| par_local.borrow_local_mut().push(entity));
    par_local.drain_into(&mut entities_needing_specialization);
}

fn extract_entities_needs_roof_specialization(
    entities_needing: Extract<Res<EntitiesNeedingSpecialization<RoofMask2d>>>,
    mut ticks: ResMut<EntitySpecializationTickPair<RoofMask2d>>,
    mut removed: Extract<RemovedComponents<RoofMask2d>>,
    mut cache: ResMut<SpecializedMaterial2dPipelineCache<RoofMask2d>>,
    views: Query<&MainEntity, With<ExtractedView>>,
    change_tick: SystemChangeTick,
) {
    for entity in removed.read() {
        ticks.remove(&MainEntity::from(entity));
        for view in &views {
            if let Some(c) = cache.get_mut(view) {
                c.remove(&MainEntity::from(entity));
            }
        }
    }
    for entity in entities_needing.iter() {
        ticks.insert((*entity).into(), change_tick.this_run());
    }
}

fn specialize_roof_meshes(
    render_meshes: Res<RenderAssets<RenderMesh>>,
    pipeline_cache: Res<PipelineCache>,
    mut render_mesh_instances: ResMut<RenderMesh2dInstances>,
    mut mask_pipelines: ResMut<bevy::render::render_resource::SpecializedMeshPipelines<MaskPipeline>>,
    mask_pipeline: Res<MaskPipeline>,
    view_key_cache: Res<ViewKeyCache>,
    views: Query<(&MainEntity, &RenderVisibleEntities)>,
    render_material_instances: Res<RenderRoofMaterials>,
    mut specialized_cache: ResMut<SpecializedMaterial2dPipelineCache<RoofMask2d>>,
    material_ticks: Res<EntitySpecializationTickPair<RoofMask2d>>,
    view_ticks: Res<crate::voronoi::VoronoiViewSpecializationTicks>,
    ticks: SystemChangeTick,
) {
    if render_material_instances.is_empty() {
        return;
    }

    for (view_entity, visible_entities) in &views {
        let Some(view_key) = view_key_cache.get(view_entity) else {
            continue;
        };
        let Some(view_tick) = view_ticks.get(view_entity) else {
            continue;
        };

        let view_cache = specialized_cache.entry(*view_entity).or_default();

        for (_, entity) in visible_entities.iter::<Mesh2d>() {
            if !render_material_instances.contains_key(entity) {
                continue;
            }

            let Some(entity_tick) = material_ticks.get(entity) else {
                continue;
            };

            let last = view_cache.get(entity).map(|(t, _)| *t);
            let needs = last.is_none_or(|t| {
                view_tick.is_newer_than(t, ticks.this_run())
                    || entity_tick.is_newer_than(t, ticks.this_run())
            });
            if !needs {
                continue;
            }

            let Some(mesh_instance) = render_mesh_instances.get_mut(entity) else {
                continue;
            };
            let Some(mesh) = render_meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };

            let pipeline_id = match mask_pipelines.specialize(
                &pipeline_cache,
                &mask_pipeline,
                *view_key
                    | Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology()),
                &mesh.layout,
            ) {
                Ok(id) => id,
                Err(err) => {
                    error!("{}", err);
                    continue;
                }
            };

            view_cache.insert(*entity, (ticks.this_run(), pipeline_id));
        }
    }
}

fn queue_roof_meshes(
    draw_functions: Res<DrawFunctions<RoofMaskPhase>>,
    render_meshes: Res<RenderAssets<RenderMesh>>,
    mut render_mesh_instances: ResMut<RenderMesh2dInstances>,
    mut render_phase: ResMut<ViewSortedRenderPhases<RoofMaskPhase>>,
    views: Query<(&MainEntity, &ExtractedView, &RenderVisibleEntities)>,
    render_material_instances: Res<RenderRoofMaterials>,
    mut specialized_cache: ResMut<SpecializedMaterial2dPipelineCache<RoofMask2d>>,
) {
    if render_material_instances.is_empty() {
        return;
    }

    for (view_entity, view, visible_entities) in &views {
        let Some(phase) = render_phase.get_mut(&view.retained_view_entity) else {
            continue;
        };

        let draw_fn = draw_functions.read().id::<DrawRoofMesh>();
        let view_cache = specialized_cache.entry(*view_entity).or_default();

        for (render_entity, entity) in visible_entities.iter::<Mesh2d>() {
            if !render_material_instances.contains_key(entity) {
                continue;
            }

            let Some(mesh_instance) = render_mesh_instances.get_mut(entity) else {
                continue;
            };
            let Some(mesh) = render_meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let Some((_, pipeline_id)) = view_cache.get(entity) else {
                continue;
            };

            phase.add(RoofMaskPhase {
                sort_key: FloatOrd(
                    mesh_instance.transforms.world_from_local.translation.z,
                ),
                pipeline: *pipeline_id,
                draw_function: draw_fn,
                entity: (*render_entity, *entity),
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: mesh.indexed(),
            });
        }
    }
}

pub struct PreparedRoofMaskMaterialBindGroup {
    bind_group: BindGroup,
    _height_uniform: UniformBuffer<Vec4>,
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct RoofMaskMaterialBindGroups(MainEntityHashMap<PreparedRoofMaskMaterialBindGroup>);

fn prepare_roof_material_bind_groups(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    pipeline: Res<MaskPipeline>,
    fallback_image: Res<FallbackImage>,
    roof_materials: Res<RenderRoofMaterials>,
    mut bind_groups: ResMut<RoofMaskMaterialBindGroups>,
) {
    bind_groups.clear();

    let material_layout = pipeline_cache.get_bind_group_layout(&pipeline.material_layout);
    for (entity, _) in roof_materials.iter() {
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());
        let mut height_uniform = UniformBuffer::from(Vec4::ZERO);
        height_uniform.write_buffer(&render_device, &render_queue);
        let Some(height_binding) = height_uniform.binding() else {
            continue;
        };
        let bind_group = render_device.create_bind_group(
            "roof_mask_material_bind_group",
            &material_layout,
            &BindGroupEntries::sequential((&fallback_image.d2.texture_view, &sampler, height_binding)),
        );
        bind_groups.insert(
            *entity,
            PreparedRoofMaskMaterialBindGroup {
                bind_group,
                _height_uniform: height_uniform,
            },
        );
    }
}

pub type DrawRoofMesh = (
    SetItemPipeline,
    SetMesh2dViewBindGroup<0>,
    SetMesh2dBindGroup<1>,
    SetRoofMaskMaterialBindGroup<2>,
    DrawMesh2d,
);

pub struct SetRoofMaskMaterialBindGroup<const I: usize>;

impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetRoofMaskMaterialBindGroup<I> {
    type Param = SRes<RoofMaskMaterialBindGroups>;
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: Option<()>,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let bind_groups = bind_groups.into_inner();
        let Some(bind_group) = bind_groups.get(&item.main_entity()) else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, &bind_group.bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct RoofMask2dPlugin;

impl Plugin for RoofMask2dPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntitiesNeedingSpecialization<RoofMask2d>>()
            .add_systems(
                PostUpdate,
                (sync_roof_mask_meshes, check_roof_materials_needing_specialization),
            );

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedMaterial2dPipelineCache<RoofMask2d>>()
            .init_resource::<EntitySpecializationTickPair<RoofMask2d>>()
            .init_resource::<RenderRoofMaterials>()
            .init_resource::<RoofMaskMaterialBindGroups>()
            .init_resource::<DrawFunctions<RoofMaskPhase>>()
            .init_resource::<ViewSortedRenderPhases<RoofMaskPhase>>()
            .add_render_command::<RoofMaskPhase, DrawRoofMesh>()
            .add_systems(
                ExtractSchedule,
                (
                    extract_entities_needs_roof_specialization,
                    extract_roof_materials,
                )
                    .after(extract_light2d_phases),
            )
            .add_systems(
                Render,
                (
                    specialize_roof_meshes
                        .in_set(RenderSystems::PrepareMeshes)
                        .after(prepare_assets::<RenderMesh>),
                    queue_roof_meshes
                        .in_set(RenderSystems::QueueMeshes)
                        .after(prepare_assets::<RenderMesh>),
                    batch_and_prepare_sorted_render_phase::<RoofMaskPhase, Mesh2dPipeline>
                        .in_set(RenderSystems::PrepareResources),
                    prepare_roof_material_bind_groups.in_set(RenderSystems::PrepareBindGroups),
                ),
            );
    }
}
