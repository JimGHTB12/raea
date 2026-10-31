//! Physics for compositions: raea-sys-bake's bakes as native calls and `raea physics`, in the composition's folder.

use std::path::PathBuf;

use raea_sys_bake::{Place, bake, goo, part, simulate};
use serde_json::{Value, json};

type R<T> = Result<T, String>;

/// The composition's folder: its asset store, and its media.
fn at<T>(f: impl FnOnce(&Place) -> T) -> T {
    let dir = crate::native::composition_dir().unwrap_or_else(|| PathBuf::from("."));
    let media = |src: &str| super::media_path(&dir, src);
    f(&Place { store: raea_io_assets::Store::beside(&dir.join("edit.tsx")).dir, media: &media, goo_setup: raea_io_assets::setup_script("goo") })
}

pub(crate) fn register() {
    let calls: [(&str, &str, &str, &str, fn(&Place, &Value) -> R<Value>); 3] = [
        ("physics.part",
         "{ src: string; nodes: string[]; material?: { missing?: boolean; color?: string; emissive?: string; opacity?: number; roughness?: number; unlit?: boolean } }",
         "{ file: string; centre: [number, number, number]; lo: [number, number, number]; hi: [number, number, number]; tris: number }",
         "glTF nodes cut out as their own .glb (where they stood), optionally with one material over them (kept in the asset store)",
         part),
        ("physics.bake",
         "{ fps: number; frames: number; gravity?: [number, number, number]; substeps?: number; colliders?: { min: [number, number, number]; max: [number, number, number]; friction?: number; restitution?: number }[]; bodies: Record<string, unknown>[] }",
         "{ fps: number; frames: number; file: string; bodies: Record<string, { p: number[]; q: number[]; centre: [number, number, number] }> }",
         "rigid bodies stepped by Rapier, deterministic, made once per request and kept: each body's rigid motion from rest per frame",
         bake),
        ("physics.goo",
         "{ spec: Record<string, unknown> }",
         "{ file: string; stats: Record<string, unknown> }",
         "goo (MLS-MPM, tools/goo) baked to a mesh sequence (.rseq), made once per spec and kept",
         goo),
    ];
    for (name, takes, gives, doc, f) in calls {
        raea_native::register(name, takes, gives, doc, move |v| at(|p| f(p, v)));
    }
}

pub const USAGE: &str = "raea physics bake REQUEST.json [-o OUT.json] | physics goo SPEC.json | physics part SRC.glb NODE...";

/// `raea physics bake REQUEST.json [-o OUT.json] [--dir EDIT_DIR]`, `raea physics goo SPEC.json`
pub fn cli(args: &[String]) -> R<String> {
    let usage = USAGE;
    let verb = args.get(1).map(String::as_str).ok_or(usage)?;
    let fl = |n: &str| args.iter().position(|a| a == n).and_then(|i| args.get(i + 1)).cloned();
    let read = |p: &str| -> R<Value> { serde_json::from_slice(&std::fs::read(p).map_err(|e| format!("{p}: {e}"))?).map_err(|e| format!("{p}: {e}")) };
    let v = match verb {
        "bake" => at(|p| bake(p, &read(args.get(2).ok_or(usage)?)?))?,
        "simulate" => at(|p| simulate(p, &read(args.get(2).ok_or(usage)?)?))?,
        "goo" => at(|p| goo(p, &json!({"spec": read(args.get(2).ok_or(usage)?)?})))?,
        "part" => at(|p| part(p, &json!({"src": args.get(2).ok_or(usage)?, "nodes": args[3..].iter().filter(|a| !a.starts_with('-')).collect::<Vec<_>>()})))?,
        _ => return Err(usage.into()),
    };
    match fl("-o") {
        Some(o) => {
            std::fs::write(&o, v.to_string()).map_err(|e| format!("{o}: {e}"))?;
            Ok(o)
        }
        None => Ok(v.get("file").and_then(Value::as_str).map(String::from).unwrap_or_else(|| v.to_string())),
    }
}
