//! `raea world EDIT.tsx`: a composition's frame as one world (docs/direction.feature DR1), read with React stepped and
//! nothing drawn; the page laid out unless `--no-layout`. `--find` is one query of it, raea/ecs's find: the entities
//! with every component `with` names, none `without` names, and each `where` field equal (`"Layer.medium": "2d-type"`).

use raea_runtime_world::{Ask, Entity, Facts, Render};
use raea_runtime_bundle::flag;
use serde_json::{json, Value};

pub const USAGE: &str = "raea world EDIT.tsx [--composition ID] [--frame N | --at A-B|A,B] [--find JSON] [--no-layout] [--render OUT | --no-render] [--json]   (a frame as one world: its layers, text, parts, camera, shots, clips, cues and lines as entities with components, each measured, with what its newest render measured (contrast, light and movement, loudness, onsets, covers) or what waits for one; --find '{\"with\": [\"Text\", \"Layer\"], \"where\": {\"Layer.medium\": \"2d-type\"}}' is one query across media)";

/// A query: the components an entity has, those it hasn't, and fields (Component.field.field) equal to a value.
#[derive(Debug, Default, PartialEq)]
pub struct Find {
    pub with: Vec<String>,
    pub without: Vec<String>,
    pub fields: Vec<(String, Value)>,
}

impl Find {
    pub fn parse(s: &str) -> Result<Find, String> {
        let v: Value = serde_json::from_str(s).map_err(|e| format!("--find takes JSON ({{\"with\": [...], \"without\": [...], \"where\": {{\"Comp.field\": value}}}}): {e}"))?;
        let names = |k: &str| -> Result<Vec<String>, String> {
            match &v[k] {
                Value::Null => Ok(Vec::new()),
                Value::Array(a) => a.iter().map(|n| n.as_str().map(str::to_string).ok_or_else(|| format!("--find's {k} lists component names"))).collect(),
                _ => Err(format!("--find's {k} is a list of component names")),
            }
        };
        let fields = match &v["where"] {
            Value::Null => Vec::new(),
            Value::Object(m) => m.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            _ => return Err("--find's where is an object of \"Component.field\": value".into()),
        };
        let known = ["with", "without", "where"];
        if let Some(k) = v.as_object().and_then(|o| o.keys().find(|k| !known.contains(&k.as_str()))) {
            return Err(format!("--find has no \"{k}\" (it takes with, without and where)"));
        }
        Ok(Find { with: names("with")?, without: names("without")?, fields })
    }

    fn holds(&self, e: &Entity) -> bool {
        let at = |path: &str| -> Option<&Value> {
            let mut keys = path.split('.');
            let mut v = e.components.get(keys.next()?)?;
            for k in keys {
                v = v.get(k)?;
            }
            Some(v)
        };
        self.with.iter().all(|c| e.components.contains_key(c)) && !self.without.iter().any(|c| e.components.contains_key(c)) && self.fields.iter().all(|(p, v)| at(p) == Some(v))
    }
}

/// The entities of `w` that `q` finds, in id order.
pub fn find<'a>(w: &'a Facts, q: &Find) -> Vec<&'a Entity> {
    w.entities.iter().filter(|e| q.holds(e)).collect()
}

fn frames(args: &[String]) -> Result<Vec<i64>, String> {
    let n = |s: &str, k: &str| s.trim().parse::<i64>().map_err(|_| format!("{k} takes frame numbers, got \"{s}\""));
    match (flag(args, "--frame").as_deref(), flag(args, "--at").as_deref()) {
        (Some(_), Some(_)) => Err("give --frame or --at, not both".into()),
        (Some(f), None) => Ok(vec![n(f, "--frame")?]),
        // as render's --at: a list (A,B,C), or a range (A-B), each frame its own world
        (None, Some(r)) => match r.split_once('-') {
            Some((a, b)) => {
                let (a, b) = (n(a, "--at")?, n(b, "--at")?);
                if b < a {
                    return Err(format!("--at {r} runs backwards"));
                }
                Ok((a..=b).collect())
            }
            None => r.split(',').map(|f| n(f, "--at")).collect(),
        },
        (None, None) => Ok(vec![0]),
    }
}

/// One entity as a line: its id, parent and each component's gist.
fn line(e: &Entity) -> String {
    let s = raea_runtime_world::text_of;
    let r2 = |v: &Value| v.as_array().map(|a| a.iter().map(|x| format!("{:.2}", x.as_f64().unwrap_or(0.0))).collect::<Vec<_>>().join(", ")).unwrap_or_default();
    let mut out = format!("#{}{}", e.id, e.parent.map(|p| format!(" in #{p}")).unwrap_or_default());
    for (name, c) in &e.components {
        let gist = match name.as_str() {
            "Layer" => format!("{} at {}{}", s(&c["medium"]), c["order"], if c["opacity"].as_f64().unwrap_or(1.0) < 1.0 { format!(", opacity {}", c["opacity"]) } else { String::new() }),
            "Text" => format!("\"{}\"{}{}{}", s(&c["words"]), c["size"].as_f64().map(|v| format!(" {v} px")).unwrap_or_default(), if c["box"].is_array() { format!(" box ({})", r2(&c["box"])) } else { String::new() }, c["needs"].as_str().map(|n| format!(" (needs {n})")).unwrap_or_default()),
            "Part" => format!("{} \"{}\"{}{}", s(&c["kind"]), s(&c["name"]), if c["min"].is_array() { format!(" box ({}) to ({})", r2(&c["min"]), r2(&c["max"])) } else { " (not measured)".into() }, c["triangles"].as_u64().map(|t| format!(", {t} triangles")).unwrap_or_default()),
            "Camera" => format!("eye ({}) looking at ({}){}", r2(&c["eye"]), r2(&c["look"]), if c["velocity"].is_array() { format!(", moving ({}) m/s", r2(&c["velocity"])) } else { String::new() }),
            "Shot" => format!("{} frames {} to {}", c["name"].as_str().unwrap_or("(unnamed)"), c["from"], c["to"].as_i64().map_or("the end".into(), |t| t.to_string())),
            "Clip" => format!("{} at {}s of it, {} words", s(&c["source"]), c["time"].as_f64().map_or("?".into(), |t| format!("{t:.2}")), c["words"].as_array().map_or(0, Vec::len)),
            "Cue" => format!("{} from {:.2}s{}", s(&c["source"]), c["at"].as_f64().unwrap_or(0.0), c["gain"].as_f64().map(|g| format!(" at {g} dB")).unwrap_or_default()),
            "Line" => format!("\"{}\" from {:.2}s{}", s(&c["text"]), c["from"].as_f64().unwrap_or(0.0), c["needs"].as_str().map(|n| format!(" (needs {n})")).unwrap_or_default()),
            "Source" => format!("{}:{}{}", s(&c["file"]), c["line"], c["component"].as_str().map(|n| format!(" <{n}>")).unwrap_or_default()),
            "Contrast" => format!("{}:1 against its pixels", c["ratio"]),
            "Picture" => picture(c),
            "Onset" => c["at"].as_f64().map_or_else(|| "not heard where it starts".into(), |a| format!("heard at {a:.2}s ({:+} ms)", c["late_ms"].as_f64().unwrap_or(0.0))),
            "Covers" => c.as_array().into_iter().flatten().map(|x| format!("#{} {:.0}%", x["layer"], x["share"].as_f64().unwrap_or(0.0) * 100.0)).collect::<Vec<_>>().join(", "),
            _ => c.get("to").map_or_else(|| c.to_string(), |t| format!("#{t}")),
        };
        out.push_str(&format!("  {name} {gist}"));
    }
    out
}

/// A picture's light and movement as words.
fn picture(c: &Value) -> String {
    let moved = c["motion"].as_f64().map(|m| format!(", motion {m:.3}")).unwrap_or_default();
    format!("luma {:.3}{moved}{}", c["luma"].as_f64().unwrap_or(0.0), c["frames"].as_u64().map(|n| format!(" over {n} frames")).unwrap_or_default())
}

/// The frame's resources as lines: the render its facts came from, its mix and picture, what waits for one.
fn resources(w: &Facts) -> Vec<String> {
    let r = &w.resources;
    let mut out = Vec::new();
    if let Some(x) = r.get("Rendered") {
        let stale = if x["stale"] == true { " (made before the composition last changed: its measures are of that version)" } else { "" };
        out.push(format!("  rendered: {}{}{stale}", raea_runtime_world::text_of(&x["out"]), x["record"].as_str().map(|i| format!(" ({i})")).unwrap_or_default()));
    }
    if let Some(m) = r.get("Mix") {
        out.push(match m["lufs"].as_f64() {
            Some(l) => format!("  mix: {l:.1} LUFS{}", m["peak"].as_f64().map(|p| format!(", true peak {p:.1} dBTP")).unwrap_or_default()),
            None => "  mix: no sound".into(),
        });
    }
    if let Some(p) = r.get("Picture") {
        out.push(format!("  picture: {}", picture(p)));
    }
    if let Some(Value::Object(ws)) = r.get("Waits") {
        let mut by: Vec<(String, Vec<&str>)> = Vec::new();
        for (c, on) in ws {
            let on = raea_runtime_world::text_of(on);
            match by.iter_mut().find(|(o, _)| *o == on) {
                Some((_, cs)) => cs.push(c),
                None => by.push((on, vec![c])),
            }
        }
        out.extend(by.into_iter().map(|(on, cs)| format!("  waits for {on}: {}", cs.join(", "))));
    }
    out
}

/// Which render's facts join: `--render OUT`, none with `--no-render`, else the newest in the project's record.
fn render(args: &[String]) -> Result<Render, String> {
    match (flag(args, "--render"), args.iter().any(|a| a == "--no-render")) {
        (Some(_), true) => Err("give --render or --no-render, not both".into()),
        (Some(o), false) => Ok(Render::From(o.into())),
        (None, true) => Ok(Render::Off),
        (None, false) => Ok(Render::Newest),
    }
}

fn run(args: &[String]) -> Result<String, String> {
    let edit = args.iter().find(|a| a.ends_with(".tsx")).ok_or(USAGE)?;
    let q = flag(args, "--find").as_deref().map(Find::parse).transpose()?;
    let ask = Ask { frames: frames(args)?, layout: !args.iter().any(|a| a == "--no-layout"), render: render(args)? };
    let worlds = raea_runtime_world::read(std::path::Path::new(edit), &raea_runtime_bundle::comp_opts(args)?, &ask)?;
    if args.iter().any(|a| a == "--json") {
        let one = |w: &Facts| match &q {
            Some(q) => json!({"frame": w.frame, "found": find(w, q)}),
            None => serde_json::to_value(w).unwrap_or(Value::Null),
        };
        let v = if worlds.len() == 1 { one(&worlds[0]) } else { Value::Array(worlds.iter().map(one).collect()) };
        return Ok(serde_json::to_string_pretty(&v).unwrap_or_default());
    }
    let mut out = Vec::new();
    for w in &worlds {
        let shown = q.as_ref().map_or_else(|| w.entities.iter().collect(), |q| find(w, q));
        out.push(format!("frame {} ({}x{} at {} fps): {} of {} entities", w.frame, w.width, w.height, w.fps, shown.len(), w.entities.len()));
        out.extend(resources(w));
        out.extend(shown.into_iter().map(line));
    }
    Ok(out.join("\n"))
}

/// `raea world ...`: its exit code.
pub fn cli(args: &[String]) -> i32 {
    let got = run(args);
    match &got {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("raea world: {e}"),
    }
    if got.is_ok() { 0 } else { 2 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raea_runtime_world::{Layer, Medium, Part, Text};

    fn world() -> Facts {
        let mut w = Facts::new(0, 1920, 1080, 30.0);
        let lyric = w.spawn(None);
        w.put(lyric, &Layer { medium: Medium::Type2d, order: 2, opacity: 1.0, blend: None });
        w.put(lyric, &Text { words: "GIVE IT BACK".into(), font: None, size: Some(96.0), box_: Some([0.1, 0.1, 0.6, 0.2]), extruded: 0.0, lines: Some(1), needs: None });
        let scene = w.spawn(None);
        w.put(scene, &Layer { medium: Medium::Scene3d, order: 0, opacity: 1.0, blend: None });
        let sign = w.spawn(Some(scene));
        w.put(sign, &Text { words: "OPEN".into(), font: None, size: None, box_: None, extruded: 0.2, lines: Some(1), needs: None });
        w.put(sign, &Part { kind: "text3d".into(), name: "sign".into(), tags: vec![], at: [0.0; 3], rotation: [0.0, 0.0, 0.0, 1.0], scale: [1.0; 3], min: None, max: None, triangles: None, src: None, material: None, color: None });
        let hidden = w.spawn(None);
        w.put(hidden, &Layer { medium: Medium::Type2d, order: 3, opacity: 1.0, blend: None });
        w.put(hidden, &Text { words: "UNDER".into(), font: None, size: None, box_: None, extruded: 0.0, lines: None, needs: None });
        w.relate(hidden, "MaskedBy", sign);
        w
    }

    #[test]
    fn one_find_spans_media() {
        let w = world();
        let flat = Find::parse(r#"{"with": ["Text", "Layer"], "where": {"Layer.medium": "2d-type"}, "without": ["MaskedBy"]}"#).unwrap();
        let got: Vec<u32> = find(&w, &flat).iter().map(|e| e.id).collect();
        assert_eq!(got, vec![0], "the 2D lyric no part masks, not the 3D sign or the masked line");
        let words_in_3d = Find::parse(r#"{"with": ["Text", "Part"], "where": {"Text.extruded": 0.2}}"#).unwrap();
        assert_eq!(find(&w, &words_in_3d).len(), 1);
        assert!(line(&w.entities[0]).contains("\"GIVE IT BACK\" 96 px box (0.10, 0.10, 0.60, 0.20)"), "{}", line(&w.entities[0]));
    }

    #[test]
    fn a_bad_query_or_range_says_what_it_takes() {
        assert!(Find::parse(r#"{"has": ["Text"]}"#).unwrap_err().contains("no \"has\""));
        assert!(Find::parse("Text").is_err());
        let a = |s: &[&str]| s.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert_eq!(frames(&a(&["e.tsx", "--at", "3-5"])).unwrap(), vec![3, 4, 5]);
        assert!(frames(&a(&["--at", "5-3"])).unwrap_err().contains("backwards"));
        assert_eq!(frames(&a(&["--at", "3,7"])).unwrap(), vec![3, 7], "a list, as render's --at");
        assert_eq!(frames(&a(&["e.tsx"])).unwrap(), vec![0]);
    }
}
