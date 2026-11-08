//! A compute as written (docs/control.feature FC2d): a shader file compiled as it is, its `@group(0)` bindings its
//! own (storage textures and uniforms, as Bevy's compute_shader_game_of_life declares them, and storage buffers: slab.rs), the entry point and which
//! image each storage texture binds given frame by frame (a <Compute file>'s `entry` and `bind`), as an example's
//! render-graph node picks a pipeline and a bind group each frame.

use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::binding_types::{storage_buffer_sized, texture_storage_2d, uniform_buffer_sized};
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::GpuImage;
use serde_json::Value;

pub use crate::slab::{Bound, Ranged};
use crate::slab::bound;

/// One `@group(0) @binding(N) var...` of the source.
#[derive(Clone, Debug, PartialEq)]
pub enum Decl {
    /// `var NAME: texture_storage_2d<FORMAT, ACCESS>`
    Storage { binding: u32, name: String, format: TextureFormat, access: StorageTextureAccess },
    /// `var<uniform> NAME: TYPE`
    Uniform { binding: u32, name: String, ty: String },
    /// `var<storage(, read | read_write)> NAME: TYPE`
    Buffer { binding: u32, name: String, ty: String, read_only: bool },
}

/// A compute as written: its declarations, its images by key, its uniforms' bytes by name, and this frame's entry
/// point and bindings (none: nothing dispatched this frame).
#[derive(Clone, Debug)]
pub struct Written {
    pub decls: Vec<Decl>,
    pub images: BTreeMap<String, Handle<Image>>,
    pub uniforms: BTreeMap<String, Vec<u8>>,
    pub entry: Option<String>,
    /// each storage texture's image key, by its name
    pub bind: BTreeMap<String, String>,
    /// its row's id: each frame's `entry` and `bind` columns are read from it (follow)
    pub id: String,
    /// the columns last followed
    pub seen: (String, String),
    /// the source's `@compute` entry points
    pub entries: Vec<String>,
    /// each storage buffer's binding, by its name
    pub buffers: BTreeMap<String, Bound>,
    /// its uniforms' values, packed again when its row's keyed `uniforms` column changes (follow_uniforms), and each
    /// frame when a field is a mesh's range in its slab (`ranged`)
    pub packer: Ranged,
    pub ranged: bool,
    /// the `uniforms` column last followed
    pub given: String,
}

fn format(f: &str) -> Option<TextureFormat> {
    Some(match f {
        "rgba32float" => TextureFormat::Rgba32Float,
        "rgba16float" => TextureFormat::Rgba16Float,
        "rgba8unorm" => TextureFormat::Rgba8Unorm,
        "r32float" => TextureFormat::R32Float,
        "r32uint" => TextureFormat::R32Uint,
        "rgba32uint" => TextureFormat::Rgba32Uint,
        _ => return None,
    })
}

/// The source's `@group(0)` declarations, in order; a binding of another kind is refused.
pub fn decls(src: &str) -> Result<Vec<Decl>, String> {
    let mut out = Vec::new();
    for raea_sys_shader::bindings::Binding { binding, space, name, ty } in raea_sys_shader::bindings::group0(src)? {
        let space = space.as_str();
        out.push(match (space, ty.strip_prefix("texture_storage_2d<").and_then(|t| t.strip_suffix('>'))) {
            ("uniform", _) => Decl::Uniform { binding, name, ty },
            (s, _) if s.split(',').next().map(str::trim) == Some("storage") => Decl::Buffer { binding, name, ty, read_only: !s.contains("read_write") },
            ("", Some(args)) => {
                let (f, a) = args.split_once(',').map(|(f, a)| (f.trim(), a.trim())).ok_or_else(|| format!("{name}: texture_storage_2d<FORMAT, ACCESS>"))?;
                let format = format(f).ok_or_else(|| format!("{name}: storage format {f} (rgba32float, rgba16float, rgba8unorm, r32float, r32uint, rgba32uint)"))?;
                let access = match a {
                    "read" => StorageTextureAccess::ReadOnly,
                    "write" => StorageTextureAccess::WriteOnly,
                    "read_write" => StorageTextureAccess::ReadWrite,
                    _ => return Err(format!("{name}: access {a} (read, write, read_write)")),
                };
                Decl::Storage { binding, name, format, access }
            }
            _ => return Err(format!("{name}: a compute as written binds texture_storage_2d, var<uniform> and var<storage>, got var<{space}> {ty}")),
        });
    }
    Ok(out)
}

/// This frame's entry point and bindings from a spec ({entry, bind}), checked against the declarations and images.
pub fn frame(w: &mut Written, o: &serde_json::Map<String, Value>) -> Result<(), String> {
    w.entry = match o.get("entry") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) if w.entries.contains(s) => Some(s.clone()),
        Some(Value::String(s)) => return Err(format!("entry {s}: the source's entry points are {:?}", w.entries)),
        Some(x) => return Err(format!("a compute's entry is an entry point's name or null, got {x}")),
    };
    w.bind.clear();
    for (k, x) in o.get("bind").and_then(Value::as_object).into_iter().flatten() {
        let key = x.as_str().filter(|s| w.images.contains_key(*s)).ok_or_else(|| format!("bind {k}: one of its images ({:?}), got {x}", w.images.keys().collect::<Vec<_>>()))?;
        if !w.decls.iter().any(|d| matches!(d, Decl::Storage { name, .. } if name == k)) {
            return Err(format!("bind {k}: the source declares no storage texture {k}"));
        }
        w.bind.insert(k.clone(), key.to_string());
    }
    if w.entry.is_some() {
        if let Some(Decl::Storage { name, .. }) = w.decls.iter().find(|d| matches!(d, Decl::Storage { name, .. } if !w.bind.contains_key(name))) {
            return Err(format!("this frame's entry binds every storage texture: {name} has no image"));
        }
    }
    Ok(())
}

/// This frame's `entry` ("": none) and `bind` (JSON, {NAME: KEY}) columns onto it, when they've changed.
pub fn follow(w: &mut Written, entry: &str, bind: &str) -> Result<(), String> {
    if w.seen.0 == entry && w.seen.1 == bind {
        return Ok(());
    }
    w.seen = (entry.to_string(), bind.to_string());
    let bind: Value = if bind.is_empty() { Value::Null } else { serde_json::from_str(bind).map_err(|e| format!("bind: {e}"))? };
    let mut o = serde_json::Map::from_iter([("bind".to_string(), bind)]);
    if !entry.is_empty() {
        o.insert("entry".into(), entry.into());
    }
    frame(w, &o)
}

/// This frame's `uniforms` column (JSON, {NAME: value}) packed into its uniforms, when it has changed ("": as built).
pub fn follow_uniforms(w: &mut Written, uniforms: &str) -> Result<(), String> {
    if uniforms.is_empty() || w.given == uniforms {
        return Ok(());
    }
    w.given = uniforms.to_string();
    w.packer.given = serde_json::from_str(uniforms).map_err(|e| format!("uniforms: {e}"))?;
    if !w.ranged {
        w.uniforms = w.packer.pack(&|_, _| Some(0))?.0;
    }
    Ok(())
}

/// Every entry and binding its row's columns take over the run (raea-sys-frame's keyed_all), checked at load; that a
/// frame's binding binds every storage texture its entry needs is checked frame by frame (follow).
pub fn check(w: &mut Written, entries: &[String], binds: &[String]) -> Result<(), String> {
    if let Some(e) = entries.iter().find(|e| !e.is_empty() && !w.entries.contains(e)) {
        return Err(format!("entry {e}: the source's entry points are {:?}", w.entries));
    }
    for b in binds {
        follow(w, "", b)?;
    }
    w.seen = Default::default();
    Ok(())
}

/// The `@compute` functions' names.
fn entries(src: &str) -> Vec<String> {
    src.match_indices("@compute").filter_map(|(i, _)| {
        let rest = &src[i..];
        let name = rest[rest.find("fn ")? + 3..].trim_start();
        Some(name[..name.find(|c: char| !(c.is_alphanumeric() || c == '_'))?].to_string())
    }).collect()
}

/// A compute as written from its spec: {file, workgroups, images: {KEY: "render:NAME:WxH" | {src, sampler}}, uniforms: {NAME: value},
/// defs}; its module registered as written. Its entry and bindings are its row's columns, followed each frame.
pub fn build(world: &mut World, o: &serde_json::Map<String, Value>, id: &str, image: &mut impl FnMut(&mut World, &Value) -> Result<Handle<Image>, String>) -> Result<(u128, Written), String> {
    let path = o.get("file").and_then(Value::as_str).ok_or("a compute as written names its file")?;
    let src = std::fs::read_to_string(path).map_err(|e| format!("a compute's file {path}: {e}"))?;
    let decls = decls(&src).map_err(|e| format!("{path}: {e}"))?;
    let defs = raea_sys_material::shader_material::defs(o, "a compute")?;
    let formats: Vec<TextureFormat> = decls.iter().filter_map(|d| if let Decl::Storage { format, .. } = d { Some(*format) } else { None }).collect();
    if formats.windows(2).any(|f| f[0] != f[1]) {
        return Err(format!("{path}: its storage textures share one format (each image may bind any of them)"));
    }
    let mut images = BTreeMap::new();
    for (k, x) in o.get("images").and_then(Value::as_object).into_iter().flatten() {
        // "render:NAME:WxH", or {src, sampler}: the sampler it is drawn with (Bevy's ImagePlugin::default_nearest, say)
        let (src, sampler) = match x {
            Value::Object(m) => (m.get("src").unwrap_or(&Value::Null), m.get("sampler").map(raea_sys_sampler::Sampling::of).transpose().map_err(|e| format!("image {k}: {e}"))?),
            _ => (x, None),
        };
        let h = image(world, src).map_err(|e| format!("image {k}: {e}"))?;
        if let Some(mut img) = world.resource_mut::<Assets<Image>>().get_mut(&h) {
            if let Some(f) = formats.first() {
                img.texture_descriptor.format = *f;
                // its zeroed pixels resized to the format (a render image is made rgba8)
                let px = (img.width() * img.height()) as usize * f.block_copy_size(None).unwrap_or(16) as usize;
                img.data = img.data.take().map(|_| vec![0; px]);
            }
            img.texture_descriptor.usage |= TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
            if let Some(s) = sampler {
                img.sampler = bevy::image::ImageSampler::Descriptor(s.over(raea_sys_sampler::descriptor_of(&img.sampler)));
            }
        }
        images.insert(k.clone(), h);
    }
    let buffers = bound(&src, &defs, &decls, o.get("buffers")).map_err(|e| format!("{path}: {e}"))?;
    let declared = raea_sys_shader::bindings::uniforms(&src)?;
    let given = o.get("uniforms").and_then(Value::as_object).cloned().unwrap_or_default();
    let ranged = Ranged::new(&src, &defs, declared, given);
    // packed with each slab range 0, to check them; a run's own ranges are packed frame by frame (lib.rs dispatch)
    let (uniforms, has_ranges) = ranged.pack(&|b, _| matches!(buffers.get(b), Some(Bound::Mesh { .. })).then_some(0)).map_err(|e| format!("{path}: {e}"))?;
    let (packer, ranged) = (ranged, has_ranges);
    let entries = entries(&src);
    let shader = raea_sys_shader::register(world, src, &defs, "raea compute as written", "raea_sys_compute");
    Ok((shader, Written { decls, images, uniforms, entry: None, bind: BTreeMap::new(), id: id.to_string(), seen: Default::default(), entries, buffers, packer, ranged, given: String::new() }))
}

/// The bind group layout its declarations make, in the compute stage.
pub fn layout(decls: &[Decl]) -> Vec<BindGroupLayoutEntry> {
    decls.iter().map(|d| match d {
        Decl::Storage { binding, format, access, .. } => texture_storage_2d(*format, *access).build(*binding, ShaderStages::COMPUTE),
        Decl::Uniform { binding, .. } => uniform_buffer_sized(false, None).build(*binding, ShaderStages::COMPUTE),
        Decl::Buffer { binding, read_only, .. } => storage_buffer_sized(*read_only, None).build(*binding, ShaderStages::COMPUTE),
    }).collect()
}

/// This frame's bind group: each storage texture's bound image, each uniform's bytes (`uniforms`: its own, or packed
/// with this frame's slab ranges), each storage buffer's buffer (None while an image or a buffer isn't on the GPU yet).
pub fn group(device: &RenderDevice, layout: &BindGroupLayout, w: &Written, images: &RenderAssets<GpuImage>, bufs: &BTreeMap<String, Buffer>, uniforms: &BTreeMap<String, Vec<u8>>) -> Option<BindGroup> {
    let mut buffers: Vec<(u32, Buffer)> = Vec::new();
    for d in &w.decls {
        match d {
            Decl::Uniform { binding, name, .. } => buffers.push((*binding, device.create_buffer_with_data(&BufferInitDescriptor { label: Some("raea_compute_written"), contents: uniforms.get(name)?, usage: BufferUsages::UNIFORM }))),
            Decl::Buffer { binding, name, .. } => buffers.push((*binding, bufs.get(name)?.clone())),
            Decl::Storage { .. } => {}
        }
    }
    let mut views = Vec::new();
    for d in &w.decls {
        if let Decl::Storage { binding, name, .. } = d {
            views.push((*binding, &images.get(w.images.get(w.bind.get(name)?)?.id())?.texture_view));
        }
    }
    let entries = raea_sys_shader::bindings::entries(&buffers, views);
    Some(device.create_bind_group("raea_compute_written", layout, &entries))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_files_own_bindings_are_read_and_a_frame_binds_its_images() {
        let src = "@group(0) @binding(0) var input: texture_storage_2d<rgba32float, read>;\n\n@group(0) @binding(1) var output: texture_storage_2d<rgba32float, write>;\n\n@group(0) @binding(2) var<uniform> config: Cfg;\n// @group(0) @binding(3) var x: f32;\n";
        let d = decls(src).unwrap();
        assert_eq!(d.len(), 3);
        assert_eq!(d[0], Decl::Storage { binding: 0, name: "input".into(), format: TextureFormat::Rgba32Float, access: StorageTextureAccess::ReadOnly });
        assert_eq!(d[2], Decl::Uniform { binding: 2, name: "config".into(), ty: "Cfg".into() });
        assert_eq!(decls("@group(0) @binding(1) var<storage, read_write> b: array<f32>;").unwrap()[0], Decl::Buffer { binding: 1, name: "b".into(), ty: "array<f32>".into(), read_only: false });
        assert!(decls("@group(0) @binding(0) var<workgroup> b: f32;").unwrap_err().contains("var<storage>"));
        let mut w = Written { decls: d, images: [("a".to_string(), Handle::default()), ("b".to_string(), Handle::default())].into(), uniforms: BTreeMap::new(), entry: None, bind: BTreeMap::new(), id: "compute0".into(), seen: Default::default(), entries: vec!["init".into(), "update".into()], buffers: BTreeMap::new(), packer: super::Ranged::new("", &[], vec![], Default::default()), ranged: false, given: String::new() };
        assert_eq!(entries("@compute @workgroup_size(8, 8, 1)\nfn init(@builtin(x) i: u32) {}\n@compute @workgroup_size(1)\nfn update_2() {}"), ["init", "update_2"]);
        assert!(check(&mut w, &["init".into(), "".into()], &[r#"{"input": "a"}"#.into()]).is_ok());
        assert!(check(&mut w, &["main".into()], &[]).unwrap_err().contains("entry main"));
        let o = |v: Value| v.as_object().cloned().unwrap();
        frame(&mut w, &o(serde_json::json!({"entry": "update", "bind": {"input": "b", "output": "a"}}))).unwrap();
        assert_eq!((w.entry.as_deref(), w.bind["input"].as_str()), (Some("update"), "b"));
        assert!(frame(&mut w, &o(serde_json::json!({"entry": "init", "bind": {"input": "a"}}))).unwrap_err().contains("output"));
        assert!(frame(&mut w, &o(serde_json::json!({"entry": null}))).is_ok());
        assert!(frame(&mut w, &o(serde_json::json!({"bind": {"input": "c"}}))).unwrap_err().contains("one of its images"));
        follow(&mut w, "init", r#"{"input": "a", "output": "b"}"#).unwrap();
        assert_eq!((w.entry.as_deref(), w.bind["output"].as_str()), (Some("init"), "b"));
        follow(&mut w, "", "").unwrap();
        assert_eq!(w.entry, None);
    }
}
