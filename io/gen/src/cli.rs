//! `raea gen …`: the verbs over flows, made's catalog and ComfyUI workflows. Each returns what it prints.

use std::path::{Path, PathBuf};

use serde_json::{Value as J, json};

use crate::comfy;
use crate::flow::{Flow, Step};
use crate::made::Made;
use crate::run::{Local, Run};

pub const USAGE: &str = "raea gen models [--kind image|video|audio] [--query WORDS] | balance | types [-o FILE] | flow FLOW.ts [-o FLOW.json]
       raea gen image|video|music|sfx|speech|mesh|text \"PROMPT\" [--model M] [--image FILE]... [--seconds S] [--aspect W:H] [-o DIR] [--max-cents N] [--plan]   (mesh: --model tripo:H3.1|tripo:P2|fal:<endpoint>)
       raea gen plan FLOW.json|FLOW.ts [--exact] | raea gen run FLOW.json|FLOW.ts [-o DIR] [--max-cents N] [--exact] [--comfy-on-replicate] [--no-local] [--no-tripo]
       raea gen comfy import WORKFLOW.json [-o FLOW.json] | comfy export FLOW.json|FLOW.ts [-o WORKFLOW.json] | comfy needs WORKFLOW.json [-o NEEDS.json]
       raea gen comfy run WORKFLOW.json [-o DIR]   (any workflow as it is, on the ComfyUI server: its input files uploaded, every output fetched)
       raea gen comfy view WORKFLOW.json [-o FILE] | comfy set WORKFLOW.json CHANGES.json   (nodes, settings by name, links, groups; edits written back)
       raea gen comfy schema [URL] | comfy coverage DIR | comfy up";

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(String::as_str)
}

/// A JSON file's value, or an error naming the file.
fn read_json(p: impl AsRef<Path>) -> Result<J, String> {
    let p = p.as_ref();
    let b = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
    serde_json::from_slice(&b).map_err(|e| format!("{}: not JSON ({e})", p.display()))
}

/// A flow file: JSON as it is, or TypeScript (raea/gen's `flow()` as the default export) run with bun under the SDK's
/// import names.
fn read_flow(p: &str) -> Result<Flow, String> {
    if ![".ts", ".mts", ".tsx"].iter().any(|e| p.ends_with(e)) {
        return Flow::parse(&read_json(p)?);
    }
    let file = std::path::absolute(p).map_err(|e| format!("{p}: {e}"))?;
    let sdk = raea_io_home::home().join("sdk");
    let entry = std::env::temp_dir().join(format!("raea-gen-flow-{}.ts", std::process::id()));
    let js = "const m = await import(process.argv[2]);\nconst f = typeof m.default === \"function\" ? await m.default() : await m.default;\nconsole.log(JSON.stringify(f));\n";
    std::fs::write(&entry, js).map_err(|e| format!("{}: {e}", entry.display()))?;
    let out = std::process::Command::new("bun")
        .arg("--tsconfig-override")
        .arg(sdk.join(raea_io_home::sdk::LIST))
        .arg(&entry)
        .arg(&file)
        .output();
    let _ = std::fs::remove_file(&entry);
    let out = out.map_err(|e| format!("{p}: a TypeScript flow runs with bun, which didn't start ({e})"))?;
    // bun reports an override that isn't the file's own folder as an internal error; it isn't one
    let err: String = String::from_utf8_lossy(&out.stderr).lines().filter(|l| !l.contains("directory mismatch")).collect::<Vec<_>>().join("\n");
    if !out.status.success() {
        return Err(format!("{p}: {}", err.trim()));
    }
    let v: J = serde_json::from_slice(&out.stdout).map_err(|e| format!("{p}: its default export isn't a flow ({e})"))?;
    Flow::parse(&v)
}

fn write_or_print(v: &J, out: Option<&str>) -> Result<String, String> {
    let text = serde_json::to_string_pretty(v).unwrap_or_default();
    match out {
        Some(o) => std::fs::write(o, &text).map(|_| format!("wrote {o}")).map_err(|e| format!("{o}: {e}")),
        None => Ok(text),
    }
}

fn runner<'a>(args: &[String], base: &Path, local: Option<&'a dyn Local>) -> Run<'a> {
    let dir = flag(args, "-o").map(PathBuf::from).unwrap_or_else(|| base.join("gen"));
    let mut r = Run::new(dir, base.to_path_buf());
    if let Some(c) = flag(args, "--max-cents").and_then(|c| c.parse().ok()) {
        r.max_cents = c;
    }
    r.policy.exact = args.iter().any(|a| a == "--exact");
    r.policy.comfy_on_replicate = args.iter().any(|a| a == "--comfy-on-replicate") || crate::setting("RAEA_COMFY_ON") .as_deref() == Some("replicate");
    r.policy.tripo = !args.iter().any(|a| a == "--no-tripo") && crate::tripo::available();
    if let Some(l) = local.filter(|_| !args.iter().any(|a| a == "--no-local")) {
        r.policy.local = l.names();
        r.local = Some(l);
    }
    r
}

pub fn command(args: &[String], local: Option<&dyn Local>) -> Result<String, String> {
    let verb = args.first().map(String::as_str).unwrap_or("");
    match verb {
        "models" => {
            let kind = flag(args, "--kind");
            let q = flag(args, "--query").map(str::to_ascii_lowercase);
            let mut out = String::new();
            for m in Made::from_settings()?.models()? {
                if kind.is_some_and(|k| m["kind"] != k) || q.as_ref().is_some_and(|q| !m.to_string().to_ascii_lowercase().contains(q)) {
                    continue;
                }
                out += &format!("{:<34} {:<6} {}\n", m["id"].as_str().unwrap_or(""), m["kind"].as_str().unwrap_or(""), m["pricing"]["display"].as_str().unwrap_or(""));
            }
            Ok(out.trim_end().to_string())
        }
        "balance" => {
            let made = Made::from_settings().and_then(|m| m.balance_cents()).map(|c| format!("{:.1} credits (${:.2}) on made.app", c as f64 / 10.0, c as f64 / 100.0)).unwrap_or_else(|e| format!("made.app: {e}"));
            // a Tripo key's credits are its API wallet (developers.tripo3d.ai/en/billing), not the tripo3d.ai dashboard's
            let tripo = crate::tripo::balance().map(|(b, f)| format!("{b} credits ({f} frozen) on Tripo's API wallet, {}", crate::tripo::BILLING)).unwrap_or_else(|e| format!("tripo: {e}"));
            Ok(format!("{made}\n{tripo}"))
        }
        "plan" | "run" => {
            let path = args.get(1).ok_or(USAGE)?;
            let flow = read_flow(path)?;
            let base = Path::new(path).parent().map(Path::to_path_buf).unwrap_or_default();
            let r = runner(args, &base, local);
            write_or_print(&if verb == "plan" { r.plan(&flow)? } else { r.run(&flow)? }, None)
        }
        "image" | "video" | "music" | "sfx" | "speech" | "mesh" | "text" => {
            let prompt = args.get(1).filter(|p| !p.starts_with("--")).ok_or(USAGE)?;
            let images: Vec<&str> = args.windows(2).filter(|w| w[0] == "--image").map(|w| w[1].as_str()).collect();
            let mut flow = Flow::default();
            for (k, f) in images.iter().enumerate() {
                let abs = std::fs::canonicalize(f).map_err(|e| format!("{f}: {e}"))?;
                flow.steps.insert(format!("in{k}"), Step::new("media.load", json!({"path": abs})));
            }
            let refs: Vec<J> = (0..images.len()).map(|k| json!(format!("@in{k}"))).collect();
            let (op, mut inputs) = match verb {
                "image" if refs.is_empty() => ("image.generate", json!({"prompt": prompt})),
                "image" => ("image.edit", json!({"prompt": prompt, "image": if refs.len() == 1 { refs[0].clone() } else { J::Array(refs.clone()) }})),
                "video" => ("video.generate", match refs.len() { 0 => json!({"prompt": prompt}), 1 => json!({"prompt": prompt, "start_image": refs[0]}), _ => json!({"prompt": prompt, "start_image": refs[0], "end_image": refs[1]}) }),
                "music" => ("audio.music", json!({"prompt": prompt})),
                "sfx" => ("audio.sfx", json!({"prompt": prompt})),
                "speech" => ("audio.speech", json!({"text": prompt})),
                "mesh" => ("mesh.generate", if refs.is_empty() { json!({"prompt": prompt}) } else { json!({"prompt": prompt, "image": refs[0]}) }),
                _ => ("text.generate", if refs.is_empty() { json!({"prompt": prompt}) } else { json!({"prompt": prompt, "image": refs[0]}) }),
            };
            if let Some(s) = flag(args, "--seconds").and_then(|s| s.parse::<f64>().ok()) {
                inputs["seconds"] = json!(s);
            }
            if let Some(a) = flag(args, "--aspect") {
                inputs["aspect_ratio"] = json!(a);
            }
            let mut step = Step::new(op, inputs);
            step.model = flag(args, "--model").map(|m| if m.contains(':') { m.to_string() } else { format!("made:{m}") });
            flow.steps.insert("out".into(), step);
            flow.out = vec!["out".into()];
            flow.check()?;
            let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
            let r = runner(args, &cwd, local);
            write_or_print(&if args.iter().any(|a| a == "--plan") { r.plan(&flow)? } else { r.run(&flow)? }, None)
        }
        "comfy" => comfy_verb(&args[1..]),
        "flow" => write_or_print(&read_flow(args.get(1).ok_or(USAGE)?)?.to_json(), flag(args, "-o")),
        "types" => {
            let ts = crate::types::typescript();
            match flag(args, "-o") {
                Some(o) => std::fs::write(o, &ts).map(|_| format!("wrote {o}")).map_err(|e| format!("{o}: {e}")),
                None => Ok(ts),
            }
        }
        _ => Err(USAGE.to_string()),
    }
}

fn comfy_verb(args: &[String]) -> Result<String, String> {
    match args.first().map(String::as_str).unwrap_or("") {
        "import" => {
            let l = comfy::import(&read_json(args.get(1).ok_or(USAGE)?)?)?;
            l.flow.check()?;
            let kept = l.flow.steps.values().filter(|s| s.op == "comfy").count();
            eprintln!("raea gen: {} steps ({} named, {} kept as ComfyUI nodes), {} outputs", l.flow.steps.len(), l.flow.steps.len() - kept, kept, l.flow.out.len());
            write_or_print(&l.flow.to_json(), flag(args, "-o"))
        }
        "needs" => {
            let path = args.get(1).ok_or(USAGE)?;
            write_or_print(&comfy::needs::needs(&read_json(path)?, &comfy::schema::load()), flag(args, "-o"))
        }
        "export" => {
            let flow = read_flow(args.get(1).ok_or(USAGE)?)?;
            write_or_print(&comfy::export(&flow)?, flag(args, "-o"))
        }
        "schema" => {
            let mut c = comfy::server::Comfy::from_settings();
            if let Some(u) = args.get(1) {
                c.url = u.trim_end_matches('/').to_string();
            }
            Ok(format!("{} node classes from {} kept in {}", c.save_schema()?, c.url, crate::made::cached(comfy::schema::CACHE_NAME).display()))
        }
        "up" => {
            let c = comfy::server::Comfy::from_settings();
            Ok(format!("ComfyUI at {}: {}", c.url, if c.up() { "answering" } else { "not answering (raea gen comfy setup installs one; RAEA_COMFY_URL names a hosted one)" }))
        }
        "coverage" => coverage(Path::new(args.get(1).ok_or(USAGE)?)),
        "run" => run_workflow(args),
        // a workflow as a whiteboard shows it, and a board's edits written back into it (apps/app whiteboards, CF1-2)
        "view" => write_or_print(&comfy::view::view(&read_json(args.get(1).ok_or(USAGE)?)?, &comfy::schema::load())?, flag(args, "-o")),
        "set" => {
            let path = args.get(1).ok_or(USAGE)?;
            let mut w = read_json(path)?;
            let n = comfy::view::set(&mut w, &read_json(args.get(2).ok_or(USAGE)?)?, &comfy::schema::load())?;
            // written whole, then moved over the old one: a reader never sees half a workflow
            let tmp = format!("{path}.{}.tmp", std::process::id());
            std::fs::write(&tmp, serde_json::to_string_pretty(&w).map_err(|e| e.to_string())? + "\n").map_err(|e| format!("{tmp}: {e}"))?;
            std::fs::rename(&tmp, path).map_err(|e| format!("{path}: {e}"))?;
            Ok(format!("{n} changes written to {path}"))
        }
        _ => Err(USAGE.to_string()),
    }
}

/// Every workflow under a folder imported: how many read, how many steps came out named or kept as ComfyUI, which
/// node classes were kept most, and each failure's reason.
fn coverage(dir: &Path) -> Result<String, String> {
    let schema = comfy::schema::load();
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "json")).collect();
    files.sort();
    let (mut read, mut failed, mut named, mut kept, mut whole) = (0, Vec::new(), 0, 0, 0);
    let mut ops: std::collections::BTreeMap<String, usize> = Default::default();
    let mut kept_classes: std::collections::BTreeMap<String, usize> = Default::default();
    let mut unknown: std::collections::BTreeMap<String, usize> = Default::default();
    let mut why: std::collections::BTreeMap<String, usize> = Default::default();
    for f in &files {
        let Ok(v) = read_json(f) else { continue };
        if !(v["nodes"].is_array() || v.as_object().is_some_and(|o| o.values().any(|n| matches!(n["class_type"], J::String(_))))) {
            continue;
        }
        let g = match comfy::read::read(&v, &schema) {
            Ok(g) => g,
            Err(e) => { failed.push(format!("{}: {e}", f.display())); continue; }
        };
        for n in g.values().filter(|n| !schema.contains_key(&n.class)) {
            *unknown.entry(n.class.clone()).or_default() += 1;
        }
        let l = comfy::lift::lift(&g, &schema);
        if let Err(e) = l.flow.check() {
            failed.push(format!("{}: {e}", f.file_name().unwrap_or_default().to_string_lossy()));
            continue;
        }
        read += 1;
        if l.flow.steps.values().all(|s| s.op != "comfy" || s.op == "media.load") && !l.flow.out.is_empty() {
            whole += 1;
        }
        for s in l.flow.steps.values() {
            *ops.entry(s.op.clone()).or_default() += 1;
            if s.op == "comfy" {
                kept += 1;
                if let Some(w) = s.from.as_deref().and_then(|f| f.split_once("kept: ")).map(|(_, w)| w) {
                    let w: String = w.split_whitespace().map(|t| if t.chars().any(|c| c.is_ascii_digit()) { "_" } else { t }).collect::<Vec<_>>().join(" ");
                    *why.entry(w).or_default() += 1;
                }
                let out = s.inputs["output"].as_str().unwrap_or("");
                *kept_classes.entry(s.inputs["nodes"][out]["class_type"].as_str().unwrap_or("?").to_string()).or_default() += 1;
            } else if s.op != "media.load" {
                named += 1;
            }
        }
    }
    let top = |m: &std::collections::BTreeMap<String, usize>, n: usize| {
        let mut v: Vec<(&String, &usize)> = m.iter().collect();
        v.sort_by(|a, b| b.1.cmp(a.1));
        v.into_iter().take(n).map(|(k, c)| format!("{k} {c}")).collect::<Vec<_>>().join(", ")
    };
    Ok(serde_json::to_string_pretty(&json!({
        "workflows": files.len(), "read": read, "failed": failed.len(), "every_step_named": whole,
        "steps_named": named, "steps_kept_comfy": kept, "ops": ops,
        "kept_most": top(&kept_classes, 25), "classes_without_schema": top(&unknown, 15), "samplers_kept_because": top(&why, 20),
        "failures": failed.iter().take(15).collect::<Vec<_>>(),
    })).unwrap_or_default())
}

/// A workflow run as it is on the ComfyUI server: each Load node's file (beside the workflow, or a path) uploaded
/// first, every output node's files fetched into DIR.
fn run_workflow(args: &[String]) -> Result<String, String> {
    let path = args.get(1).ok_or(USAGE)?;
    let base = Path::new(path).parent().map(Path::to_path_buf).unwrap_or_default();
    let schema = comfy::schema::load();
    let mut g = comfy::read::read(&read_json(path)?, &schema)?;
    let server = comfy::server::Comfy::from_settings();
    if !server.wake(std::time::Duration::from_secs(600)) {
        return Err(format!("nothing answers at {} (tools/comfy/setup.sh serve; or RAEA_COMFY_URL)", server.url));
    }
    for n in g.values_mut() {
        for key in ["image", "file", "video", "audio"] {
            let Some(name) = n.inputs.get(key).and_then(|v| v.as_str()).map(str::to_string) else { continue };
            let f = if Path::new(&name).is_absolute() { PathBuf::from(&name) } else { base.join(&name) };
            if n.class.starts_with("Load") && f.is_file() {
                n.inputs.insert(key.into(), json!(server.upload(&f)?));
            }
        }
    }
    let pid = server.queue(&comfy::read::to_api(&g))?;
    let entry = server.wait(&pid, std::time::Duration::from_secs(3600))?;
    let dir = PathBuf::from(flag(args, "-o").unwrap_or("comfy-out"));
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut files = Vec::new();
    // which node made each file too, so a whiteboard puts it beside its Save node (apps/app CF2)
    let mut outputs = serde_json::Map::new();
    for node in entry["outputs"].as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default() {
        let made = server.fetch(&entry, &node, &dir).unwrap_or_default();
        outputs.insert(node.clone(), json!(made));
        files.extend(made);
    }
    write_or_print(&json!({"prompt_id": pid, "files": files, "outputs": outputs}), None)
}
