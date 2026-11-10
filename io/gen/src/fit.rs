//! A generated model measured and fitted before a scene uses it (raea-core-meshcheck): `raea gen check MODEL.glb`, and
//! every mesh.generate output of a run, by its step's `size` input (`{h, dims, turn, faces}`: its real height in
//! metres, its real width, depth and height, the degrees to turn its front to +z, its triangle budget). With `h` or
//! `turn`, the output under its own name is rewritten fitted (scaled, turned, its origin at the bottom centre); the
//! step's cached file stays as the generator made it.

use std::path::Path;

use raea_core_facts::{Finding, finding};
use raea_core_meshcheck::{Glb, Want, check, fix};
use serde_json::{Value as J, json};

use crate::flow::Flow;

pub const USAGE: &str = "raea gen check MODEL.glb [--h METRES] [--dims W,D,H] [--turn DEG] [--faces N | --seen METRES] [--parts a,b] [--rig] [--solid] [-o FIXED.glb] [--json]   (a model measured: size, pivot, facing, triangles, loose pieces, holes, textures, and the Blender script each fault needs; -o writes it fitted)";

/// A step's `fit` input, or the flags, as what the person wants of the model.
pub fn want_of(v: &J) -> Want {
    let dims = v["dims"].as_array().filter(|a| a.len() == 3).map(|a| [a[0].as_f64().unwrap_or(0.0), a[1].as_f64().unwrap_or(0.0), a[2].as_f64().unwrap_or(0.0)]);
    let parts = v["parts"].as_array().into_iter().flatten().filter_map(|p| p.as_str().map(str::to_string)).collect();
    Want { h: v["h"].as_f64(), dims, faces: v["faces"].as_u64().map(|f| f as usize), turn: v["turn"].as_f64().unwrap_or(0.0), seen: v["seen"].as_f64(), parts, rig: v["rig"] == true, solid: v["solid"] == true, file: None }
}

/// The report's `found` as raea's Findings (docs/direction.feature DR3), each about the model, worst first, with
/// their count; its passes are not findings. A fix's command was in its words, and is its fix now.
fn as_findings(mut r: J) -> J {
    let file = r["file"].as_str().unwrap_or("model").to_string();
    let fs = finding::ranked(
        r["found"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| Finding::of_value(v, "raea gen check"))
            .map(|mut f| {
                if let Some(cut) = f.fix.as_ref().and_then(|x| f.says.strip_suffix(&format!("; fix: {x}"))) {
                    f.says = cut.to_string();
                }
                if f.about.is_empty() { f.about(&file, None) } else { f }
            })
            .collect(),
    );
    if let Some(o) = r.as_object_mut() {
        for k in ["found", "errors", "warnings"] {
            o.remove(k);
        }
    }
    r["count"] = json!(finding::count(&fs));
    r["findings"] = json!(fs);
    r
}

/// Whether a report holds an error.
fn failed(r: &J) -> bool {
    r["findings"].as_array().into_iter().flatten().any(|f| f["level"] == "error")
}

/// One model measured, and written fitted to `out` when given (`out` may be the model itself).
pub fn check_file(path: &Path, want: &Want, out: Option<&Path>) -> Result<J, String> {
    let p = path.display().to_string();
    if let Some(f) = raea_core_meshcheck::blender::unloadable(&p) {
        return Ok(as_findings(json!({"file": p, "found": [f]})));
    }
    let g = Glb::read(&p)?;
    let mut r = check(&g, g.bytes.len(), &Want { file: Some(p.clone()), ..want.clone() })?;
    r["file"] = json!(p);
    if let Some(o) = out {
        let f = &r["fixed"];
        let t = f["translate"].as_array().ok_or("the model has nothing to fit")?;
        let t = [t[0].as_f64().unwrap_or(0.0), t[1].as_f64().unwrap_or(0.0), t[2].as_f64().unwrap_or(0.0)];
        let bytes = fix(&g, f["scale"].as_f64().unwrap_or(1.0), want.turn, t)?;
        // written whole, then moved over: a scene loading it never reads half a model
        let tmp = o.with_extension(format!("glb.{}.tmp", std::process::id()));
        std::fs::write(&tmp, &bytes).map_err(|e| format!("{}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, o).map_err(|e| format!("{}: {e}", o.display()))?;
        // a licence or provenance sidecar travels with the model
        for side in ["json", "c2pa"] {
            let (a, b) = (path.with_extension(format!("glb.{side}")), o.with_extension(format!("glb.{side}")));
            if a.exists() && a != b {
                let _ = std::fs::copy(&a, &b);
            }
        }
        r["wrote"] = json!(o.display().to_string());
    }
    Ok(as_findings(r))
}

/// `raea gen check ...`
pub fn command(args: &[String]) -> Result<String, String> {
    let flag = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let file = args.get(1).filter(|f| !f.starts_with("--")).ok_or(USAGE)?;
    let num = |k: &str| flag(k).map(|v| v.parse::<f64>().map_err(|_| format!("{k} takes a number, got {v:?}"))).transpose();
    let dims = flag("--dims").map(|d| {
        let v: Vec<f64> = d.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        if v.len() == 3 { Ok([v[0], v[1], v[2]]) } else { Err(format!("--dims takes W,D,H in metres, got {d:?}")) }
    }).transpose()?;
    let parts = flag("--parts").map(|p| raea_core_meshcheck::parts(&p)).unwrap_or_default();
    let on = |k: &str| args.iter().any(|a| a == k);
    let want = Want { h: num("--h")?, dims, faces: num("--faces")?.map(|f| f as usize), turn: num("--turn")?.unwrap_or(0.0), seen: num("--seen")?, parts, rig: on("--rig"), solid: on("--solid"), file: None };
    let out = flag("-o");
    let r = check_file(Path::new(file), &want, out.as_deref().map(Path::new))?;
    let text = if args.iter().any(|a| a == "--json") {
        serde_json::to_string_pretty(&r).unwrap_or_default()
    } else {
        say(&r)
    };
    if failed(&r) {
        println!("{text}");
        std::process::exit(1);
    }
    Ok(text)
}

/// The findings as lines, worst first, then what was measured, then their count.
pub fn say(r: &J) -> String {
    let mut t: Vec<String> = r["findings"].as_array().into_iter().flatten().filter_map(|f| serde_json::from_value::<Finding>(f.clone()).ok()).map(|f| f.line()).collect();
    let m = &r["measured"];
    // a file raea can't read has no measures: its summary says so instead of nulls
    let measured = if m["triangles"].is_null() { "not read".to_string() } else { format!("{} triangles, {} x {} x {} m", m["triangles"], m["size"][0], m["size"][1], m["size"][2]) };
    t.push(format!("{}: {measured}{}", r["file"].as_str().unwrap_or("model"), r["wrote"].as_str().map(|w| format!("; fitted copy {w}")).unwrap_or_default()));
    t.push(format!("raea gen check: {}", r["count"].as_str().unwrap_or("")));
    t.join("\n")
}

/// After a run: every mesh.generate output checked, and fitted in place under its own name when its step says how
/// (`size.h` or `size.turn`). The reports go into the run's result as `checks`.
pub fn after_run(flow: &Flow, result: &mut J) {
    let mut checks = serde_json::Map::new();
    for name in &flow.out {
        let Some(step) = flow.steps.get(name).filter(|s| s.op == "mesh.generate") else { continue };
        let fit = step.inputs.get("size").cloned().unwrap_or(J::Null);
        let want = want_of(&fit);
        let files: Vec<String> = match &result["out"][name] {
            J::Array(a) => a.iter().filter_map(|f| f.as_str().map(str::to_string)).collect(),
            J::String(s) => vec![s.clone()],
            _ => vec![],
        };
        for f in files.iter().filter(|f| f.to_ascii_lowercase().ends_with(".glb")) {
            let p = Path::new(f);
            let fitted = want.h.is_some() || want.turn != 0.0;
            let r = check_file(p, &want, fitted.then_some(p)).unwrap_or_else(|e| as_findings(json!({"file": f, "found": [{"check": "read", "level": "error", "says": e}]})));
            eprintln!("raea gen: {}", say(&r).replace('\n', "\nraea gen: "));
            checks.insert(name.clone(), r);
        }
    }
    if !checks.is_empty() {
        result["checks"] = J::Object(checks);
    }
}
