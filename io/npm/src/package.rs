//! package.json and raea.lock, read and written, and a dependency spec read as a package request.

use super::*;

/// "alias@npm:pkg@1" → ("alias", "npm:pkg@1"); "@scope/pkg@^2" → ("@scope/pkg", "^2"); "react" → ("react", "").
pub(crate) fn split_spec(spec: &str) -> (String, String) {
    let at = if let Some(rest) = spec.strip_prefix('@') { rest.find('@').map(|i| i + 1) } else { spec.find('@') };
    match at {
        Some(i) => (spec[..i].to_string(), spec[i + 1..].to_string()),
        None => (spec.to_string(), String::new()),
    }
}

/// A dependency as Deno's resolver asks for it: `npm:other@range` is that package under this name; a range or dist-tag
/// is this package. Git, file, link and workspace dependencies aren't the registry's, and are refused by name.
pub(crate) fn req_of(alias: &str, spec: &str) -> R<PackageReq> {
    let spec = spec.trim();
    let text = match spec.strip_prefix("npm:") {
        Some(rest) => rest.to_string(),
        None => {
            let local = ["file:", "link:", "workspace:", "git:", "git+", "github:", "http:", "https:", "portal:"].iter().any(|p| spec.starts_with(p))
                || (spec.contains('/') && !spec.starts_with('@'));
            if local {
                return Err(format!("{alias}: \"{spec}\" isn't a registry version, which is all raea installs"));
            }
            format!("{alias}@{}", if spec.is_empty() { "*" } else { spec })
        }
    };
    PackageReq::from_str_loose(&text).map_err(|e| format!("{alias}: {e}"))
}

/// package.json, its keys and their text kept as they were; only the dependency lists raea changes are rewritten
/// (sorted, as npm writes them).
pub(crate) struct PackageJson {
    pub(crate) keys: indexmap::IndexMap<String, Box<RawValue>>,
}

impl PackageJson {
    pub(crate) fn read(root: &Path) -> R<Self> {
        let path = root.join("package.json");
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => "{}".into(),
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        Ok(PackageJson { keys: serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))? })
    }

    pub(crate) fn list(&self, key: &str) -> BTreeMap<String, String> {
        self.keys.get(key).and_then(|v| serde_json::from_str(v.get()).ok()).unwrap_or_default()
    }

    /// dependencies, then devDependencies (one in both counts once, as dependencies)
    pub(crate) fn deps(&self) -> Vec<(String, String)> {
        let mut all = self.list("dependencies");
        for (k, v) in self.list("devDependencies") {
            all.entry(k).or_insert(v);
        }
        all.into_iter().collect()
    }

    pub(crate) fn put(&mut self, key: &str, list: BTreeMap<String, String>) {
        if list.is_empty() {
            self.keys.shift_remove(key);
            return;
        }
        let body: Vec<String> = list.iter().map(|(k, v)| format!("    {}: {}", serde_json::json!(k), serde_json::json!(v))).collect();
        let raw = RawValue::from_string(format!("{{\n{}\n  }}", body.join(",\n"))).unwrap();
        match self.keys.get_mut(key) {
            Some(v) => *v = raw,
            None => {
                self.keys.insert(key.to_string(), raw);
            }
        }
    }

    pub(crate) fn set(&mut self, key: &str, name: &str, range: &str) {
        let mut l = self.list(key);
        l.insert(name.to_string(), range.to_string());
        self.put(key, l);
    }

    pub(crate) fn unset(&mut self, key: &str, name: &str) -> bool {
        let mut l = self.list(key);
        let had = l.remove(name).is_some();
        if had {
            self.put(key, l);
        }
        had
    }

    pub(crate) fn write(&self, root: &Path) -> R<()> {
        let body: Vec<String> = self.keys.iter().map(|(k, v)| format!("  {}: {}", serde_json::json!(k), v.get())).collect();
        let text = if body.is_empty() { "{}\n".to_string() } else { format!("{{\n{}\n}}\n", body.join(",\n")) };
        let path = root.join("package.json");
        write_atomic(&path, text.as_bytes()).map_err(|e| format!("{}: {e}", path.display()))
    }
}

/// raea.lock: Deno's resolution of the project's dependencies, as JSON.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Lock {
    pub(crate) version: u32,
    pub(crate) snapshot: SerializedNpmResolutionSnapshot,
}

impl Lock {
    pub(crate) fn read(root: &Path) -> Option<Lock> {
        serde_json::from_str(&std::fs::read_to_string(root.join("raea.lock")).ok()?).ok().filter(|l: &Lock| l.version == 1)
    }

    pub(crate) fn write(&self, root: &Path) -> R<()> {
        // sorted, so a lock changes only where the resolution did
        let mut v = serde_json::to_value(self).map_err(|e| e.to_string())?;
        if let Some(p) = v["snapshot"]["packages"].as_array_mut() {
            p.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        }
        let path = root.join("raea.lock");
        write_atomic(&path, format!("{}\n", serde_json::to_string_pretty(&v).unwrap()).as_bytes()).map_err(|e| format!("{}: {e}", path.display()))
    }
}
