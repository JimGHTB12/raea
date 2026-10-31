//! The fs.* calls: a project's files under its root, with undo and redo.

use super::*;

/// The fs.* calls in the registry.
pub(crate) fn calls() -> Vec<raea_core_calls::Call> {
    use std::sync::Arc;
    const STAT: &str = "{ isFile: boolean; isDirectory: boolean; isSymbolicLink: boolean; mode: number; size: number; mtimeMs: number; dev: number; ino: number }";
    const ROW: &str = "Record<string, unknown>";
    let mk = raea_core_calls::Call::new;
    let p = |v: &Value| s(v, "path").map(str::to_string);
    vec![
        mk("fs.open", "{ root: string }", "{ id: number; root: string }", "a project folder opened as a filesystem (made if missing)", Arc::new(|v| {
            let root = Path::new(s(v, "root")?);
            fs::create_dir_all(root).map_err(|e| format!("{}: {e}", root.display()))?;
            let root = root.canonicalize().map_err(|e| e.to_string())?;
            let mut pr = Project { root: root.clone(), group: None, seq: 0, state: "0".repeat(64), done: Vec::new(), redo: Vec::new() };
            for r in pr.rows() {
                pr.seq = r["seq"].as_u64().unwrap_or(pr.seq);
                if let Some(st) = r["state_after"].as_str() {
                    pr.state = st.to_string();
                }
                let t = r["target"].as_u64();
                match r["kind"].as_str() {
                    Some("undo") => {
                        pr.done.pop();
                        pr.redo.extend(t);
                    }
                    Some("redo") => {
                        pr.redo.pop();
                        pr.done.extend(t);
                    }
                    _ if r["written"].as_array().is_some_and(|w| !w.is_empty()) => {
                        pr.done.push(r["seq"].as_u64().unwrap_or(0));
                        pr.redo.clear();
                    }
                    _ => {}
                }
            }
            let mut g = projects().lock().unwrap();
            g.0 += 1;
            let id = g.0;
            g.1.insert(id, pr);
            Ok(json!({"id": id, "root": root.display().to_string()}))
        })),
        mk("fs.begin", "{ id: number; cmd: string }", "null", "a command starts: what follows is its row", Arc::new(|v| {
            let id = v.get("id").and_then(Value::as_u64).ok_or("id")?;
            let mut g = projects().lock().unwrap();
            let pr = g.1.get_mut(&id).ok_or("no such project")?;
            if pr.group.is_some() {
                pr.end(-1, Some("interrupted by the next command"), "cmd", None)?;
            }
            pr.begin(s(v, "cmd")?);
            Ok(Value::Null)
        })),
        mk("fs.end", "{ id: number; exit?: number; error?: string }", &format!("{ROW} | null"), "the command ends: its action row", Arc::new(|v| {
            let id = v.get("id").and_then(Value::as_u64).ok_or("id")?;
            let mut g = projects().lock().unwrap();
            let pr = g.1.get_mut(&id).ok_or("no such project")?;
            pr.end(v.get("exit").and_then(Value::as_i64).unwrap_or(0), v.get("error").and_then(Value::as_str), "cmd", None)
        })),
        mk("fs.undo", "{ id: number }", &format!("{ROW} | null"), "the last command's changes taken back (null when there's none)", Arc::new(|v| undo_redo(v, true))),
        mk("fs.redo", "{ id: number }", &format!("{ROW} | null"), "the last undone command's changes made again", Arc::new(|v| undo_redo(v, false))),
        mk("fs.log", "{ id: number; last?: number }", &format!("{ROW}[]"), "the project's action rows (the last n)", Arc::new(|v| {
            let n = v.get("last").and_then(Value::as_u64).map(|n| n as usize);
            with(v, |pr| {
                let rows = pr.rows();
                let k = n.map(|n| rows.len().saturating_sub(n)).unwrap_or(0);
                Ok(json!(rows[k..]))
            })
        })),
        mk("fs.touch", "{ id: number; path: string }", "null", "a path's state kept as it is now, before a tool outside the filesystem changes it (so the command's row has it, and undo can restore it)", Arc::new(move |v| {
            let path = p(v)?;
            with(v, |pr| {
                let r = pr.parent_followed(&path, "open")?;
                pr.touch(&r)?;
                Ok(Value::Null)
            })
        })),
        mk("fs.stat", "{ id: number; path: string }", STAT, "a path's facts, links followed", Arc::new(move |v| {
            let path = p(v)?;
            with(v, |pr| {
                let r = pr.follow_read(&path, "stat")?;
                let m = fs::metadata(&r).map_err(|_| err("ENOENT", "no such file or directory", "stat", &path))?;
                Ok(pr.stat_json(&m))
            })
        })),
        mk("fs.lstat", "{ id: number; path: string }", STAT, "a path's facts, a link itself", Arc::new(move |v| {
            let path = p(v)?;
            with(v, |pr| {
                let r = pr.parent_followed_read(&path, "lstat")?;
                let m = fs::symlink_metadata(&r).map_err(|_| err("ENOENT", "no such file or directory", "lstat", &path))?;
                Ok(pr.stat_json(&m))
            })
        })),
        mk("fs.exists", "{ id: number; path: string }", "boolean", "whether a path is there (links followed)", Arc::new(move |v| {
            let path = p(v)?;
            with(v, |pr| Ok(json!(pr.follow_read(&path, "access").is_ok())))
        })),
        mk("fs.read", "{ id: number; path: string }", "string", "a file's bytes, base64", Arc::new(move |v| {
            let path = p(v)?;
            with(v, |pr| {
                let r = pr.follow_read(&path, "open")?;
                if r.is_dir() {
                    return Err(err("EISDIR", "illegal operation on a directory", "read", &path));
                }
                let b = fs::read(&r).map_err(|_| err("ENOENT", "no such file or directory", "open", &path))?;
                if let Some(g) = pr.group.as_mut() {
                    g.read.insert(clean(&path));
                }
                Ok(json!(enc64(&b)))
            })
        })),
        mk("fs.write", "{ id: number; path: string; data: string; append?: boolean; exclusive?: boolean; mode?: number }", "null", "bytes written (or appended) to a file", Arc::new(move |v| {
            let path = p(v)?;
            let data = dec64(s(v, "data")?)?;
            let append = v.get("append").and_then(Value::as_bool).unwrap_or(false);
            let exclusive = v.get("exclusive").and_then(Value::as_bool).unwrap_or(false);
            let mode = v.get("mode").and_then(Value::as_u64);
            with(v, |pr| {
                // a link is written through, inside the project
                let mut r = pr.parent_followed(&path, "open")?;
                if exclusive {
                    if fs::symlink_metadata(&r).is_ok() {
                        return Err(err("EEXIST", "file already exists", "open", &path));
                    }
                } else if fs::symlink_metadata(&r).is_ok_and(|m| m.file_type().is_symlink()) {
                    r = pr.follow(&path, "open")?;
                }
                if r.is_dir() {
                    return Err(err("EISDIR", "illegal operation on a directory", "write", &path));
                }
                pr.touch(&r)?;
                let mut f = fs::OpenOptions::new().create(true).write(true).append(append).truncate(!append).open(&r).map_err(|e| format!("{path}: {e}"))?;
                f.write_all(&data).map_err(|e| e.to_string())?;
                if let Some(m) = mode {
                    let _ = set_mode(&r, m as u32);
                }
                if let Some(g) = pr.group.as_mut() {
                    g.bytes += data.len() as u64;
                }
                Ok(Value::Null)
            })
        })),
        mk("fs.mkdir", "{ id: number; path: string; recursive?: boolean; exclusive?: boolean; mode?: number }", "null", "a folder made", Arc::new(move |v| {
            let path = p(v)?;
            let recursive = v.get("recursive").and_then(Value::as_bool).unwrap_or(false);
            let exclusive = v.get("exclusive").and_then(Value::as_bool).unwrap_or(false);
            let mode = v.get("mode").and_then(Value::as_u64);
            with(v, |pr| {
                let c = clean(&path);
                if recursive {
                    let mut cur = String::new();
                    for part in c.split('/').filter(|x| !x.is_empty()) {
                        cur = format!("{cur}/{part}");
                        let r = pr.parent_followed(&cur, "mkdir")?;
                        match fs::metadata(&r) {
                            Ok(m) if m.is_dir() => continue,
                            Ok(_) => return Err(err("EEXIST", "file already exists", "mkdir", &cur)),
                            Err(_) => {
                                pr.touch(&r)?;
                                fs::create_dir(&r).map_err(|e| e.to_string())?;
                            }
                        }
                    }
                    return Ok(Value::Null);
                }
                let r = pr.parent_followed(&c, "mkdir")?;
                if let Ok(m) = fs::symlink_metadata(&r) {
                    return Err(err("EEXIST", if m.is_dir() && !exclusive { "directory already exists" } else { "file already exists" }, "mkdir", &c));
                }
                pr.touch(&r)?;
                fs::create_dir(&r).map_err(|e| e.to_string())?;
                if let Some(m) = mode {
                    let _ = set_mode(&r, m as u32);
                }
                Ok(Value::Null)
            })
        })),
        mk("fs.readdir", "{ id: number; path: string }", "{ name: string; isFile: boolean; isDirectory: boolean; isSymbolicLink: boolean }[]", "a folder's entries, by name", Arc::new(move |v| {
            let path = p(v)?;
            with(v, |pr| {
                let r = pr.follow_read(&path, "scandir")?;
                if !r.is_dir() {
                    return Err(err("ENOTDIR", "not a directory", "scandir", &path));
                }
                let mut out = Vec::new();
                for e in fs::read_dir(&r).map_err(|e| e.to_string())?.flatten() {
                    let name = e.file_name().to_string_lossy().to_string();
                    if r == pr.root && name == META {
                        continue;
                    }
                    let ft = e.file_type().map_err(|e| e.to_string())?;
                    out.push(json!({"name": name, "isFile": ft.is_file(), "isDirectory": ft.is_dir(), "isSymbolicLink": ft.is_symlink()}));
                }
                out.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
                Ok(Value::Array(out))
            })
        })),
        mk("fs.rm", "{ id: number; path: string; recursive?: boolean; force?: boolean }", "null", "a file, link or folder removed", Arc::new(move |v| {
            let path = p(v)?;
            let recursive = v.get("recursive").and_then(Value::as_bool).unwrap_or(false);
            let force = v.get("force").and_then(Value::as_bool).unwrap_or(false);
            with(v, |pr| {
                let c = clean(&path);
                if c == "/" {
                    return Err(err("EPERM", "operation not permitted", "rm", &c));
                }
                let r = match pr.parent_followed(&c, "rm") {
                    Ok(r) => r,
                    Err(_) if force => return Ok(Value::Null),
                    Err(e) => return Err(e),
                };
                let Ok(m) = fs::symlink_metadata(&r) else {
                    return if force { Ok(Value::Null) } else { Err(err("ENOENT", "no such file or directory", "rm", &c)) };
                };
                if m.is_dir() && !m.file_type().is_symlink() {
                    if !recursive && fs::read_dir(&r).map(|mut d| d.next().is_some()).unwrap_or(false) {
                        return Err(err("ENOTEMPTY", "directory not empty", "rm", &c));
                    }
                    pr.touch(&r)?;
                    fs::remove_dir_all(&r).map_err(|e| e.to_string())?;
                } else {
                    pr.touch(&r)?;
                    fs::remove_file(&r).map_err(|e| e.to_string())?;
                }
                Ok(Value::Null)
            })
        })),
        mk("fs.cp", "{ id: number; src: string; dest: string; recursive?: boolean }", "null", "a file (or with recursive, a folder) copied", Arc::new(|v| {
            let (src, dest) = (s(v, "src")?.to_string(), s(v, "dest")?.to_string());
            let recursive = v.get("recursive").and_then(Value::as_bool).unwrap_or(false);
            with(v, |pr| {
                let from = pr.follow_read(&src, "cp")?;
                let to = pr.parent_followed(&dest, "cp")?;
                if to.starts_with(&from) && from.is_dir() {
                    return Err(err("EINVAL", "can't copy a folder into itself", "cp", &dest));
                }
                copy_tree(pr, &from, &to, recursive, &src)?;
                Ok(Value::Null)
            })
        })),
        mk("fs.mv", "{ id: number; src: string; dest: string }", "null", "a path moved", Arc::new(|v| {
            let (src, dest) = (s(v, "src")?.to_string(), s(v, "dest")?.to_string());
            with(v, |pr| {
                let from = pr.parent_followed(&src, "rename")?;
                if fs::symlink_metadata(&from).is_err() {
                    return Err(err("ENOENT", "no such file or directory", "rename", &src));
                }
                let to = pr.parent_followed(&dest, "rename")?;
                if to.starts_with(&from) && to != from {
                    return Err(err("EINVAL", "can't move a folder into itself", "rename", &dest));
                }
                pr.touch(&from)?;
                pr.touch(&to)?;
                fs::rename(&from, &to).map_err(|e| format!("{dest}: {e}"))?;
                pr.touch(&to)?;
                Ok(Value::Null)
            })
        })),
        mk("fs.chmod", "{ id: number; path: string; mode: number }", "null", "a path's mode set", Arc::new(move |v| {
            let path = p(v)?;
            let mode = v.get("mode").and_then(Value::as_u64).ok_or("mode")? as u32;
            with(v, |pr| {
                let r = pr.follow(&path, "chmod")?;
                pr.touch(&r)?;
                set_mode(&r, mode).map_err(|e| e.to_string())?;
                Ok(Value::Null)
            })
        })),
        mk("fs.symlink", "{ id: number; target: string; path: string }", "null", "a link made (an absolute target is a project path)", Arc::new(move |v| {
            let path = p(v)?;
            let target = s(v, "target")?.to_string();
            with(v, |pr| {
                let r = pr.parent_followed(&path, "symlink")?;
                if fs::symlink_metadata(&r).is_ok() {
                    return Err(err("EEXIST", "file already exists", "symlink", &path));
                }
                pr.touch(&r)?;
                // an absolute target names a project path: the link points at it under the root, and reads back as it
                let t: PathBuf = if target.starts_with('/') {
                    pr.root.join(clean(&target).trim_start_matches('/'))
                } else {
                    // a relative target that climbs out of the folder (lexically, from the link's own folder) is
                    // refused: links that lead outside are the host's to place, so reads through them stay in reach
                    let mut at = r.parent().unwrap_or(&pr.root).to_path_buf();
                    for c in Path::new(&target).components() {
                        match c {
                            Component::ParentDir => {
                                if !at.pop() || !at.starts_with(&pr.root) {
                                    return Err(err("EACCES", "permission denied (a link out of the project)", "symlink", &path));
                                }
                            }
                            Component::Normal(x) => at.push(x),
                            _ => {}
                        }
                    }
                    if !at.starts_with(&pr.root) {
                        return Err(err("EACCES", "permission denied (a link out of the project)", "symlink", &path));
                    }
                    PathBuf::from(&target)
                };
                #[cfg(unix)]
                std::os::unix::fs::symlink(&t, &r).map_err(|e| e.to_string())?;
                Ok(Value::Null)
            })
        })),
        mk("fs.place_link", "{ id: number; path: string; target: string }", "null", "the host's: a link in the folder to a real path anywhere (a space's media), read-only through the shell", Arc::new(move |v| {
            let path = p(v)?;
            let target = s(v, "target")?.to_string();
            with(v, |pr| {
                let r = pr.parent_followed(&path, "symlink")?;
                if fs::symlink_metadata(&r).is_ok() {
                    return Err(err("EEXIST", "file already exists", "symlink", &path));
                }
                #[cfg(unix)]
                std::os::unix::fs::symlink(&target, &r).map_err(|e| e.to_string())?;
                Ok(Value::Null)
            })
        })),
        mk("fs.link", "{ id: number; existing: string; path: string }", "null", "a hard link made", Arc::new(move |v| {
            let path = p(v)?;
            let existing = s(v, "existing")?.to_string();
            with(v, |pr| {
                let from = pr.follow(&existing, "link")?;
                let to = pr.parent_followed(&path, "link")?;
                if fs::symlink_metadata(&to).is_ok() {
                    return Err(err("EEXIST", "file already exists", "link", &path));
                }
                if from.is_dir() {
                    return Err(err("EPERM", "operation not permitted", "link", &existing));
                }
                pr.touch(&to)?;
                fs::hard_link(&from, &to).map_err(|e| e.to_string())?;
                Ok(Value::Null)
            })
        })),
        mk("fs.readlink", "{ id: number; path: string }", "string", "a link's target (a project path when it points inside)", Arc::new(move |v| {
            let path = p(v)?;
            with(v, |pr| {
                let r = pr.parent_followed_read(&path, "readlink")?;
                let t = fs::read_link(&r).map_err(|_| err("EINVAL", "invalid argument", "readlink", &path))?;
                Ok(json!(if t.is_absolute() { pr.virt(&t).unwrap_or_else(|| "(outside the project)".into()) } else { t.display().to_string() }))
            })
        })),
        mk("fs.realpath", "{ id: number; path: string }", "string", "a path with every link followed", Arc::new(move |v| {
            let path = p(v)?;
            with(v, |pr| {
                let r = pr.follow_read(&path, "realpath")?;
                Ok(json!(pr.virt(&r).unwrap_or_else(|| clean(&path))))
            })
        })),
        mk("fs.utimes", "{ id: number; path: string; mtimeMs: number }", "null", "a path's modification time set", Arc::new(move |v| {
            let path = p(v)?;
            let ms = v.get("mtimeMs").and_then(Value::as_f64).ok_or("mtimeMs")?;
            with(v, |pr| {
                let r = pr.follow(&path, "utime")?;
                let t = std::time::UNIX_EPOCH + std::time::Duration::from_secs_f64((ms / 1e3).max(0.0));
                fs::File::options().write(r.is_file()).read(true).open(&r).and_then(|f| f.set_modified(t)).map_err(|e| e.to_string())?;
                Ok(Value::Null)
            })
        })),
        mk("fs.paths", "{ id: number; limit?: number }", "string[]", "every path in the project (to a limit)", Arc::new(|v| {
            let limit = v.get("limit").and_then(Value::as_u64).unwrap_or(100_000) as usize;
            with(v, |pr| {
                let mut out = vec!["/".to_string()];
                let mut stack = vec![pr.root.clone()];
                while let Some(d) = stack.pop() {
                    for e in fs::read_dir(&d).into_iter().flatten().flatten() {
                        if d == pr.root && e.file_name() == META {
                            continue;
                        }
                        let p = e.path();
                        if let Some(v) = pr.virt(&p) {
                            out.push(v);
                        }
                        if out.len() >= limit {
                            return Ok(json!(out));
                        }
                        if e.file_type().is_ok_and(|t| t.is_dir()) {
                            stack.push(p);
                        }
                    }
                }
                out.sort();
                Ok(json!(out))
            })
        })),
    ]
}


fn undo_redo(v: &Value, undo: bool) -> R<Value> {
    let id = v.get("id").and_then(Value::as_u64).ok_or("id")?;
    let mut g = projects().lock().unwrap();
    let pr = g.1.get_mut(&id).ok_or("no such project")?;
    if pr.group.is_some() {
        pr.end(-1, Some("interrupted by undo"), "cmd", None)?;
    }
    let target = if undo { pr.done.last().copied() } else { pr.redo.last().copied() };
    let Some(t) = target else { return Ok(Value::Null) };
    let row = pr.rows().into_iter().find(|r| r["seq"].as_u64() == Some(t)).ok_or("that command's row is gone")?;
    pr.begin(&format!("{} {}", if undo { "undo" } else { "redo" }, row["cmd"].as_str().unwrap_or("")));
    let r = pr.apply(&row, if undo { "before" } else { "after" });
    let out = pr.end(if r.is_ok() { 0 } else { 1 }, r.as_ref().err().map(String::as_str), if undo { "undo" } else { "redo" }, Some(t))?;
    r.map(|_| out)
}
