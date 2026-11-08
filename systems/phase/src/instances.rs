//! A mesh drawn many times in one draw by your shader (docs/control.feature FC2d), as Bevy's
//! shader_advanced/custom_shader_instancing draws its cubes: a Mesh3d entity with ShaderInstances is drawn in each
//! view's Transparent3d phase by Bevy's mesh pipeline with its vertex and fragment shader swapped for your file as
//! written, and a second vertex buffer of your words stepped per instance (laid out as its `attributes` say: locations
//! after the mesh's position, normal and uv at 0 to 2), once per instance of `stride` bytes.

use bevy::core_pipeline::core_3d::{Transparent3d, TransparentSortingInfo3d};
use bevy::ecs::system::lifetimeless::{Read, SRes};
use bevy::ecs::system::SystemParamItem;
use bevy::mesh::{MeshVertexBufferLayoutRef, VertexBufferLayout};
use bevy::pbr::{self, MeshInputUniform, MeshPipeline, MeshPipelineKey, MeshUniform, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup, SetMeshViewBindingArrayBindGroup, ViewKeyCache};
use bevy::prelude::*;
use bevy::render::batching::gpu_preprocessing::BatchedInstanceBuffers;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::mesh::allocator::MeshAllocator;
use bevy::render::mesh::{RenderMesh, RenderMeshBufferInfo};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases};
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::sync_world::MainEntity;
use bevy::render::view::ExtractedView;
use bevy::render::{Render, RenderApp, RenderSystems};

use crate::PhaseAttribute;

/// A mesh's instances: your shader file (as written), and each instance's `stride` bytes of 32-bit words, read as its
/// `attributes` say.
#[derive(Component, Clone, Debug, Default, Reflect, ExtractComponent)]
#[extract_app(RenderApp)]
#[reflect(Component, Default)]
#[type_path = "raea_sys_phase"]
pub struct ShaderInstances {
    pub shader: String,
    pub stride: u64,
    pub attributes: Vec<PhaseAttribute>,
    pub words: Vec<f32>,
    /// its shader file, registered as written (raea-sys-shader), and the id register gave it
    #[reflect(ignore)]
    pub registered: Option<(String, u128)>,
}

pub fn install(app: &mut App) {
    app.register_type::<ShaderInstances>();
    app.add_systems(PostUpdate, crate::register::<ShaderInstances>);
    app.add_plugins(ExtractComponentPlugin::<ShaderInstances>::default());
    let Some(r) = app.get_sub_app_mut(RenderApp) else { return };
    r.add_render_command::<Transparent3d, DrawInstances>()
        .init_resource::<SpecializedMeshPipelines<InstancesPipeline>>()
        .add_systems(Render, (queue.in_set(RenderSystems::QueueMeshes), prepare.in_set(RenderSystems::PrepareResources)));
}

/// The instances' buffer on the GPU, and the words it was made from.
#[derive(Component)]
pub struct InstanceBuffer {
    buffer: Buffer,
    count: u32,
    made: Vec<f32>,
}

fn prepare(mut commands: Commands, device: Res<RenderDevice>, items: Query<(Entity, &ShaderInstances, Option<&InstanceBuffer>)>) {
    for (e, s, had) in &items {
        if s.stride == 0 || had.is_some_and(|b| b.made == s.words) {
            continue;
        }
        let buffer = device.create_buffer_with_data(&BufferInitDescriptor { label: Some("raea_shader_instances"), contents: bytemuck::cast_slice(&s.words), usage: BufferUsages::VERTEX | BufferUsages::COPY_DST });
        let count = (s.words.len() as u64 * 4 / s.stride) as u32;
        commands.entity(e).insert(InstanceBuffer { buffer, count, made: s.words.clone() });
    }
}

#[derive(Resource)]
pub struct InstancesPipeline {
    pub(crate) mesh: MeshPipeline,
}

/// A variant: the view and mesh's key, the shader, and the instance buffer's layout.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct InstancesKey {
    mesh: MeshPipelineKey,
    shader: u128,
    stride: u64,
    attributes: Vec<(String, u64, u32)>,
}

impl SpecializedMeshPipeline for InstancesPipeline {
    type Key = InstancesKey;

    fn specialize(&self, key: InstancesKey, layout: &MeshVertexBufferLayoutRef) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut d = self.mesh.specialize(key.mesh, layout)?;
        let shader = Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(key.shader), core::marker::PhantomData);
        let attributes = key.attributes.iter().map(|(f, offset, location)| VertexAttribute { format: crate::draw::vertex_format(f).unwrap_or(VertexFormat::Float32x4), offset: *offset, shader_location: *location }).collect();
        d.vertex.shader = shader.clone();
        d.vertex.buffers.push(VertexBufferLayout { array_stride: key.stride, step_mode: VertexStepMode::Instance, attributes });
        if let Some(f) = d.fragment.as_mut() {
            f.shader = shader;
        }
        Ok(d)
    }
}

type DrawInstances = (SetItemPipeline, SetMeshViewBindGroup<0>, SetMeshViewBindingArrayBindGroup<1>, SetMeshBindGroup<2>, DrawMeshInstanced);

/// Each mesh with ShaderInstances (and a shader registered) queued in every view's Transparent3d phase.
#[allow(clippy::too_many_arguments)]
fn queue(
    draws: Res<DrawFunctions<Transparent3d>>,
    pipeline: Res<InstancesPipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<InstancesPipeline>>,
    cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    instances: Res<RenderMeshInstances>,
    batched: Option<Res<BatchedInstanceBuffers<MeshUniform, MeshInputUniform>>>,
    items: Query<(Entity, &MainEntity, &ShaderInstances)>,
    mut phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    views: Query<&ExtractedView>,
    keys: Res<ViewKeyCache>,
) {
    let draw = draws.read().id::<DrawInstances>();
    for view in &views {
        let Some(phase) = phases.get_mut(&view.retained_view_entity) else { continue };
        let Some(&view_key) = keys.get(&view.retained_view_entity) else { continue };
        for (entity, main, s) in &items {
            let (Some(shader), Some(instance)) = (s.registered.as_ref().map(|r| r.1), instances.render_mesh_queue_data(*main)) else { continue };
            let Some(mesh) = meshes.get(instance.mesh_asset_id()) else { continue };
            if crate::draw::unknown_format(&s.attributes) {
                error!("ShaderInstances: an attribute's format isn't Bevy's VertexFormat (Float32x4, Uint32, ...)");
                continue;
            }
            let key = InstancesKey {
                mesh: view_key | MeshPipelineKey::from_primitive_topology_and_strip_index(mesh.primitive_topology(), mesh.index_format()),
                shader,
                stride: s.stride,
                attributes: s.attributes.iter().map(|a| (a.format.clone(), a.offset, a.location)).collect(),
            };
            let id = match pipelines.specialize(&cache, &pipeline, key, &mesh.layout) {
                Ok(id) => id,
                Err(err) => {
                    error!("ShaderInstances: {err}");
                    continue;
                }
            };
            let centre = pbr::get_mesh_instance_world_from_local(*main, instance.current_uniform_index, &instances, batched.as_deref()).transform_point3(mesh.aabb_center);
            phase.add_retained(Transparent3d {
                sorting_info: TransparentSortingInfo3d::Sorted { mesh_center: centre, depth_bias: 0.0 },
                entity: (entity, *main),
                pipeline: id,
                draw_function: draw,
                distance: 0.0,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: true,
            });
        }
    }
}

/// The mesh's vertex buffer and the instances' buffer bound, and the mesh drawn once per instance.
pub struct DrawMeshInstanced;

impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type Param = (SRes<RenderAssets<RenderMesh>>, SRes<RenderMeshInstances>, SRes<MeshAllocator>);
    type ViewQuery = ();
    type ItemQuery = Read<InstanceBuffer>;

    fn render<'w>(item: &P, _: (), buffer: Option<&'w InstanceBuffer>, (meshes, instances, allocator): SystemParamItem<'w, '_, Self::Param>, pass: &mut TrackedRenderPass<'w>) -> RenderCommandResult {
        let allocator = allocator.into_inner();
        let Some(instance) = instances.render_mesh_queue_data(item.main_entity()) else { return RenderCommandResult::Skip };
        let Some(gpu) = meshes.into_inner().get(instance.mesh_asset_id()) else { return RenderCommandResult::Skip };
        let (Some(buffer), Some(vertices)) = (buffer, allocator.mesh_vertex_slice(&instance.mesh_asset_id())) else { return RenderCommandResult::Skip };
        pass.set_vertex_buffer(0, vertices.buffer.slice(..));
        pass.set_vertex_buffer(1, buffer.buffer.slice(..));
        match &gpu.buffer_info {
            RenderMeshBufferInfo::Indexed { index_format, count } => {
                let Some(indices) = allocator.mesh_index_slice(&instance.mesh_asset_id()) else { return RenderCommandResult::Skip };
                pass.set_index_buffer(indices.buffer.slice(..), *index_format);
                pass.draw_indexed(indices.range.start..indices.range.start + count, vertices.range.start as i32, 0..buffer.count);
            }
            RenderMeshBufferInfo::NonIndexed => pass.draw(vertices.range, 0..buffer.count),
        }
        RenderCommandResult::Success
    }
}
