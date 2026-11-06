//! `raea doctor [--fix]` (docs/skills.feature SK10): drift between a project's files and what this raea reads. Old brief
//! files raea no longer reads, a workspace or project file missing what this version writes (or not included from the
//! repo's git config), and an agent's copy of the raea skill from another version. `raea context` runs the same checks
//! and prints CONTEXT_STALE when any finds something, so they stay cheap: file reads and git config, no engine.
use super::*;
use crate::workspace::{get, git, include, new_id, set};

/// One thing doctor found, and whether `--fix` can fix it.
pub struct Finding {
    pub says: String,
    fix: Fix,
}

enum Fix {
    No,
    /// a key missing from a git-config file: (file, key, value)
    Set(PathBuf, String, String),
    /// the include in a repo's config that makes `git config` read its committed file: (repo, file)
    Include(PathBuf, PathBuf),
    /// an agent's skills reinstalled from this raea
    Skills(String),
}

impl Finding {
    pub fn fixable(&self) -> bool {
        !matches!(self.fix, Fix::No)
    }
}

/// Old brief files under a folder (raea reads only brief.ts, script.ts and taste.ts since 11-05), a few levels down.
fn old_briefs(dir: &Path, depth: usize, out: &mut Vec<Finding>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if depth > 0 && !name.starts_with('.') && name != "node_modules" && name != "media" && name != "out" {
                old_briefs(&p, depth - 1, out);
            }
        } else if [".brief", ".board"].iter().any(|x| name.ends_with(x)) {
            out.push(Finding { says: format!("{}: an old brief file, which raea no longer reads; rewrite it as brief.ts (export default brief({{...}}) from \"raea/brief\") and delete it", p.display()), fix: Fix::No });
        }
    }
}

/// A committed git-config file's keys this version writes, and its include from the repo's config.
fn format(repo: &Path, file: &Path, keys: &[(&str, String)], out: &mut Vec<Finding>) {
    for (k, v) in keys {
        if get(file, k).is_none() {
            out.push(Finding { says: format!("{}: no {k} (an older format); --fix sets it to {v}", file.display()), fix: Fix::Set(file.to_path_buf(), k.to_string(), v.clone()) });
        }
    }
    // only a folder that is its own repo includes its file (a workspace, a split project); an inline project shares
    // the workspace's repo, whose config must not take one project's keys
    let own = git(repo, &["rev-parse", "--show-toplevel"]).ok().and_then(|t| Path::new(&t).canonicalize().ok()) == repo.canonicalize().ok();
    let first = keys.first().map(|(k, _)| *k).unwrap_or_default();
    let read = git(repo, &["config", first]).ok().filter(|v| !v.is_empty());
    if own && read.is_none() && get(file, first).is_some() {
        out.push(Finding { says: format!("{}: git config doesn't read it (no include in the repo's config); --fix adds it", file.display()), fix: Fix::Include(repo.to_path_buf(), file.to_path_buf()) });
    }
}

/// The `version:` in a SKILL.md's front matter.
fn skill_version(md: &Path) -> Option<String> {
    let text = std::fs::read_to_string(md).ok()?;
    let front = text.strip_prefix("---\n")?.split("\n---\n").next()?;
    front.lines().find_map(|l| l.strip_prefix("version:")).map(|v| v.trim().to_string())
}

/// What doctor finds from a folder: `skills` are (agent, its skills folder), `version` this raea's.
pub fn findings(from: &Path, skills: &[(&str, PathBuf)], version: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    let ws = up(from, ".raea/workspace");
    let project = up(from, ".raea/project");
    let base = project.clone().or_else(|| ws.clone()).unwrap_or_else(|| from.to_path_buf());
    old_briefs(&base, if project.is_some() { 2 } else { 1 }, &mut out);
    if let Some(ws) = &ws {
        let name = ws.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let keys = [("workspace.id", new_id("ws")), ("workspace.name", name), ("workspace.layout", "monorepo".to_string())];
        format(ws, &ws.join(".raea/workspace"), &keys, &mut out);
    }
    if let Some(p) = &project {
        let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let keys = [("project.id", new_id("pj")), ("project.name", name)];
        format(p, &p.join(".raea/project"), &keys, &mut out);
    }
    for (agent, dir) in skills {
        let md = dir.join("raea/SKILL.md");
        if !md.is_file() {
            continue;
        }
        let have = skill_version(&md).unwrap_or_else(|| "unversioned".into());
        if have != version {
            out.push(Finding { says: format!("{}: the raea skill {agent} reads is {have}, this raea is {version}; --fix reinstalls it", md.display()), fix: Fix::Skills(agent.to_string()) });
        }
    }
    out
}

/// `raea doctor [--fix] [DIR]`: what it found and, with --fix, what it fixed. `reinstall` puts an agent's skills back
/// (the raea CLI's `skills install`).
pub fn doctor(args: &[String], skills: &[(&str, PathBuf)], version: &str, reinstall: &dyn Fn(&str) -> R<String>) -> R<String> {
    let fix = args.iter().any(|a| a == "--fix");
    let from = PathBuf::from(args.iter().find(|a| !a.starts_with("--")).map(String::as_str).unwrap_or("."));
    let found = findings(&from, skills, version);
    if found.is_empty() {
        return Ok("CHECKED  nothing to fix: this folder's files and the agents' raea skill match this raea".into());
    }
    let mut lines = Vec::new();
    let mut fixed = 0;
    for f in &found {
        let done = match (&f.fix, fix) {
            (Fix::Set(file, k, v), true) => set(file, k, v).map(|_| true),
            (Fix::Include(repo, file), true) => include(repo, file).map(|_| true),
            (Fix::Skills(agent), true) => reinstall(agent).map(|_| true),
            _ => Ok(false),
        };
        match done {
            Ok(true) => {
                fixed += 1;
                lines.push(format!("  fixed  {}", f.says));
            }
            Ok(false) => lines.push(format!("  {}  {}", if f.fixable() { "fix  " } else { "by hand" }, f.says)),
            Err(e) => lines.push(format!("  failed {}: {e}", f.says)),
        }
    }
    // a workspace's own files changed: commit them, as every change raea makes there is
    if fix && fixed > 0 {
        for marker in [".raea/workspace", ".raea/project"] {
            if let Some(repo) = up(&from, marker) {
                let _ = crate::workspace::commit(&repo, &[marker], "doctor --fix");
            }
        }
    }
    let can = found.iter().filter(|f| f.fixable()).count();
    let head = if fix { format!("APPLIED  {fixed} of {} fixed", found.len()) } else { format!("CHECKED  {} to fix ({can} by raea doctor --fix)", found.len()) };
    Ok(format!("{head}\n{}", lines.join("\n")))
}
