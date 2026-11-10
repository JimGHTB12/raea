//! `raea solid view FILE.ts -o DIR [--set k=v,...]`: one solid's review sheets (research note agent-tool-surface.md's
//! `solid view`). The file's parts are put in a small scene, each on its own clay-friendly colour, on a slab under
//! a key and fill light, with a hero camera from the front-right corner, and `raea render --review` takes the fixed
//! views of it in colour and clay. It prints a sidecar: the sheets, the solid's box, whether it is in more than one
//! piece, and solid info's findings, so one call gives the numbers and the pictures. The judge reads the sheets only.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

const COLOURS: [&str; 8] = ["#c9b79c", "#8fa6b8", "#b48a78", "#9cb48a", "#b8a46a", "#a08cb4", "#7fb0a8", "#c4a0a0"];

/// The scene the sheets are taken of: the file's parts (built with `set` over its params), a slab under them, lights
/// and a camera from the box.
fn scene(file: &Path, set: &Map<String, Value>, lo: [f64; 3], hi: [f64; 3]) -> String {
    let c = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
    let r = (((hi[0] - lo[0]).powi(2) + (hi[1] - lo[1]).powi(2) + (hi[2] - lo[2]).powi(2)).sqrt() / 2.0).max(0.02);
    let d = r / (20f64.to_radians().sin()) * 1.1;
    let k = 1.0 / 3f64.sqrt();
    let eye = [c[0] + d * k, c[1] + d * k * 0.7, c[2] + d * k];
    let slab = ((hi[0] - lo[0]).max(hi[2] - lo[2]) * 0.8).max(0.05);
    let thick = (r * 0.02).max(0.002);
    format!(
        r##"import {{ AbsoluteFill, Composition, registerRoot }} from "raea";
import {{ Cuboid }} from "raea/3d";
import {{ StandardMaterial }} from "raea/pbr";
import {{ Mesh3d }} from "raea/mesh";
import {{ AmbientLight, DirectionalLight }} from "raea/light";
import {{ Transform }} from "raea/transform";
import {{ Camera3d }} from "raea/camera";
import * as m from {file:?};
const spec: Record<string, unknown> = (m as any).params ?? {{}};
const p: Record<string, unknown> = {{}};
for (const [k, v] of Object.entries(spec)) p[k] = Array.isArray(v) ? v[2] : (v && typeof v === "object" && "default" in (v as any)) ? (v as any).default : v;
Object.assign(p, {set});
let parts: Record<string, any> | null = null;
const get = () => {{
  if (parts) return parts;
  const made = typeof (m as any).default === "function" ? (m as any).default(p) : (m as any).default;
  return (parts = typeof made?.expr === "string" ? {{ solid: made }} : made);
}};
const COLOURS = {colours:?};
function View() {{
  return (
    <AbsoluteFill style={{{{ background: "#1c1c1e" }}}}>
      <Camera3d transform={{Transform.fromXyz({ex}, {ey}, {ez}).lookingAt([{cx}, {cy}, {cz}])}} projection={{{{ Perspective: {{ fov: (40 * Math.PI) / 180 }} }}}} />
      <AmbientLight brightness={{250}} />
      <DirectionalLight transform={{Transform.fromXyz({lx}, {ly}, {lz}).lookingAt([{cx}, {cy}, {cz}])}} illuminance={{9000}} shadow_maps_enabled />
      <Mesh3d transform={{Transform.fromXyz({cx}, {sy}, {cz})}}><Cuboid half_size={{[{slab}, {thick}, {slab}]}} /><StandardMaterial base_color="#55565a" perceptual_roughness={{0.9}} /></Mesh3d>
      {{Object.entries(get()).map(([k, s], i) => (
        <Mesh3d key={{k}} shape={{s}}><StandardMaterial base_color={{COLOURS[i % COLOURS.length]}} perceptual_roughness={{0.6}} /></Mesh3d>
      ))}}
    </AbsoluteFill>
  );
}}
registerRoot(() => <Composition id="solid" component={{View}} width={{1280}} height={{960}} fps={{30}} durationInFrames={{1}} />);
"##,
        // relative, as the scene sits beside it: an absolute import is bundled with its raea imports left external
        file = format!("./{}", file.file_stem().and_then(|s| s.to_str()).unwrap_or("solid")),
        set = Value::Object(set.clone()),
        colours = COLOURS,
        ex = eye[0],
        ey = eye[1],
        ez = eye[2],
        cx = c[0],
        cy = c[1],
        cz = c[2],
        lx = c[0] + d * 0.6,
        ly = c[1] + d,
        lz = c[2] + d * 0.8,
        sy = lo[1] - thick,
        slab = slab,
        thick = thick,
    )
}

/// The sheets made and the sidecar said, from the file's solid info (`info`, its JSON).
pub(crate) fn view(file: &Path, set: &Map<String, Value>, info: &Value, out: &Path, json_out: bool) -> Result<String, String> {
    let b = &info["solid"]["box"];
    let v3 = |k: &str| -> Result<[f64; 3], String> {
        let a = b[k].as_array().filter(|a| a.len() == 3).ok_or("solid view: the solid is empty: nothing to look at")?;
        Ok([a[0].as_f64().unwrap_or(0.0), a[1].as_f64().unwrap_or(0.0), a[2].as_f64().unwrap_or(0.0)])
    };
    let (lo, hi) = (v3("min")?, v3("max")?);
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    // the scene sits beside the file, so its import resolves as the file's own do
    let tsx = file.with_file_name(format!(".{}.view.tsx", file.file_stem().and_then(|s| s.to_str()).unwrap_or("solid")));
    std::fs::write(&tsx, scene(file, set, lo, hi)).map_err(|e| format!("{}: {e}", tsx.display()))?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    // a scene made for this look and deleted after it: nothing for the project's record (DR3)
    let run = std::process::Command::new(exe).arg("render").arg(&tsx).arg("--review").arg(out).current_dir(tsx.parent().unwrap_or(Path::new("."))).env("RAEA_RECORD", "off").output().map_err(|e| e.to_string())?;
    let said = format!("{}{}", String::from_utf8_lossy(&run.stdout), String::from_utf8_lossy(&run.stderr));
    if !run.status.success() || !out.join("colour.png").is_file() {
        // the scene is left for a look: what failed is in it or in the file
        let why: Vec<&str> = said.lines().filter(|l| l.contains("Error") || l.contains("REFUSED") || l.contains("error")).take(3).collect();
        return Err(format!("solid view: the review render of {} failed: {}", tsx.display(), if why.is_empty() { said.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("no output").to_string() } else { why.join(" | ") }));
    }
    let _ = std::fs::remove_file(&tsx);
    let review: Value = serde_json::from_str(&std::fs::read_to_string(out.join("review.json")).unwrap_or_default()).unwrap_or(Value::Null);
    let found: Vec<Value> = info["found"].as_array().into_iter().flatten().filter(|f| f["level"] != "ok").cloned().chain(review["found"].as_array().into_iter().flatten().cloned()).collect();
    let side = json!({
        "file": file.display().to_string(), "set": set, "sheets": {"colour": out.join("colour.png"), "clay": out.join("clay.png")},
        "box": info["solid"]["box"], "disjoint": info["solid"]["pieces"].as_u64().unwrap_or(0) > 1,
        "parts": info["parts"].as_array().into_iter().flatten().enumerate().map(|(i, p)| json!({"part": p["part"], "colour": COLOURS[i % COLOURS.len()]})).collect::<Vec<_>>(),
        "tiles": review["tiles"], "errors": info["errors"], "found": found,
        "caption": format!("{} Each part in its own colour on the colour sheet (parts[].colour).", review["caption"].as_str().unwrap_or("")),
    });
    std::fs::write(out.join("view.json"), serde_json::to_string_pretty(&side).unwrap_or_default()).map_err(|e| e.to_string())?;
    if json_out {
        return Ok(serde_json::to_string_pretty(&side).unwrap_or_default());
    }
    let mut t = vec![format!("APPLIED  {} and {} (hero, front, right, top, iso, low), the sidecar in {}", out.join("colour.png").display(), out.join("clay.png").display(), out.join("view.json").display())];
    t.push(format!("parts: {}", side["parts"].as_array().into_iter().flatten().map(|p| format!("{} {}", p["part"].as_str().unwrap_or(""), p["colour"].as_str().unwrap_or(""))).collect::<Vec<_>>().join(", ")));
    for f in &found {
        t.push(format!("  {}: {}", f["level"].as_str().unwrap_or(""), f["says"].as_str().unwrap_or("")));
    }
    Ok(t.join("\n"))
}

/// Where a part is named in the file: the first line that gives it as a key or a binding (`body:`, `const body =`).
pub(crate) fn at(src: &str, part: &str) -> Option<usize> {
    let word = |l: &str, i: usize| i == 0 || !l[..i].ends_with(|c: char| c.is_alphanumeric() || c == '_');
    src.lines().position(|l| {
        l.match_indices(part).any(|(i, _)| {
            let rest = l[i + part.len()..].trim_start();
            word(l, i) && (rest.starts_with(':') || (rest.starts_with('=') && !rest.starts_with("==")) || rest.starts_with(','))
        })
    }).map(|n| n + 1)
}

pub(crate) fn out_dir(args: &[String]) -> Option<PathBuf> {
    args.iter().position(|a| a == "-o").and_then(|i| args.get(i + 1)).map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_part_is_found_where_it_is_named() {
        let src = "import x from 'y';\nconst body = a();\nreturn { body, handle: h, handlebar: 2 };\n";
        assert_eq!(super::at(src, "body"), Some(2));
        assert_eq!(super::at(src, "handle"), Some(3));
        assert_eq!(super::at(src, "spout"), None);
    }
}
