//! Your own Material (docs/control.feature FC2a, Bevy's shader/shader_material): raea/pbr's `<Material>` is Bevy's
//! `impl Material`, not an extension of StandardMaterial. Its fragment shader (and vertex shader, when it names one) is
//! the author's file compiled as written but for the numbers of its `@group(MATERIAL_BIND_GROUP)` bindings, which go to
//! this one Rust type's fixed slots in the order the source declares them (ui_bind.rs, as raea's UiMaterial does):
//! uniforms of any type up to 256 bytes at 0 to 7, packed in their declared types (layout.rs), textures at 8, 10, 12,
//! 14 and their samplers at 9, 11, 13, 15. A Rust Material's bind group is its type's, so raea's one type takes every
//! author's layout this way. Its alpha mode, depth bias and faces are the material's, as Bevy's methods give them.
//!
//! A file's `import super::NAME` reads NAME.wesl beside it: the file is registered as embedded://raea_sys_shader/
//! d{folder}/m{hash}.wesl and each module it imports from its folder as embedded://raea_sys_shader/d{folder}/NAME.wesl,
//! which is the module path WESL resolves `super::NAME` to.

use std::hash::{Hash, Hasher};

use bevy::asset::uuid_handle;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Face, RenderPipelineDescriptor, SpecializedMeshPipelineError};
use bevy::shader::{Shader, ShaderRef};
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dKey, Material2dPipeline, Material2dPlugin};
use serde_json::Value;

use crate::ui_bind::{rebind_in, Rebound};
use crate::{layout, register, Inputs, OwnSurface, Shaded};

/// The fragment shader every pipeline starts from, swapped in `specialize` for the material's own (its uuid).
const PLACEHOLDER: Handle<Shader> = uuid_handle!("5d1e7a3c-2b9f-4e6d-8a1c-3f7b9d2e6c41");
const STUB: &str = "import bevy_pbr::render::forward_io::VertexOutput;\n@fragment\nfn fragment(in: VertexOutput) -> @location(0) vec4<f32> { return vec4(1.0, 0.0, 1.0, 1.0); }\n";
/// The same for a 2D one (Material2d's), whose vertex output is mesh2d's.
const PLACEHOLDER_2D: Handle<Shader> = uuid_handle!("5d1e7a3c-2b9f-4e6d-8a1c-3f7b9d2e6c42");
const STUB_2D: &str = "import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;\n@fragment\nfn fragment(in: VertexOutput) -> @location(0) vec4<f32> { return vec4(1.0, 0.0, 1.0, 1.0); }\n";

/// How a source may write the material's group: 3 in 3D, 2 in 2D.
const GROUPS: &[&str] = &["constants::MATERIAL_BIND_GROUP", "#{MATERIAL_BIND_GROUP}", "3"];
const GROUPS_2D: &[&str] = &["constants::MATERIAL_BIND_GROUP", "#{MATERIAL_BIND_GROUP}", "2"];

/// One Rust type for every author's Material: eight uniform buffers of 256 bytes, four textures with their samplers,
/// the shaders that read them, and how it blends and culls.
#[derive(Asset, AsBindGroup, TypePath, Clone, Debug)]
#[bind_group_data(OwnKey)]
pub struct OwnMaterial {
    #[uniform(0)]
    u0: [Vec4; 16],
    #[uniform(1)]
    u1: [Vec4; 16],
    #[uniform(2)]
    u2: [Vec4; 16],
    #[uniform(3)]
    u3: [Vec4; 16],
    #[uniform(4)]
    u4: [Vec4; 16],
    #[uniform(5)]
    u5: [Vec4; 16],
    #[uniform(6)]
    u6: [Vec4; 16],
    #[uniform(7)]
    u7: [Vec4; 16],
    #[texture(8)]
    #[sampler(9)]
    t0: Option<Handle<Image>>,
    #[texture(10)]
    #[sampler(11)]
    t1: Option<Handle<Image>>,
    #[texture(12)]
    #[sampler(13)]
    t2: Option<Handle<Image>>,
    #[texture(14)]
    #[sampler(15)]
    t3: Option<Handle<Image>>,
    shader: u128,
    /// the vertex shader's uuid, 0 for Bevy's mesh vertex shader
    vertex: u128,
    alpha: AlphaMode,
    depth_bias: f32,
    /// the faces it culls (the pipeline's primitive.cull_mode); None keeps the pipeline's (3D culls Back, 2D none)
    cull: Option<Option<Face>>,
}

/// What picks a pipeline: the material's shaders and the faces it culls.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct OwnKey {
    shader: u128,
    vertex: u128,
    cull: Option<Option<Face>>,
}

impl From<&OwnMaterial> for OwnKey {
    fn from(m: &OwnMaterial) -> Self {
        OwnKey { shader: m.shader, vertex: m.vertex, cull: m.cull }
    }
}

/// Material's and Material2d's specialize, one body for both (each trait takes its own pipeline and key types): the
/// material's shaders put where that trait's placeholder is (`swap`).
macro_rules! specialize {
    ($pipeline:ty, $key:ident, $placeholder:expr) => {
        fn specialize(_: &$pipeline, descriptor: &mut RenderPipelineDescriptor, _: &MeshVertexBufferLayoutRef, key: $key<Self>) -> Result<(), SpecializedMeshPipelineError> {
            swap(descriptor, &$placeholder, key.bind_group_data)
        }
    };
}

impl Material for OwnMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(PLACEHOLDER)
    }
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha
    }
    fn depth_bias(&self) -> f32 {
        self.depth_bias
    }
    specialize!(MaterialPipeline, MaterialPipelineKey, PLACEHOLDER);
}

impl Material2d for OwnMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(PLACEHOLDER_2D)
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        match self.alpha {
            AlphaMode::Opaque => AlphaMode2d::Opaque,
            AlphaMode::Mask(c) => AlphaMode2d::Mask(c),
            _ => AlphaMode2d::Blend,
        }
    }
    fn depth_bias(&self) -> f32 {
        self.depth_bias
    }
    specialize!(Material2dPipeline, Material2dKey, PLACEHOLDER_2D);
}

/// A pipeline given the material's own shaders: its fragment where it is `placeholder`, its vertex shader (if any) in
/// that same pipeline (the main pass's: the prepass and shadow pipelines keep Bevy's), and the faces it culls.
fn swap(descriptor: &mut RenderPipelineDescriptor, placeholder: &Handle<Shader>, k: OwnKey) -> Result<(), SpecializedMeshPipelineError> {
    let Some(f) = descriptor.fragment.as_mut().filter(|f| f.shader.id() == placeholder.id()) else { return Ok(()) };
    let shader = |id: u128| Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(id), core::marker::PhantomData);
    f.shader = shader(k.shader);
    if k.vertex != 0 {
        descriptor.vertex.shader = shader(k.vertex);
    }
    if let Some(c) = k.cull {
        descriptor.primitive.cull_mode = c;
    }
    Ok(())
}

pub fn install(app: &mut App) {
    let _ = app.world_mut().resource_mut::<Assets<Shader>>().insert(&PLACEHOLDER, Shader::from_wesl(STUB, "embedded://raea_sys_shader/own_placeholder.wesl"));
    let _ = app.world_mut().resource_mut::<Assets<Shader>>().insert(&PLACEHOLDER_2D, Shader::from_wesl(STUB_2D, "embedded://raea_sys_shader/own_placeholder_2d.wesl"));
    app.add_plugins((MaterialPlugin::<OwnMaterial>::default(), Material2dPlugin::<OwnMaterial>::default()));
}

/// The material a `<Material>` spec asks for: {fragment, vertex?, uniforms, textures, defs, own: {alpha_mode,
/// depth_bias, cull_mode}}, its shaders registered as written (bindings renumbered, imports beside them); a 2D one's
/// (`two_d`, a Material2d's) binds group 2.
pub fn build(world: &mut World, v: &Value, image: &mut impl FnMut(&mut World, &Value) -> Result<Handle<Image>, String>, two_d: bool) -> Result<OwnMaterial, String> {
    let groups = if two_d { GROUPS_2D } else { GROUPS };
    let what = "a Material";
    let o = v.as_object().ok_or_else(|| format!("{what} is {{fragment, uniforms, textures}}, got {v}"))?;
    let path = o.get("fragment").and_then(Value::as_str).ok_or_else(|| format!("{what}'s fragment is its shader file"))?;
    // the uniforms are packed in the source's own types below, so Inputs reads the rest
    let mut rest = o.clone();
    let given = rest.remove("uniforms").and_then(|u| u.as_object().cloned()).unwrap_or_default();
    let ins = Inputs::read(world, &rest, what, image)?;
    let Rebound { src, uniforms, textures, .. } = rebind_in(&ins.src, groups, what).map_err(|e| format!("{path}: {e}"))?;
    let [u0, u1, u2, u3, u4, u5, u6, u7] = layout::slots(&src, &uniforms, &given, &ins.defs).map_err(|e| format!("{path}: {e}"))?;
    if let Some((k, _)) = ins.textures.iter().find(|(k, _)| !textures.contains(k)) {
        return Err(format!("{path} has no texture {k}"));
    }
    let mut t: [Option<Handle<Image>>; 4] = Default::default();
    for (i, name) in textures.iter().enumerate() {
        t[i] = Some(ins.textures.iter().find(|(k, _)| k == name).map(|x| x.1.clone()).ok_or_else(|| format!("{path}'s texture {name} has no picture"))?);
    }
    let krate = folder(world, path, &src)?;
    let shader = register(world, src, &ins.defs, "raea own material", &krate);
    let vertex = match o.get("vertex").and_then(Value::as_str) {
        None => 0,
        Some(p) => {
            let s = std::fs::read_to_string(p).map_err(|e| format!("{what}'s vertex {p}: {e}"))?;
            let s = rebind_in(&s, groups, what).map_err(|e| format!("{p}: {e}"))?.src;
            let krate = folder(world, p, &s)?;
            register(world, s, &ins.defs, "raea own material vertex", &krate)
        }
    };
    let own = o.get("own").and_then(Value::as_object).cloned().unwrap_or_default();
    let alpha = alpha_mode(own.get("alpha_mode"))?;
    let depth_bias = own.get("depth_bias").and_then(Value::as_f64).unwrap_or(0.0) as f32;
    let cull = match own.get("cull_mode") {
        None => None,
        Some(Value::Null) => Some(None),
        Some(v) if v == "Front" => Some(Some(Face::Front)),
        Some(v) if v == "Back" => Some(Some(Face::Back)),
        Some(v) => return Err(format!("a Material's cull_mode is \"Front\", \"Back\" or null (none), got {v}")),
    };
    let [t0, t1, t2, t3] = t;
    Ok(OwnMaterial { u0, u1, u2, u3, u4, u5, u6, u7, t0, t1, t2, t3, shader, vertex, alpha, depth_bias, cull })
}

/// Bevy's AlphaMode as the SDK writes it: "Opaque", "Blend", {"Mask": cutoff}, "Premultiplied", "AlphaToCoverage",
/// "Add" or "Multiply".
fn alpha_mode(v: Option<&Value>) -> Result<AlphaMode, String> {
    Ok(match v {
        None => AlphaMode::Opaque,
        Some(Value::String(s)) => match s.as_str() {
            "Opaque" => AlphaMode::Opaque,
            "Blend" => AlphaMode::Blend,
            "Premultiplied" => AlphaMode::Premultiplied,
            "AlphaToCoverage" => AlphaMode::AlphaToCoverage,
            "Add" => AlphaMode::Add,
            "Multiply" => AlphaMode::Multiply,
            _ => return Err(format!("a Material's alpha_mode is Opaque, Blend, Mask, Premultiplied, AlphaToCoverage, Add or Multiply, got {s:?}")),
        },
        Some(v) => match v.get("Mask").and_then(Value::as_f64) {
            Some(c) => AlphaMode::Mask(c as f32),
            None => return Err(format!("a Material's alpha_mode is a name or {{\"Mask\": cutoff}}, got {v}")),
        },
    })
}

/// A picture file decoded now as an Image (sRGB, the default sampler, as asset_server.load reads a colour texture), so
/// the first frame that draws it has it.
pub fn picture(world: &mut World, path: &str) -> Result<Handle<Image>, String> {
    use bevy::asset::RenderAssetUsages;
    use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let ext = std::path::Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("png");
    let image = Image::from_buffer(&bytes, ImageType::Extension(ext), CompressedImageFormats::NONE, true, ImageSampler::Default, RenderAssetUsages::default()).map_err(|e| format!("{path}: {e}"))?;
    Ok(world.resource_mut::<Assets<Image>>().add(image))
}

/// The embedded folder a file's module goes in (raea_sys_shader/d{hash of its folder}), with every module it imports
/// by `super::NAME` registered there from its own folder, and theirs in turn.
fn folder(world: &mut World, path: &str, src: &str) -> Result<String, String> {
    let dir = std::path::Path::new(path).parent().unwrap_or(std::path::Path::new("."));
    let mut h = std::collections::hash_map::DefaultHasher::new();
    dir.hash(&mut h);
    let krate = format!("raea_sys_shader/d{:016x}", h.finish());
    let mut todo: Vec<String> = siblings(src);
    let mut done: Vec<String> = Vec::new();
    while let Some(name) = todo.pop() {
        if done.contains(&name) {
            continue;
        }
        let file = ["wesl", "wgsl"].iter().map(|x| dir.join(format!("{name}.{x}"))).find(|f| f.exists()).ok_or_else(|| format!("{path} imports super::{name}, and {} has no {name}.wesl", dir.display()))?;
        let s = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        todo.extend(siblings(&s));
        let at = format!("embedded://{krate}/{name}.wesl");
        let mut h = std::collections::hash_map::DefaultHasher::new();
        (&at, &s).hash(&mut h);
        let id = AssetId::<Shader>::Uuid { uuid: bevy::asset::uuid::Uuid::from_u64_pair(h.finish(), 0x5d1e_7a3c) };
        let _ = world.resource_mut::<Assets<Shader>>().insert(id, Shader::from_wesl(s, at));
        done.push(name);
    }
    Ok(krate)
}

/// The modules a source imports from its own folder: each `super::NAME` of its `import` statements.
fn siblings(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in src.lines().map(str::trim).filter(|l| l.starts_with("import ")) {
        for (i, _) in line.match_indices("super::") {
            // super::NAME::item, or super::{NAME::item, OTHER::item}
            let after = &line[i + 7..];
            let parts: Vec<&str> = match after.strip_prefix('{') {
                Some(r) => r.split('}').next().unwrap_or("").split(',').collect(),
                None => vec![after],
            };
            for p in parts {
                let name: String = p.trim().chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
                if !name.is_empty() && !out.contains(&name) {
                    out.push(name);
                }
            }
        }
    }
    out
}

/// A part given its own Material: its MeshMaterial3d<StandardMaterial> swapped for the OwnMaterial, the standard one
/// kept aside as `Shaded` (so the rows' sync finds it) and the scene's <SurfaceShader> kept off it.
pub fn put(world: &mut World, e: Entity, m: OwnMaterial) -> bool {
    let Some(std) = world.get::<MeshMaterial3d<StandardMaterial>>(e).map(|m| m.0.clone()) else { return false };
    let h = world.resource_mut::<Assets<OwnMaterial>>().add(m);
    world.entity_mut(e).remove::<MeshMaterial3d<StandardMaterial>>().insert((MeshMaterial3d(h), Shaded { std }, OwnSurface));
    true
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_files_own_folder_imports_are_found() {
        let src = "import bevy_pbr::forward_io::VertexOutput;\nimport super::custom_material_import::COLOR_MULTIPLIER;\nimport super::{a::X, b_2::Y};\nfn f() { super::not_an_import(); }\n";
        assert_eq!(super::siblings(src), vec!["custom_material_import".to_string(), "a".into(), "b_2".into()]);
        assert!(super::alpha_mode(Some(&serde_json::json!("Blend"))).unwrap() == bevy::prelude::AlphaMode::Blend);
        assert!(super::alpha_mode(Some(&serde_json::json!({"Mask": 0.5}))).unwrap() == bevy::prelude::AlphaMode::Mask(0.5));
        assert!(super::alpha_mode(Some(&serde_json::json!("Clear"))).is_err());
    }
}
