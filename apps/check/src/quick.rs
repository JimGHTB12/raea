//! `raea check EDIT.tsx --quick` (docs/skills.feature SK10). The quick check renders three stills of a composition (a
//! quarter in, the middle, the last frame) with their manifest, about a second, and runs the full check's text rules
//! and the person's text taste on it: clipped and cramped text, safe areas, sizes. Every agent's after-save hook runs
//! it (hook.rs): after a .tsx that registers a root is saved, its findings go back to the agent; a clean check says
//! nothing.
use super::*;
use std::path::Path;

/// The composition's length in frames, read from its source (the first `durationInFrames={N}`), 90 without one.
fn frames(src: &str) -> u64 {
    src.split("durationInFrames={").nth(1).and_then(|r| r.split('}').next()).and_then(|n| n.trim().parse().ok()).unwrap_or(90)
}

/// `raea check EDIT.tsx --quick [--shape S] [--platform P] [-o check.json]`: (report, P0 or P1 found). `tastes` are the
/// person's, evaluated by the raea CLI when --taste names them.
pub fn quick(args: &[String], tastes: Vec<raea_core_brief::Taste>) -> R<(String, bool)> {
    let file = args.iter().find(|a| a.ends_with(".tsx")).ok_or("raea check --quick takes a composition: EDIT.tsx")?;
    let src = std::fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
    let n = frames(&src).max(1);
    let at = format!("{},{},{}", n / 4, n / 2, n - 1);
    let tmp = std::env::temp_dir().join(format!("raea-quick-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).map_err(|e| format!("{}: {e}", tmp.display()))?;
    let manifest = tmp.join("render.json");
    let me = std::env::current_exe().map_err(|e| e.to_string())?;
    // the stills are part of this check, which keeps the one entry in the project's record (DR3)
    let out = std::process::Command::new(me).arg("render").arg(file).args(["--at", &at, "-o"]).arg(format!("{}/", tmp.display())).arg("--manifest").arg(&manifest).env("RAEA_RECORD", "off").output().map_err(|e| e.to_string())?;
    // rendered when its manifest was written: a render whose stills clip exits 1 (DR3), and those findings are read here
    let result = if manifest.is_file() {
        on_manifest(file, &manifest, args, tastes)
    } else {
        // a render that fails is the finding: the agent sees why
        let (err, out) = (String::from_utf8_lossy(&out.stderr), String::from_utf8_lossy(&out.stdout));
        let mut why = Vec::new();
        for line in err.lines().chain(out.lines()) {
            if line.contains("REFUSED") || line.contains("rror") {
                why.push(line);
            }
        }
        let f = raea_core_facts::Finding::new("render", raea_core_facts::Level::Error, why.join("\n"), "raea check --quick");
        Ok((format!("raea check {file}: it doesn't render\n{}\nraea check: {}", f.line(), raea_core_facts::finding::count(&[f])), true))
    };
    let _ = std::fs::remove_dir_all(&tmp);
    result
}

/// `raea check --quick`'s half (docs/skills.feature SK10): the text rules and the person's text taste on the manifest
/// of a few stills (`raea render EDIT.tsx --at ... --manifest`), no video. What it can't see (contrast, freezes,
/// sound, blotching) waits for the full check on the export.
fn on_manifest(file: &str, manifest: &Path, args: &[String], tastes: Vec<raea_core_brief::Taste>) -> R<(String, bool)> {
    let at = manifest.display();
    let m: Value = serde_json::from_str(&std::fs::read_to_string(manifest).map_err(|e| format!("{at}: {e}"))?).map_err(|e| format!("{at}: {e}"))?;
    let (mut issues, mut passed) = (Vec::new(), Vec::new());
    let runs = text_runs(&m);
    let (w, h) = (m["width"].as_f64().unwrap_or(1080.0), m["height"].as_f64().unwrap_or(1920.0));
    text_rules(&runs, None, w, h, flag(args, "--shape"), flag(args, "--platform"), &mut issues, &mut passed);
    taste_text(&taste_rules(&tastes), &runs, w, h, &mut issues, &mut passed);
    // and what the render of the stills found (the 3D's clipping, DR3)
    let more = serde_json::from_value(m["findings"].clone()).unwrap_or_default();
    report(file, issues, more, passed, flag(args, "-o"), m["fps"].as_f64().unwrap_or(30.0))
}

#[cfg(test)]
mod tests {
    #[test]
    fn frames_come_from_the_source() {
        assert_eq!(super::frames("<Composition durationInFrames={240} fps={30} />"), 240);
        assert_eq!(super::frames("no composition"), 90);
    }
}
