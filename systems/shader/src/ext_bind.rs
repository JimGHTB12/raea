//! An extension file's own bindings (docs/control.feature FC2b): a MaterialExtension's fragment file is compiled as
//! written but for the numbers of its material-group bindings, which go to ShaderExt's fixed slots in the order the
//! source declares them: its uniform (a struct of its fields) at 100, its texture_2ds at 101, 103, 105 and 107, and
//! their one sampler at 102 (the standard material and the view already bind 13 of the fragment stage's 16). A
//! declaration under @if(BINDLESS) is left as written: ShaderExt isn't bindless, so it compiles out (as Bevy's
//! extended_material_bindless binds its @else set without bindless).

use bevy::prelude::*;

use crate::ui_bind::{rebind_with, Rebound};

/// ShaderExt's slot for a rebind_in slot (a uniform's 0 to 7, a texture's 8, 10, 12, 14, a sampler's 9, 11, ...).
fn place(slot: usize) -> Result<usize, String> {
    match slot {
        0 => Ok(100),
        9 => Ok(102),
        8 | 10 | 12 | 14 => Ok(101 + slot - 8),
        1..=7 => Err("an extension binds one uniform, a struct of its fields".into()),
        _ => Err("an extension's textures share one sampler, its first texture's".into()),
    }
}

/// An extension file's source with its bindings in ShaderExt's slots, and its textures in their slots' order (`given`
/// by name; one it declares and isn't given binds Bevy's fallback image). A texture it doesn't declare is refused.
pub fn fit(src: &str, given: &[(String, Handle<Image>)]) -> Result<(String, Vec<Option<Handle<Image>>>), String> {
    let Rebound { src, textures, .. } = rebind_with(src, crate::own::GROUPS, "an extension", false, &place)?;
    if let Some((k, _)) = given.iter().find(|(k, _)| !textures.contains(k)) {
        return Err(format!("texture {k}: the extension's fragment doesn't declare it (it declares {textures:?})"));
    }
    let tex = textures.iter().map(|n| given.iter().find(|(k, _)| k == n).map(|(_, h)| h.clone())).collect();
    Ok((src, tex))
}

#[cfg(test)]
mod tests {
    #[test]
    fn an_extensions_own_bindings_go_to_its_slots_and_its_bindless_set_is_left() {
        let src = "@if(BINDLESS) {\n@group(constants::MATERIAL_BIND_GROUP) @binding(100) var<storage> a: array<M>;\n}\n@else {\n\
                   @group(constants::MATERIAL_BIND_GROUP) @binding(50) var<uniform> m: M;\n\
                   @group(constants::MATERIAL_BIND_GROUP) @binding(51) var t: texture_2d<f32>;\n\
                   @group(constants::MATERIAL_BIND_GROUP) @binding(52) var s: sampler;\n}\n";
        let (out, tex) = super::fit(src, &[]).unwrap();
        assert!(out.contains("@binding(100) var<storage> a") && out.contains("@binding(100) var<uniform> m"), "{out}");
        assert!(out.contains("@binding(101) var t") && out.contains("@binding(102) var s"), "{out}");
        assert_eq!(tex, vec![None]);
        let two = "@group(3) @binding(0) var<uniform> a: f32;\n@group(3) @binding(1) var<uniform> b: f32;\n";
        assert!(super::fit(two, &[]).unwrap_err().contains("one uniform"));
        assert!(super::fit(src, &[("u".into(), Default::default())]).unwrap_err().contains("texture u"));
    }
}
