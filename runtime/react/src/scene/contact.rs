//! The scene.* native calls: a model's box and seat facing (raea-sys-contact) and a sky's daylight
//! (raea-sys-light-meter), read from the composition's folder.

use serde_json::Value;

/// A src a native call names, as a file in the composition's folder.
fn media(call: &str, v: &Value, what: &str) -> Result<std::path::PathBuf, String> {
    let src = v.get("src").and_then(Value::as_str).ok_or_else(|| format!("{call}: src, {what}"))?;
    let dir = crate::native::composition_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    Ok(super::media_path(&dir, src))
}

const GLB: &str = "a .glb (staticFile(\"x.glb\"))";

/// `scene.sky`: a sky panorama read for daylight (raea_sys_light_meter::sky_of).
fn sky_call(v: &Value) -> Result<Value, String> {
    let path = media("scene.sky", v, "an .hdr panorama (staticFile(\"x.hdr\"))")?;
    raea_sys_light_meter::sky_of(&path.to_string_lossy(), v.get("rotation").and_then(Value::as_f64).unwrap_or(0.0) as f32)
}

pub(crate) fn register() {
    raea_native::register(
        "scene.sky",
        "{ src: string; rotation?: number }",
        "{ sun: { dir: [number, number, number]; elevation: number; horizontal: number; cct: number | null } | null; sky: { horizontal: number; luminance: number; cct: number | null } }",
        "a sky panorama read for daylight: its sun's direction once turned by rotation (as <Environment rotation>), and the light on a level surface from the sun's disc and from the rest of the sky (panorama units), the sky's mean luminance, each one's colour temperature",
        sky_call,
    );
    // a .glb's own geometry, found in the composition's folder
    let models: [(&str, &str, &str, &str, fn(&std::path::Path, &Value) -> Result<Value, String>); 2] = [
        ("scene.facing",
         "{ src: string }",
         "{ faces: [number, number] | null; seat?: number; lean?: number }",
         "which way a seat faces as it comes, on the plan (x, z), read from its triangles: from its back's centre across its seat's; null for a seat with nothing over it (a backless stool); seat is its height",
         |p, _| raea_sys_contact::facing(p)),
        ("scene.bounds",
         "{ src: string; rotation?: [number, number, number]; scale?: number | [number, number, number] }",
         "{ min: [number, number, number]; max: [number, number, number] }",
         "a glTF model's box turned (Euler XYZ radians) and scaled about its origin, from all its nodes: set on a surface at h, its origin goes at h - min y",
         raea_sys_contact::bounds),
    ];
    for (name, takes, gives, doc, f) in models {
        raea_native::register(name, takes, gives, doc, move |v| f(&media(name, v, GLB)?, v));
    }
}
