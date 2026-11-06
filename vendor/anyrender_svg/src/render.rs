// Copyright 2024 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::util;
use anyrender::PaintScene;
use anyrender::filters::{Filter, FilterEffect};
use kurbo::{Affine, BezPath, Shape as _};
use peniko::{BlendMode, Fill};
use std::sync::Arc;
use usvg::{Node, Path};

pub(crate) fn render_group<S: PaintScene, F: FnMut(&mut S, &usvg::Node)>(
    scene: &mut S,
    group: &usvg::Group,
    transform: Affine,
    global_transform: Affine,
    error_handler: &mut F,
) {
    render_children(scene, group, transform, global_transform, false, error_handler);
}

/// raea: a group's children; `rel`: placed by their own transforms under `transform` (a mask's or a pattern's content,
/// drawn where it's used), not their absolute ones
fn render_children<S: PaintScene, F: FnMut(&mut S, &usvg::Node)>(
    scene: &mut S,
    group: &usvg::Group,
    transform: Affine,
    global_transform: Affine,
    rel: bool,
    error_handler: &mut F,
) {
    for node in group.children() {
        let own = match node {
            usvg::Node::Group(g) => g.transform(),
            _ => usvg::Transform::default(),
        };
        let transform = if rel { transform * util::to_affine(&own) } else { transform * util::to_affine(&node.abs_transform()) };
        // (a group's children under `rel` go on from its own place; otherwise from their absolute ones)
        let inner = if rel { transform } else { Affine::IDENTITY };
        match node {
            usvg::Node::Group(g) => {
                // raea: a mask (its content drawn into a layer composited DestIn over the group's, its luminance as
                // alpha) and filter effects (a layer raea-page's GPU passes filter); each in a layer of its own
                let space = global_transform * transform;
                let masked = g.mask().is_some();
                if masked {
                    scene.push_layer(Fill::NonZero, BlendMode::default(), 1.0, Affine::IDENTITY, &space.transform_rect_bbox(group_bounds(g)).inflate(2.0, 2.0), None, None);
                }
                let filtered = filter_of(g).map(|(f, region)| {
                    scene.push_layer(Fill::NonZero, BlendMode::default(), 1.0, space, &region, Some(Arc::new(f)), None);
                });
                let alpha = g.opacity().get();
                let is_fully_opaque = alpha >= 1.0;
                let mix = util::to_mix(g.blend_mode());

                // raea: a clip path's every shape (paths, text's glyphs, under their own transforms and the clip
                // path's), not only its first path
                let clip_path = g.clip_path().map(|c| {
                    let mut out = BezPath::new();
                    let mut rule = None;
                    clip_shapes(c.root(), util::to_affine(&c.transform()), &mut out, &mut rule);
                    (out, rule)
                });

                let did_push_layer = match clip_path {
                    // If there is a clip path, then push a layer that clips using it
                    Some((local_path, rule)) => {
                        scene.push_layer(
                            match rule {
                                Some(usvg::FillRule::EvenOdd) => Fill::EvenOdd,
                                _ => Fill::NonZero,
                            },
                            BlendMode {
                                mix,
                                compose: peniko::Compose::SrcOver,
                            },
                            alpha,
                            global_transform * transform,
                            &local_path,
                            None,
                            None,
                        );

                        true
                    }
                    // Else if there is blending to be done then push a layer with a rectangular clip
                    #[allow(deprecated)]
                    _ if mix != peniko::Mix::Normal || !is_fully_opaque => {
                        // Use the layer bounding box as the clip path. It is in the group's
                        // content coordinate space, so map it to canvas space and outset it
                        // by 2px on each side (like resvg) so anti-aliased pixels at the
                        // layer edge aren't clipped.
                        let bounding_box = g.layer_bounding_box();
                        let rect = kurbo::Rect::from_origin_size(
                            (bounding_box.x() as f64, bounding_box.y() as f64),
                            (bounding_box.width() as f64, bounding_box.height() as f64),
                        );
                        let rect = (global_transform * transform)
                            .transform_rect_bbox(rect)
                            .inflate(2.0, 2.0);
                        scene.push_layer(
                            Fill::NonZero,
                            BlendMode {
                                mix,
                                compose: peniko::Compose::SrcOver,
                            },
                            alpha,
                            Affine::IDENTITY,
                            &rect,
                            None,
                            None,
                        );

                        true
                    }
                    // Else if there is no clip or blending then don't push a layer
                    _ => false,
                };

                render_children(scene, g, inner, global_transform, rel, error_handler);

                if did_push_layer {
                    scene.pop_layer();
                }
                if filtered.is_some() {
                    scene.pop_layer();
                }
                if let Some(mask) = g.mask() {
                    draw_mask(scene, mask, space, error_handler);
                    scene.pop_layer();
                }
            }
            usvg::Node::Path(path) => {
                if !path.is_visible() {
                    continue;
                }
                let local_path = util::to_bez_path(path);

                // raea: vector-effect="non-scaling-stroke" (blitz-dom marks such a path by its id): stroked in its
                // SVG's viewport space, the path carried there by its own transform, so the width isn't scaled by it
                let in_svg = transform;
                let non_scaling = path.id().starts_with(NON_SCALING_STROKE_ID);
                let transform = global_transform * transform;
                let stroke_it = |scene: &mut S, error_handler: &mut F| {
                    if non_scaling {
                        stroke_in(scene, error_handler, path, global_transform, in_svg, &local_path, node);
                    } else {
                        stroke(scene, error_handler, path, transform, &local_path, node);
                    }
                };
                match path.paint_order() {
                    usvg::PaintOrder::FillAndStroke => {
                        fill(scene, error_handler, path, transform, &local_path, node);
                        stroke_it(scene, error_handler);
                    }
                    usvg::PaintOrder::StrokeAndFill => {
                        stroke_it(scene, error_handler);
                        fill(scene, error_handler, path, transform, &local_path, node);
                    }
                }
            }
            usvg::Node::Image(img) => {
                if !img.is_visible() {
                    continue;
                }
                match img.kind() {
                    usvg::ImageKind::JPEG(_)
                    | usvg::ImageKind::PNG(_)
                    | usvg::ImageKind::GIF(_)
                    | usvg::ImageKind::WEBP(_) => {
                        #[cfg(feature = "image")]
                        {
                            // raea: decoded once per image, not on every paint (a feed card's posters kept a core busy)
                            let Some(image) = util::cached_raster_image(img.kind(), img.rendering_mode()) else {
                                error_handler(scene, node);
                                continue;
                            };
                            let image_ts = global_transform * util::to_affine(&img.abs_transform());
                            scene.draw_image(image.as_ref(), image_ts);
                        }

                        #[cfg(not(feature = "image"))]
                        {
                            error_handler(scene, node);
                            continue;
                        }
                    }
                    usvg::ImageKind::SVG(svg) => {
                        render_group(
                            scene,
                            svg.root(),
                            transform,
                            global_transform,
                            error_handler,
                        );
                    }
                }
            }
            usvg::Node::Text(text) => {
                // raea: text thickened as Chrome draws it (text_width): each glyph path filled, then stroked that wide
                if let Some(w) = text_width(text, global_transform * transform)
                    && draw_widened(scene, text.flattened(), global_transform * inner, rel, w, error_handler)
                {
                    continue;
                }
                // The children of the flattened text group already carry the
                // text node's absolute transform, so recurse with the identity
                // transform (like the group case) to avoid applying it twice.
                render_children(
                    scene,
                    text.flattened(),
                    inner,
                    global_transform,
                    rel,
                    error_handler,
                );
            }
        }
    }
}

fn fill<S: PaintScene, F: FnMut(&mut S, &usvg::Node)>(
    scene: &mut S,
    error_handler: &mut F,
    path: &Path,
    transform: Affine,
    local_path: &BezPath,
    node: &Node,
) {
    if let Some(fill) = &path.fill() {
        if let usvg::Paint::Pattern(pattern) = fill.paint() {
            let rule = match fill.rule() {
                usvg::FillRule::NonZero => Fill::NonZero,
                usvg::FillRule::EvenOdd => Fill::EvenOdd,
            };
            draw_pattern(scene, pattern, fill.opacity().get(), rule, transform, local_path, error_handler);
        } else if let Some((paint, brush_transform)) = util::to_brush(fill.paint(), fill.opacity()) {
            scene.fill(
                match fill.rule() {
                    usvg::FillRule::NonZero => Fill::NonZero,
                    usvg::FillRule::EvenOdd => Fill::EvenOdd,
                },
                transform,
                paint.as_ref(),
                Some(brush_transform),
                local_path,
            );
        } else {
            error_handler(scene, node);
        }
    }
}

fn stroke<S: PaintScene, F: FnMut(&mut S, &usvg::Node)>(
    scene: &mut S,
    error_handler: &mut F,
    path: &Path,
    transform: Affine,
    local_path: &BezPath,
    node: &Node,
) {
    if let Some(stroke) = &path.stroke() {
        if let usvg::Paint::Pattern(pattern) = stroke.paint() {
            // raea: a pattern stroke, as a fill of the stroke's outline
            let outline = kurbo::stroke(local_path.iter(), &util::to_stroke(stroke), &Default::default(), 0.1);
            draw_pattern(scene, pattern, stroke.opacity().get(), Fill::NonZero, transform, &outline, error_handler);
        } else if let Some((paint, brush_transform)) = util::to_brush(stroke.paint(), stroke.opacity()) {
            let conv_stroke = util::to_stroke(stroke);
            scene.stroke(
                &conv_stroke,
                transform,
                paint.as_ref(),
                Some(brush_transform),
                local_path,
            );
        } else {
            error_handler(scene, node);
        }
    }
}

thread_local! {
    /// raea: the tree being drawn's font database (render_svg_tree_with sets it): which face each text glyph came from
    pub(crate) static FONTS: std::cell::RefCell<Option<Arc<usvg::fontdb::Database>>> = const { std::cell::RefCell::new(None) };
}

/// raea: how widely a text's glyphs are stroked over their fill, in the text's own units (None: not at all), as the
/// page draws its other text (crate::TEXT_LOOK, set by blitz-paint: CoreText's smoothing grow by the size on screen and
/// whether the colour is light, and synthetic bold where a weight of 600 or more fell on a lighter face without a
/// weight axis). Taken from the text's first span, as nearly all SVG text is one; `at` is the text's whole transform.
fn text_width(text: &usvg::Text, at: Affine) -> Option<f64> {
    let look = crate::TEXT_LOOK.get()?;
    let span = text.layouted().first()?;
    let size = f64::from(span.font_size.get());
    let k = at.determinant().abs().sqrt();
    if k < 1e-6 {
        return None;
    }
    let light = match span.fill.as_ref().map(usvg::Fill::paint) {
        Some(usvg::Paint::Color(c)) => {
            let lin = |v: u8| {
                let v = f64::from(v) / 255.0;
                if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
            };
            0.2126 * lin(c.red) + 0.7152 * lin(c.green) + 0.0722 * lin(c.blue) > 0.5
        }
        _ => false,
    };
    let wanted = text.chunks().first().and_then(|c| c.spans().first()).map_or(400, |s| s.font().weight());
    let fake_bold = wanted >= 600
        && span.positioned_glyphs.first().is_some_and(|g| {
            FONTS.with(|f| {
                let db = f.borrow();
                let Some(face) = db.as_ref().and_then(|db| db.face(g.font)) else { return false };
                let axis = db.as_ref().and_then(|db| db.with_face_data(g.font, |data, i| ttf_wght_axis(data, i))).unwrap_or(false);
                face.weight.0 < 600 && !axis
            })
        });
    let w = look(size * k, light, fake_bold) / k;
    (w > 0.0).then_some(w)
}

/// raea: whether a face has a weight axis (fvar's wght), read from its sfnt table directory (`index` into a collection)
fn ttf_wght_axis(data: &[u8], index: u32) -> bool {
    let u16_at = |o: usize| data.get(o..o + 2).map(|b| u16::from_be_bytes([b[0], b[1]]));
    let u32_at = |o: usize| data.get(o..o + 4).map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]));
    let font = if data.get(0..4) == Some(b"ttcf") { u32_at(12 + 4 * index as usize).map(|o| o as usize) } else { Some(0) };
    let Some(font) = font else { return false };
    let Some(tables) = u16_at(font + 4) else { return false };
    let fvar = (0..usize::from(tables)).map(|i| font + 12 + 16 * i).find(|&r| data.get(r..r + 4) == Some(b"fvar"));
    let Some(fvar) = fvar.and_then(|r| u32_at(r + 8)).map(|o| o as usize) else { return false };
    let (Some(axes), Some(count), Some(size)) = (u16_at(fvar + 4), u16_at(fvar + 8), u16_at(fvar + 10)) else { return false };
    (0..usize::from(count)).any(|i| data.get(fvar + usize::from(axes) + i * usize::from(size)..).is_some_and(|a| a.starts_with(b"wght")))
}

/// raea: a flattened text's glyph paths (`at` the transform its children go under) each filled and stroked `w` wide in its fill (one translucent fill in a layer
/// of its opacity, so the overlap isn't drawn twice), then its own stroke. False, drawing nothing, when the text holds
/// anything else (a COLR, SVG or bitmap glyph's group), which is drawn as it was.
fn draw_widened<S: PaintScene, F: FnMut(&mut S, &usvg::Node)>(scene: &mut S, group: &usvg::Group, at: Affine, rel: bool, w: f64, error_handler: &mut F) -> bool {
    if !group.children().iter().all(|n| matches!(n, usvg::Node::Path(p) if !matches!(p.fill().map(usvg::Fill::paint), Some(usvg::Paint::Pattern(_))))) {
        return false;
    }
    for node in group.children() {
        let usvg::Node::Path(path) = node else { continue };
        if !path.is_visible() {
            continue;
        }
        let local = util::to_bez_path(path);
        // (placed as render_children places them: under `rel`, where the text is; otherwise by their absolute transforms)
        let transform = if rel { at } else { at * util::to_affine(&node.abs_transform()) };
        if let Some(fill) = path.fill()
            && let Some((paint, brush_transform)) = util::to_brush(fill.paint(), usvg::Opacity::ONE)
        {
            let alpha = fill.opacity().get();
            if alpha < 1.0 {
                scene.push_layer(Fill::NonZero, BlendMode::default(), alpha, transform, &local.bounding_box().inflate(w, w), None, None);
            }
            let rule = if fill.rule() == usvg::FillRule::EvenOdd { Fill::EvenOdd } else { Fill::NonZero };
            scene.fill(rule, transform, paint.as_ref(), Some(brush_transform), &local);
            scene.stroke(&kurbo::Stroke::new(w), transform, paint.as_ref(), Some(brush_transform), &local);
            if alpha < 1.0 {
                scene.pop_layer();
            }
        }
        stroke(scene, error_handler, path, transform, &local, node);
    }
    true
}

/// raea: blitz-dom's mark (its NON_SCALING_STROKE_ID) on a path with vector-effect="non-scaling-stroke"
const NON_SCALING_STROKE_ID: &str = "raea-nss-";

thread_local! {
    /// raea: while a tree drawn by `render_svg_tree_in` is drawn, its transform split into the page's (`host`) and the
    /// viewBox's mapping (a non-scaling stroke is drawn under the first alone)
    pub(crate) static VIEWPORT: std::cell::Cell<Option<(Affine, Affine)>> = const { std::cell::Cell::new(None) };
}

/// raea: a non-scaling stroke: the path taken by `pre` (its place in the tree) and the viewBox (VIEWPORT) into its
/// SVG's viewport, and stroked under the page's transform alone, its paint carried along; a pattern stroke as an
/// ordinary one.
fn stroke_in<S: PaintScene, F: FnMut(&mut S, &usvg::Node)>(
    scene: &mut S,
    error_handler: &mut F,
    path: &Path,
    transform: Affine,
    pre: Affine,
    local_path: &BezPath,
    node: &Node,
) {
    let Some(st) = path.stroke() else { return };
    // the tree's transform is the page's, then the viewBox's: the path is carried through the viewBox too (a tree
    // drawn without the split has none to leave out)
    let (host, pre) = match VIEWPORT.with(|v| v.get()) {
        Some((host, viewport)) => (host, viewport * pre),
        None => (transform, pre),
    };
    match util::to_brush(st.paint(), st.opacity()) {
        Some((paint, brush_transform)) if !matches!(st.paint(), usvg::Paint::Pattern(_)) => {
            scene.stroke(&util::to_stroke(st), host, paint.as_ref(), Some(pre * brush_transform), &(pre * local_path.clone()));
        }
        _ => stroke(scene, error_handler, path, transform * pre, local_path, node),
    }
}

/// raea: a group's drawn bounds in its own space: what its layers clip to.
fn group_bounds(g: &usvg::Group) -> kurbo::Rect {
    let b = g.layer_bounding_box();
    kurbo::Rect::new(b.left() as f64, b.top() as f64, b.right() as f64, b.bottom() as f64)
}

/// raea: lib.rs clip_path_shape
pub(crate) fn tree_clip(tree: &usvg::Tree) -> (BezPath, Fill) {
    let mut out = BezPath::new();
    let mut rule = None;
    if let Some(usvg::Node::Group(g)) = tree.root().children().first()
        && let Some(c) = g.clip_path()
    {
        clip_shapes(c.root(), util::to_affine(&c.transform()), &mut out, &mut rule);
    }
    (out, if rule == Some(usvg::FillRule::EvenOdd) { Fill::EvenOdd } else { Fill::NonZero })
}

/// raea: a clip path's shapes, in the clipped element's space, appended to `out` (its first shape's clip-rule).
fn clip_shapes(group: &usvg::Group, transform: Affine, out: &mut BezPath, rule: &mut Option<usvg::FillRule>) {
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => clip_shapes(g, transform * util::to_affine(&g.transform()), out, rule),
            usvg::Node::Path(p) => {
                if !p.is_visible() {
                    continue;
                }
                rule.get_or_insert(p.fill().map_or(usvg::FillRule::NonZero, |f| f.rule()));
                let mut b = util::to_bez_path(p);
                b.apply_affine(transform);
                out.extend(b);
            }
            usvg::Node::Text(t) => clip_shapes(t.flattened(), transform, out, rule),
            usvg::Node::Image(_) => {}
        }
    }
}

/// raea: a mask over what's been drawn since its layer was pushed: its content (and its own mask's) in a layer
/// composited DestIn, luminance taken as alpha by a colour matrix (raea-page's GPU pass), clipped to the mask's region.
fn draw_mask<S: PaintScene, F: FnMut(&mut S, &usvg::Node)>(scene: &mut S, mask: &usvg::Mask, space: Affine, error_handler: &mut F) {
    let r = mask.rect();
    let region = kurbo::Rect::new(r.left() as f64, r.top() as f64, r.right() as f64, r.bottom() as f64);
    let filter = match mask.kind() {
        usvg::MaskType::Luminance => Some(Arc::new(Filter::single(FilterEffect::ColorMatrix(anyrender::filters::color_transformation::ColorMatrix(luminance_to_alpha(true)))))),
        usvg::MaskType::Alpha => None,
    };
    let dest_in = BlendMode { mix: peniko::Mix::Normal, compose: peniko::Compose::DestIn };
    scene.push_layer(Fill::NonZero, dest_in, 1.0, space, &region, filter, None);
    render_children(scene, mask.root(), space * util::to_affine(&mask.root().transform()), Affine::IDENTITY, true, error_handler);
    scene.pop_layer();
    if let Some(m) = mask.mask() {
        draw_mask(scene, m, space, error_handler);
    }
}

/// raea: luminanceToAlpha's matrix; `straight`: colour left (a mask's: its alpha is what's used)
pub(crate) fn luminance_to_alpha(straight: bool) -> [f32; 20] {
    let k = if straight { 1.0 } else { 0.0 };
    [k, 0.0, 0.0, 0.0, 0.0, 0.0, k, 0.0, 0.0, 0.0, 0.0, 0.0, k, 0.0, 0.0, 0.2125, 0.7154, 0.0721, 0.0, 0.0]
}

/// raea: `shape` filled with a pattern: its tile's content drawn at each tile over the shape's bounds, clipped to the
/// shape and each tile, at the fill's opacity.
fn draw_pattern<S: PaintScene, F: FnMut(&mut S, &usvg::Node)>(scene: &mut S, pattern: &usvg::Pattern, opacity: f32, rule: Fill, transform: Affine, shape: &BezPath, error_handler: &mut F) {
    let r = pattern.rect();
    let (w, h) = (r.width() as f64, r.height() as f64);
    let place = util::to_affine(&pattern.transform());
    // the shape's bounds in the pattern's space: the tiles that cover it
    let b = place.inverse().transform_rect_bbox(shape.bounding_box());
    let (i0, i1) = (((b.x0 - r.x() as f64) / w).floor() as i64, ((b.x1 - r.x() as f64) / w).ceil() as i64);
    let (j0, j1) = (((b.y0 - r.y() as f64) / h).floor() as i64, ((b.y1 - r.y() as f64) / h).ceil() as i64);
    if (i1 - i0).saturating_mul(j1 - j0) > 20_000 {
        return;
    }
    scene.push_layer(rule, BlendMode::default(), opacity, transform, shape, None, None);
    let tile = kurbo::Rect::new(0.0, 0.0, w, h);
    for j in j0..j1 {
        for i in i0..i1 {
            let at = transform * place * Affine::translate((r.x() as f64 + i as f64 * w, r.y() as f64 + j as f64 * h));
            scene.push_clip_layer(Fill::NonZero, at, &tile);
            render_children(scene, pattern.root(), at * util::to_affine(&pattern.root().transform()), Affine::IDENTITY, true, error_handler);
            scene.pop_layer();
        }
    }
    scene.pop_layer();
}

/// raea: the first filtered group's filters in `tree` as one graph, and its region (blitz-dom's css_filter_tree: a CSS
/// filter list with url()s, on a rect the element's size). `elements`: the ids of the list's <filter>s; the other
/// filters are its functions, which work in the colour space of the filter before them, as Chrome chains them
/// (`url(#f) grayscale(1)` greys in linear light after a linearRGB <filter>; a function first is in sRGB).
pub(crate) fn tree_filter(tree: &usvg::Tree, elements: &[String]) -> Option<(Filter, kurbo::Rect)> {
    fn find(g: &usvg::Group, elements: &[String]) -> Option<(Filter, kurbo::Rect)> {
        if !g.filters().is_empty() {
            return crate::filter::filter_in(g, Some(elements));
        }
        g.children().iter().find_map(|n| match n {
            usvg::Node::Group(c) => find(c, elements),
            _ => None,
        })
    }
    find(tree.root(), elements)
}

/// raea: a group's filters as one graph for raea-page's GPU passes (each filter's SourceGraphic the last one's result),
/// and the first filter's region in the group's space; None when it has none or one has nothing in it.
fn filter_of(g: &usvg::Group) -> Option<(Filter, kurbo::Rect)> {
    crate::filter::filter_in(g, None)
}
