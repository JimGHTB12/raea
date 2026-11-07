//! An app's `export const app` (docs/targets.feature T0a), read from its source before anything is bundled or run:
//! which target hosts it and its window. The value must be a literal (objects, arrays, strings, numbers, booleans),
//! so the CLI knows the host without running the app. Anything else is refused with the file and line, never read as
//! the default. With no `app` export, or no `target` in it, the target is web.

use std::path::Path;

use serde_json::{Map, Value};

use crate::literal::{self, Error};

/// Where an app runs: Blitz on Bevy (web, the default) or Rune (native).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Web,
    Native,
}

impl Target {
    pub fn parse(s: &str) -> Result<Target, String> {
        match s {
            "web" => Ok(Target::Web),
            "native" => Ok(Target::Native),
            other => Err(format!("target {other:?}: \"web\" or \"native\"")),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Target::Web => "web",
            Target::Native => "native",
        }
    }
}

/// What the app's file says, with the defaults filled in where it says nothing.
#[derive(Clone, Debug, PartialEq)]
pub struct AppConfig {
    pub target: Target,
    pub title: Option<String>,
    pub size: Option<(f32, f32)>,
    pub min_size: Option<(f32, f32)>,
    /// whether the file had an `app` export at all
    pub declared: bool,
}

const KEYS: [&str; 4] = ["target", "title", "size", "minSize"];

/// `path`'s app config, `target` (from `--target`) over what the file says.
pub fn read(path: &Path, target: Option<Target>) -> Result<AppConfig, String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut c = parse(&src).map_err(|e| format!("{}:{}: {}", path.display(), e.line, e.why))?;
    if let Some(t) = target {
        c.target = t;
    }
    Ok(c)
}

/// The config in `src`: the default when it exports no `app`.
pub fn parse(src: &str) -> Result<AppConfig, Error> {
    let code = literal::blank_comments(src);
    // where to look for the export: strings blanked too, so one written inside a string isn't it
    let bare = literal::blank_strings(&code);
    let mut c = AppConfig { target: Target::Web, title: None, size: None, min_size: None, declared: false };
    if let Some(at) = other_export(&bare) {
        return Err(Error::at(src, at, "`app` is exported, but not as `export const app = { ... }` (a literal the CLI can read)"));
    }
    let Some(start) = find_export(&bare, "app") else { return Ok(c) };
    c.declared = true;
    let (value, _) = literal::value(src, &code, start)?;
    let Value::Object(o) = value else { return Err(Error::at(src, start, "`app` must be an object literal")) };
    fields(src, start, &o, &mut c)?;
    Ok(c)
}

fn fields(src: &str, at: usize, o: &Map<String, Value>, c: &mut AppConfig) -> Result<(), Error> {
    if let Some(k) = o.keys().find(|k| !KEYS.contains(&k.as_str())) {
        return Err(Error::at(src, at, &format!("`app` has no field {k:?} (it has {})", KEYS.join(", "))));
    }
    if let Some(t) = o.get("target") {
        let s = t.as_str().ok_or_else(|| Error::at(src, at, "`app.target` is a string"))?;
        c.target = Target::parse(s).map_err(|e| Error::at(src, at, &e))?;
    }
    if let Some(t) = o.get("title") {
        c.title = Some(t.as_str().ok_or_else(|| Error::at(src, at, "`app.title` is a string"))?.to_string());
    }
    c.size = o.get("size").map(|v| pair(src, at, "size", v)).transpose()?;
    c.min_size = o.get("minSize").map(|v| pair(src, at, "minSize", v)).transpose()?;
    Ok(())
}

fn pair(src: &str, at: usize, key: &str, v: &Value) -> Result<(f32, f32), Error> {
    let bad = || Error::at(src, at, &format!("`app.{key}` is [width, height], two positive numbers"));
    let a = v.as_array().filter(|a| a.len() == 2).ok_or_else(bad)?;
    let (w, h) = (a[0].as_f64().ok_or_else(bad)?, a[1].as_f64().ok_or_else(bad)?);
    if w <= 0.0 || h <= 0.0 {
        return Err(bad());
    }
    Ok((w as f32, h as f32))
}

/// Where `export const NAME` starts its value: past `=`, its type annotation skipped.
fn find_export(code: &str, name: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(i) = code[from..].find("export") {
        let at = from + i;
        let rest = &code[at + 6..];
        let t = rest.trim_start();
        if let Some(r) = t.strip_prefix("const")
            && r.starts_with(char::is_whitespace)
            && let Some(r) = r.trim_start().strip_prefix(name)
            && !r.starts_with(|ch: char| ch.is_alphanumeric() || ch == '_' || ch == '$')
        {
            let off = code.len() - r.len();
            return code[off..].find('=').map(|e| off + e + 1);
        }
        from = at + 6;
    }
    None
}

/// An `app` exported some other way: `export let/var/function app`, or `app` in an `export { ... }` list.
fn other_export(code: &str) -> Option<usize> {
    let word = |s: &str| s.starts_with("app") && !s[3..].starts_with(|ch: char| ch.is_alphanumeric() || ch == '_' || ch == '$');
    let mut from = 0;
    while let Some(i) = code[from..].find("export") {
        let at = from + i;
        let t = code[at + 6..].trim_start();
        for kw in ["let", "var", "function", "class"] {
            if let Some(r) = t.strip_prefix(kw)
                && r.starts_with(char::is_whitespace)
                && word(r.trim_start())
            {
                return Some(at);
            }
        }
        if let Some(list) = t.strip_prefix('{').and_then(|r| r.split_once('}')).map(|(l, _)| l)
            && list.split(',').any(|n| n.trim() == "app" || n.trim().ends_with(" as app"))
        {
            return Some(at);
        }
        from = at + 6;
    }
    None
}

/// The steps an app file exports (`export const steps = [...]`, a literal as `app` is) as a check's script JSON, or
/// None when it exports none: `raea app check` plays them when no `--script` is given, so an example's steps travel
/// with it (docs/targets.feature T5a)
pub fn steps(src: &str) -> Result<Option<String>, Error> {
    let code = literal::blank_comments(src);
    let Some(start) = find_export(&literal::blank_strings(&code), "steps") else { return Ok(None) };
    match literal::value(src, &code, start)?.0 {
        v @ Value::Array(_) => Ok(Some(v.to_string())),
        _ => Err(Error::at(src, start, "`steps` must be an array literal of the check's steps")),
    }
}
