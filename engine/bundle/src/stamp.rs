//! Source stamps (docs/editing.feature, E2): each element a composition's own files write carries the place in the code
//! that wrote it, so the render's manifest can say which JSX drew a run of text or a video, whatever made its words.
//!
//! The stamp is a source-to-source pass over a TSX file of the edit (its copy under the work folder's src/), before any
//! compile, that keeps every line where it was:
//! - an HTML element (`<h1>`, `<span>`, `<video>`…) gets `data-raea-loc="<file>:<line>:<col>"` and
//!   `data-raea-use={__raeaUse()}`: the use sites of the edit's components it was drawn inside, nearest first
//!   ("card.tsx:3:5|promo.tsx:12:7"), read from a context;
//! - a use of one of the edit's own components (`<Card>`, defined in the file or imported from a relative path) is
//!   wrapped in that context's provider with its place added, so a component used twice draws two different chains.
//!   Not inside an element of a package's component (raea's <Series>, <TransitionSeries>), which may read its children's
//!   types: there the use isn't wrapped, and its elements name only where they're written.
//!
//! Lines and columns are 1-based, columns in UTF-16 units as TypeScript counts them (the host's edit.ts finds the
//! element again with the TypeScript compiler). The helper module is written beside the entry (`raea-stamp.tsx`) and
//! imported on the file's first line, so no line moves.
use oxc::allocator::Allocator;
use oxc::ast::ast::*;
use oxc::ast_visit::{walk, Visit};
use oxc::parser::Parser;
use oxc::span::{GetSpan, SourceType};
use std::collections::HashMap;
use std::path::Path;

/// HTML elements stamped: what a composition's page draws. Other lowercase tags (bevy-react's `node`/`text`/`image`, an
/// R3F `mesh`, SVG's parts) are left alone, since their props may mean something to what draws them.
const HTML: &[&str] = &[
    "div", "span", "p", "h1", "h2", "h3", "h4", "h5", "h6", "section", "article", "header", "footer", "main", "aside", "nav", "figure", "figcaption", "blockquote", "pre", "code", "strong", "em", "b", "i", "u", "s", "small", "mark", "sub", "sup", "label", "ul", "ol", "li", "dl", "dt", "dd", "img", "video", "audio", "picture", "a", "button", "table", "tr", "td", "th", "svg", "canvas", "br", "hr",
];

/// raea's modules whose components draw parts (a Mesh3d, a Model, a Sprite, an Img): their uses are stamped as an HTML
/// element is, and the scene gives the part a LayerKey component from it (runtime/scene bevy_parts), so a 3D part is
/// a layer as an element is (docs/review.feature R2a). Never a prop the part's kind reads.
const PARTS3D: &[&str] = &["raea/3d", "raea/mesh", "raea/sprite", "raea"];

/// The parts among them, by the name they're exported as: what draws a thing on its own. Never a shape or a material
/// inside one (a Mesh3d reads its <Cuboid> child's props, and an extra prop there made it no shape at all).
/// raea's page parts too (an Img, a Video, an AbsoluteFill), whose element is written in the SDK, unstamped: their use
/// is the layer's place in the code; and an Audio, whose use is its cue's place (`raea world`, docs/direction.feature DR1)
const PART_NAMES: &[&str] = &["Mesh3d", "Model", "SceneRoot", "Entity", "Sprite", "Text3D", "Text2d", "ImagePlane", "ShaderMesh", "Plate", "Decal", "Splat", "Grass", "Mesh2d", "Screen", "Img", "Video", "OffthreadVideo", "AbsoluteFill", "Audio"];

/// Package components that read their children's types, whose children are left as written (a wrapper there would
/// hide a <Series.Sequence> from its <Series>, a <Cuboid> from its <Mesh3d>): every component of raea's 3D, material,
/// light and camera modules, and these by name
const READERS_MODULES: &[&str] = &["raea/3d", "raea/mesh", "raea/sprite", "raea/pbr", "raea/light", "raea/camera"];
const READERS: &[&str] = &["Series", "TransitionSeries"];

/// The helper module stamped files import: the context and its two readers.
pub const HELPER: &str = r#"// raea's source stamps (engine/bundle stamp.rs): the use sites of the edit's components, nearest first
import { createContext, useContext } from "react";
const At = createContext("");
const read = (): string => {
  try {
    return useContext(At);
  } catch {
    // JSX made outside a render (a module's constant): no component around it
    return "";
  }
};
export const __RaeaAt = At.Provider;
export const __raeaUse = (): string | undefined => read() || undefined;
export const __raeaChain = (loc: string, key?: unknown): string => {
  const at = key === undefined || key === null ? loc : `${loc}#${String(key)}`;
  const up = read();
  return up ? `${at}|${up}` : at;
};
"#;

/// `source` (TSX, or JSX when `jsx`), the file at `rel` (the edit's folder's path, with `/`), stamped; `helper` is
/// the import path of the helper module. None when there was nothing to stamp or it doesn't parse (it then compiles
/// as written, and says why there).
pub fn stamp(source: &str, rel: &str, jsx: bool, helper: &str) -> Option<String> {
    let alloc = Allocator::default();
    let st = if jsx { SourceType::jsx() } else { SourceType::tsx() };
    let parsed = Parser::new(&alloc, source, st).parse();
    if !parsed.diagnostics.is_empty() {
        return None;
    }
    let mut imports = HashMap::new();
    let mut parts3d = std::collections::HashSet::new();
    let mut readers = std::collections::HashSet::new();
    for s in &parsed.program.body {
        let Statement::ImportDeclaration(d) = s else { continue };
        let (local, module) = module_of(rel, d.source.value.as_str());
        let is3d = PARTS3D.contains(&module.as_str());
        let reads = READERS_MODULES.contains(&module.as_str());
        for sp in d.specifiers.iter().flatten() {
            imports.insert(sp.local().name.to_string(), local);
            let name = sp.local().name.to_string();
            if !local && (reads || READERS.contains(&name.as_str())) {
                readers.insert(name);
            }
            let exported = match sp {
                ImportDeclarationSpecifier::ImportSpecifier(x) => x.imported.name().to_string(),
                _ => String::new(),
            };
            if is3d && PART_NAMES.contains(&exported.as_str()) {
                parts3d.insert(sp.local().name.to_string());
            }
        }
    }
    let mut v = Stamper { src: source, rel, imports, parts3d, readers, parents: Vec::new(), edits: Vec::new(), lines: line_starts(source) };
    v.visit_program(&parsed.program);
    if v.edits.is_empty() {
        return None;
    }
    let mut out = String::with_capacity(source.len() + v.edits.len() * 64);
    out.push_str(&format!("import {{ __RaeaAt, __raeaUse, __raeaChain }} from {};", json(helper)));
    let mut edits = v.edits;
    // by place; at one place, a wrapper's close before a later element's open (stable sort keeps the visit's order)
    edits.sort_by_key(|e| e.0);
    let mut at = 0;
    for (pos, end, text) in edits {
        if pos < at {
            continue;
        }
        out.push_str(&source[at..pos]);
        out.push_str(&text);
        at = end;
    }
    out.push_str(&source[at..]);
    Some(out)
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// an HTML element, a fragment, or one of the edit's components: its children may be wrapped
    Open,
    /// a package's component: its children are left as written
    Package,
}

struct Stamper<'s> {
    src: &'s str,
    rel: &'s str,
    /// each imported name: true when it's from a relative path (the edit's own)
    imports: HashMap<String, bool>,
    /// the names imported from PARTS3D's modules
    parts3d: std::collections::HashSet<String>,
    /// the package components that read their children's types (READERS)
    readers: std::collections::HashSet<String>,
    parents: Vec<Kind>,
    /// (at, end, text): `src[at..end]` replaced by `text`
    edits: Vec<(usize, usize, String)>,
    lines: Vec<usize>,
}

impl Stamper<'_> {
    fn loc(&self, at: usize) -> String {
        let line = self.lines.partition_point(|&s| s <= at);
        let start = self.lines[line - 1];
        let col = self.src[start..at].encode_utf16().count() + 1;
        format!("{}:{line}:{col}", self.rel)
    }
}

impl Stamper<'_> {
    /// The source of an element's `key` value, as an expression: `{i}` gives `i`, `"a"` gives `"a"`
    fn key_expr(&self, open: &JSXOpeningElement) -> Option<String> {
        open.attributes.iter().find_map(|a| match a {
            JSXAttributeItem::Attribute(x) if matches!(&x.name, JSXAttributeName::Identifier(n) if n.name == "key") => match x.value.as_ref()? {
                JSXAttributeValue::ExpressionContainer(c) => c.expression.as_expression().map(|e| self.src[e.span().start as usize..e.span().end as usize].to_string()),
                JSXAttributeValue::StringLiteral(l) => Some(json(l.value.as_str())),
                _ => None,
            },
            _ => None,
        })
    }
}

impl<'a> Visit<'a> for Stamper<'_> {
    fn visit_jsx_element(&mut self, it: &JSXElement<'a>) {
        let open = &it.opening_element;
        let mut kind = Kind::Open;
        match &open.name {
            JSXElementName::Identifier(id) if HTML.contains(&id.name.as_str()) => {
                let mut text = format!(" data-raea-loc={} data-raea-use={{__raeaUse()}}", json(&self.loc(it.span.start as usize)));
                // its place in a list (a .map()'s key), so five cards drawn by one line have five keys
                if let Some(k) = self.key_expr(open) {
                    text.push_str(&format!(" data-raea-idx={{{k}}}"));
                }
                let at = open.name.span().end as usize;
                self.edits.push((at, at, text));
            }
            JSXElementName::Identifier(_) => {}
            JSXElementName::IdentifierReference(r) => {
                let name = r.name.as_str();
                // the edit's own: defined here (not imported) or imported from a relative path
                let own = self.imports.get(name).copied().unwrap_or(true);
                if !own {
                    // a package component that reads its children's types (a Series, a 3D part's shape and material)
                    // gets them as written; any other (an AbsoluteFill, a Sequence) is a plain element's parent
                    if self.readers.contains(name) {
                        kind = Kind::Package;
                    }
                    if self.parts3d.contains(name) {
                        let mut text = format!(" data-raea-loc={} data-raea-use={{__raeaUse()}}", json(&self.loc(it.span.start as usize)));
                        if let Some(k) = self.key_expr(open) {
                            text.push_str(&format!(" data-raea-idx={{{k}}}"));
                        }
                        let at = r.span.end as usize;
                        self.edits.push((at, at, text));
                    }
                } else if self.parents.last() != Some(&Kind::Package) {
                    let (start, end) = (it.span.start as usize, it.span.end as usize);
                    // a key belongs on what's in the list: the wrapper
                    let key = open.attributes.iter().find_map(|a| match a {
                        JSXAttributeItem::Attribute(x) if matches!(&x.name, JSXAttributeName::Identifier(n) if n.name == "key") => Some(x.span),
                        _ => None,
                    });
                    let idx = self.key_expr(open).map(|k| format!(", {k}")).unwrap_or_default();
                    let mut wrap = format!("<__RaeaAt value={{__raeaChain({}{idx})}}", json(&self.loc(start)));
                    if let Some(k) = key {
                        wrap.push(' ');
                        wrap.push_str(&self.src[k.start as usize..k.end as usize]);
                        self.edits.push((k.start as usize, k.end as usize, String::new()));
                    }
                    wrap.push('>');
                    self.edits.push((start, start, wrap));
                    self.edits.push((end, end, "</__RaeaAt>".into()));
                }
            }
            // <Foo.Bar>, <a:b>, <this>: a package's or unknown, left alone
            _ => kind = Kind::Package,
        }
        self.parents.push(kind);
        walk::walk_jsx_element(self, it);
        self.parents.pop();
    }

    fn visit_jsx_fragment(&mut self, it: &JSXFragment<'a>) {
        self.parents.push(Kind::Open);
        walk::walk_jsx_fragment(self, it);
        self.parents.pop();
    }
}

/// Whether an import from the file at `rel` (its path under src/) is the edit's own, and the module it names as an
/// edit would write it. The work folder's copies import raea's SDK at motion/ beside src/ ("raea/mesh" is
/// "../motion/mesh" there: fresh.rs rewrite_into), so a relative path is the edit's own only while it stays in src/,
/// and one into motion/ is raea's ("raea/mesh"); a bare one is a package's as it is.
fn module_of(rel: &str, spec: &str) -> (bool, String) {
    if !spec.starts_with('.') {
        return (false, spec.to_string());
    }
    let mut at: Vec<&str> = rel.split('/').collect();
    at.pop();
    let mut out = false;
    for part in spec.split('/') {
        match part {
            "." | "" => {}
            ".." if at.pop().is_none() => out = true,
            ".." => {}
            p if out => at.push(p),
            p => at.push(p),
        }
    }
    if !out {
        return (true, spec.to_string());
    }
    match at.split_first() {
        // "raea" itself is motion/prelude there (raea-runtime-bundle rewrite.rs)
        Some((&"motion", rest)) if rest.is_empty() || rest == ["index"] || rest == ["prelude"] => (false, "raea".into()),
        Some((&"motion", rest)) => (false, format!("raea/{}", rest.join("/"))),
        _ => (false, spec.to_string()),
    }
}

fn line_starts(s: &str) -> Vec<usize> {
    std::iter::once(0).chain(s.match_indices('\n').map(|(i, _)| i + 1)).collect()
}

fn json(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

/// Whether `file` is one of the edit's own sources under `src` (the work folder's copy of its folder), and its path
/// there with `/`.
pub fn own(file: &Path, src: &Path) -> Option<String> {
    let rel = file.strip_prefix(src).ok()?;
    if rel.components().any(|c| c.as_os_str() == "node_modules") {
        return None;
    }
    Some(rel.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const H: &str = "/w/raea-stamp.tsx";

    #[test]
    fn html_elements_carry_their_place_and_lines_stay() {
        let src = "export default function A() {\n  return <div>\n    <h1>Hi</h1>\n  </div>\n}\n";
        let out = stamp(src, "a.tsx", false, H).unwrap();
        assert_eq!(out.lines().count(), src.lines().count());
        assert!(out.contains(r#"<div data-raea-loc="a.tsx:2:10" data-raea-use={__raeaUse()}>"#), "{out}");
        assert!(out.contains(r#"<h1 data-raea-loc="a.tsx:3:5" data-raea-use={__raeaUse()}>Hi</h1>"#), "{out}");
        assert!(out.starts_with("import { __RaeaAt, __raeaUse, __raeaChain } from \"/w/raea-stamp.tsx\";export default"));
    }

    #[test]
    fn own_components_are_wrapped_with_their_use_and_keep_the_key() {
        let src = "import { Card } from \"./card\";\nimport { Series } from \"raea\";\nconst X = () => <>{[1, 2].map((i) => <Card key={i} n={i} />)}</>;\n";
        let out = stamp(src, "p.tsx", false, H).unwrap();
        assert!(out.contains(r#"<__RaeaAt value={__raeaChain("p.tsx:3:38", i)} key={i}><Card  n={i} /></__RaeaAt>"#), "{out}");
    }

    #[test]
    fn an_html_element_in_a_list_carries_its_key() {
        let src = "const X = () => <ul>{[\"a\", \"b\"].map((t) => <li key={t}>{t}</li>)}<li key=\"z\" /></ul>;\n";
        let out = stamp(src, "l.tsx", false, H).unwrap();
        assert!(out.contains(r#"<li data-raea-loc="l.tsx:1:44" data-raea-use={__raeaUse()} data-raea-idx={t} key={t}>"#), "{out}");
        assert!(out.contains(r#"data-raea-idx={"z"} key="z""#), "{out}");
    }

    #[test]
    fn a_3d_part_from_raeas_3d_modules_is_stamped_and_other_packages_are_not() {
        let src = "import { Mesh3d, Cuboid } from \"raea/mesh\";\nimport { Model } from \"raea/3d\";\nimport { Series } from \"raea\";\nconst X = () => <Series>{[0, 1].map((i) => <Mesh3d key={i} />)}<Model src=\"a.glb\" /></Series>;\n";
        let out = stamp(src, "s.tsx", false, H).unwrap();
        assert!(out.contains(r#"<Mesh3d data-raea-loc="s.tsx:4:44" data-raea-use={__raeaUse()} data-raea-idx={i} key={i} />"#), "{out}");
        assert!(out.contains(r#"<Model data-raea-loc="s.tsx:4:64" data-raea-use={__raeaUse()} src="a.glb" />"#), "{out}");
        assert!(out.contains("<Series>{"), "{out}");
        // a shape inside a part is the part's, left as written
        let out = stamp("import { Mesh3d, Cuboid } from \"raea/mesh\";\nconst Y = () => <Mesh3d><Cuboid /></Mesh3d>;\n", "s.tsx", false, H).unwrap();
        assert!(out.contains("<Cuboid />"), "{out}");
    }

    #[test]
    fn the_work_folders_sdk_imports_are_raeas_not_the_edits() {
        assert_eq!(module_of("comp.tsx", "./card"), (true, "./card".into()));
        assert_eq!(module_of("lib/x.tsx", "../card"), (true, "../card".into()));
        assert_eq!(module_of("comp.tsx", "../motion/mesh"), (false, "raea/mesh".into()));
        assert_eq!(module_of("lib/x.tsx", "../../motion"), (false, "raea".into()));
        assert_eq!(module_of("comp.tsx", "../motion/prelude"), (false, "raea".into()));
        assert_eq!(module_of("comp.tsx", "raea/3d"), (false, "raea/3d".into()));
        // as the work folder has it: the SDK's parts stamped, its shapes and own wrappers not
        let src = "import { Cuboid, Mesh3d } from \"../motion/mesh\";\nimport { Camera3d } from \"../motion/camera\";\nconst X = () => <><Camera3d /><Mesh3d><Cuboid /></Mesh3d></>;\n";
        let out = stamp(src, "comp.tsx", false, H).unwrap();
        assert!(out.contains("<><Camera3d /><Mesh3d data-raea-loc=\"comp.tsx:3:31\" data-raea-use={__raeaUse()}><Cuboid /></Mesh3d></>"), "{out}");
    }

    #[test]
    fn a_plain_package_component_is_a_parent_like_an_element() {
        let src = "import { AbsoluteFill } from \"raea\";\nimport { Card } from \"./card\";\nconst X = () => <AbsoluteFill><Card /></AbsoluteFill>;\n";
        let out = stamp(src, "p.tsx", false, H).unwrap();
        assert!(out.contains(r#"<AbsoluteFill data-raea-loc="p.tsx:3:17" data-raea-use={__raeaUse()}><__RaeaAt value={__raeaChain("p.tsx:3:31")}><Card /></__RaeaAt></AbsoluteFill>"#), "{out}");
    }

    #[test]
    fn a_package_components_children_are_left_as_written() {
        let src = "import { Series } from \"raea\";\nfunction Own() { return null }\nconst X = () => <Series><Own /></Series>;\nconst Y = () => <div><Own /></div>;\n";
        let out = stamp(src, "p.tsx", false, H).unwrap();
        assert!(out.contains("<Series><Own /></Series>"), "{out}");
        assert!(out.contains(r#"<__RaeaAt value={__raeaChain("p.tsx:4:22")}><Own /></__RaeaAt>"#), "{out}");
    }

    #[test]
    fn other_tags_and_unparsable_files_are_left_alone() {
        assert_eq!(stamp("const a = <mesh><boxGeometry /></mesh>;\n", "a.tsx", false, H), None);
        assert_eq!(stamp("const a = <div>;\n", "a.tsx", false, H), None);
    }

    #[test]
    fn columns_count_as_typescript_does() {
        let out = stamp("const a = \"é😀\"; const b = <p>x</p>;\n", "a.tsx", false, H).unwrap();
        // é is 1 unit, 😀 is 2: the <p> is at UTF-16 column 28
        assert!(out.contains(r#"data-raea-loc="a.tsx:1:28""#), "{out}");
    }

    #[test]
    fn only_the_edits_own_files() {
        assert_eq!(own(Path::new("/w/src/lib/card.tsx"), Path::new("/w/src")).as_deref(), Some("lib/card.tsx"));
        assert_eq!(own(Path::new("/w/src/node_modules/x/a.tsx"), Path::new("/w/src")), None);
        assert_eq!(own(Path::new("/w/motion/index.tsx"), Path::new("/w/src")), None);
    }
}
