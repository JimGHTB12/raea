//! WGSL from a K kernel: one compute shader per kernel, every intermediate a local.

use std::collections::HashMap;

use raea_core_k::{E, Kernel, Line};

struct Gen<'a> {
    buffers: &'a [&'a str],
    textures: &'a [&'a str],
    scalars: &'a HashMap<&'a str, i32>,
    pre: Vec<String>,
    fresh: usize,
    /// sum's loop variables in scope
    bound: Vec<String>,
    /// every name the kernel binds: one WGSL var each, assigned in order (rebinding is assignment)
    declared: std::collections::BTreeSet<String>,
}

impl Gen<'_> {
    fn name(&self, n: &str) -> Result<String, String> {
        if self.bound.iter().any(|b| b == n) {
            Ok(format!("kl_{n}"))
        } else if n == "i" || self.declared.contains(n) {
            Ok(format!("k_{n}"))
        } else if let Some(v) = self.scalars.get(n) {
            Ok(format!("({v})"))
        } else {
            Err(format!("unknown name {n}"))
        }
    }
    fn e(&mut self, e: &E) -> Result<String, String> {
        Ok(match e {
            E::Num(n) => format!("({})", *n as i32),
            E::Name(n) => self.name(n)?,
            E::Neg(a) => format!("(-{})", self.e(a)?),
            E::Bin(op, a, b) => {
                let (x, y) = (self.e(a)?, self.e(b)?);
                match op {
                    '+' | '-' | '*' => format!("({x} {op} {y})"),
                    // K's % divides (WGSL's % is the remainder, K's !)
                    '%' => format!("({x} / {y})"),
                    '!' => format!("({x} % {y})"),
                    // min, max and clamp take their bounds plus kz, a zero the compiler can't prove (read from the
                    // shape uniform): knowing a value is a byte, the Adreno X1-85's Vulkan compiler fused clamp, shl and or
                    // into a pack that set the top two bytes of half the NV12 luma words, striping bright video (#82)
                    '&' => format!("min({x}, {y} + kz)"),
                    '|' => format!("max({x}, {y} + kz)"),
                    '<' | '>' => format!("select(0, 1, {x} {op} {y})"),
                    _ => format!("select(0, 1, {x} == {y})"),
                }
            }
            E::Call(f, a) => {
                let buf = |k: usize| -> Result<String, String> {
                    match &a[k] {
                        E::Name(n) if self.buffers.contains(&n.as_str()) => Ok(format!("b_{n}")),
                        _ => Err(format!("{f}: the first argument is an input buffer")),
                    }
                };
                match f.as_str() {
                    "tex" => {
                        let t = match &a[0] {
                            E::Name(n) if self.textures.contains(&n.as_str()) => n.clone(),
                            _ => return Err("tex[T;x;y]: T is a texture input".into()),
                        };
                        format!("kpack(textureLoad(t_{t}, vec2<i32>({}, {}), 0))", self.e(&a[1])?, self.e(&a[2])?)
                    }
                    "byte" => {
                        let b = buf(0)?;
                        let k = self.e(&a[1])?;
                        format!("i32(({b}[u32({k}) >> 2u] >> (8u * (u32({k}) & 3u))) & 255u)")
                    }
                    "word" => {
                        let b = buf(0)?;
                        format!("bitcast<i32>({b}[u32({})])", self.e(&a[1])?)
                    }
                    "pop" => format!("i32(countOneBits(u32({})))", self.e(&a[0])?),
                    "xor" => format!("({} ^ {})", self.e(&a[0])?, self.e(&a[1])?),
                    "or" => format!("({} | {})", self.e(&a[0])?, self.e(&a[1])?),
                    "and" => format!("({} & {})", self.e(&a[0])?, self.e(&a[1])?),
                    // shifted as u32: a byte shifted into the sign bit (an NV12 word's top byte >= 128) overflows an i32
                    // shift, which WGSL leaves to the compiler. Same bits as the interpreter's wrapping_shl.
                    "shl" => format!("bitcast<i32>(u32({}) << (u32({}) & 31u))", self.e(&a[0])?, self.e(&a[1])?),
                    "shr" => format!("({} >> (u32({}) & 31u))", self.e(&a[0])?, self.e(&a[1])?),
                    "clamp" => format!("clamp({}, {} + kz, {} + kz)", self.e(&a[0])?, self.e(&a[1])?, self.e(&a[2])?),
                    "sel" => {
                        let (c, x, y) = (self.e(&a[0])?, self.e(&a[1])?, self.e(&a[2])?);
                        format!("select({y}, {x}, {c} != 0)")
                    }
                    "pack" => {
                        let v: Vec<String> = (0..4).map(|k| self.e(&a[k]).map(|s| format!("(u32({s}) & 255u)"))).collect::<Result<_, _>>()?;
                        format!("bitcast<i32>({} | ({} << 8u) | ({} << 16u) | ({} << 24u))", v[0], v[1], v[2], v[3])
                    }
                    "sum" | "low" => {
                        let E::Name(k) = &a[0] else { return Err(format!("{f}[k;n;expr]: k is a name")) };
                        let (init, fold) = if f == "low" { ("2147483647", "min") } else { ("0", "") };
                        let n = self.e(&a[1])?;
                        let id = self.fresh;
                        self.fresh += 1;
                        self.bound.push(k.clone());
                        // the body's own pre-statements belong inside the loop
                        let outer = std::mem::take(&mut self.pre);
                        let body = self.e(&a[2])?;
                        let inner = std::mem::replace(&mut self.pre, outer);
                        self.bound.pop();
                        self.pre.push(format!(
                            "var s_{id}: i32 = {init};\n  for (var kl_{k}: i32 = 0; kl_{k} < {n}; kl_{k} = kl_{k} + 1) {{ {} s_{id} = {}; }}",
                            inner.join(" "),
                            if fold.is_empty() { format!("s_{id} + {body}") } else { format!("{fold}(s_{id}, {body} + kz)") }
                        ));
                        format!("s_{id}")
                    }
                    other => return Err(format!("unknown function {other}")),
                }
            }
        })
    }
}

/// As `wgsl`, with texture inputs too (bound after the buffers, as texture_2d<f32> views whose texels are read back
/// as exact bytes: give them a Unorm view, not sRGB). Binding order: buffers, textures, outputs, the shape uniform.
pub fn wgsl_tex(k: &Kernel, buffers: &[&str], textures: &[&str], scalars: &HashMap<&str, i32>, outputs: &[&str]) -> Result<String, String> {
    fn names(lines: &[Line], out: &mut std::collections::BTreeSet<String>) {
        for l in lines {
            match l {
                Line::Bind(n, _) => {
                    out.insert(n.clone());
                }
                Line::When(_, b) => names(b, out),
            }
        }
    }
    fn emit(g: &mut Gen, lines: &[Line], body: &mut String, ind: usize) -> Result<(), String> {
        let pad = " ".repeat(ind);
        for l in lines {
            match l {
                Line::Bind(name, e) => {
                    let x = g.e(e)?;
                    for p in g.pre.drain(..) {
                        body.push_str(&format!("{pad}{p}\n"));
                    }
                    body.push_str(&format!("{pad}k_{name} = {x};\n"));
                }
                Line::When(c, b) => {
                    let x = g.e(c)?;
                    for p in g.pre.drain(..) {
                        body.push_str(&format!("{pad}{p}\n"));
                    }
                    body.push_str(&format!("{pad}if ({x} != 0) {{\n"));
                    emit(g, b, body, ind + 2)?;
                    body.push_str(&format!("{pad}}}\n"));
                }
            }
        }
        Ok(())
    }
    let mut declared = std::collections::BTreeSet::new();
    names(&k.lines, &mut declared);
    declared.remove("i");
    let mut g = Gen { buffers, textures, scalars, pre: Vec::new(), fresh: 0, bound: Vec::new(), declared: declared.clone() };
    let mut body = String::new();
    for n in &declared {
        body.push_str(&format!("  var k_{n}: i32 = 0;\n"));
    }
    emit(&mut g, &k.lines, &mut body, 2)?;
    let mut s = String::new();
    for (b, name) in buffers.iter().enumerate() {
        s.push_str(&format!("@group(0) @binding({b}) var<storage, read> b_{name}: array<u32>;\n"));
    }
    for (t, name) in textures.iter().enumerate() {
        s.push_str(&format!("@group(0) @binding({}) var t_{name}: texture_2d<f32>;\n", buffers.len() + t));
    }
    let base = buffers.len() + textures.len();
    for (o, name) in outputs.iter().enumerate() {
        s.push_str(&format!("@group(0) @binding({}) var<storage, read_write> o_{name}: array<u32>;\n", base + o));
    }
    s.push_str("fn kpack(c: vec4<f32>) -> i32 { let u = vec4<u32>(round(clamp(c, vec4<f32>(0.0), vec4<f32>(1.0)) * 255.0)); return bitcast<i32>(u.x | (u.y << 8u) | (u.z << 16u) | (u.w << 24u)); }\n");
    s.push_str("struct Shape { n: u32, stride: u32 }\n");
    s.push_str(&format!("@group(0) @binding({}) var<uniform> shape: Shape;\n", base + outputs.len()));
    s.push_str("@compute @workgroup_size(256)\nfn main(@builtin(global_invocation_id) gid: vec3<u32>) {\n");
    s.push_str("  let ii = gid.x + gid.y * shape.stride;\n  if (ii >= shape.n) { return; }\n  let k_i: i32 = i32(ii);\n  let kz: i32 = i32(shape.n >> 31u);\n");
    s.push_str(&body);
    for name in outputs {
        s.push_str(&format!("  o_{name}[ii] = bitcast<u32>(k_{name});\n"));
    }
    s.push_str("}\n");
    Ok(s)
}
