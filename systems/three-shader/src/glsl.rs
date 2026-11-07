//! A three.js ShaderMaterial's GLSL as a three shader's WGSL (docs/r3f.feature R3F11), by naga's GLSL frontend.
//!
//! three hands its shaders GLSL ES (1.0 or 3.0) with its own built-ins declared in front: the matrices, the camera,
//! `position`, `normal`, `uv`. Here every global a stage declares (a uniform, an attribute, a varying, an output) and
//! every built-in it writes (`gl_Position`, `gl_FragColor`, `gl_FragCoord`, ...) becomes a plain global of the module,
//! so naga's module has no inputs or outputs of its own: `main` is a function like any other. The wrapper module.rs's
//! contract asks for (`three_vertex`, `three_fragment`) sets those globals from the mesh, the uniforms and the varyings,
//! calls `main`, and reads back what it wrote. Samplers are a texture and a sampler bound where the material binds its
//! textures, the name a macro for `sampler2D(texture, sampler)`.
//!
//! The includes (`#include <common>`) are expanded by the SDK from three's ShaderChunk before the source gets here, and
//! a material's `defines` come as `#define` lines in front.

use crate::module::Ty;
use std::collections::BTreeMap;
pub use crate::scan::{scan, Scanned, Var};

/// three's built-in uniforms: what the vertex stage of a ShaderMaterial is given, and the fragment's
const VERTEX_BUILTINS: [(&str, &str); 7] = [
    ("modelMatrix", "mat4"),
    ("modelViewMatrix", "mat4"),
    ("projectionMatrix", "mat4"),
    ("viewMatrix", "mat4"),
    ("normalMatrix", "mat3"),
    ("cameraPosition", "vec3"),
    ("isOrthographic", "bool"),
];
const FRAGMENT_BUILTINS: [&str; 3] = ["viewMatrix", "cameraPosition", "isOrthographic"];
/// attributes the mesh gives at fixed places (module.rs's vertex_entry); any other is one of its own (a0..a3)
const MESH_ATTRIBUTES: [&str; 4] = ["position", "normal", "uv", "color"];

/// the built-ins GLSL writes or reads, each a global here: (GLSL name, global, type)
const GL: [(&str, &str, &str); 9] = [
    ("gl_Position", "rg_position", "vec4"),
    ("gl_PointSize", "rg_pointsize", "float"),
    ("gl_FragColor", "rg_frag", "vec4"),
    ("gl_FragCoord", "rg_fragcoord", "vec4"),
    ("gl_FrontFacing", "rg_front", "bool"),
    ("gl_PointCoord", "rg_pointcoord", "vec2"),
    ("gl_VertexID", "rg_vertexid", "int"),
    ("gl_InstanceID", "rg_instanceid", "int"),
    ("gl_FragDepth", "rg_fragdepth", "float"),
];

/// What a ShaderMaterial gives: its two stages, whether it's a RawShaderMaterial (which declares three's built-ins
/// itself), its `vertexColors`.
pub struct Source<'a> {
    pub vertex: &'a str,
    pub fragment: &'a str,
    pub raw: bool,
    pub vertex_colors: bool,
    /// it draws three.js Points: each point a quad whose corners come as `uv` (0..1), sized by gl_PointSize
    pub points: bool,
}

/// Both stages translated: each the WGSL that goes after module.rs's prelude, the uniforms they declare (laid out by
/// the caller), the samplers (bound in this order) and the mesh's own attributes (a0.. in this order: name, floats).
#[derive(Debug)]
pub struct Translated {
    pub vertex: String,
    pub fragment: String,
    pub uniforms: Vec<(String, Ty, Option<u32>)>,
    pub samplers: Vec<String>,
    pub attributes: Vec<(String, usize)>,
}

fn wgsl_ty(glsl: &str, what: &str) -> Result<Ty, String> {
    Ty::parse(glsl).ok_or_else(|| format!("{what}: its type {glsl} isn't a number, vector or matrix"))
}

/// The GLSL naga reads for one stage: the globals as plain declarations, the samplers bound, the built-ins renamed.
fn stage_glsl(s: &Scanned, vertex: bool, src: &Source, samplers: &[String], extra: &[Var]) -> Result<String, String> {
    // the samplers first: the macros below rename `texture2D`, which is also the GLSL 450 texture type's name
    let mut g = String::from("#version 450\n");
    for v in &s.samplers {
        if v.glsl != "sampler2D" {
            return Err(format!("sampler {}: a {} isn't supported yet (sampler2D is)", v.name, v.glsl));
        }
        let k = samplers.iter().position(|x| *x == v.name).ok_or_else(|| format!("sampler {}: not among the material's textures", v.name))?;
        g += &format!("layout(set = 0, binding = {}) uniform texture2D rgt{k}x;\nlayout(set = 0, binding = {}) uniform sampler rgs{k}x;\n#define {} sampler2D(rgt{k}x, rgs{k}x)\n", 1 + k * 2, 2 + k * 2, v.name);
    }
    g += "#define highp\n#define mediump\n#define lowp\n";
    for (gl, ours, _) in GL {
        g += &format!("#define {gl} {ours}\n");
    }
    g += "#define gl_FragData rg_fragdata\n";
    if vertex {
        g += "#define texture2D(s, c) textureLod(s, c, 0.0)\n#define texture(s, c) textureLod(s, c, 0.0)\n#define textureCube(s, c) textureLod(s, c, 0.0)\n";
    } else {
        g += "#define texture2D texture\n#define textureCube texture\n#define texture2DLodEXT textureLod\n#define textureCubeLodEXT textureLod\n#define texture2DProj textureProj\n#define gl_FragDepthEXT gl_FragDepth\n";
    }
    if src.vertex_colors {
        g += "#define USE_COLOR\n";
    }
    for (gl, ours, ty) in GL {
        let _ = gl;
        g += &format!("{ty} {ours};\n");
    }
    g += "vec4 rg_fragdata[1];\n";
    let declared = |n: &str| s.uniforms.iter().chain(&s.attributes).chain(&s.varyings).chain(&s.outputs).chain(extra).any(|v| v.name == n) || s.samplers.iter().any(|v| v.name == n);
    if !src.raw {
        for (n, ty) in VERTEX_BUILTINS {
            if (vertex || FRAGMENT_BUILTINS.contains(&n)) && !declared(n) {
                g += &format!("{ty} {n};\n");
            }
        }
        if vertex {
            for (n, ty) in [("position", "vec3"), ("normal", "vec3"), ("uv", "vec2")] {
                if !declared(n) {
                    g += &format!("{ty} {n};\n");
                }
            }
            if src.vertex_colors && !declared("color") {
                g += "vec3 color;\n";
            }
        }
    }
    for v in s.uniforms.iter().chain(&s.attributes).chain(&s.varyings).chain(&s.outputs).chain(extra) {
        let arr = v.array.map(|n| format!("[{n}]")).unwrap_or_default();
        g += &format!("{} {}{arr};\n", v.glsl, v.name);
    }
    g += &s.body;
    Ok(g)
}

/// naga's module for one stage's GLSL, `main` an ordinary function `rg_main`, every private global renamed `rg{n}x`;
/// returns its WGSL and the renames (GLSL name -> WGSL name).
fn naga_wgsl(glsl: &str, vertex: bool) -> Result<(String, BTreeMap<String, String>), String> {
    let stage = if vertex { naga::ShaderStage::Vertex } else { naga::ShaderStage::Fragment };
    let mut m = naga::front::glsl::Frontend::default().parse(&naga::front::glsl::Options::from(stage), glsl).map_err(|e| {
        let what = if vertex { "vertex" } else { "fragment" };
        format!("the {what} shader: {}", e.emit_to_string(glsl).lines().take(12).collect::<Vec<_>>().join("\n"))
    })?;
    let mut names = BTreeMap::new();
    for (i, (_, g)) in m.global_variables.iter_mut().enumerate() {
        if g.space == naga::AddressSpace::Private {
            if let Some(n) = &g.name {
                let ours = format!("rg{i}x");
                names.insert(n.clone(), ours.clone());
                g.name = Some(ours);
            }
        }
    }
    m.entry_points.clear();
    for (_, f) in m.functions.iter_mut() {
        if f.name.as_deref() == Some("main") {
            f.name = Some("rg_main".into());
        }
    }
    let info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&m)
        .map_err(|e| format!("the {} shader: {}", if vertex { "vertex" } else { "fragment" }, e.emit_to_string(glsl).lines().take(12).collect::<Vec<_>>().join("\n")))?;
    let mut w = naga::back::wgsl::write_string(&m, &info, naga::back::wgsl::WriterFlags::empty()).map_err(|e| format!("writing WGSL: {e}"))?;
    // the textures go in the material's bind group (module.rs binds slots at 0 and textures from 1)
    w = w.replace("@group(0) @binding(", "@group(constants::MATERIAL_BIND_GROUP) @binding(");
    Ok((w, names))
}

/// The struct both stages read their varyings through: clip position, then each varying the vertex stage writes, in
/// its order (location i).
fn varyings_struct(vs: &[Var], points: bool) -> Result<String, String> {
    let mut m = String::from("struct ThreeVaryings {\n    @builtin(position) clip: vec4<f32>,\n");
    if points {
        m += &format!("    @location({}) corner: vec2<f32>,\n", vs.len());
    }
    for (i, v) in vs.iter().enumerate() {
        let ty = wgsl_ty(&v.glsl, &format!("varying {}", v.name))?;
        if v.array.is_some() {
            return Err(format!("varying {}: an array varying isn't supported yet", v.name));
        }
        let flat = matches!(ty, Ty::I32 | Ty::U32 | Ty::IVec(_) | Ty::UVec(_)) || v.flat;
        if matches!(ty, Ty::Bool | Ty::BVec(_) | Ty::Mat(_)) {
            return Err(format!("varying {}: a {} can't be a varying here", v.name, v.glsl));
        }
        m += &format!("    @location({i}){} v{i}: {},\n", if flat { " @interpolate(flat)" } else { "" }, ty.wgsl());
    }
    Ok(m + "};\n")
}

/// `NAME = value;` for each uniform a stage declares (an array element by element)
fn set_uniforms(uniforms: &[Var], names: &BTreeMap<String, String>) -> String {
    let mut m = String::new();
    for u in uniforms {
        let Some(g) = names.get(&u.name) else { continue };
        match u.array {
            None => m += &format!("    {g} = u_{}();\n", u.name),
            Some(n) => m += &format!("    for (var i = 0u; i < {n}u; i++) {{ {g}[i] = u_{}(i); }}\n", u.name),
        }
    }
    m
}

/// three's built-in uniforms, set where the stage has them
fn set_builtins(names: &BTreeMap<String, String>, model: bool) -> String {
    let mut m = String::new();
    let set = |n: &str, v: &str, m: &mut String| {
        if let Some(g) = names.get(n) {
            *m += &format!("    {g} = {v};\n");
        }
    };
    if model {
        set("modelMatrix", "model", &mut m);
        set("modelViewMatrix", "three_view() * model", &mut m);
        set("normalMatrix", "three_normal_matrix(three_view() * model)", &mut m);
    }
    set("viewMatrix", "three_view()", &mut m);
    set("projectionMatrix", "three_projection()", &mut m);
    set("cameraPosition", "three_camera_position()", &mut m);
    set("isOrthographic", "three_projection()[3][3] == 1.0", &mut m);
    m
}

const NORMAL_MATRIX: &str = "fn three_normal_matrix(mv: mat4x4<f32>) -> mat3x3<f32> {\n    let a = mat3x3<f32>(mv[0].xyz, mv[1].xyz, mv[2].xyz);\n    let c0 = cross(a[1], a[2]);\n    let c1 = cross(a[2], a[0]);\n    let c2 = cross(a[0], a[1]);\n    return mat3x3<f32>(c0, c1, c2) * (1.0 / dot(a[0], c0));\n}\n";

/// A ShaderMaterial's two stages as a three shader's (module.rs's contract).
pub fn translate(src: &Source, textures: &[String]) -> Result<Translated, String> {
    let vs = scan(src.vertex, true)?;
    let fs = scan(src.fragment, false)?;
    // the samplers bound: the material's textures that either stage samples, in the material's order
    let samplers: Vec<String> = textures.iter().filter(|t| vs.samplers.iter().chain(&fs.samplers).any(|s| s.name == **t)).cloned().collect();
    for s in vs.samplers.iter().chain(&fs.samplers) {
        if !samplers.contains(&s.name) {
            return Err(format!("sampler {}: the material gives no texture for it", s.name));
        }
    }
    // a varying the fragment stage reads that the vertex stage doesn't write is declared there too (zero, as three
    // would refuse to link it)
    let missing: Vec<Var> = fs.varyings.iter().filter(|v| !vs.varyings.iter().any(|x| x.name == v.name)).cloned().collect();
    let mut vary = vs.varyings.clone();
    vary.extend(missing.iter().cloned());
    let (vw, vn) = naga_wgsl(&stage_glsl(&vs, true, src, &samplers, &missing)?, true)?;
    let (fw, fn_) = naga_wgsl(&stage_glsl(&fs, false, src, &samplers, &[])?, false)?;
    // the uniforms both stages declare, once each (the same name is the same uniform in three)
    let mut uniforms: Vec<(String, Ty, Option<u32>)> = Vec::new();
    let builtin = |u: &&Var| VERTEX_BUILTINS.iter().any(|(n, _)| *n == u.name);
    for u in vs.uniforms.iter().chain(&fs.uniforms) {
        if builtin(&u) || uniforms.iter().any(|x| x.0 == u.name) {
            continue;
        }
        uniforms.push((u.name.clone(), wgsl_ty(&u.glsl, &format!("uniform {}", u.name))?, u.array));
    }
    let user_uniforms: Vec<Var> = vs.uniforms.iter().chain(&fs.uniforms).filter(|u| !builtin(u)).cloned().collect();
    // the mesh's attributes: the fixed four, and the rest each a vec4 slot of its own
    let mut attributes = Vec::new();
    let mut set_attr = String::new();
    for a in &vs.attributes {
        let ty = wgsl_ty(&a.glsl, &format!("attribute {}", a.name))?;
        let Some(g) = vn.get(&a.name) else { continue };
        let swz = ["", ".x", ".xy", ".xyz", ""][ty.floats().min(4)];
        let from = match MESH_ATTRIBUTES.iter().position(|n| *n == a.name) {
            Some(0) => "v.position".to_string(),
            Some(1) => "v.normal".to_string(),
            Some(2) => "v.uv".to_string(),
            Some(_) => format!("v.color{}", if ty.floats() == 3 { ".xyz" } else { swz }),
            None => {
                if attributes.len() == crate::module::ATTRIBUTES {
                    return Err(format!("attribute {}: a mesh gives a shader at most {} attributes of its own", a.name, crate::module::ATTRIBUTES));
                }
                attributes.push((a.name.clone(), ty.floats()));
                format!("v.a{}{swz}", attributes.len() - 1)
            }
        };
        let cast = match ty {
            Ty::I32 => format!("i32({from})"),
            Ty::IVec(n) => format!("vec{n}<i32>({from})"),
            Ty::U32 => format!("u32({from})"),
            Ty::UVec(n) => format!("vec{n}<u32>({from})"),
            _ => from,
        };
        set_attr += &format!("    {g} = {cast};\n");
    }
    // three's own attributes for a ShaderMaterial (declared in its prefix, not the source)
    if !src.raw {
        for (n, from) in [("position", "v.position"), ("normal", "v.normal"), ("uv", "v.uv"), ("color", "v.color.xyz")] {
            if !vs.attributes.iter().any(|a| a.name == n) && !vs.uniforms.iter().any(|a| a.name == n) {
                if let Some(g) = vn.get(n) {
                    set_attr += &format!("    {g} = {from};\n");
                }
            }
        }
    }
    let vstruct = varyings_struct(&vary, src.points)?;
    // vertex: built-ins, attributes, uniforms, main, the varyings out
    let mut v = String::new();
    v += &vw;
    v += "\n";
    v += &vstruct;
    v += NORMAL_MATRIX;
    v += "\nfn three_vertex(v: ThreeVertex) -> ThreeVaryings {\n    let model = three_model(v.instance);\n";
    v += &set_builtins(&vn, true);
    v += &set_attr;
    v += &set_uniforms(&user_uniforms, &vn);
    if let Some(g) = vn.get("rg_vertexid") {
        v += &format!("    {g} = i32(v.index);\n");
    }
    if let Some(g) = vn.get("rg_pointsize") {
        v += &format!("    {g} = 1.0;\n");
    }
    v += "    rg_main();\n    var o: ThreeVaryings;\n";
    v += &format!("    o.clip = {};\n", vn.get("rg_position").cloned().unwrap_or_else(|| "vec4<f32>(0.0)".into()));
    if src.points {
        // the point's quad: its corner moved gl_PointSize pixels across the target, in clip space
        let size = vn.get("rg_pointsize").cloned().unwrap_or_else(|| "1.0".into());
        v += &format!("    let corner = v.uv * 2.0 - 1.0;\n    o.clip = vec4<f32>(o.clip.xy + corner * {size} / three_viewport().zw * o.clip.w, o.clip.zw);\n    o.corner = corner;\n");
    }
    for (i, x) in vary.iter().enumerate() {
        if let Some(g) = vn.get(&x.name) {
            v += &format!("    o.v{i} = {g};\n");
        }
    }
    v += "    return o;\n}\n";
    // fragment: built-ins, uniforms, varyings in, gl_FragCoord and gl_FrontFacing, main, its colour out
    let mut f = String::new();
    f += &fw;
    f += "\n";
    f += &vstruct;
    f += NORMAL_MATRIX;
    f += "\nfn three_fragment(in: ThreeVaryings, front: bool) -> vec4<f32> {\n";
    f += &set_builtins(&fn_, false);
    f += &set_uniforms(&user_uniforms, &fn_);
    for (i, x) in vary.iter().enumerate() {
        if let Some(g) = fn_.get(&x.name) {
            f += &format!("    {g} = in.v{i};\n");
        }
    }
    if let Some(g) = fn_.get("rg_fragcoord") {
        // GLSL counts y up from the bottom of the target, WGSL down from its top
        f += &format!("    {g} = vec4<f32>(in.clip.x, three_viewport().w - in.clip.y, in.clip.z, in.clip.w);\n");
    }
    if let (Some(g), true) = (fn_.get("rg_pointcoord"), src.points) {
        // gl_PointCoord counts down from the sprite's top
        f += &format!("    {g} = vec2<f32>(in.corner.x * 0.5 + 0.5, 0.5 - in.corner.y * 0.5);\n");
    }
    if let Some(g) = fn_.get("rg_front") {
        f += &format!("    {g} = front;\n");
    }
    f += "    rg_main();\n";
    let colour = match fs.outputs.first() {
        Some(o) => fn_.get(&o.name).cloned(),
        None if src.fragment.contains("gl_FragData") => fn_.get("rg_fragdata").map(|g| format!("{g}[0]")),
        None => fn_.get("rg_frag").cloned(),
    };
    f += &format!("    return {};\n}}\n", colour.unwrap_or_else(|| "vec4<f32>(0.0, 0.0, 0.0, 1.0)".into()));
    Ok(Translated { vertex: v, fragment: f, uniforms, samplers, attributes })
}

#[cfg(test)]
mod tests {
    use super::*;

    const VERT: &str = "precision highp float;\n// three's typical wave\nuniform float uTime;\nvarying vec2 vUv;\nattribute float aRandom;\nvoid main() {\n  vUv = uv;\n  vec3 p = position;\n  p.z += sin(p.x * 4.0 + uTime) * 0.1 * aRandom;\n  gl_Position = projectionMatrix * modelViewMatrix * vec4(p, 1.0);\n}\n";
    const FRAG: &str = "uniform vec3 uColor;\nuniform sampler2D map;\nvarying vec2 vUv;\nvoid main() {\n  vec4 t = texture2D(map, vUv);\n  if (t.a < 0.1) discard;\n  gl_FragColor = vec4(uColor * t.rgb, 1.0);\n}\n";

    #[test]
    fn declarations_come_out_of_the_source() {
        let s = scan(VERT, true).unwrap();
        assert_eq!(s.uniforms, vec![Var { name: "uTime".into(), glsl: "float".into(), array: None, flat: false }]);
        assert_eq!(s.varyings[0].name, "vUv");
        assert_eq!(s.attributes[0].name, "aRandom");
        assert!(!s.body.contains("uniform") && !s.body.contains("precision"));
        assert!(s.body.contains("void main()"));
        let f = scan("#define N 3\nuniform vec2 pts[N], one;\nlayout(location = 0) out highp vec4 outColor;\nvoid main(){ outColor = vec4(pts[0], one); }", false).unwrap();
        assert_eq!(f.uniforms.iter().map(|u| (u.name.as_str(), u.array)).collect::<Vec<_>>(), vec![("pts", Some(3)), ("one", None)]);
        assert_eq!(f.outputs[0].name, "outColor");
        assert!(f.body.contains("#define N 3"));
    }

    #[test]
    fn a_shader_material_translates_to_the_contract() {
        let t = translate(&Source { vertex: VERT, fragment: FRAG, raw: false, vertex_colors: false, points: false }, &["map".into()]).unwrap();
        assert_eq!(t.uniforms.iter().map(|u| u.0.as_str()).collect::<Vec<_>>(), vec!["uTime", "uColor"]);
        assert_eq!(t.samplers, vec!["map".to_string()]);
        assert_eq!(t.attributes, vec![("aRandom".to_string(), 1)]);
        assert!(t.vertex.contains("fn three_vertex(v: ThreeVertex) -> ThreeVaryings"), "{}", t.vertex);
        assert!(t.vertex.contains("= u_uTime();"));
        assert!(t.vertex.contains("= v.a0.x;"));
        assert!(t.vertex.contains("textureSampleLevel") || !t.vertex.contains("textureSample("));
        assert!(t.fragment.contains("fn three_fragment(in: ThreeVaryings, front: bool) -> vec4<f32>"), "{}", t.fragment);
        assert!(t.fragment.contains("@group(constants::MATERIAL_BIND_GROUP) @binding(1)"));
        assert!(t.fragment.contains("discard"));
    }

    #[test]
    fn a_raw_shader_material_declares_its_own_built_ins() {
        let vert = "#version 300 es\nprecision highp float;\nuniform mat4 modelViewMatrix;\nuniform mat4 projectionMatrix;\nin vec3 position;\nin vec4 color;\nout vec4 vColor;\nvoid main(){ vColor = color; gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0); }";
        let frag = "#version 300 es\nprecision highp float;\nin vec4 vColor;\nout vec4 fragColor;\nvoid main(){ fragColor = vColor; }";
        let t = translate(&Source { vertex: vert, fragment: frag, raw: true, vertex_colors: false, points: false }, &[]).unwrap();
        assert!(t.uniforms.is_empty());
        assert!(t.attributes.is_empty());
        assert!(t.vertex.contains("= v.color;"), "{}", t.vertex);
    }

    #[test]
    fn errors_name_the_stage() {
        let e = translate(&Source { vertex: VERT, fragment: "void main(){ gl_FragColor = nope; }", raw: false, vertex_colors: false, points: false }, &["map".into()]).unwrap_err();
        assert!(e.contains("fragment"), "{e}");
    }
}
