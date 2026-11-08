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
export const __raeaChain = (loc: string): string => {
  const up = read();
  return up ? `${loc}|${up}` : loc;
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
    for s in &parsed.program.body {
        let Statement::ImportDeclaration(d) = s else { continue };
        let local = d.source.value.starts_with('.');
        for sp in d.specifiers.iter().flatten() {
            imports.insert(sp.local().name.to_string(), local);
        }
    }
    let mut v = Stamper { src: source, rel, imports, parents: Vec::new(), edits: Vec::new(), lines: line_starts(source) };
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

impl<'a> Visit<'a> for Stamper<'_> {
    fn visit_jsx_element(&mut self, it: &JSXElement<'a>) {
        let open = &it.opening_element;
        let mut kind = Kind::Open;
        match &open.name {
            JSXElementName::Identifier(id) if HTML.contains(&id.name.as_str()) => {
                let text = format!(" data-raea-loc={} data-raea-use={{__raeaUse()}}", json(&self.loc(it.span.start as usize)));
                let at = open.name.span().end as usize;
                self.edits.push((at, at, text));
            }
            JSXElementName::Identifier(_) => {}
            JSXElementName::IdentifierReference(r) => {
                let name = r.name.as_str();
                // the edit's own: defined here (not imported) or imported from a relative path
                let own = self.imports.get(name).copied().unwrap_or(true);
                if !own {
                    kind = Kind::Package;
                } else if self.parents.last() != Some(&Kind::Package) {
                    let (start, end) = (it.span.start as usize, it.span.end as usize);
                    // a key belongs on what's in the list: the wrapper
                    let key = open.attributes.iter().find_map(|a| match a {
                        JSXAttributeItem::Attribute(x) if matches!(&x.name, JSXAttributeName::Identifier(n) if n.name == "key") => Some(x.span),
                        _ => None,
                    });
                    let mut wrap = format!("<__RaeaAt value={{__raeaChain({})}}", json(&self.loc(start)));
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
        assert!(out.contains(r#"<__RaeaAt value={__raeaChain("p.tsx:3:38")} key={i}><Card  n={i} /></__RaeaAt>"#), "{out}");
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
