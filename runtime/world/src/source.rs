//! Where a thing was written (the Source component): the source stamps an element or part carries (engine/bundle
//! stamp.rs: data-raea-loc, data-raea-use, data-raea-idx), and the component's name read from the code at its nearest
//! use, or the tag written at its place when it was drawn in no component of the edit's own.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use raea_core_facts::Source;
use serde_json::{Map, Value};

thread_local! {
    /// each source file's lines, read once a read
    static FILES: RefCell<HashMap<PathBuf, String>> = RefCell::new(HashMap::new());
}

fn text(props: &Map<String, Value>, k: &str) -> Option<String> {
    props.get(k).map(raea_core_facts::text_of)
}

/// "file:line:col" as its file, line and column (1-based).
fn place(loc: &str) -> Option<(String, u32, u32)> {
    let mut it = loc.rsplitn(3, ':');
    let col = it.next()?.parse().ok()?;
    let line = it.next()?.parse().ok()?;
    Some((it.next()?.to_string(), line, col))
}

/// The JSX tag written at `file:line:col` under `dir`: `<Rider` gives "Rider".
pub(crate) fn tag_at(dir: &Path, loc: &str) -> Option<String> {
    let (file, line, col) = place(loc)?;
    let path = dir.join(&file);
    FILES.with(|f| {
        let mut f = f.borrow_mut();
        if !f.contains_key(&path) {
            let read = std::fs::read_to_string(&path).unwrap_or_default();
            f.insert(path.clone(), read);
        }
        let text = &f[&path];
        // columns count UTF-16 units, as TypeScript does
        let units: Vec<u16> = text.lines().nth(line.checked_sub(1)? as usize)?.encode_utf16().collect();
        let rest = String::from_utf16_lossy(units.get(col.checked_sub(1)? as usize..)?);
        let tag = rest.strip_prefix('<')?;
        let name = &tag[..tag.find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '.')).unwrap_or(tag.len())];
        (!name.is_empty()).then(|| name.to_string())
    })
}

/// A stamped element's or part's Source; None when it carries no stamp (raea's own elements, a package's).
pub(crate) fn of(props: &Map<String, Value>, dir: &Path) -> Option<Source> {
    let loc = text(props, "data-raea-loc")?;
    let uses = text(props, "data-raea-use").unwrap_or_default();
    let (file, line, _) = place(&loc)?;
    let near = uses.split('|').find(|u| !u.is_empty());
    let component = near.and_then(|u| tag_at(dir, u)).or_else(|| tag_at(dir, &loc));
    Some(Source { file: Some(file), line: Some(line), component, key: raea_sys_layers::key_from_props(props) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stamp_reads_as_its_file_line_and_the_component_used_there() {
        let dir = std::env::temp_dir().join(format!("raea-world-source-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("promo.tsx"), "export const P = () => (\n  <Stage>\n    <Rider seat=\"a\" />\n  </Stage>\n);\n").unwrap();
        std::fs::write(dir.join("rider.tsx"), "export const Rider = () => <Mesh3d name=\"body\" />;\n").unwrap();
        let mut props = Map::new();
        props.insert("data-raea-loc".into(), Value::String("rider.tsx:1:28".into()));
        props.insert("data-raea-use".into(), Value::String("promo.tsx:3:5".into()));
        let s = of(&props, &dir).unwrap();
        assert_eq!((s.file.as_deref(), s.line), (Some("rider.tsx"), Some(1)));
        assert_eq!(s.component.as_deref(), Some("Rider"), "the use's component, not the tag inside it");
        assert!(s.key.as_deref().is_some_and(|k| k.starts_with("rider.tsx:1:28")), "{:?}", s.key);
        // drawn in no component of the edit's: the tag at its place
        props.remove("data-raea-use");
        assert_eq!(of(&props, &dir).unwrap().component.as_deref(), Some("Mesh3d"));
        assert!(of(&Map::new(), &dir).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
