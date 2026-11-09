//! What in a project isn't connected to raea's project system (docs/projects.feature PJ8), each with its file and line,
//! what's wrong and the fix; `raea doctor --fix` applies the ones it can (`apply`).
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::{File, How, Prop, Registry, is_path, resolve, stem};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Level {
    /// not connected: something reads it wrong or not at all
    Warn,
    /// said so the person knows (a size only the render knows), not wrong
    Info,
}

/// What `--fix` does.
#[derive(Debug, Clone)]
pub enum Fix {
    /// registers a default export: an import line at the top (when one is missing) and a registerRoot at the end
    Register { file: PathBuf, import: Option<String>, root: String },
    /// a controller moved to the workspace's (or the project's) controllers/
    Move { from: PathBuf, to: PathBuf },
}

#[derive(Debug, Clone)]
pub struct Lint {
    /// relative to the scanned folder ("" for the folder itself)
    pub file: String,
    pub line: usize,
    pub rule: &'static str,
    pub level: Level,
    pub says: String,
    pub fix: String,
    pub action: Option<Fix>,
}

impl Lint {
    pub fn json(&self) -> Value {
        json!({
            "file": self.file, "line": self.line, "rule": self.rule,
            "level": if self.level == Level::Warn { "warn" } else { "info" },
            "says": self.says, "fix": self.fix, "fixable": self.action.is_some(),
        })
    }

    /// file:line: what (the fix)
    pub fn line(&self) -> String {
        let at = if self.file.is_empty() { ".".to_string() } else if self.line > 0 { format!("{}:{}", self.file, self.line) } else { self.file.clone() };
        format!("{at}: {} ({})", self.says, self.fix)
    }
}

/// A path with the home folder as ~, as a person reads it.
fn tilde(p: &Path) -> String {
    match std::env::var_os("HOME").map(PathBuf::from) {
        Some(h) if p.starts_with(&h) => format!("~/{}", p.strip_prefix(&h).unwrap_or(p).display()),
        _ => p.display().to_string(),
    }
}

fn warn(file: &str, line: usize, rule: &'static str, says: String, fix: impl Into<String>) -> Lint {
    Lint { file: file.into(), line, rule, level: Level::Warn, says, fix: fix.into(), action: None }
}

pub(super) fn lint(reg: &Registry, files: &[File], used_by: &BTreeMap<String, BTreeSet<String>>) -> Vec<Lint> {
    let mut out = Vec::new();
    if !reg.project && !(reg.compositions.is_empty() && reg.controllers.is_empty()) {
        out.push(warn("", 0, "not-a-project", "this folder isn't in a raea project (no .raea/project at or above it)".into(), "raea project new NAME and move the files in, or raea workspace adopt the folder"));
    }
    let comp_files: BTreeSet<&str> = reg.compositions.iter().map(|c| c.file.as_str()).collect();
    let controller_files: BTreeSet<&str> = reg.controllers.iter().map(|c| c.file.as_str()).collect();

    for f in files {
        if let Some(why) = &f.facts.broken {
            out.push(warn(&f.rel, 1, "broken", format!("doesn't parse: {why}"), "fix the syntax: nothing reads a file that doesn't parse"));
            continue;
        }
        // a .tsx that is neither a composition nor reached from one
        if f.rel.ends_with(".tsx") && !comp_files.contains(f.rel.as_str()) && !used_by.contains_key(&f.rel) && !controller_files.contains(f.rel.as_str()) {
            out.push(warn(&f.rel, 1, "orphan", "isn't a composition, and no composition imports it".into(), "register it (a default export or a registerRoot <Composition>), import it from a composition, or delete it"));
        }
        for (spec, line) in f.facts.imports.iter().filter(|(s, _)| is_path(s)) {
            match resolve(&f.path, spec) {
                None => out.push(warn(&f.rel, *line, "unresolved", format!("imports {spec}, which isn't there"), "fix the path, or add the file")),
                Some(to) if !to.starts_with(&reg.dir) => out.push(warn(
                    &f.rel,
                    *line,
                    "outside",
                    format!("imports {} from outside the project", tilde(&to)),
                    "copy it into the project (raea part add, for raea's parts), so the project renders on its own",
                )),
                _ => {}
            }
        }
    }

    // one id, two compositions (a composition's versions share theirs)
    let mut by_id: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for c in &reg.compositions {
        by_id.entry(c.id.as_str()).or_default().insert(stem(&c.file));
    }
    for c in &reg.compositions {
        let names = &by_id[c.id.as_str()];
        if names.len() > 1 {
            let others: Vec<&str> = names.iter().map(String::as_str).filter(|n| *n != stem(&c.file)).collect();
            let who = match others.as_slice() {
                [one] => format!("{one} does"),
                [rest @ .., last] => format!("{} and {last} do", rest.join(", ")),
                [] => continue,
            };
            out.push(warn(&c.file, c.line, "duplicate-id", format!("registers \"{}\", as {who}", c.id), "give one of them its own id: `raea render --composition ID` picks by it"));
        }
    }

    for c in &reg.compositions {
        let f = files.iter().find(|f| f.rel == c.file).expect("a composition's file was read");
        match c.how {
            How::Registered => {
                let computed: Vec<String> = [("width", &c.width), ("height", &c.height), ("fps", &c.fps), ("durationInFrames", &c.frames)]
                    .into_iter()
                    .filter_map(|(k, v)| match v {
                        Some(Prop::Computed(e)) => Some(format!("{k} = {e}")),
                        None => Some(format!("{k} missing")),
                        _ => None,
                    })
                    .collect();
                if !computed.is_empty() {
                    out.push(Lint { file: c.file.clone(), line: c.line, rule: "computed", level: Level::Info, says: format!("<Composition id=\"{}\">: {}", c.id, computed.join(", ")), fix: "known when it renders; write them out to have them listed before".into(), action: None });
                }
            }
            How::Default => {
                let known = [&c.width, &c.height, &c.fps, &c.frames].iter().all(|v| matches!(v, Some(Prop::Num(_))));
                let action = match (&c.component, known) {
                    (Some(name), true) => {
                        let n = |v: &Option<Prop>| match v {
                            Some(Prop::Num(n)) => *n,
                            _ => 0.0,
                        };
                        let missing: Vec<&str> = ["Composition", "registerRoot"].into_iter().filter(|x| !f.facts.raea_names.iter().any(|h| h == x)).collect();
                        Some(Fix::Register {
                            file: f.path.clone(),
                            import: (!missing.is_empty()).then(|| format!("import {{ {} }} from \"raea\";\n", missing.join(", "))),
                            root: format!("\nregisterRoot(() => <Composition id=\"{}\" component={{{name}}} width={{{}}} height={{{}}} fps={{{}}} durationInFrames={{{}}} />);\n", c.id, n(&c.width), n(&c.height), n(&c.fps), n(&c.frames)),
                        })
                    }
                    _ => None,
                };
                let fix = match (&c.component, known) {
                    (Some(_), true) => "raea doctor --fix registers it with its last render's size, fps and length".to_string(),
                    (None, _) => "name the default export (export default function Name) and register it: registerRoot(() => <Composition id component width height fps durationInFrames />)".to_string(),
                    (Some(_), false) => "render it once (raea render), then raea doctor --fix registers it with that size, fps and length; or write its registerRoot".to_string(),
                };
                out.push(Lint { file: c.file.clone(), line: 1, rule: "unregistered", level: Level::Info, says: format!("registered from its default export as \"{}\": raea renders it at 1080x1920, 30 fps unless told", c.id), fix, action });
            }
        }
    }

    // controllers: where they're kept, one id each, and every override naming one that's there
    let ws = crate::up(&reg.dir, ".raea/workspace");
    let home = ws.clone().unwrap_or_else(|| reg.dir.clone()).join("controllers");
    let mut ids: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for c in &reg.controllers {
        let f = files.iter().find(|f| f.rel == c.file).expect("a controller's file was read");
        if let Some(id) = &c.id {
            ids.entry(id.clone()).or_default().push(&c.file);
        }
        let in_home = f.path.parent().and_then(Path::file_name).is_some_and(|n| n == "controllers");
        if !in_home {
            let id = c.id.clone().unwrap_or_else(|| stem(&c.file));
            let to = home.join(format!("{id}.ts"));
            out.push(Lint {
                file: c.file.clone(),
                line: c.line,
                rule: "controller-home",
                level: Level::Warn,
                says: "a controller outside controllers/: `raea controller` and the app don't read it".into(),
                fix: format!("move it to {}", to.display()),
                action: (!to.exists()).then(|| Fix::Move { from: f.path.clone(), to }),
            });
        }
    }
    for (id, at) in ids.iter().filter(|(_, at)| at.len() > 1) {
        for f in at {
            out.push(warn(f, 1, "controller-duplicate", format!("controller \"{id}\" is declared in {} files", at.len()), "keep one: the others' fields override nothing"));
        }
    }
    // the workspace's controllers too, read the same way, for overrides naming them
    let mut known: BTreeSet<String> = ids.keys().cloned().collect();
    for e in std::fs::read_dir(&home).into_iter().flatten().flatten() {
        let src = std::fs::read_to_string(e.path()).unwrap_or_default();
        if let Some(id) = super::read::read(&src, false).controller.and_then(|c| c.id) {
            known.insert(id);
        }
    }
    let mut named = BTreeSet::new();
    for (path, _) in crate::controllers::overrides(&reg.dir) {
        let id = path.split('.').next().unwrap_or_default().to_string();
        if !known.contains(&id) && named.insert(id.clone()) {
            out.push(warn(".raea/project", 0, "override-missing", format!("overrides controller \"{id}\", which isn't there"), format!("raea controller ls lists the ones there; remove the override with git config --file .raea/project --remove-section controller.{id}")));
        }
    }
    out
}

/// Applies a fix; what it did, for doctor's output.
pub fn apply(fix: &Fix) -> Result<String, String> {
    match fix {
        Fix::Register { file, import, root } => {
            let src = std::fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
            let text = format!("{}{}{root}", import.clone().unwrap_or_default(), src.trim_end_matches('\n'));
            std::fs::write(file, text).map_err(|e| format!("{}: {e}", file.display()))?;
            Ok(format!("registered {}", file.display()))
        }
        Fix::Move { from, to } => {
            if let Some(d) = to.parent() {
                std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
            }
            std::fs::rename(from, to).map_err(|e| format!("{}: {e}", from.display()))?;
            Ok(format!("moved {} to {}", from.display(), to.display()))
        }
    }
}
