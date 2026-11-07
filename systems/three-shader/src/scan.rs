//! The scanner glsl.rs translates with: a stage's GLSL split into the globals it declares (attributes, varyings,
//! uniforms, samplers, outputs) and the rest of its source, the body naga compiles.

use std::collections::BTreeMap;

/// A global a stage declares.
#[derive(Clone, Debug, PartialEq)]
pub struct Var {
    pub name: String,
    /// its GLSL type (`vec3`, `mat4`, `sampler2D`)
    pub glsl: String,
    pub array: Option<u32>,
    pub flat: bool,
}

#[derive(Default, Debug)]
pub struct Scanned {
    pub attributes: Vec<Var>,
    pub varyings: Vec<Var>,
    pub uniforms: Vec<Var>,
    /// sampler2D (and samplerCube, refused later) uniforms
    pub samplers: Vec<Var>,
    pub outputs: Vec<Var>,
    /// the source without those declarations, comments and `precision`, `#version` and `#extension` lines
    pub body: String,
}

/// Removes // and /* */ comments, keeping newlines.
fn uncomment(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let b = src.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
        } else if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
            i += 2;
            while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                if b[i] == b'\n' {
                    out.push('\n');
                }
                i += 1;
            }
            i += 2;
        } else {
            out.push(b[i] as char);
            i += 1;
        }
    }
    out
}

const QUALIFIERS: [&str; 10] = ["highp", "mediump", "lowp", "flat", "smooth", "noperspective", "centroid", "invariant", "const", "precise"];

/// The `#define NAME <integer>`s, for array sizes given by a macro.
fn int_defines(src: &str) -> BTreeMap<String, u32> {
    let mut m = BTreeMap::new();
    for l in src.lines() {
        let t: Vec<&str> = l.trim().split_whitespace().collect();
        if t.len() == 3 && t[0] == "#define" {
            if let Ok(n) = t[2].parse::<u32>() {
                m.insert(t[1].to_string(), n);
            }
        }
    }
    m
}

/// A global declaration's role and its variables, or None when the statement isn't one this cares about.
fn declaration(stmt: &str, vertex: bool, ints: &BTreeMap<String, u32>) -> Result<Option<(&'static str, Vec<Var>)>, String> {
    // layout(...) qualifiers say nothing three's GLSL needs here
    let mut s = stmt.trim().to_string();
    while let Some(i) = s.find("layout") {
        let Some(open) = s[i..].find('(') else { break };
        let Some(close) = s[i + open..].find(')') else { break };
        s.replace_range(i..i + open + close + 1, " ");
    }
    let s = s.replace(',', " , ").replace('[', " [ ").replace(']', " ] ").replace('=', " = ");
    let toks: Vec<&str> = s.split_whitespace().collect();
    let Some(&first) = toks.first() else { return Ok(None) };
    let mut flat = false;
    let mut role = None;
    let mut k = 0;
    while k < toks.len() {
        match toks[k] {
            "uniform" => role = Some("uniform"),
            "attribute" => role = Some("attribute"),
            "varying" => role = Some("varying"),
            "in" => role = Some(if vertex { "attribute" } else { "varying" }),
            "out" => role = Some(if vertex { "varying" } else { "output" }),
            "flat" => flat = true,
            q if QUALIFIERS.contains(&q) => {}
            _ => break,
        }
        k += 1;
    }
    let Some(role) = role else { return Ok(None) };
    if first == "precision" {
        return Ok(None);
    }
    let Some(&ty) = toks.get(k) else { return Err(format!("a declaration with no type: {stmt:?}")) };
    if toks.get(k + 1) == Some(&"{") {
        return Err(format!("an interface block isn't supported yet: {stmt:?}"));
    }
    let mut vars = Vec::new();
    let mut i = k + 1;
    while i < toks.len() {
        let name = toks[i];
        let mut array = None;
        i += 1;
        if toks.get(i) == Some(&"[") {
            let n = toks.get(i + 1).copied().unwrap_or("");
            array = Some(n.parse::<u32>().ok().or_else(|| ints.get(n).copied()).ok_or_else(|| format!("{name}: its array length {n:?} isn't a number or a #define of one"))?);
            i += 3;
        }
        if toks.get(i) == Some(&"=") {
            // an initializer (desktop GLSL allows one on a uniform): skipped to the next name
            while i < toks.len() && toks[i] != "," {
                i += 1;
            }
        }
        vars.push(Var { name: name.to_string(), glsl: ty.to_string(), array, flat });
        if toks.get(i) == Some(&",") {
            i += 1;
        }
    }
    Ok(Some((role, vars)))
}

/// A stage's globals taken out of its source.
pub fn scan(src: &str, vertex: bool) -> Result<Scanned, String> {
    let src = uncomment(src);
    let ints = int_defines(&src);
    let mut out = Scanned::default();
    let mut body = String::with_capacity(src.len());
    let mut depth = 0i32;
    let mut stmt = String::new();
    let mut line_start = true;
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        // a directive line, outside a statement's text: kept as it is (but #version, #extension and #pragma)
        if line_start && depth == 0 && stmt.trim().is_empty() {
            let mut lead = String::new();
            let mut cc = Some(c);
            while let Some(x) = cc.filter(|x| matches!(x, ' ' | '\t')) {
                lead.push(x);
                cc = chars.next();
            }
            if cc == Some('#') {
                let mut line = String::from("#");
                for x in chars.by_ref() {
                    if x == '\n' {
                        break;
                    }
                    line.push(x);
                }
                let t = line.trim_start_matches('#').trim_start();
                if !(t.starts_with("version") || t.starts_with("extension") || t.starts_with("pragma")) {
                    // on a line of its own: the newline after a function's `}` went to the statement being gathered,
                    // so a #define after a function (three's <common>) was mid-line (naga: UnexpectedHash)
                    if !body.is_empty() && !body.ends_with('\n') {
                        body.push('\n');
                    }
                    body += &line;
                }
                body.push('\n');
                stmt.clear();
                line_start = true;
                continue;
            }
            stmt += &lead;
            let Some(c2) = cc else { break };
            line_start = false;
            if c2 == '\n' {
                stmt.push('\n');
                line_start = true;
                continue;
            }
            step(c2, &mut depth, &mut stmt, &mut body, &mut out, vertex, &ints)?;
            continue;
        }
        line_start = c == '\n';
        step(c, &mut depth, &mut stmt, &mut body, &mut out, vertex, &ints)?;
    }
    body += &stmt;
    out.body = body;
    Ok(out)
}

/// One character: inside a body it goes straight out; at the top level it gathers into a statement, which ends at `;`
/// (a declaration this takes out, or anything else kept) or at `{` (a function's or struct's head, kept).
fn step(c: char, depth: &mut i32, stmt: &mut String, body: &mut String, out: &mut Scanned, vertex: bool, ints: &BTreeMap<String, u32>) -> Result<(), String> {
    if *depth > 0 {
        body.push(c);
        if c == '{' {
            *depth += 1;
        } else if c == '}' {
            *depth -= 1;
        }
        return Ok(());
    }
    match c {
        '{' => {
            *body += stmt;
            body.push('{');
            stmt.clear();
            *depth = 1;
        }
        ';' => {
            let t = stmt.trim();
            if t.starts_with("precision ") {
                *body += &"\n".repeat(stmt.matches('\n').count());
            } else {
                match declaration(t, vertex, ints)? {
                    Some((role, vars)) => {
                        let list = match role {
                            "attribute" => &mut out.attributes,
                            "varying" => &mut out.varyings,
                            "output" => &mut out.outputs,
                            _ => &mut out.uniforms,
                        };
                        for v in vars {
                            if role == "uniform" && v.glsl.starts_with("sampler") {
                                out.samplers.push(v);
                            } else {
                                list.push(v);
                            }
                        }
                        *body += &"\n".repeat(stmt.matches('\n').count());
                    }
                    None => {
                        *body += stmt;
                        body.push(';');
                    }
                }
            }
            stmt.clear();
        }
        _ => stmt.push(c),
    }
    Ok(())
}
