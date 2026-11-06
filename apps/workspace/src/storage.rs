//! A project changing storage with its history (docs/projects.feature PJ4): `raea project split NAME` makes an inline
//! project its own repo, a submodule, from `git subtree split` (its commits, rewritten to the folder); `raea project
//! join NAME` brings a submodule back in as a folder, its history merged under projects/NAME (`git subtree add`).
//! .raea/workspace and .gitmodules change in the same commit.
use super::*;
use workspace::{commit, get, git, include, set};

fn workspace_and(name: &str) -> R<(PathBuf, String, PathBuf)> {
    let here = std::env::current_dir().map_err(|e| e.to_string())?;
    let ws = workspace::up(&here, ".raea/workspace").ok_or("not in a workspace")?;
    let key = workspace::slug(name);
    let rel = get(&ws.join(".raea/workspace"), &format!("project.{key}.path")).ok_or_else(|| format!("no project {name} in this workspace: raea workspace lists them"))?;
    let dir = ws.join(&rel);
    Ok((ws, rel, dir))
}

fn storage_of(ws: &Path, name: &str) -> String {
    get(&ws.join(".raea/workspace"), &format!("project.{}.storage", workspace::slug(name))).unwrap_or_else(|| "inline".into())
}

fn clean(ws: &Path) -> R<()> {
    if !git(ws, &["status", "--porcelain", "--untracked-files=no"])?.is_empty() {
        return Err("the workspace has changes not yet committed: commit or drop them first, so the move is one change".into());
    }
    Ok(())
}

/// `raea project split NAME [--remote URL]`: an inline project made a submodule, its history kept.
pub fn split(name: &str, remote: Option<&str>) -> R<String> {
    let (ws, rel, dir) = workspace_and(name)?;
    if storage_of(&ws, name) == "submodule" {
        return Err(format!("{name} is a submodule already"));
    }
    clean(&ws)?;
    let branch = format!("raea-split-{}", workspace::slug(name));
    let tip = git(&ws, &["subtree", "split", "-q", &format!("--prefix={rel}"), "-b", &branch])?;
    let n = git(&ws, &["rev-list", "--count", &branch])?;
    // the project's folder leaves the workspace's tree, then comes back as its own repo with that history
    git(&ws, &["rm", "-r", "-q", "--", &rel])?;
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    git(&dir, &["init", "-q"])?;
    git(&dir, &["fetch", "-q", &ws.to_string_lossy(), &branch])?;
    git(&dir, &["reset", "-q", "--hard", "FETCH_HEAD"])?;
    git(&ws, &["branch", "-q", "-D", &branch])?;
    if Path::new(&dir.join(".gitattributes")).is_file() || ws.join(".gitattributes").is_file() {
        // media stays in LFS: the workspace's attributes come along
        let _ = std::fs::copy(ws.join(".gitattributes"), dir.join(".gitattributes"));
        let _ = git(&dir, &["add", ".gitattributes"]).and_then(|_| git(&dir, &["commit", "-q", "-m", "raea: LFS for media/ and out/", "--", ".gitattributes"]));
    }
    let url = match remote {
        Some(u) => {
            git(&dir, &["remote", "add", "origin", u])?;
            git(&dir, &["push", "-q", "-u", "origin", "HEAD"])?;
            u.to_string()
        }
        None => format!("./{rel}"),
    };
    git(&ws, &["-c", "protocol.file.allow=always", "submodule", "add", "-q", &url, &rel])?;
    set(&ws.join(".raea/workspace"), &format!("project.{}.storage", workspace::slug(name)), "submodule")?;
    include(&dir, &dir.join(".raea/project"))?;
    commit(&ws, &[".raea/workspace", ".gitmodules", &rel], &format!("project {name} split into its own repo ({n} commits kept)"))?;
    Ok(format!("{}: {name} is a submodule now, its own repo with {n} commits (tip {})", dir.display(), &tip[..tip.len().min(9)]))
}

/// `raea project join NAME`: a submodule brought back in as a folder, its history merged under its path.
pub fn join(name: &str) -> R<String> {
    let (ws, rel, dir) = workspace_and(name)?;
    if storage_of(&ws, name) != "submodule" {
        return Err(format!("{name} is a folder in the workspace already"));
    }
    clean(&ws)?;
    if !git(&dir, &["status", "--porcelain"])?.is_empty() {
        return Err(format!("{name} has changes not yet committed: commit them first, so they come along"));
    }
    // the submodule's repo is kept aside while its folder leaves, then its history comes in under the same path
    let aside = std::env::temp_dir().join(format!("raea-join-{}-{}", workspace::slug(name), std::process::id()));
    let _ = std::fs::remove_dir_all(&aside);
    git(&ws, &["clone", "-q", "--no-local", &dir.to_string_lossy(), &aside.to_string_lossy()])?;
    git(&ws, &["submodule", "deinit", "-q", "-f", "--", &rel])?;
    git(&ws, &["rm", "-q", "-f", "--", &rel])?;
    let modules = PathBuf::from(git(&ws, &["rev-parse", "--path-format=absolute", "--git-common-dir"])?).join("modules").join(&rel);
    let _ = std::fs::remove_dir_all(&modules);
    let _ = std::fs::remove_dir_all(&dir);
    set(&ws.join(".raea/workspace"), &format!("project.{}.storage", workspace::slug(name)), "inline")?;
    let gitmodules = ws.join(".gitmodules");
    let mut paths = vec![".raea/workspace"];
    if gitmodules.is_file() {
        paths.push(".gitmodules");
    }
    commit(&ws, &paths, &format!("project {name} leaves its submodule"))?;
    git(&ws, &["subtree", "add", "-q", &format!("--prefix={rel}"), &aside.to_string_lossy(), "HEAD", "-m", &format!("raea: project {name} joined, its history under {rel}")])?;
    let _ = std::fs::remove_dir_all(&aside);
    Ok(format!("{}: {name} is a folder in the workspace now, its history merged", dir.display()))
}
