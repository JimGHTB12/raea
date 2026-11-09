//! An own Material with a binding_array<texture_2d<f32>> (docs/control.feature FC2d, Bevy's
//! shader_advanced/texture_binding_array): OwnMaterial's bind group but its uniform buffers (wgpu has no binding array beside one), with the array at ui_bind's ARRAY slot, its
//! pictures (own_images.rs) in order and Bevy's fallback image in the slots past them, as Bevy's example binds its
//! BindlessMaterial. Bevy's derive can't own a texture array, so this type builds its bind group directly. Its
//! pipelines are OwnMaterial's: the same key, shaders and specialize.

use std::num::NonZero;

use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    AsBindGroup, AsBindGroupError, BindGroupBuilder, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingResources, BindingType, BufferBindingType, PipelineCache, PreparedBindGroup, RenderPipelineDescriptor, ShaderStages,
    SpecializedMeshPipelineError, TextureSampleType, TextureViewDimension,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::storage::GpuShaderBuffer;
use bevy::render::material_bind_groups::FallbackBuffer;
use bevy::render::texture::{FallbackImage, GpuImage};
use bevy::shader::ShaderRef;

use crate::own::{OwnKey, OwnMaterial, PLACEHOLDER};
use crate::ui_bind::{ARRAY, ARRAY_LEN, OWN_BASE};

/// An OwnMaterial whose source binds a texture array, its pictures `own.array`.
#[derive(Asset, TypePath, Clone, Debug)]
pub struct OwnArrayMaterial {
    pub own: OwnMaterial,
}

const BINDING: u32 = (OWN_BASE + ARRAY) as u32;

impl AsBindGroup for OwnArrayMaterial {
    type Data = OwnKey;
    type Param = (<OwnMaterial as AsBindGroup>::Param, SRes<RenderAssets<GpuImage>>, SRes<FallbackImage>);

    fn label() -> &'static str {
        "raea_own_array_material"
    }

    fn bind_group_data(&self) -> OwnKey {
        self.own.bind_group_data()
    }

    fn as_bind_group(&self, layout: &BindGroupLayoutDescriptor, device: &RenderDevice, cache: &PipelineCache, fallback_buffer: &FallbackBuffer, buffers: &RenderAssets<GpuShaderBuffer>, (inner, images, fallback): &mut SystemParamItem<'_, '_, Self::Param>) -> Result<PreparedBindGroup, AsBindGroupError> {
        let layout = cache.get_bind_group_layout(layout);
        let mut b = BindGroupBuilder::default();
        self.own.build_bind_group(&layout, device, inner, true, &mut b)?;
        let mut views = vec![&*fallback.d2.texture_view; ARRAY_LEN];
        for (i, h) in self.own.array.iter().take(ARRAY_LEN).enumerate() {
            views[i] = &*images.get(h).ok_or(AsBindGroupError::RetryNextUpdate)?.texture_view;
        }
        let keep: Vec<u32> = Self::bind_group_layout_entries(device, true).iter().map(|e| e.binding).collect();
        let mut entries: Vec<BindGroupEntry> = b.binding_resources.iter().filter(|(i, _)| keep.contains(i)).map(|(i, r)| BindGroupEntry { binding: *i, resource: r.get_binding(fallback_buffer, buffers) }).collect();
        entries.push(BindGroupEntry { binding: BINDING, resource: BindingResource::TextureViewArray(&views) });
        Ok(PreparedBindGroup { bindings: BindingResources(vec![]), bind_group: device.create_bind_group(Self::label(), &layout, &entries) })
    }

    fn build_bind_group(&self, _: &BindGroupLayout, _: &RenderDevice, _: &mut SystemParamItem<'_, '_, Self::Param>, _: bool, _: &mut BindGroupBuilder) -> Result<(), AsBindGroupError> {
        // an array of views can't be owned, so `as_bind_group` builds it (as Bevy's texture_binding_array does)
        Err(AsBindGroupError::CreateBindGroupDirectly)
    }

    fn bind_group_layout_entries(device: &RenderDevice, _: bool) -> Vec<BindGroupLayoutEntry> {
        // wgpu refuses a binding array beside a uniform buffer, so this one binds none (own_images.rs refuses uniforms)
        let mut e = OwnMaterial::bind_group_layout_entries(device, true);
        e.retain(|e| !matches!(e.ty, BindingType::Buffer { ty: BufferBindingType::Uniform, .. }));
        let ty = BindingType::Texture { sample_type: TextureSampleType::Float { filterable: true }, view_dimension: TextureViewDimension::D2, multisampled: false };
        e.push(BindGroupLayoutEntry { binding: BINDING, visibility: ShaderStages::FRAGMENT, ty, count: NonZero::new(ARRAY_LEN as u32) });
        e
    }
}

impl Material for OwnArrayMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(PLACEHOLDER)
    }
    fn alpha_mode(&self) -> AlphaMode {
        self.own.alpha.unwrap_or(AlphaMode::Opaque)
    }
    fn depth_bias(&self) -> f32 {
        <OwnMaterial as Material>::depth_bias(&self.own)
    }
    fn specialize(_: &MaterialPipeline, descriptor: &mut RenderPipelineDescriptor, layout: &MeshVertexBufferLayoutRef, key: MaterialPipelineKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        crate::own::swap(descriptor, layout, &PLACEHOLDER, key.bind_group_data)
    }
}

/// The array Material wrapping a part's OwnMaterial, made once per OwnMaterial; None when it binds no array.
pub(crate) fn wrap(world: &mut World, h: &Handle<OwnMaterial>) -> Option<Handle<OwnArrayMaterial>> {
    #[derive(Resource, Default)]
    struct Wrapped(std::collections::HashMap<AssetId<OwnMaterial>, Handle<OwnArrayMaterial>>);
    if let Some(a) = world.get_resource_or_init::<Wrapped>().0.get(&h.id()).cloned() {
        return Some(a);
    }
    let own = world.resource::<Assets<OwnMaterial>>().get(h).filter(|m| !m.array.is_empty())?.clone();
    let a = world.resource_mut::<Assets<OwnArrayMaterial>>().add(OwnArrayMaterial { own });
    world.get_resource_or_init::<Wrapped>().0.insert(h.id(), a.clone());
    Some(a)
}
