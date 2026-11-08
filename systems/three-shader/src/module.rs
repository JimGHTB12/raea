//! The WGSL a three shader is compiled as: the prelude both stages share (three's built-ins, the uniform accessors, the
//! textures), the entry points over the source's `three_vertex` and `three_fragment`, and the uniform layout.
//!
//! The contract a stage's source meets (docs/r3f.feature R3F11), whether naga wrote it from GLSL (glsl.rs) or three's
//! own TSL builder wrote it (sdk/r3f's input/tsl.ts):
//!   vertex:   fn three_vertex(v: ThreeVertex) -> ThreeVaryings
//!   fragment: fn three_fragment(in: ThreeVaryings, front: bool) -> vec4<f32>
//! and both declare the same `struct ThreeVaryings { @builtin(position) clip: vec4<f32>, @location(0) ... }`. The
//! prelude gives `ThreeVertex` (position, normal, uv, color, a0..a3: a mesh's own attributes packed four floats a
//! slot, instance, index), `three_model(instance)`, `three_view()`, `three_projection()` (Bevy's clip_from_view, reverse
//! Z), `three_camera_position()`, `three_viewport()`, `three_time()`, a `u_NAME()` per uniform (`u_NAME(i)` for an
//! array), and `tex_NAME` / `samp_NAME` per texture.

/// vec4 slots in the uniform buffer (4 KB, under every device's 16 KB uniform binding; lettra's default scramble pool
/// takes 78), and the two the material keeps for itself, its last: three's time (time, delta, frame), then how the
/// fragment's colour is read (lib.rs's `display`)
pub const SLOTS: usize = 256;
pub const USER_SLOTS: usize = SLOTS - 2;
pub const TIME_SLOT: usize = SLOTS - 2;
pub const DISPLAY_SLOT: usize = SLOTS - 1;
/// textures a material may bind, and a mesh's own attributes (each up to four floats)
pub const TEXTURES: usize = 4;
pub const ATTRIBUTES: usize = 4;

/// A uniform's type: WGSL's name for it, the floats an element holds, the slots an element takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ty {
    F32,
    I32,
    U32,
    Bool,
    /// vecN of f32 (N 2..4)
    Vec(u8),
    IVec(u8),
    UVec(u8),
    BVec(u8),
    /// matNxN of f32 (N 2..4)
    Mat(u8),
}

impl Ty {
    /// WGSL's name (`vec3<f32>`), or GLSL's (`vec3`, `ivec2`, `mat4`, `float`)
    pub fn parse(s: &str) -> Option<Ty> {
        let s = s.replace(' ', "");
        let n = |c: char| c.to_digit(10).filter(|d| (2..=4).contains(d)).map(|d| d as u8);
        Some(match s.as_str() {
            "f32" | "float" => Ty::F32,
            "i32" | "int" => Ty::I32,
            "u32" | "uint" => Ty::U32,
            "bool" => Ty::Bool,
            _ => {
                let c = s.chars().nth(if s.starts_with("vec") || s.starts_with("mat") { 3 } else { 4 })?;
                let d = n(c)?;
                match s.as_str() {
                    x if x == format!("vec{d}") || x == format!("vec{d}<f32>") => Ty::Vec(d),
                    x if x == format!("ivec{d}") || x == format!("vec{d}<i32>") => Ty::IVec(d),
                    x if x == format!("uvec{d}") || x == format!("vec{d}<u32>") => Ty::UVec(d),
                    x if x == format!("bvec{d}") || x == format!("vec{d}<bool>") => Ty::BVec(d),
                    x if x == format!("mat{d}") || x == format!("mat{d}x{d}") || x == format!("mat{d}x{d}<f32>") => Ty::Mat(d),
                    _ => return None,
                }
            }
        })
    }

    pub fn wgsl(self) -> String {
        match self {
            Ty::F32 => "f32".into(),
            Ty::I32 => "i32".into(),
            Ty::U32 => "u32".into(),
            Ty::Bool => "bool".into(),
            Ty::Vec(n) => format!("vec{n}<f32>"),
            Ty::IVec(n) => format!("vec{n}<i32>"),
            Ty::UVec(n) => format!("vec{n}<u32>"),
            Ty::BVec(n) => format!("vec{n}<bool>"),
            Ty::Mat(n) => format!("mat{n}x{n}<f32>"),
        }
    }

    /// floats an element is given as (a matrix column-major, as three's `elements`)
    pub fn floats(self) -> usize {
        match self {
            Ty::F32 | Ty::I32 | Ty::U32 | Ty::Bool => 1,
            Ty::Vec(n) | Ty::IVec(n) | Ty::UVec(n) | Ty::BVec(n) => n as usize,
            Ty::Mat(n) => (n * n) as usize,
        }
    }

    /// vec4 slots an element takes
    pub fn slots(self) -> usize {
        match self {
            Ty::Mat(n) => n as usize,
            _ => 1,
        }
    }

    /// the element at slot `k` (a WGSL expression) of `s` (the slots array)
    fn read(self, s: &str, k: &str) -> String {
        let sw = |n: u8| ["", "", ".xy", ".xyz", ""][n as usize];
        match self {
            Ty::F32 => format!("{s}[{k}].x"),
            Ty::I32 => format!("i32({s}[{k}].x)"),
            Ty::U32 => format!("u32({s}[{k}].x)"),
            Ty::Bool => format!("({s}[{k}].x != 0.0)"),
            Ty::Vec(n) => format!("{s}[{k}]{}", sw(n)),
            Ty::IVec(n) => format!("vec{n}<i32>({s}[{k}]{})", sw(n)),
            Ty::UVec(n) => format!("vec{n}<u32>({s}[{k}]{})", sw(n)),
            Ty::BVec(n) => format!("({s}[{k}]{} != vec{n}<f32>(0.0))", sw(n)),
            Ty::Mat(n) => {
                let cols: Vec<String> = (0..n).map(|c| format!("{s}[{k} + {c}u]{}", sw(n))).collect();
                format!("mat{n}x{n}<f32>({})", cols.join(", "))
            }
        }
    }
}

/// A uniform laid out: its first slot.
#[derive(Clone, Debug)]
pub struct Uniform {
    pub name: String,
    pub ty: Ty,
    /// an array's length
    pub array: Option<u32>,
    pub base: usize,
}

impl Uniform {
    pub fn len(&self) -> usize {
        self.array.unwrap_or(1) as usize
    }
}

/// Uniforms laid out one after another from slot 0; an error past the slots there are.
pub fn layout(list: &[(String, Ty, Option<u32>)]) -> Result<Vec<Uniform>, String> {
    let mut at = 0;
    let mut out = Vec::new();
    for (name, ty, array) in list {
        let u = Uniform { name: name.clone(), ty: *ty, array: *array, base: at };
        at += ty.slots() * u.len();
        if at > USER_SLOTS {
            return Err(format!("a three shader's uniforms take at most {USER_SLOTS} vec4 slots; {name} passes them ({at})"));
        }
        out.push(u);
    }
    Ok(out)
}

/// `values` (each uniform's floats, an array's elements one after another) written into the slots
pub fn pack(layout: &[Uniform], values: &serde_json::Map<String, serde_json::Value>, slots: &mut [[f32; 4]; SLOTS]) -> Result<(), String> {
    for u in layout {
        let Some(v) = values.get(&u.name) else { continue };
        let f: Vec<f32> = match v {
            serde_json::Value::Number(n) => vec![n.as_f64().unwrap_or(0.0) as f32],
            serde_json::Value::Bool(b) => vec![*b as u8 as f32],
            serde_json::Value::Array(a) => a.iter().map(|x| match x {
                serde_json::Value::Bool(b) => Ok(*b as u8 as f32),
                _ => x.as_f64().map(|f| f as f32).ok_or_else(|| format!("uniform {}: numbers, got {x}", u.name)),
            }).collect::<Result<_, _>>()?,
            _ => return Err(format!("uniform {}: a number or a list of numbers, got {v}", u.name)),
        };
        let per = u.ty.floats();
        if f.len() < per {
            return Err(format!("uniform {} ({}): {per} numbers an element, got {}", u.name, u.ty.wgsl(), f.len()));
        }
        for (i, e) in f.chunks(per).take(u.len()).enumerate() {
            if e.len() < per {
                break;
            }
            let at = u.base + i * u.ty.slots();
            match u.ty {
                Ty::Mat(n) => {
                    for c in 0..n as usize {
                        for r in 0..n as usize {
                            slots[at + c][r] = e[c * n as usize + r];
                        }
                    }
                }
                _ => {
                    for (c, x) in e.iter().enumerate() {
                        slots[at][c] = *x;
                    }
                }
            }
        }
    }
    Ok(())
}

const GROUP: &str = "constants::MATERIAL_BIND_GROUP";

/// The prelude of a stage: Bevy's view and mesh functions, the slots, the built-ins, the uniform accessors, and (when
/// `textures` is given: a WGSL source; naga's GLSL declares its own) the textures.
pub fn prelude(uniforms: &[Uniform], textures: Option<&[String]>, attributes: usize) -> String {
    let mut m = String::from(
        "import bevy_pbr::render::mesh_functions::get_world_from_local;\n\
         import bevy_pbr::render::mesh_view_bindings::{view, globals};\n\n",
    );
    m += &format!("struct ThreeSlots {{ s: array<vec4<f32>, {SLOTS}> }};\n@group({GROUP}) @binding(0) var<uniform> three_slots: ThreeSlots;\n");
    if let Some(ts) = textures {
        for (i, t) in ts.iter().enumerate() {
            m += &format!("@group({GROUP}) @binding({}) var tex_{t}: texture_2d<f32>;\n@group({GROUP}) @binding({}) var samp_{t}: sampler;\n", 1 + i * 2, 2 + i * 2);
        }
    }
    m += "\nstruct ThreeVertex {\n    position: vec3<f32>,\n    normal: vec3<f32>,\n    uv: vec2<f32>,\n    color: vec4<f32>,\n";
    for k in 0..attributes.max(1) {
        m += &format!("    a{k}: vec4<f32>,\n");
    }
    m += "    instance: u32,\n    index: u32,\n};\n\n";
    m += "fn three_model(instance: u32) -> mat4x4<f32> { return get_world_from_local(instance); }\n\
          fn three_view() -> mat4x4<f32> { return view.view_from_world; }\n\
          fn three_projection() -> mat4x4<f32> { return view.clip_from_view; }\n\
          fn three_camera_position() -> vec3<f32> { return view.world_position; }\n\
          fn three_viewport() -> vec4<f32> { return view.viewport; }\n";
    m += &format!("fn three_time() -> f32 {{ return three_slots.s[{TIME_SLOT}].x; }}\nfn three_delta() -> f32 {{ return three_slots.s[{TIME_SLOT}].y; }}\nfn three_frame() -> f32 {{ return three_slots.s[{TIME_SLOT}].z; }}\n");
    for u in uniforms {
        let ty = u.ty.wgsl();
        match u.array {
            None => m += &format!("fn u_{}() -> {ty} {{ return {}; }}\n", u.name, u.ty.read("three_slots.s", &format!("{}u", u.base))),
            Some(_) => m += &format!("fn u_{}(i: u32) -> {ty} {{ let k = {}u + i * {}u; return {}; }}\n", u.name, u.base, u.ty.slots(), u.ty.read("three_slots.s", "k")),
        }
    }
    m + "\n"
}

/// The vertex entry: the mesh's attributes as a ThreeVertex, the source's `three_vertex` on it. Locations: position 0,
/// normal 1, uv 2, colour 3, the mesh's own attributes 4 and up (lib.rs's `specialize` lays the buffer out so).
pub fn vertex_entry(attributes: usize) -> String {
    let mut m = String::from("\nstruct ThreeMeshIn {\n    @builtin(instance_index) instance: u32,\n    @builtin(vertex_index) index: u32,\n    @location(0) position: vec3<f32>,\n    @location(1) normal: vec3<f32>,\n    @location(2) uv: vec2<f32>,\n    @location(3) color: vec4<f32>,\n");
    for k in 0..attributes {
        m += &format!("    @location({}) a{k}: vec4<f32>,\n", 4 + k);
    }
    m += "};\n\n@vertex\nfn vertex(m: ThreeMeshIn) -> ThreeVaryings {\n    var v: ThreeVertex;\n    v.position = m.position;\n    v.normal = m.normal;\n    v.uv = m.uv;\n    v.color = m.color;\n";
    for k in 0..attributes {
        m += &format!("    v.a{k} = m.a{k};\n");
    }
    m + "    v.instance = m.instance;\n    v.index = m.index;\n    return three_vertex(v);\n}\n"
}

/// The fragment entry: the source's `three_fragment`, its colour read as `display` says (tone.wesl's three_untone).
pub fn fragment_entry() -> String {
    let mut m = String::from(TONE);
    m = m.replace("GROUP", GROUP).replace("DISPLAY_SLOT", &DISPLAY_SLOT.to_string());
    m + "\n@fragment\nfn fragment(in: ThreeVaryings, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {\n    return three_untone(three_fragment(in, front));\n}\n"
}

const TONE: &str = include_str!("tone.wesl");

/// The argument spans of the call whose `(` is at `open` (byte offsets, each trimmed), and where its `)` is.
fn args(src: &str, open: usize) -> Option<(Vec<(usize, usize)>, usize)> {
    let b = src.as_bytes();
    let (mut depth, mut start, mut out) = (0i32, open + 1, Vec::new());
    for (i, &c) in b.iter().enumerate().skip(open) {
        match c {
            b'(' | b'[' => depth += 1,
            b')' | b']' => {
                depth -= 1;
                if depth == 0 {
                    out.push((start, i));
                    return Some((out, i));
                }
            }
            b',' if depth == 1 => {
                out.push((start, i));
                start = i + 1;
            }
            _ => {}
        }
    }
    None
}

/// Every sample of the textures `names` with its v turned over (three's flipY): textureSample* read at
/// (u, 1 - v), textureLoad at the mirrored row.
pub fn flip_samples(src: &str, names: &[String]) -> String {
    if names.is_empty() {
        return src.to_string();
    }
    let mut out = String::with_capacity(src.len());
    let mut at = 0;
    while let Some(i) = src[at..].find("texture").map(|i| at + i) {
        let rest = &src[i..];
        let name_end = rest.char_indices().find(|&(_, c)| !c.is_ascii_alphanumeric() && c != '_').map_or(rest.len(), |(k, _)| k);
        let func = &rest[..name_end];
        let open = i + name_end;
        let call = (func.starts_with("textureSample") || func == "textureLoad") && src[open..].starts_with('(');
        let parsed = call.then(|| args(src, open)).flatten();
        match parsed {
            Some((list, close)) if list.first().is_some_and(|(a, b)| names.iter().any(|n| src[*a..*b].trim() == n)) => {
                let k = if func == "textureLoad" { 1 } else { 2 };
                let Some(&(ca, cb)) = list.get(k) else {
                    out += &src[at..open];
                    at = open;
                    continue;
                };
                let coords = src[ca..cb].trim();
                let flipped = if func == "textureLoad" {
                    let tex = src[list[0].0..list[0].1].trim();
                    let level = list.get(2).map(|(a, b)| src[*a..*b].trim()).unwrap_or("0");
                    format!("vec2<i32>(({coords}).x, i32(textureDimensions({tex}, {level}).y) - 1 - ({coords}).y)")
                } else {
                    format!("vec2<f32>(({coords}).x, 1.0 - ({coords}).y)")
                };
                out += &src[at..ca];
                out += &flipped;
                out += &src[cb..=close];
                at = close + 1;
            }
            _ => {
                out += &src[at..open.max(i + 1)];
                at = open.max(i + 1);
            }
        }
    }
    out + &src[at..]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flipped_textures_are_read_upside_down() {
        let s = "let a = textureSample(rgt0x, rgs0x, vUv);\nlet b = textureSampleLevel(rgt1x, rgs1x, f(x, y), 0.0);\nlet c = textureLoad(rgt0x, p, 0);";
        let f = flip_samples(s, &["rgt0x".into()]);
        assert!(f.contains("textureSample(rgt0x, rgs0x,vec2<f32>((vUv).x, 1.0 - (vUv).y))"), "{f}");
        assert!(f.contains("textureSampleLevel(rgt1x, rgs1x, f(x, y), 0.0)"));
        assert!(f.contains("textureLoad(rgt0x,vec2<i32>((p).x, i32(textureDimensions(rgt0x, 0).y) - 1 - (p).y), 0)"), "{f}");
    }

    #[test]
    fn types_read_from_wgsl_and_glsl() {
        assert_eq!(Ty::parse("vec3<f32>"), Some(Ty::Vec(3)));
        assert_eq!(Ty::parse("vec3"), Some(Ty::Vec(3)));
        assert_eq!(Ty::parse("ivec2"), Some(Ty::IVec(2)));
        assert_eq!(Ty::parse("mat4"), Some(Ty::Mat(4)));
        assert_eq!(Ty::parse("mat3x3<f32>"), Some(Ty::Mat(3)));
        assert_eq!(Ty::parse("float"), Some(Ty::F32));
        assert_eq!(Ty::parse("sampler2D"), None);
    }

    #[test]
    fn uniforms_pack_in_order_and_read_back() {
        let l = layout(&[("uTime".into(), Ty::F32, None), ("m".into(), Ty::Mat(3), None), ("pts".into(), Ty::Vec(2), Some(3))]).unwrap();
        assert_eq!(l.iter().map(|u| u.base).collect::<Vec<_>>(), vec![0, 1, 4]);
        let mut s = [[0f32; 4]; SLOTS];
        let v: serde_json::Value = serde_json::json!({"uTime": 2.5, "m": [1, 2, 3, 4, 5, 6, 7, 8, 9], "pts": [1, 2, 3, 4, 5, 6]});
        pack(&l, v.as_object().unwrap(), &mut s).unwrap();
        assert_eq!(s[0][0], 2.5);
        assert_eq!(s[2], [4.0, 5.0, 6.0, 0.0]);
        assert_eq!(s[6], [5.0, 6.0, 0.0, 0.0]);
        let p = prelude(&l, Some(&["map".into()]), 0);
        assert!(p.contains("fn u_uTime() -> f32 { return three_slots.s[0u].x; }"));
        assert!(p.contains("fn u_m() -> mat3x3<f32> { return mat3x3<f32>(three_slots.s[1u + 0u].xyz, three_slots.s[1u + 1u].xyz, three_slots.s[1u + 2u].xyz); }"));
        assert!(p.contains("fn u_pts(i: u32) -> vec2<f32> { let k = 4u + i * 1u; return three_slots.s[k].xy; }"));
        assert!(p.contains("var tex_map: texture_2d<f32>;"));
        // 63 mat4s fit in the 254 slots of its own, 64 don't
        assert!(layout(&[("big".into(), Ty::Mat(4), Some(63))]).is_ok());
        assert!(layout(&[("big".into(), Ty::Mat(4), Some(64))]).is_err());
    }
}
