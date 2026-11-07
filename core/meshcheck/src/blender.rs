//! The Blender step a finding calls for (docs/blender.feature BL10-BL16): each finding a script fixes carries it as
//! text ("fix: raea blender run blender/X.py ...") and in its JSON (`fix.blender`, `fix.run`). The scripts are the
//! library in sdk/guides/craft/bevy/blender/, which `raea blender run blender/X.py` copies into the project the first
//! time. Blender is the workshop between an asset's source and media/: these are the only reasons to go there.

use serde_json::{Value, json};

/// A finding's fix through a library script: `args` as the command line takes them.
pub fn fix(script: &str, args: &str) -> Value {
    json!({"blender": script, "run": format!("raea blender run blender/{script} {args}")})
}

/// The finding's words with its fix's command after them.
pub fn says(words: &str, fix: &Value) -> String {
    format!("{words}; fix: {}", fix["run"].as_str().unwrap_or(""))
}

/// The most triangles a model seen from `seen` metres should carry: 150 000 at 1 m or closer (a hero in the hand),
/// falling with the square of the distance (the screen area it covers), never under 2 000.
pub fn budget(seen: f64) -> usize {
    ((150_000.0 / seen.max(1.0).powi(2)) as usize).max(2_000)
}

/// `dir/name.glb` with `-tag` before its extension, for a fix's output beside its input.
pub fn beside(file: &str, tag: &str) -> String {
    let stem = file.rsplit_once('.').map(|(s, _)| s).unwrap_or(file);
    format!("{stem}-{tag}.glb")
}

/// The finding for a file raea can't load (FBX, OBJ, .blend, STL, PLY), or None for a glTF.
pub fn unloadable(file: &str) -> Option<Value> {
    let ext = file.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default();
    if matches!(ext.as_str(), "glb" | "gltf") {
        return None;
    }
    let stem = file.rsplit_once('.').map(|(s, _)| s).unwrap_or(file);
    let f = fix("convert.py", &format!("{file} {stem}.glb"));
    Some(json!({"check": "format", "level": "error", "says": says(&format!("raea loads glTF (.glb); this is .{ext}"), &f), "fix": f}))
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_budget_falls_with_distance_and_bottoms_out() {
        assert_eq!(super::budget(0.5), 150_000);
        assert_eq!(super::budget(2.0), 37_500);
        assert_eq!(super::budget(40.0), 2_000);
    }

    #[test]
    fn a_format_raea_cant_load_names_convert() {
        let f = super::unloadable("media/src/chair.fbx").unwrap();
        assert_eq!(f["fix"]["blender"], "convert.py");
        assert_eq!(f["fix"]["run"], "raea blender run blender/convert.py media/src/chair.fbx media/src/chair.glb");
        assert!(super::unloadable("media/chair.glb").is_none());
    }
}
