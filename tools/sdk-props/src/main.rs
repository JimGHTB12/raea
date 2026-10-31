//! Every exported prop of the SDK's engine components, read or not (docs/sdk.feature S8c). A component is one that
//! hands its props to the engine (node3d); each member of its props type (and of what that extends) is read when its
//! own body uses it beyond a check (a line without `throw`), or when the body forwards the props object and the
//! engine's readers name it (reads.rs). The rest are unread: typed for an author, ignored by the engine.
//!
//!   raea-sdk-props [ROOT]           the unread, one a line (component.prop, file), then the count
//!   raea-sdk-props --check [ROOT]   the ratchet (tools/agents/check, L0): no more unread than tools/sdk-props.max

mod reads;
mod scan;

use std::collections::BTreeMap;
use std::path::PathBuf;

/// `word` in `text` as a whole identifier
fn mentions(text: &str, word: &str) -> bool {
    text.match_indices(word).any(|(i, _)| !text[..i].ends_with(scan::is_id) && !text[i + word.len()..].starts_with(scan::is_id))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check = args.iter().any(|a| a == "--check");
    let root = PathBuf::from(args.iter().find(|a| !a.starts_with("--")).map(String::as_str).unwrap_or("."));
    let mut ifaces = BTreeMap::new();
    let mut comps = vec![];
    for f in scan::sources(&root) {
        let Ok(src) = std::fs::read_to_string(&f) else { continue };
        let rel = f.strip_prefix(&root).unwrap_or(&f);
        scan::interfaces(rel, &src, &mut ifaces);
        scan::components(rel, &src, &mut comps);
    }
    let engine = reads::literals(&root);
    let (mut props, mut unread) = (0, vec![]);
    for c in &comps {
        let mut all: Vec<String> = c.props.iter().flat_map(|b| scan::all_members(b, &c.file, &ifaces, 0)).collect();
        all.sort();
        all.dedup();
        for m in all {
            if m == "children" {
                continue;
            }
            props += 1;
            if !(mentions(&c.body, &m) || c.forwards && engine.contains(&m)) {
                unread.push(format!("{}.{m} ({}, {})", c.name, c.file.display(), c.kinds.join(" ")));
            }
        }
    }
    unread.sort();
    unread.dedup();
    let line = format!("sdk-props: {props} props on {} engine components, {} unread", comps.len(), unread.len());
    if !check {
        for u in &unread {
            println!("{u}");
        }
        println!("{line}");
        return;
    }
    let max_file = root.join("tools/sdk-props.max");
    let max: usize = std::fs::read_to_string(&max_file).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    if unread.len() > max {
        eprintln!("{line}: more than tools/sdk-props.max ({max}). A prop the engine never reads is a bug, not a type: read it or remove it (raea-sdk-props lists them)");
        std::process::exit(1);
    }
    if unread.len() < max {
        println!("{line} (max {max}: lower tools/sdk-props.max to {} in this commit)", unread.len());
    } else {
        println!("{line} (max {max})");
    }
}
