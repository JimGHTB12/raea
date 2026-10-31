//! Bevy's effect stack (chromatic aberration, lens distortion, vignette), patched where it loads so it keeps the
//! frame's alpha.
//!
//! Bevy 0.20's post_process.wesl writes `vec4(vignette(uv, color), 1.0)`: alpha 1 on every pixel. The paint look
//! (look.rs) clears the scene's camera see-through and lays its sky wherever the cells' alpha says nothing was drawn,
//! so once a <Grade aberration> passed Bevy's 1e-4 cut-off (below it the pass doesn't run) every such pixel came out
//! drawn, in the clear colour: a black sky (10-22, anything/intro/opening-build.tsx's glitch, FilmLook). Here alpha
//! takes the aberration's own path (the lens-distorted uv, then the same samples toward the middle, averaged), so a
//! pixel's alpha is the coverage the fringes carry: 0 where only sky reaches it, 1 inside a thing. A camera that
//! clears opaque (any other) has alpha 1 everywhere and draws exactly as before. dof.rs does the same for depth of
//! field.

use bevy::app::App;
use raea_sys_frame::replace_once;

/// The FNV-1a hash of Bevy 0.20.0's bevy_post_process/effect_stack/post_process.wesl, which the patch below was
/// written against.
const UPSTREAM_FNV: u64 = 0x9fbb_17b4_41d1_9a99;

/// The aberration's alpha: chromatic_aberration.wesl's sampling (0.20.0) on `.a`, unweighted (its LUT tints colour).
const ALPHA: &str = "import package::effect_stack::chromatic_aberration::{source_texture, common_sampler, chromatic_aberration_settings};

/// raea: the coverage under chromatic_aberration's samples (chromatic_aberration.wesl's path, averaged), so the
/// paint look's sky still shows where nothing was drawn (raea-render engine/effect_stack.rs)
fn raea_aberration_alpha(start_pos: vec2<f32>) -> f32 {
    let end_pos = mix(start_pos, vec2(0.5), chromatic_aberration_settings.intensity);
    let texel_length = length((end_pos - start_pos) * vec2<f32>(textureDimensions(source_texture)));
    let sample_count = min(u32(ceil(texel_length)), chromatic_aberration_settings.max_samples);
    if (sample_count <= 1u) {
        return textureSampleLevel(source_texture, common_sampler, start_pos, 0.0).a;
    }
    var sum = 0.0;
    for (var i = 0u; i < sample_count; i += 1u) {
        let t = (f32(i) + 0.5) / f32(sample_count);
        sum += textureSampleLevel(source_texture, common_sampler, mix(start_pos, end_pos, t), 0.0).a;
    }
    return sum / f32(sample_count);
}

@fragment";

/// Bevy's post_process.wesl with alpha kept (above). Bevy loads it when the effect stack's pipeline is first made,
/// after the plugins are built, so the copy registered here is the one it gets.
pub fn install(app: &mut App) {
    let swapped = super::bevy_api::swap_embedded_shader(app, "bevy_post_process/effect_stack/post_process.wesl", UPSTREAM_FNV, |bevy| {
        let s = replace_once(bevy, "@fragment", ALPHA);
        replace_once(&s, "return vec4(vignette(in.uv, color), 1.0);", "return vec4(vignette(in.uv, color), raea_aberration_alpha(distorted_uv));")
    });
    assert!(swapped, "raea: Bevy no longer embeds bevy_post_process/effect_stack/post_process.wesl (docs/bevy-upgrade.md)");
}


#[cfg(test)]
mod tests {
    use super::*;

    /// The patch applies to the post_process.wesl this Bevy ships (the hash and both anchors), and no opaque write is
    /// left behind.
    #[test]
    fn effect_stack_patch_applies() {
        use bevy::asset::AssetApp;
        let mut app = App::new();
        app.add_plugins((bevy::app::TaskPoolPlugin::default(), bevy::asset::AssetPlugin::default())).init_asset::<bevy::image::Image>().init_asset::<bevy::shader::Shader>();
        app.add_plugins(bevy::post_process::effect_stack::EffectStackPlugin);
        install(&mut app);
        let reg = app.world().resource::<bevy::asset::io::embedded::EmbeddedAssetRegistry>();
        let ours = reg.remove_asset(std::path::Path::new("bevy_post_process/effect_stack/post_process.wesl")).expect("registered");
        let s = std::str::from_utf8(ours.value()).unwrap();
        assert!(!s.contains("color), 1.0)"));
        assert!(s.contains("raea_aberration_alpha(distorted_uv)"));
    }
}
