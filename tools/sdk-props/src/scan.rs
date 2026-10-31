//! sdk/*/src scanned: the props interfaces and what they extend, each component, its props type and the kinds it hands
//! node3d. A scanner, not a parser: interface members are the identifiers before `:` or `?:` at the interface's own
//! brace depth, a component is `export function Name(param: Type)` and its body the braces after it. Generated files
//! (*.gen.ts, *.gen.tsx) are left out: their props are reflected fields, read by reflection (tier 2).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// A props interface: what it extends and its own members.
pub struct Iface {
    pub extends: Vec<Base>,
    pub members: Vec<String>,
}

/// An exported component that hands its props to the engine.
pub struct Component {
    pub name: String,
    pub file: PathBuf,
    /// its props type: the interfaces it names (`A & B`, `Omit<A, "x">`)
    pub props: Vec<Base>,
    /// the kinds it hands node3d
    pub kinds: Vec<String>,
    /// its body's lines that do more than check (no `throw`)
    pub body: String,
    /// its whole props object goes to node3d, or what a destructuring leaves of it (`...rest`): each member then
    /// reaches the engine as itself
    pub forwards: bool,
}

/// The TS sources under `root`'s sdk/*/src, generated files left out
pub fn sources(root: &Path) -> Vec<PathBuf> {
    let mut out = vec![];
    let mut stack: Vec<PathBuf> = std::fs::read_dir(root.join("sdk")).into_iter().flatten().flatten().map(|e| e.path().join("src")).filter(|p| p.is_dir()).collect();
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            if p.is_dir() && name != "node_modules" {
                stack.push(p);
            } else if (name.ends_with(".ts") || name.ends_with(".tsx")) && !name.contains(".gen.") && !name.ends_with(".d.ts") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

fn ident_at(s: &str) -> &str {
    let end = s.find(|c: char| !is_id(c)).unwrap_or(s.len());
    &s[..end]
}

/// The text from `open` (a `{`) to its matching `}`, braces in strings and comments counted as code (good enough for
/// the SDK's sources, whose strings rarely hold a brace that isn't paired)
fn block(src: &str, open: usize) -> &str {
    let mut depth = 0;
    for (i, c) in src[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &src[open + 1..open + i];
                }
            }
            _ => {}
        }
    }
    &src[open + 1..]
}

/// The members at an interface body's own depth: `name:`, `name?:`, `readonly name:`, `name(`
fn members(body: &str) -> Vec<String> {
    let mut out = vec![];
    let mut depth = 0;
    for line in body.lines() {
        let t = line.trim_start().trim_start_matches("readonly ");
        if depth == 0 && !t.starts_with("//") && !t.starts_with('*') && !t.starts_with("/*") {
            let id = ident_at(t);
            let rest = t[id.len()..].trim_start();
            if !id.is_empty() && (rest.starts_with(':') || rest.starts_with("?:") || rest.starts_with('(')) {
                out.push(id.to_string());
            }
        }
        depth += line.matches(['{', '(', '[']).count() as i32 - line.matches(['}', ')', ']']).count() as i32;
    }
    out
}

/// Every `interface Name [extends A, B] { ... }` in `src`
pub fn interfaces(file: &Path, src: &str, into: &mut BTreeMap<String, Vec<(PathBuf, Iface)>>) {
    for (at, _) in src.match_indices("interface ") {
        if at > 0 && !src[..at].ends_with([' ', '\n', '\t']) {
            continue;
        }
        let rest = &src[at + "interface ".len()..];
        let name = ident_at(rest);
        let Some(open) = rest.find('{') else { continue };
        let head = &rest[name.len()..open];
        let extends = head.split_once("extends").map(|(_, e)| bases(e)).unwrap_or_default();
        let body = block(rest, open);
        into.entry(name.to_string()).or_default().push((file.to_path_buf(), Iface { extends, members: members(body) }));
    }
}

/// Every `export function Name(param: Type)` in `file` whose body calls `node3d("Kind"`
pub fn components(file: &Path, src: &str, into: &mut Vec<Component>) {
    for (at, _) in src.match_indices("export function ") {
        let rest = &src[at + "export function ".len()..];
        let name = ident_at(rest);
        let Some(params) = rest[name.len()..].strip_prefix('(') else { continue };
        // the type after the parameter (a destructuring's defaults hold no colon of their own)
        let Some((param, ty)) = params.split_once(')').map(|(p, _)| p).and_then(|p| p.rsplit_once(':')) else { continue };
        let props = bases(ty);
        let param = param.trim();
        // the body: the first brace after the parameters' own closing paren (a return type may come between)
        let Some(close) = params.find(')') else { continue };
        let Some(open) = params[close..].find('{').map(|i| i + close + name.len() + 1) else { continue };
        let body = block(rest, open);
        let kinds: Vec<String> = body.match_indices("node3d(\"").map(|(i, _)| ident_at(&body[i + 8..]).to_string()).collect();
        if kinds.is_empty() || props.is_empty() {
            continue;
        }
        // what holds the props whole: the parameter (for a destructuring, what it leaves: `...rest`), and each local
        // made from one of those (`const p = dofOf("X", raw)`, `const { size, ...rest } = props`)
        let mut whole: Vec<&str> = match param.rsplit_once("...") {
            Some((_, r)) if param.starts_with('{') => vec![ident_at(r.trim())],
            _ if param.starts_with('{') => vec![],
            _ => vec![param],
        };
        for line in body.lines() {
            let t = line.trim_start().trim_start_matches("const ").trim_start_matches("let ");
            // a destructuring's defaults (`{ size = 1, ...rest }`) hold an ` = ` of their own
            let split = if t.starts_with('{') { t.split_once("} = ") } else { t.split_once(" = ") };
            let Some((lhs, rhs)) = split else { continue };
            if whole.iter().any(|w| hands(rhs, w)) {
                let local = if lhs.starts_with('{') { lhs.rsplit_once("...").map_or("", |(_, r)| ident_at(r)) } else { ident_at(lhs) };
                if !local.is_empty() {
                    whole.push(local);
                }
            }
        }
        // one of them to node3d, spread into what goes there (`{ ...props, color: hex }`) or through a helper that
        // returns it (`colours("Fog", props, [...])`)
        let forwards = whole.iter().any(|w| body.contains(&format!("...{w}")) || body.match_indices("node3d(").any(|(i, _)| body[i..].split_once(',').is_some_and(|(_, a)| hands(a.lines().next().unwrap_or(""), w))));
        // a same-file function handed one of them reads for the component when its result is used
        // (`return paintLook(props, frame, fps)`, `const runs = textRuns(owner, props)`), not when it only checks
        // (`bufferShape(props);`)
        let mut reads = body.to_string();
        for (i, _) in body.match_indices('(') {
            let f = &body[..i];
            let start = f.rfind(|c: char| !is_id(c)).map_or(0, |j| j + 1);
            let f = &f[start..];
            let args = body[i + 1..].lines().next().unwrap_or("");
            let dropped = body[..start].rsplit('\n').next().is_some_and(|l| l.trim().is_empty());
            if f.is_empty() || f == name || dropped || !whole.iter().any(|w| hands(args, w)) {
                continue;
            }
            reads.push_str(&body_of(src, f));
        }
        // a parent that finds this element among its children (`child.type === BufferMesh`) and hands its props to a
        // function: that function reads them (<Mesh>'s geometry children)
        for (i, _) in src.match_indices(&format!("type === {name})")) {
            let next = src[i..].lines().nth(1).unwrap_or("");
            if let Some((call, _)) = next.split_once(".props") {
                let call = call.split_once('(').map_or("", |(f, _)| f);
                reads.push_str(&body_of(src, &call[call.rfind(|c: char| !is_id(c)).map_or(0, |j| j + 1)..]));
            }
        }
        let body = reads.lines().filter(|l| !l.contains("throw ")).collect::<Vec<_>>().join("\n");
        into.push(Component { name: name.to_string(), file: file.to_path_buf(), props, kinds, body, forwards });
    }
}

/// `function f(...) { ... }`'s body in `src`, a newline first (empty when there is none)
fn body_of(src: &str, f: &str) -> String {
    let Some(at) = src.find(&format!("function {f}(")).filter(|_| !f.is_empty()) else { return String::new() };
    src[at..].find('{').map_or(String::new(), |o| format!("\n{}", block(&src[at..], o)))
}

/// A character of an identifier
pub fn is_id(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// `arg` hands on `param` whole: as itself, or as an argument of a helper, never only its members (`param.x`)
fn hands(arg: &str, param: &str) -> bool {
    arg.match_indices(param).any(|(i, _)| !arg[..i].ends_with(|c: char| is_id(c) || c == '.') && !arg[i + param.len()..].starts_with(|c: char| is_id(c) || c == '.'))
}

/// One base an interface or a component's props type names: `Name`, or `Omit<Name, "a" | "b">` (its members less those)
#[derive(Clone)]
pub struct Base {
    pub name: String,
    pub omit: Vec<String>,
}

/// The bases in a type's text: `A & B`, `A, B` (an extends list), `Omit<A, "x">`, generics after a name skipped
pub fn bases(ty: &str) -> Vec<Base> {
    let mut out = vec![];
    let mut rest = ty.trim();
    while !rest.is_empty() {
        rest = rest.trim_start_matches(|c: char| c == '&' || c == ',' || c.is_whitespace());
        if let Some(inner) = rest.strip_prefix("Omit<") {
            let end = inner.find('>').unwrap_or(inner.len());
            let (name, omitted) = inner[..end].split_once(',').unwrap_or((&inner[..end], ""));
            let omit = omitted.split('|').map(|s| s.trim().trim_matches('"')).filter(|s| !s.is_empty()).map(String::from).collect();
            out.push(Base { name: ident_at(name.trim()).to_string(), omit });
            rest = &inner[(end + 1).min(inner.len())..];
            continue;
        }
        let name = ident_at(rest);
        if name.is_empty() {
            break;
        }
        out.push(Base { name: name.to_string(), omit: vec![] });
        rest = &rest[name.len()..];
        // a name's own generics (`Foo<Bar>`) are not bases
        if let Some(g) = rest.strip_prefix('<') {
            let mut depth = 1;
            let end = g.char_indices().find(|&(_, c)| {
                depth += (c == '<') as i32 - (c == '>') as i32;
                depth == 0
            });
            rest = end.map_or("", |(i, _)| &g[i + 1..]);
        }
    }
    out
}

/// An interface's members with those of everything it extends, by name: the one in `file` first, then one in its
/// folder, then any (two SDK files may each have a SpotLightProps)
pub fn all_members(base: &Base, file: &Path, ifaces: &BTreeMap<String, Vec<(PathBuf, Iface)>>, depth: usize) -> Vec<String> {
    let Some(found) = ifaces.get(&base.name).filter(|_| depth < 8) else { return vec![] };
    let pick = found.iter().find(|(f, _)| f == file).or_else(|| found.iter().find(|(f, _)| f.parent() == file.parent())).unwrap_or(&found[0]);
    let mut out = pick.1.members.clone();
    for e in &pick.1.extends {
        out.extend(all_members(e, &pick.0, ifaces, depth + 1));
    }
    out.retain(|m| !base.omit.contains(m));
    out.sort();
    out.dedup();
    out
}
