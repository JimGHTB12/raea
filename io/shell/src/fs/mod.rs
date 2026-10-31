//! A project folder as the shell's filesystem (docs/media.feature, "The sandbox boundary and the action rows"): virtual
//! paths ("/a/b") are files under the project's root and nothing else; a symlink can't lead out of it, and `.raea/`
//! (where this keeps its records) isn't visible. Every command is one action row in `.raea/actions.jsonl`: the command,
//! its verb and arguments, the files it read and wrote (each written path's state before and after: content hash,
//! folder, link or absent), bytes, exit, time, error, and the project's state hash before and after (a chain over the
//! rows). Contents are kept by hash in `.raea/objects/`, so any command can be undone and redone.
//!
//!   fs.open {root} -> {id}; fs.begin {id, cmd}; fs.end {id, exit?, error?} -> row; fs.undo / fs.redo {id} -> row | null
//!   fs.stat / fs.lstat / fs.read / fs.write / fs.mkdir / fs.readdir / fs.rm / fs.cp / fs.mv / fs.chmod / fs.symlink /
//!   fs.link / fs.readlink / fs.realpath / fs.utimes / fs.exists / fs.paths / fs.log

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io::Write as _;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use serde_json::{Value, json};
use sha2::Digest as _;

use raea_core_calls::{R, dec64, enc64, s};

mod calls;

pub(crate) use calls::calls;

const META: &str = ".raea";

fn projects() -> &'static Mutex<(u64, HashMap<u64, Project>)> {
    static P: OnceLock<Mutex<(u64, HashMap<u64, Project>)>> = OnceLock::new();
    P.get_or_init(|| Mutex::new((0, HashMap::new())))
}

struct Group {
    cmd: String,
    t0: Instant,
    /// each written path's state before the command first touched it
    before: BTreeMap<String, Value>,
    read: BTreeSet<String>,
    bytes: u64,
}

struct Project {
    root: PathBuf,
    group: Option<Group>,
    seq: u64,
    state: String,
    done: Vec<u64>,
    redo: Vec<u64>,
}

/// A Node-style error message, as just-bash's filesystems throw them.
fn err(code: &str, what: &str, op: &str, path: &str) -> String {
    format!("{code}: {what}, {op} '{path}'")
}

fn hash_hex(b: &[u8]) -> String {
    sha2::Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect()
}

/// A virtual path made absolute and lexically clean ("/a/../b" is "/b"; nothing goes above "/").
fn clean(p: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for c in Path::new(p).components() {
        match c {
            Component::Normal(x) => parts.push(x.to_str().unwrap_or("")),
            Component::ParentDir => {
                parts.pop();
            }
            _ => {}
        }
    }
    format!("/{}", parts.join("/"))
}

#[cfg(unix)]
fn mode_of(m: &fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt as _;
    m.permissions().mode() & 0o7777
}
#[cfg(not(unix))]
fn mode_of(m: &fs::Metadata) -> u32 {
    if m.is_dir() { 0o755 } else { 0o644 }
}
#[cfg(unix)]
fn set_mode(p: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(p, fs::Permissions::from_mode(mode))
}
#[cfg(not(unix))]
fn set_mode(_: &Path, _: u32) -> std::io::Result<()> {
    Ok(())
}

impl Project {
    fn real(&self, v: &str) -> R<PathBuf> {
        if v.contains('\0') {
            return Err(err("EINVAL", "path contains a null byte", "open", &v.replace('\0', "\\0")));
        }
        let v = clean(v);
        if v == format!("/{META}") || v.starts_with(&format!("/{META}/")) {
            return Err(err("ENOENT", "no such file or directory", "open", &v));
        }
        Ok(self.root.join(v.trim_start_matches('/')))
    }
    fn virt(&self, real: &Path) -> Option<String> {
        real.strip_prefix(&self.root).ok().map(|r| format!("/{}", r.to_string_lossy()).replace("//", "/")).map(|s| if s.len() > 1 { s.trim_end_matches('/').to_string() } else { s })
    }
    /// The real path with every link followed, refused if it leaves the project.
    fn follow(&self, v: &str, op: &str) -> R<PathBuf> {
        let p = self.real(v)?;
        match p.canonicalize() {
            Ok(c) if c.starts_with(&self.root) => {
                if c.strip_prefix(&self.root).ok().and_then(|r| r.components().next()).is_some_and(|c| c.as_os_str() == META) {
                    return Err(err("ENOENT", "no such file or directory", op, v));
                }
                Ok(c)
            }
            Ok(_) => Err(err("EACCES", "permission denied (outside the project)", op, v)),
            Err(e) if e.raw_os_error() == Some(62) => Err(err("ELOOP", "too many levels of symbolic links", op, v)),
            Err(_) => Err(err("ENOENT", "no such file or directory", op, v)),
        }
    }
    /// `follow` for reading: a link the space was given (placed by the host: a space's media, a project's takes) may
    /// lead outside the folder, and what's behind it can be read, never written. The shell can't make such a link
    /// itself (fs.symlink refuses a target that leaves the folder), so reading through one reads only what the host
    /// put in reach.
    fn follow_read(&self, v: &str, op: &str) -> R<PathBuf> {
        match self.follow(v, op) {
            Err(e) if e.starts_with("EACCES") => self.real(v)?.canonicalize().map_err(|_| err("ENOENT", "no such file or directory", op, v)),
            r => r,
        }
    }
    /// `parent_followed` for reading (lstat, readlink): the parent may be behind a link the host placed.
    fn parent_followed_read(&self, v: &str, op: &str) -> R<PathBuf> {
        match self.parent_followed(v, op) {
            Err(e) if e.starts_with("EACCES") => {
                let c = clean(v);
                let (dir, name) = c.rsplit_once('/').unwrap();
                let d = self.follow_read(if dir.is_empty() { "/" } else { dir }, op)?;
                Ok(d.join(name))
            }
            r => r,
        }
    }
    /// The real path of `v`'s parent followed, with `v`'s own name kept (for writes, links and lstat).
    fn parent_followed(&self, v: &str, op: &str) -> R<PathBuf> {
        if v.contains('\0') {
            return Err(err("EINVAL", "path contains a null byte", op, &v.replace('\0', "\\0")));
        }
        let c = clean(v);
        if c == "/" {
            return Ok(self.root.clone());
        }
        let (dir, name) = c.rsplit_once('/').unwrap();
        let dir = if dir.is_empty() { "/" } else { dir };
        let d = self.follow(dir, op).map_err(|e| if e.starts_with("ENOENT") { err("ENOENT", "no such file or directory", op, &c) } else { e })?;
        if !d.is_dir() {
            return Err(err("ENOTDIR", "not a directory", op, &c));
        }
        if name == META && d == self.root {
            return Err(err("ENOENT", "no such file or directory", op, &c));
        }
        Ok(d.join(name))
    }

    fn objects(&self) -> PathBuf {
        self.root.join(META).join("objects")
    }
    fn keep(&self, bytes: &[u8]) -> R<String> {
        let h = hash_hex(bytes);
        let o = self.objects().join(&h);
        if !o.exists() {
            fs::create_dir_all(self.objects()).map_err(|e| e.to_string())?;
            fs::write(&o, bytes).map_err(|e| e.to_string())?;
        }
        Ok(h)
    }
    /// A path's state for the record: a file's content hash and mode, a folder, a link's target, or null.
    fn state_of(&self, real: &Path) -> R<Value> {
        match fs::symlink_metadata(real) {
            Err(_) => Ok(Value::Null),
            Ok(m) if m.file_type().is_symlink() => Ok(json!({"kind": "link", "target": fs::read_link(real).map(|t| t.to_string_lossy().to_string()).unwrap_or_default()})),
            Ok(m) if m.is_dir() => Ok(json!({"kind": "dir", "mode": mode_of(&m)})),
            Ok(m) => {
                let b = fs::read(real).map_err(|e| e.to_string())?;
                Ok(json!({"kind": "file", "hash": self.keep(&b)?, "mode": mode_of(&m), "size": b.len()}))
            }
        }
    }
    /// Remembers `real` (and, for a folder, everything under it) as it was before the command first changed it.
    fn touch(&mut self, real: &Path) -> R<()> {
        let mut paths = vec![real.to_path_buf()];
        if fs::symlink_metadata(real).is_ok_and(|m| m.is_dir()) {
            let mut stack = vec![real.to_path_buf()];
            while let Some(d) = stack.pop() {
                for e in fs::read_dir(&d).into_iter().flatten().flatten() {
                    let p = e.path();
                    if fs::symlink_metadata(&p).is_ok_and(|m| m.is_dir()) {
                        stack.push(p.clone());
                    }
                    paths.push(p);
                }
            }
        }
        for p in paths {
            let Some(v) = self.virt(&p) else { continue };
            if self.group.as_ref().is_some_and(|g| g.before.contains_key(&v)) {
                continue;
            }
            let st = self.state_of(&p)?;
            if let Some(g) = self.group.as_mut() {
                g.before.insert(v, st);
            }
        }
        Ok(())
    }

    fn begin(&mut self, cmd: &str) {
        self.group = Some(Group { cmd: cmd.into(), t0: Instant::now(), before: BTreeMap::new(), read: BTreeSet::new(), bytes: 0 });
    }

    /// Closes the open command into its row (written to actions.jsonl); null when nothing is open.
    fn end(&mut self, exit: i64, error: Option<&str>, kind: &str, target: Option<u64>) -> R<Value> {
        let Some(mut g) = self.group.take() else { return Ok(Value::Null) };
        // what appeared under a folder the command was given (a tool's outputs, an extraction): absent before
        let dirs: Vec<String> = g.before.keys().filter(|v| fs::symlink_metadata(self.root.join(v.trim_start_matches('/'))).is_ok_and(|m| m.is_dir())).cloned().collect();
        for d in dirs {
            let mut stack = vec![self.root.join(d.trim_start_matches('/'))];
            while let Some(x) = stack.pop() {
                for e in fs::read_dir(&x).into_iter().flatten().flatten() {
                    let p = e.path();
                    if p.starts_with(self.root.join(META)) {
                        continue;
                    }
                    if let Some(v) = self.virt(&p) {
                        g.before.entry(v).or_insert(Value::Null);
                    }
                    if e.file_type().is_ok_and(|t| t.is_dir()) {
                        stack.push(p);
                    }
                }
            }
        }
        let mut written = Vec::new();
        for (v, before) in &g.before {
            let after = self.state_of(&self.root.join(v.trim_start_matches('/')))?;
            if &after != before {
                written.push(json!({"path": v, "before": before, "after": after}));
            }
        }
        self.seq += 1;
        let words: Vec<&str> = g.cmd.split_whitespace().collect();
        let state_before = self.state.clone();
        let mut h = sha2::Sha256::new();
        h.update(state_before.as_bytes());
        h.update(serde_json::to_vec(&written).unwrap_or_default());
        let state_after: String = h.finalize().iter().map(|x| format!("{x:02x}")).collect();
        if !written.is_empty() {
            self.state = state_after.clone();
        }
        let row = json!({
            "seq": self.seq,
            "kind": kind,
            "target": target,
            "at": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0),
            "cmd": g.cmd,
            "verb": words.first().copied().unwrap_or(""),
            "args": words.iter().skip(1).copied().collect::<Vec<_>>(),
            "read": g.read.iter().collect::<Vec<_>>(),
            "written": written,
            "bytes": g.bytes,
            "exit": exit,
            "ms": (g.t0.elapsed().as_secs_f64() * 1e4).round() / 10.0,
            "error": error,
            "state_before": state_before,
            "state_after": if self.state == state_before { state_before.clone() } else { state_after },
        });
        match kind {
            "undo" => {
                self.done.pop();
                if let Some(t) = target {
                    self.redo.push(t);
                }
            }
            "redo" => {
                self.redo.pop();
                if let Some(t) = target {
                    self.done.push(t);
                }
            }
            _ => {
                if !row["written"].as_array().is_some_and(|w| w.is_empty()) {
                    self.done.push(self.seq);
                    self.redo.clear();
                }
            }
        }
        fs::create_dir_all(self.root.join(META)).map_err(|e| e.to_string())?;
        let mut f = fs::OpenOptions::new().create(true).append(true).open(self.root.join(META).join("actions.jsonl")).map_err(|e| e.to_string())?;
        writeln!(f, "{row}").map_err(|e| e.to_string())?;
        Ok(row)
    }

    fn rows(&self) -> Vec<Value> {
        fs::read_to_string(self.root.join(META).join("actions.jsonl")).unwrap_or_default().lines().filter_map(|l| serde_json::from_str(l).ok()).collect()
    }

    /// Puts each path of a row back to its `before` (undo) or `after` (redo) state.
    fn apply(&mut self, row: &Value, side: &str) -> R<()> {
        let mut entries: Vec<(String, Value)> = row["written"].as_array().into_iter().flatten().map(|w| (w["path"].as_str().unwrap_or("").to_string(), w[side].clone())).collect();
        // removals deepest first, then creations shallowest first
        entries.sort_by_key(|(p, _)| std::cmp::Reverse(p.matches('/').count()));
        for (v, st) in &entries {
            if st.is_null() {
                let p = self.root.join(v.trim_start_matches('/'));
                if let Ok(m) = fs::symlink_metadata(&p) {
                    self.touch(&p)?;
                    if m.is_dir() { fs::remove_dir_all(&p) } else { fs::remove_file(&p) }.map_err(|e| e.to_string())?;
                }
            }
        }
        entries.reverse();
        for (v, st) in &entries {
            if st.is_null() {
                continue;
            }
            let p = self.root.join(v.trim_start_matches('/'));
            self.touch(&p)?;
            match st["kind"].as_str() {
                Some("dir") => {
                    if fs::symlink_metadata(&p).is_ok_and(|m| !m.is_dir()) {
                        fs::remove_file(&p).map_err(|e| e.to_string())?;
                    }
                    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
                    let _ = set_mode(&p, st["mode"].as_u64().unwrap_or(0o755) as u32);
                }
                Some("file") => {
                    if fs::symlink_metadata(&p).is_ok_and(|m| m.is_dir()) {
                        fs::remove_dir_all(&p).map_err(|e| e.to_string())?;
                    } else if fs::symlink_metadata(&p).is_ok_and(|m| m.file_type().is_symlink()) {
                        fs::remove_file(&p).map_err(|e| e.to_string())?;
                    }
                    let b = fs::read(self.objects().join(st["hash"].as_str().unwrap_or(""))).map_err(|e| format!("the kept copy of {v} is gone: {e}"))?;
                    fs::write(&p, b).map_err(|e| e.to_string())?;
                    let _ = set_mode(&p, st["mode"].as_u64().unwrap_or(0o644) as u32);
                }
                Some("link") => {
                    if fs::symlink_metadata(&p).is_ok() {
                        if p.is_dir() && !fs::symlink_metadata(&p).unwrap().file_type().is_symlink() { fs::remove_dir_all(&p) } else { fs::remove_file(&p) }.map_err(|e| e.to_string())?;
                    }
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(st["target"].as_str().unwrap_or(""), &p).map_err(|e| e.to_string())?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn stat_json(&self, m: &fs::Metadata) -> Value {
        let mtime = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs_f64() * 1e3).unwrap_or(0.0);
        // dev and ino: a file's stable identity, which just-bash's cp and mv need to tell two existing files apart (without
        // them `mv new.tsx edit.tsx` refused: "cannot safely determine whether ... are the same file"); as numbers, exact
        // to 2^53, which covers APFS's inode numbers
        #[cfg(unix)]
        let (dev, ino) = { use std::os::unix::fs::MetadataExt; (m.dev() as f64, m.ino() as f64) };
        #[cfg(not(unix))]
        let (dev, ino) = (0.0, 0.0);
        json!({"isFile": m.is_file(), "isDirectory": m.is_dir(), "isSymbolicLink": m.file_type().is_symlink(), "mode": mode_of(m), "size": m.len(), "mtimeMs": mtime, "dev": dev, "ino": ino})
    }
}

fn with<T>(v: &Value, f: impl FnOnce(&mut Project) -> R<T>) -> R<T> {
    let id = v.get("id").and_then(Value::as_u64).ok_or("id: an open project")?;
    let mut g = projects().lock().unwrap();
    let p = g.1.get_mut(&id).ok_or_else(|| format!("no open project {id}"))?;
    // a call outside any command is its own row
    let implicit = p.group.is_none();
    if implicit {
        p.begin("(api)");
    }
    let r = f(p);
    if implicit {
        let wrote = p.group.as_ref().is_some_and(|g| !g.before.is_empty());
        if wrote {
            p.end(if r.is_ok() { 0 } else { 1 }, r.as_ref().err().map(String::as_str), "cmd", None)?;
        } else {
            p.group = None;
        }
    }
    r
}

fn copy_tree(p: &mut Project, from: &Path, to: &Path, recursive: bool, vsrc: &str) -> R<()> {
    let m = fs::symlink_metadata(from).map_err(|_| err("ENOENT", "no such file or directory", "cp", vsrc))?;
    if m.is_dir() {
        if !recursive {
            return Err(err("EISDIR", "illegal operation on a directory", "cp", vsrc));
        }
        p.touch(to)?;
        fs::create_dir_all(to).map_err(|e| e.to_string())?;
        for e in fs::read_dir(from).map_err(|e| e.to_string())?.flatten() {
            if from == p.root && e.file_name() == META {
                continue;
            }
            copy_tree(p, &e.path(), &to.join(e.file_name()), true, vsrc)?;
        }
    } else if m.file_type().is_symlink() {
        p.touch(to)?;
        let t = fs::read_link(from).map_err(|e| e.to_string())?;
        let _ = fs::remove_file(to);
        #[cfg(unix)]
        std::os::unix::fs::symlink(t, to).map_err(|e| e.to_string())?;
    } else {
        p.touch(to)?;
        if fs::symlink_metadata(to).is_ok_and(|m| m.is_dir()) {
            return Err(err("EISDIR", "illegal operation on a directory", "cp", &p.virt(to).unwrap_or_default()));
        }
        let n = fs::copy(from, to).map_err(|e| e.to_string())?;
        if let Some(g) = p.group.as_mut() {
            g.bytes += n;
        }
    }
    Ok(())
}
