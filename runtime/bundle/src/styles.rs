//! styles.gen.ts for a composition that imports it (`import { css } from "./styles.gen"`): the brand kit's variables
//! (brand/brand.css beside the edit, its @imports dropped: Blitz loads no remote CSS), then Tailwind 4 for every class
//! any .ts or .tsx in the edit's folder uses, with raea's brand theme (engine/cli/template/theme.css: class names onto
//! --brand-* variables). render, play, bundle and export-game write it before they bundle, as the legacy app does
//! (apps/legacy/shell/src/tailwind.rs: the same compile, which this follows byte for byte). Tailwind's compiler runs on raea's
//! JavaScriptCore (raea-native's js-exec), from the package raea ships (sdk/react/tw/node_modules/tailwindcss under
//! raea_io_home::home(): the checkout's, or an install's share/raea).
#[cfg(feature = "modules")]
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[cfg(feature = "modules")]
use serde_json::json;

type R<T> = Result<T, String>;

const FILE: &str = "styles.gen.ts";

/// The compile, as the legacy app's tailwind.rs runs it: `@import "tailwindcss"` and the theme, Tailwind's own stylesheets from
/// the package, the candidates built
#[cfg(feature = "modules")]
const COMPILE: &str = r#"import { compile } from "./dist/lib.mjs";
const { candidates, theme, files } = JSON.parse(process.stdin.text);
const c = await compile(`@import "tailwindcss";\n${theme}`, {
  base: "/",
  loadStylesheet: async (id) => {
    const name = id === "tailwindcss" ? "index.css" : id.replace(/^tailwindcss\//, "");
    const file = name.endsWith(".css") ? name : name + ".css";
    if (!(file in files)) throw new Error("tailwind: no stylesheet " + id);
    return { path: file, base: "/", content: files[file] };
  },
});
process.stdout.write(c.build(candidates));
"#;

/// The edit's folder's code (.ts and .tsx, not dot folders, node_modules or media), names sorted as readdirSync gives
/// them: what Tailwind's candidates come from and what may import styles.gen
fn sources(space: &Path) -> Vec<PathBuf> {
    fn walk(d: &Path, out: &mut Vec<PathBuf>) {
        let mut names: Vec<_> = std::fs::read_dir(d).into_iter().flatten().flatten().map(|e| e.file_name()).collect();
        names.sort();
        for name in names {
            let n = name.to_string_lossy();
            if n.starts_with('.') || n == "node_modules" || n == "media" {
                continue;
            }
            let p = d.join(&name);
            if p.is_dir() {
                walk(&p, out);
            } else if n.ends_with(".tsx") || n.ends_with(".ts") {
                out.push(p);
            }
        }
    }
    let mut files = Vec::new();
    walk(space, &mut files);
    files
}

/// styles.gen.ts in `dir`, written when some code there imports it and it's missing or older than that code or
/// brand/brand.css (an unchanged compile keeps the file and its time). A folder that doesn't import it is left alone.
pub fn ensure(dir: &Path) -> R<()> {
    let code: Vec<PathBuf> = sources(dir).into_iter().filter(|p| p.file_name().is_none_or(|n| n != FILE)).collect();
    let imports = code.iter().any(|p| std::fs::read_to_string(p).is_ok_and(|t| t.contains("/styles.gen\"") || t.contains("/styles.gen'") || t.contains("/styles.gen.ts\"")));
    if !imports {
        return Ok(());
    }
    let file = dir.join(FILE);
    let modified = |p: &Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    if let Some(made) = modified(&file) {
        let newest = code.iter().map(PathBuf::as_path).chain(std::iter::once(dir.join("brand/brand.css").as_path())).filter_map(modified).max();
        if newest.is_none_or(|n| n <= made) {
            return Ok(());
        }
    }
    let css = css(dir, &code)?;
    let body = format!("// written by raea from brand/brand.css and the project's Tailwind classes; don't edit\nexport const css = {};\n", serde_json::to_string(&css).map_err(|e| e.to_string())?);
    if std::fs::read_to_string(&file).ok().as_deref() != Some(body.as_str()) {
        std::fs::write(&file, body).map_err(|e| format!("{}: {e}", file.display()))?;
    } else {
        // the same CSS: its time moves on, so the next render skips the compile
        let _ = std::fs::File::options().append(true).open(&file).and_then(|f| f.set_modified(std::time::SystemTime::now()));
    }
    Ok(())
}

/// An app's Tailwind (docs/targets.feature T0b): every class any .ts or .tsx in `dir` uses, with Tailwind's own default
/// theme (not raea's brand theme), as plain CSS for a page. A folder with no code has no classes: no compile runs.
pub fn tailwind(dir: &Path) -> R<String> {
    let code = sources(dir);
    if code.is_empty() {
        return Ok(String::new());
    }
    compile(&code, "")
}

#[cfg(not(feature = "modules"))]
fn compile(_: &[PathBuf], _: &str) -> R<String> {
    Err("tailwind: this build leaves out the module runtime Tailwind's compiler runs on (the modules feature)".into())
}

/// The project's CSS: its brand kit's variables, then its Tailwind.
fn css(space: &Path, code: &[PathBuf]) -> R<String> {
    let theme_file = raea_io_home::home().join("engine/cli/template/theme.css");
    let theme = std::fs::read_to_string(&theme_file).map_err(|e| format!("styles.gen: {}: {e}", theme_file.display()))?;
    let tw = compile(code, &theme).map_err(|e| format!("styles.gen: {e}"))?;
    let brand = match std::fs::read_to_string(space.join("brand/brand.css")) {
        Ok(b) => b.split('\n').filter(|l| !l.trim_start().starts_with("@import")).collect::<Vec<_>>().join("\n"),
        Err(_) => String::new(),
    };
    Ok(format!("{brand}\n{tw}"))
}

/// Tailwind 4 for `code`'s candidates after `theme`, compiled on raea's JavaScriptCore
#[cfg(feature = "modules")]
fn compile(code: &[PathBuf], theme: &str) -> R<String> {
    let pkg = raea_io_home::home().join("sdk/react/tw/node_modules/tailwindcss");
    let mut files = BTreeMap::new();
    for e in std::fs::read_dir(&pkg).map_err(|e| format!("Tailwind's package isn't at {} ({e}; in a checkout: `bun install` in sdk/react/tw)", pkg.display()))?.flatten() {
        let p = e.path();
        if p.extension().is_some_and(|x| x == "css") {
            files.insert(p.file_name().unwrap().to_string_lossy().into_owned(), std::fs::read_to_string(&p).map_err(|e| e.to_string())?);
        }
    }
    let input = json!({ "candidates": candidates(code), "theme": theme, "files": files }).to_string();
    let entry = raea_native::js::Entry::Code { text: COMPILE.into(), ts: false, dir: pkg.clone() };
    let run = raea_native::js::run(&pkg, &pkg, entry, Vec::new(), Default::default(), input, Vec::new(), std::time::Duration::from_secs(60));
    if run.exit != 0 {
        return Err(format!("tailwind: {}", run.stderr.chars().take(400).collect::<String>()));
    }
    Ok(run.stdout)
}

/// Every whitespace-separated word inside a quoted string of `code`, first sighting first (void's project.ts order)
#[cfg_attr(not(feature = "modules"), allow(dead_code))]
fn candidates(code: &[PathBuf]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for f in code {
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        for s in quoted(&src) {
            for c in s.split(char::is_whitespace) {
                if !c.is_empty() && seen.insert(c.to_string()) {
                    out.push(c.to_string());
                }
            }
        }
    }
    out
}

/// The strings /(["'`])((?:(?!\1)[^\\]|\\.)*)\1/g matches: from a quote to the next unescaped same quote; a quote with
/// no closing one is passed over
#[cfg_attr(not(feature = "modules"), allow(dead_code))]
fn quoted(src: &str) -> Vec<&str> {
    let b = src.as_bytes();
    let utf8_len = |lead: u8| match lead {
        0xf0..=0xff => 4,
        0xe0..=0xef => 3,
        0xc0..=0xdf => 2,
        _ => 1,
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let q = b[i];
        if q == b'"' || q == b'\'' || q == b'`' {
            let mut j = i + 1;
            let mut closed = None;
            while j < b.len() {
                if b[j] == b'\\' {
                    if j + 1 >= b.len() {
                        break;
                    }
                    j += 1 + utf8_len(b[j + 1]);
                    continue;
                }
                if b[j] == q {
                    closed = Some(j);
                    break;
                }
                j += utf8_len(b[j]);
            }
            if let Some(end) = closed {
                out.push(&src[i + 1..end]);
                i = end + 1;
                continue;
            }
        }
        i += utf8_len(b[i]).max(1);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_quoted_words_as_project_ts() {
        assert_eq!(quoted(r#"a "b c" 'd' `e ${f}` "g\"h" 'unclosed"#), vec!["b c", "d", "e ${f}", "g\\\"h"]);
    }
}
