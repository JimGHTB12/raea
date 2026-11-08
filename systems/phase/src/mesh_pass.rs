//! A mesh drawn again in a phase of its own (docs/control.feature FC2d), as Bevy's shader_advanced/custom_render_phase
//! draws its stencil: a Mesh3d entity with a MeshPass is drawn, after the camera's main opaque pass, by a pipeline of
//! its shader file as written (its `vertex` and `fragment`, reading the mesh's positions at location 0, compressed or
//! not), with Bevy's mesh view and mesh bind groups (0 and 2, group 1 empty), straight onto the view's colour target
//! with no depth attachment, back faces culled. The phase is sorted by distance, as the example's Stencil3d is.

use std::ops::Range;

use bevy::core_pipeline::core_3d::{main_opaque_pass_3d, TransparentSortingInfo3d};
use bevy::core_pipeline::schedule::Core3d;
use bevy::core_pipeline::Core3dSystems;
use bevy::camera::visibility::{self, VisibilityClass};
use bevy::camera::{MainPassResolutionOverride, Viewport};
use bevy::ecs::entity::EntityHash;
use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::math::FloatOrd;
use bevy::mesh::{MeshAttributeCompressionFlags, MeshVertexBufferLayoutRef};
use bevy::pbr::{self, DrawMesh, MeshInputUniform, MeshPipeline, MeshPipelineKey, MeshPipelineSystems, MeshPipelineViewLayoutKey, MeshUniform, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup, SetMeshViewEmptyBindGroup, ViewKeyCache};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::render::batching::gpu_preprocessing::{batch_and_prepare_sorted_render_phase, BatchedInstanceBuffers, IndirectParametersMetadata, UntypedPhaseIndirectParametersBuffers};
use bevy::render::batching::{GetBatchData, GetFullBatchData};
use bevy::render::camera::{DirtySpecializations, ExtractedCamera, PendingQueues};
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::mesh::allocator::MeshAllocator;
use bevy::render::mesh::RenderMesh;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{sort_phase_system, AddRenderCommand, CachedRenderPipelinePhaseItem, DrawFunctionId, DrawFunctions, PhaseItem, PhaseItemExtraIndex, SetItemPipeline, SortedPhaseItem, SortedRenderPhasePlugin, ViewSortedRenderPhases};
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, ViewQuery};
use bevy::render::sync_world::{MainEntity, MainEntityHashSet};
use bevy::render::view::{ExtractedView, RenderVisibleEntities, RetainedViewEntity, ViewTarget};
use bevy::render::{Extract, Render, RenderApp, RenderDebugFlags, RenderStartup, RenderSystems};
use indexmap::IndexMap;
use nonmax::NonMaxU32;

/// A mesh's own pass: its shader file (as written) and entry points.
#[derive(Component, Clone, Debug, Default, Reflect, ExtractComponent)]
#[extract_app(RenderApp)]
#[reflect(Component, Default)]
#[require(VisibilityClass)]
#[component(on_add = visibility::add_visibility_class::<MeshPass>)]
#[type_path = "raea_sys_phase"]
pub struct MeshPass {
    pub shader: String,
    pub vertex: String,
    pub fragment: String,
    /// its shader file, registered as written (raea-sys-shader), and the id register gave it
    #[reflect(ignore)]
    pub registered: Option<(String, u128)>,
}

pub fn install(app: &mut App) {
    app.register_type::<MeshPass>();
    app.add_systems(PostUpdate, crate::register::<MeshPass>);
    app.add_plugins((ExtractComponentPlugin::<MeshPass>::default(), SortedRenderPhasePlugin::<Pass3d, MeshPipeline>::new(RenderDebugFlags::default())));
    let Some(r) = app.get_sub_app_mut(RenderApp) else { return };
    r.init_resource::<SpecializedMeshPipelines<PassPipeline>>()
        .init_resource::<DrawFunctions<Pass3d>>()
        .add_render_command::<Pass3d, DrawPass>()
        .init_resource::<ViewSortedRenderPhases<Pass3d>>()
        .init_resource::<Pending>()
        .add_systems(RenderStartup, init.after(MeshPipelineSystems))
        .add_systems(ExtractSchedule, extract_phases)
        .add_systems(Render, (queue.in_set(RenderSystems::QueueMeshes), sort_phase_system::<Pass3d>.in_set(RenderSystems::PhaseSort), batch_and_prepare_sorted_render_phase::<Pass3d, PassPipeline>.in_set(RenderSystems::PrepareResources)))
        .add_systems(Core3d, draw.after(main_opaque_pass_3d).in_set(Core3dSystems::MainPass));
}

#[derive(Resource)]
pub struct PassPipeline {
    mesh: MeshPipeline,
}

fn init(mut commands: Commands, mesh: Res<MeshPipeline>) {
    commands.insert_resource(PassPipeline { mesh: mesh.clone() });
}

/// A pass pipeline's variant: the view and mesh's key, its shader and entry points.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct PassKey {
    mesh: MeshPipelineKey,
    shader: u128,
    vertex: String,
    fragment: String,
}

impl SpecializedMeshPipeline for PassPipeline {
    type Key = PassKey;

    fn specialize(&self, key: PassKey, layout: &MeshVertexBufferLayoutRef) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut defs = Vec::new();
        let mut attributes = Vec::new();
        if layout.0.contains(Mesh::ATTRIBUTE_POSITION) {
            if layout.0.get_attribute_compression().contains(MeshAttributeCompressionFlags::COMPRESS_POSITION) {
                defs.push("VERTEX_POSITIONS_COMPRESSED".into());
            }
            attributes.push(Mesh::ATTRIBUTE_POSITION.at_shader_location(0));
        }
        let buffer = layout.0.get_layout(&attributes)?;
        let view = self.mesh.get_view_layout(MeshPipelineViewLayoutKey::from(key.mesh));
        let shader = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(key.shader), core::marker::PhantomData);
        Ok(RenderPipelineDescriptor {
            label: Some("raea_mesh_pass".into()),
            layout: vec![view.main_layout, view.empty_layout, self.mesh.mesh_layouts.model_only.clone()],
            vertex: VertexState { shader: shader.clone(), shader_defs: defs.clone(), entry_point: Some(key.vertex.into()), buffers: vec![buffer], ..default() },
            fragment: Some(FragmentState { shader, shader_defs: defs, entry_point: Some(key.fragment.into()), targets: vec![Some(ColorTargetState { format: key.mesh.target_format(), blend: None, write_mask: ColorWrites::ALL })], ..default() }),
            primitive: PrimitiveState { topology: key.mesh.primitive_topology(), strip_index_format: key.mesh.strip_index_format(), cull_mode: Some(Face::Back), ..default() },
            ..default()
        })
    }
}

type DrawPass = (SetItemPipeline, SetMeshViewBindGroup<0>, SetMeshViewEmptyBindGroup<1>, SetMeshBindGroup<2>, DrawMesh);

/// A mesh in the pass: sorted by its distance from the view, never batched.
pub struct Pass3d {
    sorting: TransparentSortingInfo3d,
    distance: FloatOrd,
    main: MainEntity,
    pipeline: CachedRenderPipelineId,
    draw_function: DrawFunctionId,
    batch_range: Range<u32>,
    extra_index: PhaseItemExtraIndex,
    indexed: bool,
}

impl PhaseItem for Pass3d {
    // a mesh drawn again: no render entity of its own (Bevy's Entity::PLACEHOLDER, as the example queues it)
    fn entity(&self) -> Entity {
        Entity::PLACEHOLDER
    }
    fn main_entity(&self) -> MainEntity {
        self.main
    }
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }
    fn batch_range(&self) -> &Range<u32> {
        &self.batch_range
    }
    fn batch_range_mut(&mut self) -> &mut Range<u32> {
        self.batch_range_and_extra_index_mut().0
    }
    fn extra_index(&self) -> PhaseItemExtraIndex {
        self.extra_index.clone()
    }
    fn batch_range_and_extra_index_mut(&mut self) -> (&mut Range<u32>, &mut PhaseItemExtraIndex) {
        (&mut self.batch_range, &mut self.extra_index)
    }
}

impl SortedPhaseItem for Pass3d {
    type SortKey = FloatOrd;
    fn sort_key(&self) -> FloatOrd {
        self.distance
    }
    fn sort(items: &mut IndexMap<(Entity, MainEntity), Pass3d, EntityHash>) {
        items.sort_by_key(|_, i: &Pass3d| i.distance);
    }
    fn recalculate_sort_keys(items: &mut IndexMap<(Entity, MainEntity), Self, EntityHash>, view: &ExtractedView) {
        let range = view.rangefinder3d();
        for i in items.values_mut() {
            i.distance = FloatOrd(i.sorting.sort_distance(&range));
        }
    }
    fn indexed(&self) -> bool {
        self.indexed
    }
}

impl CachedRenderPipelinePhaseItem for Pass3d {
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline
    }
}

/// A mesh instance's uniform, as the CPU builds it.
fn uniform(mesh_instances: &RenderMeshInstances, allocator: &MeshAllocator, main: MainEntity) -> Option<MeshUniform> {
    let RenderMeshInstances::CpuBuilding(ref m) = *mesh_instances else {
        error!("MeshPass: a CPU-built mesh uniform asked for while the GPU builds them");
        return None;
    };
    let m = m.get(&main)?;
    let first = allocator.mesh_vertex_slice(&m.mesh_asset_id()).map_or(0, |s| s.range.start);
    let meta = allocator.mesh_metadata_slice(&m.mesh_asset_id()).map(|s| s.range.start);
    Some(MeshUniform::new(&m.transforms, first, m.material_bindings_index().slot, None, None, None, Some(m.tag()), meta))
}

impl GetBatchData for PassPipeline {
    type Param = (SRes<RenderMeshInstances>, SRes<MeshAllocator>);
    type BatchSetCompareData = AssetId<Mesh>;
    type BatchCompareData = ();
    type BufferData = MeshUniform;

    fn get_batch_data((m, a): &SystemParamItem<Self::Param>, (_, main): (Entity, MainEntity)) -> Option<(MeshUniform, Option<(AssetId<Mesh>, ())>)> {
        Some((uniform(m, a, main)?, None))
    }
}

impl GetFullBatchData for PassPipeline {
    type BufferInputData = MeshInputUniform;

    fn get_index_and_compare_data((m, _): &SystemParamItem<Self::Param>, main: MainEntity) -> Option<(NonMaxU32, Option<(AssetId<Mesh>, ())>)> {
        let RenderMeshInstances::GpuBuilding(ref m) = **m else {
            error!("MeshPass: a GPU-built mesh index asked for while the CPU builds them");
            return None;
        };
        let m = m.get(&main)?;
        Some((NonMaxU32::new(m.gpu_specific.current_uniform_index())?, m.should_batch().then_some((m.mesh_asset_id(), ()))))
    }

    fn get_binned_batch_data((m, a): &SystemParamItem<Self::Param>, main: MainEntity) -> Option<MeshUniform> {
        uniform(m, a, main)
    }

    fn write_batch_indirect_parameters_metadata(indexed: bool, base_output_index: u32, batch_set_index: Option<NonMaxU32>, buffers: &mut UntypedPhaseIndirectParametersBuffers, offset: u32) {
        let p = IndirectParametersMetadata { base_output_index, batch_set_index: batch_set_index.map_or(!0, u32::from), mesh_index: 0, early_instance_count: 0, late_instance_count: 0 };
        if indexed { buffers.indexed.set(offset, p) } else { buffers.non_indexed.set(offset, p) }
    }

    fn get_binned_index(_: &SystemParamItem<Self::Param>, _: MainEntity) -> Option<NonMaxU32> {
        None
    }
}

/// Each active 3D camera's pass phase, made ready for the frame (a camera gone, its phase dropped).
fn extract_phases(mut phases: ResMut<ViewSortedRenderPhases<Pass3d>>, cameras: Extract<Query<(Entity, &Camera), With<Camera3d>>>, mut live: Local<HashSet<RetainedViewEntity>>) {
    live.clear();
    for (main, camera) in &cameras {
        if camera.is_active {
            let v = RetainedViewEntity::new(main.into(), None, 0);
            phases.prepare_for_new_frame(v);
            live.insert(v);
        }
    }
    phases.retain(|v, _| live.contains(v));
}

#[derive(Default, Deref, DerefMut, Resource)]
struct Pending(PendingQueues);

/// Each visible mesh with a MeshPass queued in its view's pass phase (one not loaded yet, or whose shader isn't
/// registered yet, tried again next frame).
#[allow(clippy::too_many_arguments)]
fn queue(
    draws: Res<DrawFunctions<Pass3d>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<PassPipeline>>,
    cache: Res<PipelineCache>,
    pipeline: Res<PassPipeline>,
    meshes: Res<RenderAssets<RenderMesh>>,
    instances: Res<RenderMeshInstances>,
    batched: Option<Res<BatchedInstanceBuffers<MeshUniform, MeshInputUniform>>>,
    mut phases: ResMut<ViewSortedRenderPhases<Pass3d>>,
    mut views: Query<(&ExtractedView, &RenderVisibleEntities)>,
    keys: Res<ViewKeyCache>,
    dirty: Res<DirtySpecializations>,
    mut pending: ResMut<Pending>,
    passes: Query<&MeshPass>,
    mut scratch: Local<MainEntityHashSet>,
) {
    let draw = draws.read().id::<DrawPass>();
    for (view, visible) in &mut views {
        let Some(phase) = phases.get_mut(&view.retained_view_entity) else { continue };
        let Some(&view_key) = keys.get(&view.retained_view_entity) else { continue };
        let Some(vis) = visible.get::<Mesh3d>() else { continue };
        let waiting = pending.prepare_for_new_frame(view.retained_view_entity);
        for &main in dirty.iter_to_dequeue(view.retained_view_entity, vis) {
            phase.remove(Entity::PLACEHOLDER, main);
        }
        for (render, main) in dirty.iter_to_queue(view.retained_view_entity, vis, &waiting.prev_frame, &mut scratch) {
            let Ok(pass) = passes.get(*render) else { continue };
            let (Some(shader), Some(instance)) = (pass.registered.as_ref().map(|r| r.1), instances.render_mesh_queue_data(*main)) else {
                waiting.current_frame.insert((*render, *main));
                continue;
            };
            let Some(mesh) = meshes.get(instance.mesh_asset_id()) else { continue };
            let key = PassKey { mesh: view_key | MeshPipelineKey::from_primitive_topology_and_strip_index(mesh.primitive_topology(), mesh.index_format()), shader, vertex: pass.vertex.clone(), fragment: pass.fragment.clone() };
            let id = match pipelines.specialize(&cache, &pipeline, key, &mesh.layout) {
                Ok(id) => id,
                Err(err) => {
                    error!("MeshPass: {err}");
                    continue;
                }
            };
            let centre = pbr::get_mesh_instance_world_from_local(*main, instance.current_uniform_index, &instances, batched.as_deref()).transform_point3(mesh.aabb_center);
            phase.add_retained(Pass3d {
                sorting: TransparentSortingInfo3d::Sorted { mesh_center: centre, depth_bias: 0.0 },
                distance: FloatOrd(0.0),
                main: *main,
                pipeline: id,
                draw_function: draw,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: mesh.indexed(),
            });
        }
    }
}

/// The pass drawn onto the view's colour target, no depth attachment, in the camera's viewport.
fn draw(world: &World, view: ViewQuery<(&ExtractedCamera, &ExtractedView, &ViewTarget, Option<&MainPassResolutionOverride>)>, phases: Res<ViewSortedRenderPhases<Pass3d>>, mut ctx: RenderContext) {
    let entity = view.entity();
    let (camera, extracted, target, over) = view.into_inner();
    let Some(phase) = phases.get(&extracted.retained_view_entity) else { return };
    if phase.items.is_empty() {
        return;
    }
    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor { label: Some("raea mesh pass"), color_attachments: &[Some(target.get_color_attachment())], depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None });
    if let Some(v) = Viewport::from_viewport_and_override(camera.viewport.as_ref(), over) {
        pass.set_camera_viewport(&v);
    }
    if let Err(err) = phase.render(&mut pass, world, entity) {
        error!("MeshPass: drawing the pass: {err:?}");
    }
}
