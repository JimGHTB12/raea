//! The engine's readers scanned: every string literal that could be a prop's name in the Rust that turns a part's
//! data-props into rows and entities (the runtime crates, engine/render, the systems crates and core/scene's helpers such as light_power; runtime/look reads
//! <Look>'s by name). A prop whose name is one of them is read; one whose name is none of them can't be.

use std::collections::BTreeSet;
use std::path::Path;

/// Where data-props are read, from the repo's root
pub const READERS: &[&str] = &["runtime", "engine/render/src", "systems", "core/scene/src"];

/// The identifier-shaped string literals (`"castShadow"`, `"x"`) in the readers' .rs files
pub fn literals(root: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack: Vec<_> = READERS.iter().map(|r| root.join(r)).collect();
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let Ok(src) = std::fs::read_to_string(&p) else { continue };
                for piece in src.split('"').skip(1).step_by(2) {
                    if !piece.is_empty() && piece.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        out.insert(piece.to_string());
                    }
                }
            }
        }
    }
    out
}
