//! `raea next` (docs/skills.feature SK10): the two or three useful next steps for a project, read from its files: the
//! brief (what it still needs, from the raea CLI's brief doctor), the storyboard its modes want, its compositions, its
//! renders in out/ and their checks (`raea check ... -o out/NAME.check.json`). A menu for the agent or the person to
//! pick from; nothing runs.
use super::*;
use std::time::SystemTime;

/// What the raea CLI read from the project's brief.
pub struct Brief {
    /// what the brief still needs (empty when it's shaped)
    pub needs: Vec<String>,
    pub modes: Vec<String>,
}

fn modified(p: &Path) -> SystemTime {
    std::fs::metadata(p).and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH)
}

fn files(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.is_file() && p.to_string_lossy().ends_with(ext)).collect();
    v.sort_by_key(|p| std::cmp::Reverse(modified(p)));
    v
}

fn name(p: &Path) -> String {
    let full = p.to_string_lossy();
    full.rsplit(['/', '\\']).next().unwrap_or_default().to_string()
}

/// The P0 and P1 issues a check file holds, as its lines.
fn blocking(check: &Path) -> Vec<String> {
    let Ok(j) = std::fs::read_to_string(check).map_err(drop).and_then(|t| serde_json::from_str::<serde_json::Value>(&t).map_err(drop)) else { return vec![] };
    let mut out = Vec::new();
    for i in j["issues"].as_array().into_iter().flatten() {
        if let (Some(p @ ("P0" | "P1")), Some(says)) = (i["p"].as_str(), i["says"].as_str()) {
            out.push(format!("{p} {says}"));
        }
    }
    out
}

/// The steps, most useful first, at most three. `stale` is what `raea doctor` found here.
pub fn steps(dir: &Path, brief: &Brief, stale: usize) -> Vec<String> {
    let mut out = Vec::new();
    if stale > 0 {
        out.push(format!("raea doctor: {stale} thing(s) in this project's files or your raea skill don't match this raea (`raea doctor --fix`)"));
    }
    if let Some(first) = brief.needs.first() {
        out.push(format!("shape the brief: {first} (ask the person, or write your best guess and add it to `assumed`)"));
    }
    let boarded = brief.modes.iter().any(|m| ["pitch", "explain", "story"].contains(&m.as_str()));
    let scripts = files(dir, ".script.ts");
    if boarded && scripts.is_empty() {
        out.push("plan it: write the storyboard as NAME.script.ts beside the brief (`raea spec NAME.script.ts` checks it) and wait for the person's approval".into());
    }
    // the compositions as raea's project system reads them (PJ8), newest first; what isn't connected is a step before
    let reg = crate::registry::scan(dir);
    let loose: Vec<&crate::registry::Lint> = reg.warnings().collect();
    if let Some(first) = loose.first() {
        out.push(format!("connect the project's files: {} thing(s) aren't connected, first {} (`raea project scan` lists them, `raea doctor --fix` fixes what it can)", loose.len(), first.line()));
    }
    let comps = reg.composition_files();
    let renders = files(&dir.join("out"), ".mp4");
    match (comps.first(), renders.first()) {
        (None, _) => out.push("make the first item: one .tsx whose root registers a Composition (`raea guide craft` first)".into()),
        (Some(c), None) => out.push(format!("render {}: stills to look at first (`raea render {} --at 0,45,89 -o frames/`), then the video into out/", name(c), name(c))),
        (Some(c), Some(r)) if modified(c) > modified(r) => out.push(format!("render again: {} changed after {}", name(c), name(r))),
        (_, Some(r)) => {
            let check = r.with_extension("check.json");
            if !check.is_file() || modified(&check) < modified(r) {
                out.push(format!("check {}: `raea check out/{} -o out/{}` (add --manifest when the render wrote one)", name(r), name(r), name(&check)));
            } else {
                match blocking(&check).first() {
                    Some(issue) => out.push(format!("polish {}: {issue} (`raea guide polish`)", name(r))),
                    None => out.push(format!("show {}: post it to the project's feed (feed_post), or give the person its path, and read their notes", name(r))),
                }
            }
        }
    }
    out.truncate(3);
    out
}

/// `raea next`'s text.
pub fn text(dir: &Path, brief: &Brief, stale: usize) -> String {
    let s = steps(dir, brief, stale);
    let mut t = format!("raea next: {}\n", dir.display());
    for (i, line) in s.iter().enumerate() {
        t.push_str(&format!("{}. {line}\n", i + 1));
    }
    t.trim_end().to_string()
}
