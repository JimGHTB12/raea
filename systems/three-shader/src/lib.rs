//! three.js's shaders drawn by Bevy (docs/r3f.feature R3F11): a ShaderMaterial's or RawShaderMaterial's GLSL, which
//! naga translates (glsl.rs), and a node material's WGSL, which three's own TSL builder writes in the SDK. Either is
//! a vertex and a fragment source meeting module.rs's contract; this crate's Material runs them with three's built-ins
//! (the model, view and projection matrices, the camera, the mesh's attributes and its own), the material's uniforms
//! in 62 vec4 slots (changed frame by frame by `three:uniforms` component rows, `patch`), up to four textures, and
//! three's time.
//!
//! What three puts on screen is what shows: a ShaderMaterial's colour is the value its canvas gets (`display`), so the
//! fragment entry takes it back through the camera's tone mapping (tone.wesl); a node material's is scene-referred
//! (`linear`) and is left for Bevy's to map, as three's renderer maps it.
//!
//! Only the main pass runs a three shader: its prepass is off (a vertex shader may move the mesh where the standard
//! prepass wouldn't), and its shadow is the mesh's own shape, as three draws a ShaderMaterial's shadow with its depth
//! material.

use bevy::prelude::*;
use bevy::mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexFormat};
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::render::render_resource::{AsBindGroup, CompareFunction, Face, RenderPipelineDescriptor, SpecializedMeshPipelineError};
use bevy::shader::{Shader, ShaderRef};
use serde_json::Value;
use std::hash::{Hash, Hasher};

pub mod glsl;
pub mod scan;
pub mod linear;
pub mod module;
mod row;
pub use row::{fitted, of_row};

use module::{Ty, Uniform, ATTRIBUTES, DISPLAY_SLOT, SLOTS, TEXTURES, TIME_SLOT};

/// A mesh's own attributes for a three shader (a0..a3), four floats each.
pub const ATTRIBUTE: [MeshVertexAttribute; ATTRIBUTES] = [
    MeshVertexAttribute::new("RaeaThreeA0", 0x7e3a_5100, VertexFormat::Float32x4),
    MeshVertexAttribute::new("RaeaThreeA1", 0x7e3a_5101, VertexFormat::Float32x4),
    MeshVertexAttribute::new("RaeaThreeA2", 0x7e3a_5102, VertexFormat::Float32x4),
    MeshVertexAttribute::new("RaeaThreeA3", 0x7e3a_5103, VertexFormat::Float32x4),
];

pub fn install(app: &mut App) {
    app.add_plugins(MaterialPlugin::<ThreeShader>::default());
    app.add_systems(PostUpdate, follow_tonemapping);
    app.add_systems(PreUpdate, linear::sync);
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
#[bind_group_data(ThreeKey)]
pub struct ThreeShader {
    #[uniform(0)]
    slots: [Vec4; SLOTS],
    #[texture(1)]
    #[sampler(2)]
    tex0: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    tex1: Option<Handle<Image>>,
    #[texture(5)]
    #[sampler(6)]
    tex2: Option<Handle<Image>>,
    #[texture(7)]
    #[sampler(8)]
    tex3: Option<Handle<Image>>,
    /// the camera's tone mapping LUT, for a `display` colour taken back through it (tone.wesl)
    #[texture(9, dimension = "3d")]
    #[sampler(10)]
    lut: Option<Handle<Image>>,
    vertex: u128,
    fragment: u128,
    attributes: u8,
    alpha: AlphaMode,
    cull: Option<Face>,
    depth_write: bool,
    depth_test: bool,
    /// whether its colour is what three's canvas shows (taken back through Bevy's tone mapping) or scene light
    display: bool,
    layout: Vec<Uniform>,
}

/// What picks a three shader's pipeline.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ThreeKey {
    vertex: u128,
    fragment: u128,
    attributes: u8,
    cull: Option<Face>,
    depth_write: bool,
    depth_test: bool,
}

impl From<&ThreeShader> for ThreeKey {
    fn from(m: &ThreeShader) -> Self {
        ThreeKey { vertex: m.vertex, fragment: m.fragment, attributes: m.attributes, cull: m.cull, depth_write: m.depth_write, depth_test: m.depth_test }
    }
}

fn handle(id: u128) -> Handle<Shader> {
    Handle::Uuid(bevy::asset::uuid::Uuid::from_u128(id), core::marker::PhantomData)
}

impl Material for ThreeShader {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha
    }
    fn enable_prepass() -> bool {
        false
    }
    fn specialize(_pipeline: &MaterialPipeline, descriptor: &mut RenderPipelineDescriptor, layout: &MeshVertexBufferLayoutRef, key: MaterialPipelineKey<Self>) -> Result<(), SpecializedMeshPipelineError> {
        let k = key.bind_group_data;
        let mut attrs = vec![Mesh::ATTRIBUTE_POSITION.at_shader_location(0), Mesh::ATTRIBUTE_NORMAL.at_shader_location(1), Mesh::ATTRIBUTE_UV_0.at_shader_location(2), Mesh::ATTRIBUTE_COLOR.at_shader_location(3)];
        for (i, a) in ATTRIBUTE.iter().take(k.attributes as usize).enumerate() {
            attrs.push(a.at_shader_location(4 + i as u32));
        }
        descriptor.vertex.buffers = vec![layout.0.get_layout(&attrs)?];
        descriptor.vertex.shader = handle(k.vertex);
        if let Some(f) = descriptor.fragment.as_mut() {
            f.shader = handle(k.fragment);
        }
        descriptor.primitive.cull_mode = k.cull;
        if let Some(d) = descriptor.depth_stencil.as_mut() {
            d.depth_write_enabled = Some(k.depth_write);
            if !k.depth_test {
                d.depth_compare = Some(CompareFunction::Always);
            }
        }
        Ok(())
    }
}

/// `module` added to Assets<Shader> under the uuid of its hash (salted with `kind`); returns the uuid.
fn register(world: &mut World, module: String, kind: &str) -> u128 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    module.hash(&mut h);
    let hi = h.finish();
    kind.hash(&mut h);
    let id = ((hi as u128) << 64) | h.finish() as u128;
    let aid = AssetId::<Shader>::Uuid { uuid: bevy::asset::uuid::Uuid::from_u128(id) };
    let mut shaders = world.resource_mut::<Assets<Shader>>();
    if shaders.get(aid).is_none() {
        let _ = shaders.insert(aid, Shader::from_wesl(module, format!("embedded://raea_sys_three_shader/{kind}{id:032x}.wesl")));
    }
    id
}

/// The two modules of a material, from its stages' WGSL: the prelude in front of each, the entry point after.
pub fn modules(vertex: &str, fragment: &str, layout: &[Uniform], textures: Option<&[String]>, attributes: usize) -> (String, String) {
    let pre = module::prelude(layout, textures, attributes);
    (format!("{pre}{vertex}\n{}", module::vertex_entry(attributes)), format!("{pre}{fragment}\n{}", module::fragment_entry()))
}

/// A three shader's spec, as the SDK's ThreeShaderMaterial gives it:
///   glsl: {vertex, fragment, raw?, vertexColors?}   a ShaderMaterial's (includes expanded, defines in front)
///   wgsl: {vertex, fragment, uniforms: [{name, type, array?}], attributes?: [name]}   the contract's, written already
///   uniforms: {name: number | [numbers]}   each one's value (a matrix column-major, an array's elements in turn)
///   textures: {name: picture | {src, colorSpace: "srgb" | "linear", sampler}}   "linear": sampled raw, as three
///                                         samples a NoColorSpace texture
///   output: "display" | "linear"          what its colour is (display for a ShaderMaterial, linear for TSL)
///   transparent, blending ("normal" | "additive" | "add" | "multiply" | "none"), side ("front" | "back" | "double"), depthWrite,
///   depthTest, alphaTest, time: [t, delta, frame]
/// Returns the material and the names of the mesh's own attributes it reads (a0.. in order).
pub fn build(world: &mut World, v: &Value, mut image: impl FnMut(&mut World, &Value) -> Result<Handle<Image>, String>) -> Result<(ThreeShader, Vec<String>), String> {
    let o = v.as_object().ok_or_else(|| format!("a three shader is {{glsl | wgsl, uniforms, textures, ...}}, got {v}"))?;
    // the textures, in the order given
    let mut tex: Vec<(String, Handle<Image>)> = Vec::new();
    if let Some(t) = o.get("textures") {
        for (k, x) in t.as_object().ok_or("a three shader's textures are {name: picture}")? {
            if tex.len() == TEXTURES {
                return Err(format!("a three shader binds at most {TEXTURES} textures; {k} is one more"));
            }
            let who = format!("three shader texture {k}");
            // {src, colorSpace: "linear"}: three samples it raw (NoColorSpace), so it's bound as its raw numbers
            let raw = x.get("colorSpace").and_then(Value::as_str) == Some("linear");
            let h = if raw {
                let pic = x.get("src").cloned().unwrap_or(Value::Null);
                let src = image(world, &pic).map_err(|e| format!("texture {k}: {e}"))?;
                let h = linear::raw(world, src);
                if let Some(s) = x.get("sampler") {
                    raea_sys_sampler::PendingSamplers::give(world, h.clone(), raea_sys_sampler::Sampling::of(s).map_err(|e| format!("texture {k}: {e}"))?, who);
                }
                h
            } else {
                raea_sys_sampler::texture(world, x, &who, &mut image).map_err(|e| format!("texture {k}: {e}"))?
            };
            tex.push((k.clone(), h));
        }
    }
    let names: Vec<String> = tex.iter().map(|t| t.0.clone()).collect();
    let (vsrc, fsrc, list, bound, attrs, own_textures) = if let Some(g) = o.get("glsl") {
        let s = |k: &str| g.get(k).and_then(Value::as_str).ok_or_else(|| format!("a three shader's glsl needs {k}"));
        let flag = |k: &str| g.get(k) == Some(&Value::Bool(true));
        let src = glsl::Source { vertex: s("vertex")?, fragment: s("fragment")?, raw: flag("raw"), vertex_colors: flag("vertexColors"), points: flag("points") };
        let t = glsl::translate(&src, &names)?;
        (t.vertex, t.fragment, t.uniforms, t.samplers, t.attributes.into_iter().map(|a| a.0).collect::<Vec<_>>(), false)
    } else if let Some(w) = o.get("wgsl") {
        let s = |k: &str| w.get(k).and_then(Value::as_str).map(str::to_string).ok_or_else(|| format!("a three shader's wgsl needs {k}"));
        let mut list = Vec::new();
        for u in w.get("uniforms").and_then(Value::as_array).into_iter().flatten() {
            let name = u.get("name").and_then(Value::as_str).ok_or("a wgsl uniform needs its name")?;
            let ty = u.get("type").and_then(Value::as_str).and_then(Ty::parse).ok_or_else(|| format!("uniform {name}: its type is f32, i32, u32, bool, a vecN or a matNxN"))?;
            list.push((name.to_string(), ty, u.get("array").and_then(Value::as_u64).map(|n| n as u32)));
        }
        let attrs: Vec<String> = w.get("attributes").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).map(str::to_string).collect();
        if attrs.len() > ATTRIBUTES {
            return Err(format!("a three shader reads at most {ATTRIBUTES} attributes of a mesh's own, got {}", attrs.len()));
        }
        (s("vertex")?, s("fragment")?, list, names.clone(), attrs, true)
    } else {
        return Err("a three shader gives glsl {vertex, fragment} or wgsl {vertex, fragment, uniforms}".into());
    };
    let layout = module::layout(&list)?;
    // three's textures with flipY count v from the picture's bottom; Bevy's images from their top
    let flip: Vec<String> = o.get("flipY").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).filter_map(|n| bound.iter().position(|b| b == n)).map(|k| if own_textures { format!("tex_{}", bound[k]) } else { format!("rgt{k}x") }).collect();
    let (vsrc, fsrc) = (module::flip_samples(&vsrc, &flip), module::flip_samples(&fsrc, &flip));
    let (vm, fm) = modules(&vsrc, &fsrc, &layout, own_textures.then_some(bound.as_slice()), attrs.len());
    let vertex = register(world, vm, "v");
    let fragment = register(world, fm, "f");
    let mut slots = [[0f32; 4]; SLOTS];
    if let Some(u) = o.get("uniforms").and_then(Value::as_object) {
        module::pack(&layout, u, &mut slots)?;
    }
    if let Some(t) = o.get("time").and_then(Value::as_array) {
        for (i, x) in t.iter().take(3).enumerate() {
            slots[TIME_SLOT][i] = x.as_f64().unwrap_or(0.0) as f32;
        }
    }
    let display = o.get("output").and_then(Value::as_str).unwrap_or("display") == "display";
    // tone.wesl reads which it is: a display colour is taken back through the camera's tone mapping, a linear one
    // mapped forward when the camera maps in its materials' shaders
    slots[DISPLAY_SLOT][1] = if display { 1.0 } else { 0.0 };
    // the textures bound in the order the shader binds them (bound), each found by name
    let by = |n: &str| tex.iter().find(|t| t.0 == n).map(|t| t.1.clone());
    let mut t = bound.iter().map(|n| by(n));
    let transparent = o.get("transparent").and_then(Value::as_bool).unwrap_or(false);
    let alpha = match (transparent, o.get("blending").and_then(Value::as_str).unwrap_or("normal")) {
        (_, "none") => AlphaMode::Opaque,
        (_, "additive" | "add") => AlphaMode::Add,
        (_, "multiply") => AlphaMode::Multiply,
        (true, _) => AlphaMode::Blend,
        (false, _) => match o.get("alphaTest").and_then(Value::as_f64).filter(|a| *a > 0.0) {
            Some(a) => AlphaMode::Mask(a as f32),
            None => AlphaMode::Opaque,
        },
    };
    let cull = match o.get("side").and_then(Value::as_str).unwrap_or("front") {
        "double" => None,
        "back" => Some(Face::Front),
        _ => Some(Face::Back),
    };
    let m = ThreeShader {
        slots: slots.map(Vec4::from_array),
        tex0: t.next().flatten(),
        tex1: t.next().flatten(),
        tex2: t.next().flatten(),
        tex3: t.next().flatten(),
        lut: None,
        vertex,
        fragment,
        attributes: attrs.len() as u8,
        alpha,
        cull,
        depth_write: o.get("depthWrite").and_then(Value::as_bool).unwrap_or(!transparent),
        depth_test: o.get("depthTest").and_then(Value::as_bool).unwrap_or(true),
        display,
        layout,
    };
    Ok((m, attrs))
}

/// A `three:uniforms` component row: the frame's uniform values ({uniforms: {name: value}, time: [t, delta, frame]})
/// onto the part's three shader.
pub fn patch(world: &mut World, e: Entity, v: &Value) -> Result<(), String> {
    let Some(h) = world.get::<MeshMaterial3d<ThreeShader>>(e).map(|m| m.0.clone()) else { return Err("three:uniforms: the part has no three shader".into()) };
    let mut mats = world.resource_mut::<Assets<ThreeShader>>();
    let Some(mut m) = mats.get_mut(&h) else { return Err("three:uniforms: its material isn't loaded".into()) };
    let mut slots = m.slots.map(|s| s.to_array());
    if let Some(u) = v.get("uniforms").and_then(Value::as_object) {
        module::pack(&m.layout, u, &mut slots)?;
    }
    if let Some(t) = v.get("time").and_then(Value::as_array) {
        for (i, x) in t.iter().take(3).enumerate() {
            slots[TIME_SLOT][i] = x.as_f64().unwrap_or(0.0) as f32;
        }
    }
    m.slots = slots.map(Vec4::from_array);
    Ok(())
}

/// The mesh a three shader reads: a colour on every vertex (white where it has none), texture coordinates (zero where
/// none) and its own attributes (`attributes`, from a buffer mesh's JSON: {name: {size, values}}, per vertex), laid
/// out per triangle corner as buffer_tris lays a buffer mesh's positions.
pub fn fit_mesh(mesh: &mut Mesh, buffer: Option<&str>, own: &[String]) -> Result<(), String> {
    let n = mesh.count_vertices();
    if mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_none() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1f32; 4]; n]);
    }
    if mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_none() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0f32; 2]; n]);
    }
    if mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_none() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0f32, 0.0, 1.0]; n]);
    }
    if own.is_empty() {
        return Ok(());
    }
    let v: Value = match buffer {
        Some(b) => serde_json::from_str(b).map_err(|e| format!("a buffer mesh isn't JSON: {e}"))?,
        None => return Err(format!("a three shader reads the mesh's own attributes ({}): give the mesh as a BufferMesh with them", own.join(", "))),
    };
    let verts = v.get("positions").and_then(Value::as_array).map(|p| p.len() / 3).unwrap_or(0);
    let idx = raea_sys_material::mesh::buffer_corners(&v, verts);
    for (k, name) in own.iter().enumerate() {
        let a = v.get("attributes").and_then(|a| a.get(name)).ok_or_else(|| format!("the shader reads attribute {name}, which the mesh doesn't give"))?;
        let size = a.get("size").and_then(Value::as_u64).unwrap_or(1).clamp(1, 4) as usize;
        let vals: Vec<f32> = a.get("values").and_then(Value::as_array).ok_or_else(|| format!("attribute {name}: {{size, values}}"))?.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect();
        if vals.len() < verts * size {
            return Err(format!("attribute {name}: {size} numbers for each of {verts} vertices, got {}", vals.len()));
        }
        let per: Vec<[f32; 4]> = idx.iter().map(|&i| {
            let mut out = [0f32; 4];
            out[..size].copy_from_slice(&vals[i * size..i * size + size]);
            out
        }).collect();
        if per.len() != n {
            return Err(format!("attribute {name}: {} corners, the mesh has {n}", per.len()));
        }
        mesh.insert_attribute(ATTRIBUTE[k].clone(), per);
    }
    Ok(())
}

/// Each three shader follows the scene camera's tone mapping, whichever it is this frame: a `display` one's colour is
/// taken back through it, a `linear` one's mapped by it where the camera maps in its materials' shaders.
fn follow_tonemapping(cams: Query<(&Camera, Option<&bevy::core_pipeline::tonemapping::Tonemapping>), With<Camera3d>>, luts: Option<Res<bevy::core_pipeline::tonemapping::TonemappingLuts>>, mut mats: ResMut<Assets<ThreeShader>>) {
    use bevy::core_pipeline::tonemapping::Tonemapping as T;
    let tm = cams.iter().filter(|(c, _)| c.is_active).min_by_key(|(c, _)| c.order).and_then(|(_, t)| t.copied()).unwrap_or(T::None);
    let (method, lut) = match tm {
        T::None | T::Linear => (1.0, None),
        T::Reinhard => (2.0, None),
        T::ReinhardLuminance => (3.0, None),
        T::AcesFitted => (4.0, None),
        T::AgX => (5.0, luts.as_ref().map(|l| l.agx.clone())),
        T::SomewhatBoringDisplayTransform => (6.0, None),
        T::TonyMcMapface => (7.0, luts.as_ref().map(|l| l.tony_mc_mapface.clone())),
        T::BlenderFilmic => (8.0, luts.as_ref().map(|l| l.blender_filmic.clone())),
        T::KhronosPbrNeutral => (9.0, None),
    };
    let ids: Vec<AssetId<ThreeShader>> = mats.iter().filter(|(_, m)| m.slots[DISPLAY_SLOT].x != 1.0 + method || m.lut != lut).map(|(id, _)| id).collect();
    for id in ids {
        if let Some(mut m) = mats.get_mut(id) {
            m.slots[DISPLAY_SLOT].x = 1.0 + method;
            m.lut = lut.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn modules_wrap_the_contract() {
        let l = super::module::layout(&[("tint".into(), super::Ty::Vec(3), None)]).unwrap();
        let (v, f) = super::modules("struct ThreeVaryings { @builtin(position) clip: vec4<f32>, };\nfn three_vertex(v: ThreeVertex) -> ThreeVaryings { var o: ThreeVaryings; o.clip = three_projection() * three_view() * three_model(v.instance) * vec4(v.position, 1.0); return o; }", "struct ThreeVaryings { @builtin(position) clip: vec4<f32>, };\nfn three_fragment(in: ThreeVaryings, front: bool) -> vec4<f32> { return vec4(u_tint(), 1.0); }", &l, Some(&[]), 0);
        assert!(v.contains("@vertex\nfn vertex(m: ThreeMeshIn) -> ThreeVaryings"));
        assert!(f.contains("return three_untone(three_fragment(in, front));"));
        assert!(f.contains("@group(constants::MATERIAL_BIND_GROUP) @binding(9) var three_lut: texture_3d<f32>;"));
    }
}
