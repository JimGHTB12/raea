//! Bevy's depth of field, patched where it loads so its blur keeps the frame's alpha.
//!
//! Bevy 0.20's dof.wesl blurs the colour and writes alpha 1 everywhere (the bokeh's box blurs sum `.rgb` and return
//! `vec4(sum / n, 1.0)`, and the Gaussian library does the same). The paint look (look.rs) clears the scene's camera
//! see-through and lays its sky wherever the cells' alpha says nothing was drawn, so with a <DepthOfField> every such
//! pixel came out drawn, in the clear colour: a black sky (10-22, anything/intro/opening-build.tsx). Here the box blurs
//! average all four channels and the Gaussian passes blur alpha by the same kernel as the library blurs colour, so a
//! pixel's alpha is the blurred coverage: 0 where only sky reaches it, 1 inside a thing, between at a blurred edge.
//! A camera that clears opaque (any other) has alpha 1 everywhere and draws exactly as before.

use bevy::app::App;

/// The FNV-1a hash of Bevy 0.20.0's bevy_post_process/dof/dof.wesl, which the patch below was written against.
const UPSTREAM_FNV: u64 = 0xd551_0770_127f_2472;

/// The Gaussian passes' alpha: bevy_post_process's gaussian_blur.wesl (0.20.0) on `.a` alone. Its kernel is copied
/// here, not patched, because the library is loaded as Bevy builds its plugin, before this could swap it.
const GAUSSIAN_ALPHA: &str = "
/// raea: gaussian_blur (bevy_post_process/gaussian_blur.wesl) on alpha alone, so the Gaussian passes keep coverage
fn raea_gaussian_alpha(frag_coord: vec4<f32>, coc: f32, frag_offset: vec2<f32>) -> f32 {
    let sigma = coc * 0.25;
    let support = i32(ceil(sigma * 1.5));
    let uv = frag_coord.xy / vec2<f32>(textureDimensions(color_texture_a));
    let offset = frag_offset / vec2<f32>(textureDimensions(color_texture_a));
    let exp_factor = -1.0 / (2.0 * sigma * sigma);
    var sum = textureSampleLevel(color_texture_a, color_texture_sampler, uv, 0.0).a;
    var weight_sum = 1.0;
    for (var i = 1; i <= support; i += 2) {
        let w0 = exp(exp_factor * f32(i) * f32(i));
        let w1 = exp(exp_factor * f32(i + 1) * f32(i + 1));
        let uv_offset = offset * (f32(i) + w1 / (w0 + w1));
        let weight = w0 + w1;
        sum += (
            textureSampleLevel(color_texture_a, color_texture_sampler, uv + uv_offset, 0.0).a +
            textureSampleLevel(color_texture_a, color_texture_sampler, uv - uv_offset, 0.0).a
        ) * weight;
        weight_sum += weight * 2.0;
    }
    return sum / weight_sum;
}

/// Calculates the horizontal component of the separable Gaussian blur.";

/// Bevy's dof.wesl with alpha kept (above). Bevy's pipeline loads the shader when a camera first has depth of field,
/// after the plugins are built, so the copy registered here is the one it gets.
pub fn install(app: &mut App) {
    let swapped = super::bevy_api::swap_embedded_shader(app, "bevy_post_process/dof/dof.wesl", UPSTREAM_FNV, |bevy| {
        // the bokeh's two box blurs (a and b): all four channels summed and averaged
        let s = replace_n(bevy, "    var sum = vec3(0.0);", "    var sum = vec4(0.0);", 2);
        let s = replace_n(&s, "uv + offset * f32(i), 0.0).rgb;", "uv + offset * f32(i), 0.0);", 2);
        let s = replace_n(&s, "return vec4(sum / vec3(1.0 + f32(support)), 1.0);", "return sum / (1.0 + f32(support));", 2);
        // the Gaussian passes: the library's colour, alpha blurred by the same kernel
        let s = replace_n(&s, "/// Calculates the horizontal component of the separable Gaussian blur.", GAUSSIAN_ALPHA, 1);
        let s = replace_n(
            &s,
            "return gaussian_blur(color_texture_a, color_texture_sampler, in.position, coc, vec2(1.0, 0.0));",
            "return vec4(gaussian_blur(color_texture_a, color_texture_sampler, in.position, coc, vec2(1.0, 0.0)).rgb, raea_gaussian_alpha(in.position, coc, vec2(1.0, 0.0)));",
            1,
        );
        replace_n(
            &s,
            "return gaussian_blur(color_texture_a, color_texture_sampler, in.position, coc, vec2(0.0, 1.0));",
            "return vec4(gaussian_blur(color_texture_a, color_texture_sampler, in.position, coc, vec2(0.0, 1.0)).rgb, raea_gaussian_alpha(in.position, coc, vec2(0.0, 1.0)));",
            1,
        )
    });
    assert!(swapped, "raea: Bevy no longer embeds bevy_post_process/dof/dof.wesl (docs/bevy-upgrade.md)");
}

/// `s` with `from` replaced, which must appear exactly `n` times.
fn replace_n(s: &str, from: &str, to: &str, n: usize) -> String {
    assert_eq!(s.matches(from).count(), n, "raea: expected `{from}` {n} times in Bevy's dof.wesl");
    s.replace(from, to)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The patch applies to the dof.wesl this Bevy ships (the hash and every anchor), and leaves no `.rgb`-only box
    /// blur behind.
    #[test]
    fn dof_patch_applies() {
        let mut app = App::new();
        app.add_plugins((bevy::asset::AssetPlugin::default(), bevy::post_process::dof::DepthOfFieldPlugin));
        install(&mut app);
        let reg = app.world().resource::<bevy::asset::io::embedded::EmbeddedAssetRegistry>();
        let ours = reg.remove_asset(std::path::Path::new("bevy_post_process/dof/dof.wesl")).expect("registered");
        let s = std::str::from_utf8(ours.value()).unwrap();
        assert!(!s.contains("vec4(sum / vec3(1.0 + f32(support)), 1.0)"));
        assert_eq!(s.matches("raea_gaussian_alpha(in.position").count(), 2);
    }
}
