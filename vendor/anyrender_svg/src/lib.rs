// Copyright 2023 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Render an SVG into any impl of [`anyrender::PaintScene`].
//!
//! This currently lacks support for some important SVG features. Known missing features include: masking, filter effects, group backgrounds
//! path shape-rendering, and patterns.

// LINEBENDER LINT SET - lib.rs - v1
// See https://linebender.org/wiki/canonical-lints/
// These lints aren't included in Cargo.toml because they
// shouldn't apply to examples and tests
#![warn(unused_crate_dependencies)]
#![warn(clippy::print_stdout, clippy::print_stderr)]
// END LINEBENDER LINT SET
#![cfg_attr(docsrs, feature(doc_cfg))]
// The following lints are part of the Linebender standard set,
// but resolving them has been deferred for now.
// Feel free to send a PR that solves one or more of these.
#![allow(missing_docs, clippy::shadow_unrelated, clippy::missing_errors_doc)]
#![cfg_attr(test, allow(unused_crate_dependencies))] // Some dev dependencies are only used in tests

mod error;
mod filter;
mod render;
mod util;

pub use error::Error;
pub use usvg;

use anyrender::PaintScene;
use kurbo::Affine;

/// Append an SVG to an [`anyrender::PaintScene`].
///
/// This will draw a red box over (some) unsupported elements.
pub fn render_svg_str<S: PaintScene>(
    scene: &mut S,
    svg: &str,
    transform: Affine,
) -> Result<(), Error> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opt)?;
    render_svg_tree(scene, &tree, transform);
    Ok(())
}

/// Append an SVG to an [`anyrender::PaintScene`] (with custom error handling).
///
/// See the [module level documentation](crate#unsupported-features) for a list of some unsupported svg features
pub fn render_svg_str_with<S: PaintScene, F: FnMut(&mut S, &usvg::Node)>(
    scene: &mut S,
    svg: &str,
    transform: Affine,
    error_handler: &mut F,
) -> Result<(), Error> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg, &opt)?;
    render_svg_tree_with(scene, &tree, transform, error_handler);
    Ok(())
}

/// Append a [`usvg::Tree`] to an [`anyrender::PaintScene`].
///
/// This will draw a red box over (some) unsupported elements.
pub fn render_svg_tree<S: PaintScene>(scene: &mut S, svg: &usvg::Tree, transform: Affine) {
    render_svg_tree_with(scene, svg, transform, &mut util::default_error_handler);
}

/// raea: the filter graph usvg made of `tree`'s first filtered group, and its region in the group's space: a CSS filter
/// list with url()s (blitz-dom's css_filter_tree; `elements` its <filter>s' ids), for raea-page's GPU passes as an
/// SVG's filters go.
pub fn filter_graph(tree: &usvg::Tree, elements: &[String]) -> Option<(anyrender::filters::Filter, kurbo::Rect)> {
    render::tree_filter(tree, elements)
}

/// raea: the clip of `tree`'s first element (blitz-dom's css_clip_tree: a `<clipPath>` an HTML element's clip-path
/// url() names), every shape of it in the element's CSS px with its clip-rule; an empty path when the clip path
/// shows nothing (usvg drops an element whose clip path has no shapes).
pub fn clip_path_shape(tree: &usvg::Tree) -> (kurbo::BezPath, peniko::Fill) {
    render::tree_clip(tree)
}

/// raea: a tree drawn under `transform`, which is `host * viewport`: `viewport` the viewBox's mapping (in CSS px),
/// `host` the page's transform (its place, transforms and device scale). A stroke with
/// vector-effect="non-scaling-stroke" is drawn under `host` alone, as Chrome draws it; the rest under `transform`.
pub fn render_svg_tree_in<S: PaintScene>(scene: &mut S, svg: &usvg::Tree, transform: Affine, host: Affine, viewport: Affine) {
    let was = render::VIEWPORT.with(|v| v.replace(Some((host, viewport))));
    render_svg_tree(scene, svg, transform);
    render::VIEWPORT.with(|v| v.set(was));
}

/// Append a [`usvg::Tree`] to an [`anyrender::PaintScene`] (with custom error handling).
///
/// See the [module level documentation](crate#unsupported-features) for a list of some unsupported svg features
pub fn render_svg_tree_with<S: PaintScene, F: FnMut(&mut S, &usvg::Node)>(
    scene: &mut S,
    svg: &usvg::Tree,
    transform: Affine,
    error_handler: &mut F,
) {
    let was = render::FONTS.with(|f| f.replace(Some(svg.fontdb().clone())));
    render::render_group(
        scene,
        svg.root(),
        Affine::IDENTITY,
        transform,
        error_handler,
    );
    render::FONTS.with(|f| f.replace(was));
}

/// raea: SVG text's glyphs grown as the page's other text is (blitz-paint's CoreText look): given a text's size in
/// device px, whether its colour is light and whether it's synthetic bold, how much wider than its outline it's
/// drawn, in device px.
pub type TextLook = fn(f64, bool, bool) -> f64;
pub(crate) static TEXT_LOOK: std::sync::OnceLock<TextLook> = std::sync::OnceLock::new();

/// raea: sets `TextLook` (once; later calls are ignored). Unset, SVG text is drawn as its outlines.
pub fn set_text_look(look: TextLook) {
    let _ = TEXT_LOOK.set(look);
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyrender::recording::{RenderCommand, Scene};
    use peniko::Fill;

    fn clip_rule(clip_attrs: &str, path_attrs: &str) -> Fill {
        let svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
                <defs><clipPath id="clip" {clip_attrs}>
                    <path {path_attrs} d="M0 0H100V100H0Z M25 25H75V75H25Z"/>
                </clipPath></defs>
                <rect width="100" height="100" clip-path="url(#clip)"/>
            </svg>"##
        );
        let mut scene = Scene::new();
        render_svg_str(&mut scene, &svg, Affine::IDENTITY).unwrap();
        scene
            .commands
            .iter()
            .find_map(|command| match command {
                RenderCommand::PushLayer(layer) => Some(layer.fill),
                _ => None,
            })
            .expect("SVG should produce a clipping layer")
    }

    #[test]
    fn clip_rule_attribute_is_preserved() {
        assert_eq!(clip_rule("", ""), Fill::NonZero);
        assert_eq!(clip_rule("", r#"clip-rule="nonzero""#), Fill::NonZero);
        assert_eq!(clip_rule("", r#"clip-rule="evenodd""#), Fill::EvenOdd);
    }

    #[test]
    fn clip_rule_style_overrides_attribute_and_inherits() {
        assert_eq!(
            clip_rule("", r#"clip-rule="nonzero" style="clip-rule: evenodd""#),
            Fill::EvenOdd
        );
        assert_eq!(
            clip_rule(r#"style="clip-rule: evenodd""#, ""),
            Fill::EvenOdd
        );
        assert_eq!(
            clip_rule(r#"clip-rule="evenodd""#, r#"style="clip-rule: nonzero""#),
            Fill::NonZero
        );
    }

    #[test]
    fn fill_rule_does_not_control_clipping() {
        assert_eq!(clip_rule("", r#"fill-rule="evenodd""#), Fill::NonZero);
        assert_eq!(
            clip_rule("", r#"fill-rule="nonzero" clip-rule="evenodd""#),
            Fill::EvenOdd
        );
    }
}
