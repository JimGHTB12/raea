//! A PhaseItem in the render world: its buffers made (prepare, again when its words or indices change), queued in
//! each view's Opaque3d phase where it's visible (queue, as custom_phase_item's queue_custom_phase_item), and drawn
//! by its pipeline (DrawPhaseItem: its vertex and index buffers, every index once).

use std::collections::HashMap;

use bevy::core_pipeline::core_3d::{Opaque3d, Opaque3dBatchSetKey, Opaque3dBinKey, CORE_3D_DEPTH_FORMAT};
use bevy::ecs::query::ROQueryItem;
use bevy::ecs::system::lifetimeless::Read;
use bevy::ecs::system::SystemParamItem;
use bevy::mesh::VertexBufferLayout;
use bevy::prelude::*;
use bevy::render::camera::{DirtySpecializations, PendingQueues};
use bevy::render::mesh::allocator::MeshSlabs;
use bevy::render::render_phase::{AddRenderCommand, BinnedRenderPhaseType, DrawFunctions, InputUniformIndex, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewBinnedRenderPhases};
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::sync_world::MainEntityHashSet;
use bevy::render::view::{ExtractedView, RenderVisibleEntities};
use bevy::render::{Render, RenderApp, RenderSystems};

use crate::PhaseItem;

/// A phase item's buffers on the GPU, and what they were made from.
#[derive(Component)]
pub struct Gpu {
    vertices: Buffer,
    indices: Buffer,
    count: u32,
    made: (Vec<f32>, Vec<u32>),
}

/// The pipelines made, by everything that decides one.
#[derive(Resource, Default)]
struct Pipelines(HashMap<String, CachedRenderPipelineId>);

#[derive(Resource, Default, Deref, DerefMut)]
struct Pending(PendingQueues);

type DrawCommands = (SetItemPipeline, DrawPhaseItem);

pub fn install(app: &mut App) {
    let Some(r) = app.get_sub_app_mut(RenderApp) else { return };
    r.init_resource::<Pipelines>()
        .init_resource::<Pending>()
        .add_render_command::<Opaque3d, DrawCommands>()
        .add_systems(Render, (prepare.in_set(RenderSystems::PrepareResources), queue.in_set(RenderSystems::Queue)));
}

fn prepare(mut commands: Commands, device: Res<RenderDevice>, items: Query<(Entity, &PhaseItem, Option<&Gpu>)>) {
    for (e, p, had) in &items {
        if had.is_some_and(|g| g.made.0 == p.words && g.made.1 == p.indices) || p.indices.is_empty() {
            continue;
        }
        let buffer = |bytes: &[u8], usage| device.create_buffer_with_data(&BufferInitDescriptor { label: Some("raea_phase_item"), contents: bytes, usage });
        let gpu = Gpu {
            vertices: buffer(bytemuck::cast_slice(&p.words), BufferUsages::VERTEX),
            indices: buffer(bytemuck::cast_slice(&p.indices), BufferUsages::INDEX),
            count: p.indices.len() as u32,
            made: (p.words.clone(), p.indices.clone()),
        };
        commands.entity(e).insert(gpu);
    }
}

pub(crate) fn vertex_format(f: &str) -> Option<VertexFormat> {
    use VertexFormat::*;
    Some(match f {
        "Float32" => Float32,
        "Float32x2" => Float32x2,
        "Float32x3" => Float32x3,
        "Float32x4" => Float32x4,
        "Uint32" => Uint32,
        "Uint32x2" => Uint32x2,
        "Uint32x3" => Uint32x3,
        "Uint32x4" => Uint32x4,
        "Sint32" => Sint32,
        "Sint32x2" => Sint32x2,
        "Sint32x3" => Sint32x3,
        "Sint32x4" => Sint32x4,
        "Unorm8x4" => Unorm8x4,
        _ => return None,
    })
}

/// Whether any of `attributes` has a format that isn't one of Bevy's VertexFormats here.
pub(crate) fn unknown_format(attributes: &[crate::PhaseAttribute]) -> bool {
    for a in attributes {
        if vertex_format(&a.format).is_none() {
            return true;
        }
    }
    false
}

fn compare(c: &str) -> Option<CompareFunction> {
    use CompareFunction::*;
    Some(match c {
        "Never" => Never,
        "Less" => Less,
        "Equal" => Equal,
        "LessEqual" => LessEqual,
        "Greater" => Greater,
        "NotEqual" => NotEqual,
        "GreaterEqual" => GreaterEqual,
        "Always" => Always,
        _ => return None,
    })
}

/// Its pipeline in a view of `samples` and `format` (an attribute or compare it doesn't know is logged, once a key).
fn pipeline(cache: &PipelineCache, made: &mut Pipelines, p: &PhaseItem, shader: u128, samples: u32, format: TextureFormat) -> Option<CachedRenderPipelineId> {
    let key = format!("{shader}|{}|{}|{}|{:?}|{}|{}|{samples}|{format:?}", p.vertex, p.fragment, p.stride, p.attributes.iter().map(|a| (&a.format, a.offset, a.location)).collect::<Vec<_>>(), p.depth_write, p.depth_compare);
    if let Some(id) = made.0.get(&key) {
        return Some(*id);
    }
    let attributes = p.attributes.iter().map(|a| vertex_format(&a.format).map(|format| VertexAttribute { format, offset: a.offset, shader_location: a.location })).collect::<Option<Vec<_>>>();
    let (Some(attributes), Some(depth_compare)) = (attributes, compare(&p.depth_compare)) else {
        error!("PhaseItem: an attribute's format (Float32x3, Uint32, ...) or its depth_compare (Always, Greater, ...) isn't Bevy's: {key}");
        return None;
    };
    let shader = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(shader), core::marker::PhantomData);
    let id = cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("raea_phase_item".into()),
        vertex: VertexState { shader: shader.clone(), entry_point: Some(p.vertex.clone().into()), buffers: vec![VertexBufferLayout { array_stride: p.stride, step_mode: VertexStepMode::Vertex, attributes }], ..default() },
        fragment: Some(FragmentState { shader, entry_point: Some(p.fragment.clone().into()), targets: vec![Some(ColorTargetState { format, blend: None, write_mask: ColorWrites::ALL })], ..default() }),
        depth_stencil: Some(DepthStencilState { format: CORE_3D_DEPTH_FORMAT, depth_write_enabled: Some(p.depth_write), depth_compare: Some(depth_compare), stencil: default(), bias: default() }),
        multisample: MultisampleState { count: samples, ..default() },
        ..default()
    });
    made.0.insert(key, id);
    Some(id)
}

#[allow(clippy::too_many_arguments)]
fn queue(
    cache: Res<PipelineCache>,
    mut made: ResMut<Pipelines>,
    mut phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    draws: Res<DrawFunctions<Opaque3d>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities, &Msaa)>,
    items: Query<&PhaseItem>,
    dirty: Res<DirtySpecializations>,
    mut pending: ResMut<Pending>,
    mut scratch: Local<MainEntityHashSet>,
) {
    let draw = draws.read().id::<DrawCommands>();
    for (view, visible, msaa) in &views {
        let Some(phase) = phases.get_mut(&view.retained_view_entity) else { continue };
        let Some(vis) = visible.get::<PhaseItem>() else { continue };
        let waiting = pending.prepare_for_new_frame(view.retained_view_entity);
        for &main in dirty.iter_to_dequeue(view.retained_view_entity, vis) {
            phase.remove(main);
        }
        let mut later = Vec::new();
        for (render, main) in dirty.iter_to_queue(view.retained_view_entity, vis, &waiting.prev_frame, &mut scratch) {
            let id = items.get(*render).ok().and_then(|p| Some((p, p.registered.as_ref()?.1))).and_then(|(p, s)| pipeline(&cache, &mut made, p, s, msaa.samples(), view.target_format));
            let Some(id) = id else {
                later.push((*render, *main));
                continue;
            };
            let set = Opaque3dBatchSetKey { draw_function: draw, pipeline: id, material_bind_group_index: None, lightmap_slab: None, slabs: MeshSlabs::default() };
            phase.add(set, Opaque3dBinKey { asset_id: AssetId::<Mesh>::default().untyped() }, (*render, *main), InputUniformIndex::default(), BinnedRenderPhaseType::NonMesh);
        }
        waiting.current_frame.extend(later);
    }
}

/// Its vertex and index buffers bound, and every index drawn once.
pub struct DrawPhaseItem;

impl<P: bevy::render::render_phase::PhaseItem> RenderCommand<P> for DrawPhaseItem {
    type Param = ();
    type ViewQuery = ();
    type ItemQuery = Read<Gpu>;

    fn render<'w>(_: &P, _: ROQueryItem<'w, '_, Self::ViewQuery>, gpu: Option<ROQueryItem<'w, '_, Self::ItemQuery>>, _: SystemParamItem<'w, '_, Self::Param>, pass: &mut TrackedRenderPass<'w>) -> RenderCommandResult {
        let Some(g) = gpu else { return RenderCommandResult::Skip };
        pass.set_vertex_buffer(0, g.vertices.slice(..));
        pass.set_index_buffer(g.indices.slice(..), IndexFormat::Uint32);
        pass.draw_indexed(0..g.count, 0, 0..1);
        RenderCommandResult::Success
    }
}
