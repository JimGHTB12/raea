//! A frame pass as written (docs/control.feature FC2d): a shader file compiled as it is, drawn over the frame with
//! Bevy's FullscreenShader vertex state, as shader_advanced/custom_post_processing's post_process_system draws its
//! own. Its `@group(0)` bindings are its own: each `texture_2d<f32>` the frame as drawn so far (the one `frame`
//! names: the ViewTarget's post_process_write source) or an image, each `sampler` Bevy's SamplerDescriptor::default()
//! (nearest) or the sampler given for it, each `var<uniform>` its value in its declared type's layout.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::binding_types::{sampler, texture_2d, uniform_buffer_sized};
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::GpuImage;
use serde_json::Value;

/// One `@group(0) @binding(N) var...` of the source.
#[derive(Clone, Debug, PartialEq)]
pub enum Decl {
    Texture { binding: u32, name: String },
    Sampler { binding: u32, name: String },
    Uniform { binding: u32, name: String, ty: String },
}

/// A pass as written: its declarations, the texture that reads the frame, its images and samplers by name, its
/// uniforms' bytes by name and its fragment entry point.
#[derive(Clone, Debug)]
pub struct Written {
    pub decls: Vec<Decl>,
    pub frame: String,
    pub images: BTreeMap<String, Handle<Image>>,
    pub samplers: BTreeMap<String, bevy::image::ImageSamplerDescriptor>,
    pub uniforms: BTreeMap<String, Vec<u8>>,
    pub entry: String,
    /// its row's id, whose keyed `uniforms` column (JSON, {NAME: value}) it follows frame by frame, and the last seen
    pub id: String,
    pub seen: String,
    src: String,
    declared: Vec<(String, String)>,
    defs: Vec<bevy::shader::ShaderDefVal>,
}

impl Written {
    /// This frame's uniforms (JSON, {NAME: value}) packed, when they've changed; whether they had.
    pub fn follow(&mut self, uniforms: &str) -> Result<bool, String> {
        if uniforms.is_empty() || self.seen == uniforms {
            return Ok(false);
        }
        self.seen = uniforms.to_string();
        let given: serde_json::Map<String, Value> = serde_json::from_str(uniforms).map_err(|e| format!("uniforms: {e}"))?;
        self.uniforms = pack(&self.src, &self.declared, &given, &self.defs)?;
        Ok(true)
    }
}

/// Each uniform's bytes by name, in its declared type's layout.
fn pack(src: &str, declared: &[(String, String)], given: &serde_json::Map<String, Value>, defs: &[bevy::shader::ShaderDefVal]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let slots = raea_sys_shader::uniform_slots(src, declared, given, defs)?;
    Ok(declared.iter().zip(slots).map(|((n, _), s)| (n.clone(), s.iter().flat_map(|v| v.to_array()).flat_map(f32::to_ne_bytes).collect())).collect())
}

/// The source's `@group(0)` declarations, in order; a binding of another kind is refused.
pub fn decls(src: &str) -> Result<Vec<Decl>, String> {
    let mut out = Vec::new();
    for raea_sys_shader::bindings::Binding { binding, space, name, ty } in raea_sys_shader::bindings::group0(src)? {
        let (uniform, ty) = (space == "uniform", ty.replace(' ', ""));
        out.push(match (uniform, ty.as_str()) {
            (true, _) => Decl::Uniform { binding, name, ty },
            (false, "texture_2d<f32>") => Decl::Texture { binding, name },
            (false, "sampler") => Decl::Sampler { binding, name },
            _ => return Err(format!("{name}: a frame pass as written binds texture_2d<f32>, sampler and var<uniform>, got {ty}")),
        });
    }
    Ok(out)
}

/// A pass as written from its spec: {file, frame, entry, textures: {NAME: image}, samplers: {NAME: sampler}, uniforms:
/// {NAME: value}, defs}; its module registered as written, its id returned with it.
pub fn build(world: &mut World, o: &serde_json::Map<String, Value>, image: &mut impl FnMut(&mut World, &Value) -> Result<Handle<Image>, String>) -> Result<(u128, Written), String> {
    let path = o.get("file").and_then(Value::as_str).ok_or("a frame pass as written names its file")?;
    let src = std::fs::read_to_string(path).map_err(|e| format!("a frame pass's file {path}: {e}"))?;
    let decls = decls(&src).map_err(|e| format!("{path}: {e}"))?;
    let defs = raea_sys_material::shader_material::defs(o, "a frame pass")?;
    let frame = o.get("frame").and_then(Value::as_str).unwrap_or_default().to_string();
    let named = |k: &str| o.get(k).and_then(Value::as_object).cloned().unwrap_or_default();
    let (textures, samplers) = (named("textures"), named("samplers"));
    let mut images = BTreeMap::new();
    let mut sampled = BTreeMap::new();
    for d in &decls {
        match d {
            Decl::Texture { name, .. } if *name == frame => {}
            Decl::Texture { name, .. } => {
                let v = textures.get(name).ok_or_else(|| format!("{path}: its texture {name} is the frame (frame: \"{name}\") or an image (textures: {{ {name}: ... }})"))?;
                images.insert(name.clone(), image(world, v).map_err(|e| format!("texture {name}: {e}"))?);
            }
            Decl::Sampler { name, .. } => {
                let s = samplers.get(name).map(raea_sys_sampler::Sampling::of).transpose().map_err(|e| format!("sampler {name}: {e}"))?;
                sampled.insert(name.clone(), s.map_or_else(Default::default, |s| s.over(Default::default())));
            }
            Decl::Uniform { .. } => {}
        }
    }
    if !frame.is_empty() && !decls.iter().any(|d| matches!(d, Decl::Texture { name, .. } if *name == frame)) {
        return Err(format!("{path}: frame {frame:?} names none of its texture_2d<f32>s"));
    }
    let declared = raea_sys_shader::bindings::uniforms(&src)?;
    let uniforms = pack(&src, &declared, &named("uniforms"), &defs).map_err(|e| format!("{path}: {e}"))?;
    let entry = o.get("entry").and_then(Value::as_str).unwrap_or("fragment").to_string();
    let shader = raea_sys_shader::register(world, src.clone(), &defs, "raea frame pass as written", "raea_sys_framepass");
    Ok((shader, Written { decls, frame, images, samplers: sampled, uniforms, entry, id: String::new(), seen: String::new(), src, declared, defs }))
}

/// The bind group layout its declarations make, in the fragment stage.
pub fn layout(decls: &[Decl]) -> Vec<BindGroupLayoutEntry> {
    decls.iter().map(|d| match d {
        Decl::Texture { binding, .. } => texture_2d(TextureSampleType::Float { filterable: true }).build(*binding, ShaderStages::FRAGMENT),
        Decl::Sampler { binding, .. } => sampler(SamplerBindingType::Filtering).build(*binding, ShaderStages::FRAGMENT),
        Decl::Uniform { binding, .. } => uniform_buffer_sized(false, None).build(*binding, ShaderStages::FRAGMENT),
    }).collect()
}

/// This frame's bind group: the frame (`source`) and its images, its samplers (`sampler` by name) and its uniforms'
/// bytes (None while an image isn't on the GPU yet).
pub fn group(device: &RenderDevice, layout: &BindGroupLayout, w: &Written, source: &TextureView, images: &RenderAssets<GpuImage>, sampler: impl Fn(&str) -> Option<Sampler>) -> Option<BindGroup> {
    let mut buffers = Vec::new();
    let mut samplers = Vec::new();
    let mut views = Vec::new();
    for d in &w.decls {
        match d {
            Decl::Uniform { binding, name, .. } => buffers.push((*binding, device.create_buffer_with_data(&BufferInitDescriptor { label: Some("raea_frame_pass_written"), contents: w.uniforms.get(name)?, usage: BufferUsages::UNIFORM }))),
            Decl::Sampler { binding, name } => samplers.push((*binding, sampler(name)?)),
            Decl::Texture { binding, name } if *name == w.frame => views.push((*binding, source)),
            Decl::Texture { binding, name } => views.push((*binding, &images.get(w.images.get(name)?.id())?.texture_view)),
        }
    }
    let mut entries = raea_sys_shader::bindings::entries(&buffers, views);
    entries.extend(samplers.iter().map(|(b, s)| BindGroupEntry { binding: *b, resource: BindingResource::Sampler(s) }));
    Some(device.create_bind_group("raea_frame_pass_written", layout, &entries))
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_files_own_bindings_are_read() {
        let src = "@group(0) @binding(0) var screen_texture: texture_2d<f32>;\n@group(0) @binding(1) var texture_sampler: sampler;\n@group(0) @binding(2) var<uniform> settings: S;\n";
        let d = super::decls(src).unwrap();
        assert_eq!(d[2], super::Decl::Uniform { binding: 2, name: "settings".into(), ty: "S".into() });
        assert!(matches!(&d[1], super::Decl::Sampler { binding: 1, name } if name == "texture_sampler"));
        assert!(super::decls("@group(0) @binding(0) var t: texture_depth_2d;").unwrap_err().contains("texture_depth_2d"));
    }
}
