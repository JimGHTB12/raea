//! The half-resolution fog's draw: the march at half size, then the upsampled apply over the view.

use super::*;

#[allow(clippy::too_many_arguments)]
pub fn draw(
    view: ViewQuery<(&ViewTarget, &ViewDepthStencilTexture, &ViewPipelines, &ViewVolumes, &HalfTextures, &MeshViewBindGroup, &Msaa)>,
    cache: Res<PipelineCache>,
    pipelines: Option<Res<Pipelines>>,
    uniforms: Res<FogUniforms>,
    images: Res<RenderAssets<GpuImage>>,
    allocator: Res<MeshAllocator>,
    hulls: Res<HullMeshes>,
    meshes: Res<RenderAssets<RenderMesh>>,
    mut ctx: RenderContext,
) {
    let Some(p) = pipelines else { return };
    let (target, depth, ids, volumes, tex, view_group, msaa) = view.into_inner();
    if volumes.0.is_empty() {
        return;
    }
    let (Some(plain), Some(textured), Some(down), Some(comp), Some(binding)) = (
        cache.get_render_pipeline(ids.march[0]),
        cache.get_render_pipeline(ids.march[1]),
        cache.get_render_pipeline(ids.down),
        cache.get_render_pipeline(ids.comp),
        uniforms.0.binding(),
    ) else {
        return;
    };
    let ms = !matches!(msaa, Msaa::Off);
    let device = ctx.render_device().clone();
    ctx.command_encoder().push_debug_group("half_fog");

    // 1. each texel's depth
    let group = device.create_bind_group("half_fog_down", &cache.get_bind_group_layout(&p.down_layouts[ms as usize]), &BindGroupEntries::with_indices(((2, super::bevy_api::depth_binding(depth)),)));
    {
        let mut pass = ctx.command_encoder().begin_render_pass(&RenderPassDescriptor {
            label: Some("half_fog_downsample"),
            color_attachments: &[Some(RenderPassColorAttachment { view: &tex.depth.default_view, depth_slice: None, resolve_target: None, ops: Operations { load: LoadOp::Clear(Default::default()), store: StoreOp::Store } })],
            ..default()
        });
        pass.set_pipeline(down);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
    }

    // 2. the march, volume by volume, into the 1/k texture (cleared to no light, everything showing through)
    let clear = wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    let mut first = true;
    for volume in &volumes.0 {
        let handle = if volume.exterior { &hulls.cube } else { &hulls.plane };
        let (Some(vertices), Some(mesh)) = (allocator.mesh_vertex_slice(&handle.id()), meshes.get(handle)) else { continue };
        let density = volume.density_texture.and_then(|t| images.get(t));
        let mut entries = DynamicBindGroupEntries::sequential((binding.clone(), BindingResource::TextureView(&tex.depth.default_view)));
        if let Some(d) = density {
            entries = entries.extend_sequential((BindingResource::TextureView(&d.texture_view), BindingResource::Sampler(&d.sampler)));
        }
        let group = device.create_bind_group(None, &cache.get_bind_group_layout(&p.march_layouts[density.is_some() as usize]), &entries);
        let load = if first { LoadOp::Clear(clear) } else { LoadOp::Load };
        first = false;
        let mut pass = ctx.command_encoder().begin_render_pass(&RenderPassDescriptor {
            label: Some("half_fog_march"),
            color_attachments: &[Some(RenderPassColorAttachment { view: &tex.fog.default_view, depth_slice: None, resolve_target: None, ops: Operations { load, store: StoreOp::Store } })],
            ..default()
        });
        pass.set_viewport(0.0, 0.0, tex.viewport.x, tex.viewport.y, 0.0, 1.0);
        pass.set_vertex_buffer(0, *vertices.buffer.slice(..));
        pass.set_pipeline(if density.is_some() { textured } else { plain });
        pass.set_bind_group(0, &view_group.main, &view_group.main_offsets);
        pass.set_bind_group(1, &group, &[volume.offset]);
        match &mesh.buffer_info {
            RenderMeshBufferInfo::Indexed { index_format, count } => {
                let Some(indices) = allocator.mesh_index_slice(&handle.id()) else { continue };
                pass.set_index_buffer(*indices.buffer.slice(..), *index_format);
                pass.draw_indexed(indices.range.start..(indices.range.start + count), vertices.range.start as i32, 0..1);
            }
            RenderMeshBufferInfo::NonIndexed => pass.draw(vertices.range.clone(), 0..1),
        }
    }
    if first {
        ctx.command_encoder().pop_debug_group();
        return;
    }

    // 3. over the frame, guided by depth
    let group = device.create_bind_group(
        "half_fog_composite",
        &cache.get_bind_group_layout(&p.comp_layouts[ms as usize]),
        &BindGroupEntries::sequential((&tex.fog.default_view, &tex.depth.default_view, super::bevy_api::depth_binding(depth))),
    );
    {
        let mut pass = ctx.command_encoder().begin_render_pass(&RenderPassDescriptor {
            label: Some("half_fog_composite"),
            color_attachments: &[Some(RenderPassColorAttachment { view: target.main_texture_view(), depth_slice: None, resolve_target: None, ops: Operations { load: LoadOp::Load, store: StoreOp::Store } })],
            ..default()
        });
        pass.set_pipeline(comp);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
    }
    ctx.command_encoder().pop_debug_group();
}
