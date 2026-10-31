//! blitz-paint's painter for a page, with filtered layers painted apart.

use super::*;

use std::hash::Hasher;
use std::sync::Arc;

use anyrender::filters::Filter;
use anyrender::{Glyph, NormalizedCoord, Paint, PaintRef, PaintScene, RenderContext};
use anyrender_vello::VelloScenePainter;
use vello::kurbo::{Affine, PathEl, Rect, Shape, Stroke};
use vello::peniko::{BlendMode, Color, Fill, FontData, ImageAlphaType, ImageBrush, ImageData, ImageFormat, ImageQuality, StyleRef};

/// blitz-paint's painter for a page: what isn't in a filtered layer goes straight into the page's scene (Vello's
/// painter); a filtered layer's content into a scene of its own, its place in its parent kept by a placeholder image.
/// `layers` are the filtered layers in the order they closed, an inner one before the one holding it.
pub struct Painter<'s> {
    root: &'s mut vello::Scene,
    open: Vec<Open>,
    pub layers: Vec<Filtered>,
    /// layers open in the page's own scene
    depth: u32,
    /// the page in device px: a layer's box is cut to it, grown by its filter's reach (what blurs in from outside)
    page: Rect,
    /// every call's arguments hashed: the same print as the last frame's draws the same picture (`print`)
    hash: rustc_hash::FxHasher,
}

impl<'s> Painter<'s> {
    pub fn new(root: &'s mut vello::Scene, width: u32, height: u32) -> Self {
        Painter { root, open: vec![], layers: vec![], depth: 0, page: Rect::new(0.0, 0.0, width as f64, height as f64), hash: Default::default() }
    }

    /// What was painted, as a number: the same as the last frame's, the page's picture hasn't changed (what's in a
    /// view's texture aside: H7's views are drawn again each frame they're placed).
    pub fn print(&self) -> u64 {
        self.hash.finish()
    }

    fn f64s(&mut self, v: &[f64]) {
        for x in v {
            self.hash.write_u64(x.to_bits());
        }
    }

    fn affine(&mut self, a: Affine) {
        self.f64s(&a.as_coeffs());
    }

    fn shape(&mut self, shape: &impl Shape) {
        if let Some(r) = shape.as_rect() {
            self.hash.write_u8(1);
            return self.f64s(&[r.x0, r.y0, r.x1, r.y1]);
        }
        for el in shape.path_elements(0.1) {
            match el {
                PathEl::MoveTo(p) => self.f64s(&[0.0, p.x, p.y]),
                PathEl::LineTo(p) => self.f64s(&[1.0, p.x, p.y]),
                PathEl::QuadTo(a, b) => self.f64s(&[2.0, a.x, a.y, b.x, b.y]),
                PathEl::CurveTo(a, b, c) => self.f64s(&[3.0, a.x, a.y, b.x, b.y, c.x, c.y]),
                PathEl::ClosePath => self.hash.write_u8(4),
            }
        }
    }

    fn color(&mut self, c: Color) {
        for x in c.components {
            self.hash.write_u32(x.to_bits());
        }
    }

    fn paint(&mut self, p: &PaintRef<'_>) {
        match p {
            Paint::Solid(c) => self.color(*c),
            Paint::Gradient(g) => self.hash.write(format!("{g:?}").as_bytes()),
            Paint::Image(i) => {
                self.hash.write_u64(i.image.data.id());
                self.hash.write(format!("{:?}{:?}", (i.image.width, i.image.height, i.image.format, i.image.alpha_type), i.sampler).as_bytes());
            }
            Paint::Resource(r) => self.hash.write(format!("{r:?}").as_bytes()),
            Paint::Custom(c) => match c.downcast_ref::<blitz_paint::TilePattern>() {
                Some(t) => self.hash.write(format!("{:?}{:?}{:?}", t.gradient, t.gradient_transform, t.tile).as_bytes()),
                // an opaque brush: never the same picture twice
                None => self.hash.write_u64(rand_like()),
            },
        }
    }

    /// The scene being painted and the transform from page to its pixels.
    fn top(&mut self) -> (&mut vello::Scene, Affine) {
        match self.open.last_mut() {
            Some(o) => (&mut o.layer.scene, Affine::translate((-o.layer.origin.0 as f64, -o.layer.origin.1 as f64))),
            None => (self.root, Affine::IDENTITY),
        }
    }

    /// One more (or one fewer) plain layer open in the scene being painted.
    fn deeper(&mut self, by: i32) {
        let d = match self.open.last_mut() {
            Some(o) => &mut o.depth,
            None => &mut self.depth,
        };
        *d = d.saturating_add_signed(by);
    }

    /// A box in device px grown by `by`, cut to the page grown by `reach` (what's off the page can still blur onto
    /// it): its origin and size in whole pixels.
    fn region(&self, b: Rect, by: Rect, reach: Rect) -> ((f64, f64), (u32, u32)) {
        let grow = |r: Rect, g: Rect| Rect::new(r.x0 + g.x0, r.y0 + g.y0, r.x1 + g.x1, r.y1 + g.y1);
        let b = grow(b, by).intersect(grow(self.page, reach));
        let (x0, y0) = (b.x0.floor(), b.y0.floor());
        ((x0, y0), ((b.x1.ceil() - x0).max(0.0) as u32, (b.y1.ceil() - y0).max(0.0) as u32))
    }

    /// `backdrop-filter`: what's painted so far under the layer's box, filtered, drawn clipped to the box before its
    /// own content. The backdrop is the scene being painted (the page, or the filtered layer holding this one).
    fn backdrop(&mut self, fill: Fill, transform: Affine, clip: &impl Shape, filter: Arc<Filter>) {
        let reach = reach(&filter, transform);
        let ((x0, y0), (w, h)) = self.region(transform.transform_rect_bbox(clip.bounding_box()), reach, reach);
        if w == 0 || h == 0 {
            return;
        }
        let scale = transform.determinant().abs().sqrt() as f32;
        let open = self.open.last().map_or(self.depth, |o| o.depth + 1);
        let (scene, at) = self.top();
        // the scene so far with its open layers closed, moved to the region's pixels
        let mut so_far = scene.clone();
        for _ in 0..open {
            so_far.pop_layer();
        }
        let mut region = vello::Scene::new();
        region.append(&so_far, Some(Affine::translate((-x0, -y0)) * at.inverse()));
        let image = ImageData { data: vello::peniko::Blob::new(Arc::new([0u8; 0])), format: ImageFormat::Rgba8, alpha_type: ImageAlphaType::Alpha, width: w, height: h };
        let place = Affine::translate((x0, y0));
        scene.push_clip_layer(fill, at * transform, clip);
        scene.fill(Fill::NonZero, at * place, &ImageBrush::new(image.clone()).with_quality(ImageQuality::Low), None, &Rect::new(0.0, 0.0, w as f64, h as f64));
        scene.pop_layer();
        self.layers.push(Filtered { scene: region, filter, scale, origin: (x0 as i32, y0 as i32), size: (w, h), image, user: Affine::translate((-x0, -y0)) * transform, linear: None, warp: None });
    }

    /// A fill whose brush is a tile repeated both ways (blitz-paint's TilePattern: a gradient layer of many tiles):
    /// the tile drawn once into a texture of its device size (the filtered layers' path, no filter) and the shape
    /// filled with it repeating, the tile's top left at the fill's origin.
    fn pattern(&mut self, style: Fill, transform: Affine, t: &blitz_paint::TilePattern, shape: &impl Shape) {
        let (_, at) = self.top();
        let s = (at * transform).determinant().abs().sqrt();
        let (tw, th) = ((t.tile.width * s).ceil().max(1.0), (t.tile.height * s).ceil().max(1.0));
        let mut scene = vello::Scene::new();
        // sampled at its pixels' centres, as every other gradient fill is (centred)
        let tile_at = Affine::scale_non_uniform(tw / t.tile.width, th / t.tile.height);
        let gradient_at = centred(&Paint::Gradient(&t.gradient), tile_at, t.gradient_transform);
        scene.fill(Fill::NonZero, tile_at, &t.gradient, gradient_at, &Rect::new(0.0, 0.0, t.tile.width, t.tile.height));
        let image = ImageData { data: vello::peniko::Blob::new(Arc::new([0u8; 0])), format: ImageFormat::Rgba8, alpha_type: ImageAlphaType::Alpha, width: tw as u32, height: th as u32 };
        // whole device pixels a tile, on whole pixels: sampled nearest, as the tiles were drawn
        let exact = (t.tile.width * s).fract() == 0.0 && (t.tile.height * s).fract() == 0.0;
        let quality = if exact { ImageQuality::Low } else { ImageQuality::Medium };
        let brush = ImageBrush::new(image.clone()).with_extend(vello::peniko::Extend::Repeat).with_quality(quality);
        let (scene_at, at) = self.top();
        scene_at.fill(style, at * transform, &brush, Some(Affine::scale_non_uniform(t.tile.width / tw, t.tile.height / th)), shape);
        let filter = Arc::new(Filter::empty());
        let linear = Linear::of(&t.gradient, t.gradient_transform, (tw / t.tile.width, th / t.tile.height));
        self.layers.push(Filtered { scene, filter, scale: s as f32, origin: (0, 0), size: (tw as u32, th as u32), image, user: tile_at, linear, warp: None });
    }

    /// Paint into the scene being painted, at its pixels.
    fn with<R>(&mut self, f: impl FnOnce(&mut VelloScenePainter<'static, '_>, Affine) -> R) -> R {
        let (scene, at) = self.top();
        f(&mut VelloScenePainter::new(scene), at)
    }
}

impl RenderContext for Painter<'_> {}

impl PaintScene for Painter<'_> {
    fn reset(&mut self) {
        self.root.reset();
        self.open.clear();
        self.layers.clear();
    }

    fn push_layer(
        &mut self,
        fill: Fill,
        blend: impl Into<BlendMode>,
        alpha: f32,
        transform: Affine,
        clip: &impl Shape,
        filter: Option<Arc<Filter>>,
        backdrop_filter: Option<Arc<Filter>>,
    ) {
        let blend = blend.into();
        self.hash.write_u8(10);
        self.hash.write_u8(fill as u8);
        self.hash.write_u8(blend.mix as u8);
        self.hash.write_u8(blend.compose as u8);
        self.hash.write_u32(alpha.to_bits());
        self.affine(transform);
        self.shape(clip);
        if filter.is_some() || backdrop_filter.is_some() {
            self.hash.write(format!("{filter:?}{backdrop_filter:?}").as_bytes());
        }
        if let Some(bd) = backdrop_filter.filter(|f| !f.nodes().is_empty()) {
            self.backdrop(fill, transform, clip, bd);
        }
        // a filter of no primitives (blitz-paint's mark of a backdrop root) still paints the layer apart
        let Some(filter) = filter else {
            self.deeper(1);
            return self.with(|p, at| p.push_layer(fill, blend, alpha, at * transform, clip, None, None));
        };
        // the layer's box in device px: blitz-paint's clip is the border box already grown by the filter's reach
        let reach = reach(&filter, transform);
        let ((x0, y0), (w, h)) = self.region(transform.transform_rect_bbox(clip.bounding_box()), Rect::ZERO, reach);
        let scale = transform.determinant().abs().sqrt() as f32;
        let mut scene = vello::Scene::new();
        // the layer's own clip, inside its scene: a rotated box clips before it blurs
        scene.push_clip_layer(fill, Affine::translate((-x0, -y0)) * transform, clip);
        // an empty blob: Vello never reads it, the filtered texture overrides it (vello's register_texture does this)
        let image = ImageData { data: vello::peniko::Blob::new(Arc::new([0u8; 0])), format: ImageFormat::Rgba8, alpha_type: ImageAlphaType::Alpha, width: w, height: h };
        let layer = Filtered { scene, filter, scale, origin: (x0 as i32, y0 as i32), size: (w, h), image, user: Affine::translate((-x0, -y0)) * transform, linear: None, warp: None };
        self.open.push(Open { layer, depth: 0, blend, alpha });
    }

    /// A CSS 3D transform's perspective (B3 H5): the content drawn in its own space into a scene of its own, which the
    /// filter passes warp onto the box its corners land in (`Passes::run`, the warp pass).
    fn push_warp_layer(&mut self, h: [f64; 9], bounds: Rect) {
        self.hash.write_u8(14);
        self.f64s(&h);
        self.f64s(&[bounds.x0, bounds.y0, bounds.x1, bounds.y1]);
        // the content's box in whole px, at most 4096 a side
        let (x0, y0) = (bounds.x0.floor(), bounds.y0.floor());
        let (w, hh) = ((bounds.x1.ceil() - x0).clamp(0.0, 4096.0) as u32, (bounds.y1.ceil() - y0).clamp(0.0, 4096.0) as u32);
        // where it lands in the page (pop_layer places the image through the scene's own `at`): the box cut where
        // it passes behind the viewer (w at least a little above 0; the warp pass draws nothing behind), its corners
        // and the cut's mapped
        let wz = |(x, y): (f64, f64)| h[6] * x + h[7] * y + h[8];
        let near = 1e-3 * h[8].abs().max(1e-6);
        let quad = [(x0, y0), (x0 + w as f64, y0), (x0 + w as f64, y0 + hh as f64), (x0, y0 + hh as f64)];
        let mut kept = vec![];
        for i in 0..4 {
            let (a, b) = (quad[i], quad[(i + 1) % 4]);
            let (za, zb) = (wz(a) - near, wz(b) - near);
            if za >= 0.0 {
                kept.push(a);
            }
            if (za >= 0.0) != (zb >= 0.0) {
                let t = za / (za - zb);
                kept.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            }
        }
        let landed = (!kept.is_empty()).then(|| {
            kept.iter().fold(Rect::new(f64::MAX, f64::MAX, f64::MIN, f64::MIN), |r, &(x, y)| {
                let z = wz((x, y)).max(near);
                let p = ((h[0] * x + h[1] * y + h[2]) / z, (h[3] * x + h[4] * y + h[5]) / z);
                Rect::new(r.x0.min(p.0), r.y0.min(p.1), r.x1.max(p.0), r.y1.max(p.1))
            })
        });
        // cut to what the scene being painted shows of the page: the page, or the filtered layer holding this one
        let (_, at) = self.top();
        let shown = match self.open.last() {
            Some(o) => at.inverse().transform_rect_bbox(Rect::new(0.0, 0.0, o.layer.size.0 as f64, o.layer.size.1 as f64)),
            None => self.page,
        };
        let out = landed.map(|r| r.intersect(shown)).filter(|r| r.width() > 0.0 && r.height() > 0.0);
        let (origin, size, to_layer) = match (out, inverse3(&h)) {
            (Some(r), Some(inv)) => {
                let (ox, oy) = (r.x0.floor(), r.y0.floor());
                let size = ((r.x1.ceil() - ox) as u32, (r.y1.ceil() - oy) as u32);
                // a pixel of the box to a texel of the layer: back through h, less the layer's origin
                let t = mul3(&[1.0, 0.0, -x0, 0.0, 1.0, -y0, 0.0, 0.0, 1.0], &mul3(&inv, &[1.0, 0.0, ox, 0.0, 1.0, oy, 0.0, 0.0, 1.0]));
                ((ox as i32, oy as i32), size, t.map(|v| v as f32))
            }
            _ => ((0, 0), (0, 0), [0.0; 9]),
        };
        let mut scene = vello::Scene::new();
        scene.push_clip_layer(Fill::NonZero, Affine::translate((-x0, -y0)), &bounds);
        let image = ImageData { data: vello::peniko::Blob::new(Arc::new([0u8; 0])), format: ImageFormat::Rgba8, alpha_type: ImageAlphaType::Alpha, width: size.0.max(1), height: size.1.max(1) };
        let warp = Warp { origin, size, to_layer };
        let layer = Filtered { scene, filter: Arc::new(Filter::empty()), scale: 1.0, origin: (x0 as i32, y0 as i32), size: (w, hh), image, user: Affine::IDENTITY, linear: None, warp: Some(warp) };
        self.open.push(Open { layer, depth: 0, blend: BlendMode::default(), alpha: 1.0 });
    }

    fn push_clip_layer(&mut self, fill: Fill, transform: Affine, clip: &impl Shape) {
        self.hash.write_u8(11);
        self.hash.write_u8(fill as u8);
        self.affine(transform);
        self.shape(clip);
        self.deeper(1);
        self.with(|p, at| p.push_clip_layer(fill, at * transform, clip));
    }

    fn pop_layer(&mut self) {
        self.hash.write_u8(12);
        match self.open.last_mut() {
            Some(o) if o.depth > 0 => o.depth -= 1,
            Some(_) => {
                let Open { mut layer, blend, alpha, .. } = self.open.pop().unwrap();
                layer.scene.pop_layer();
                // a warped layer's image is the box it lands in on the page
                let ((x, y), (w, h)) = layer.warp.as_ref().map_or((layer.origin, layer.size), |w| (w.origin, w.size));
                if w > 0 && h > 0 && layer.size.0 > 0 && layer.size.1 > 0 {
                    let place = Affine::translate((x as f64, y as f64));
                    let image = ImageBrush::new(layer.image.clone()).with_quality(ImageQuality::Low);
                    let (scene, at) = self.top();
                    let bounds = Rect::new(0.0, 0.0, w as f64, h as f64);
                    let plain = alpha >= 1.0 && blend == BlendMode::default();
                    if !plain {
                        scene.push_layer(Fill::NonZero, blend, alpha, at * place, &bounds);
                    }
                    scene.fill(Fill::NonZero, at * place, &image, None, &bounds);
                    if !plain {
                        scene.pop_layer();
                    }
                    self.layers.push(layer);
                }
                return;
            }
            None => self.depth = self.depth.saturating_sub(1),
        }
        self.with(|p, _| p.pop_layer());
    }

    fn stroke<'a>(&mut self, style: &Stroke, transform: Affine, brush: impl Into<PaintRef<'a>>, brush_transform: Option<Affine>, shape: &impl Shape) {
        let brush = brush.into();
        self.hash.write_u8(13);
        self.f64s(&[style.width, style.miter_limit, style.dash_offset, style.join as u8 as f64, style.start_cap as u8 as f64, style.end_cap as u8 as f64]);
        self.f64s(&style.dash_pattern);
        self.affine(transform);
        self.paint(&brush);
        self.affine(brush_transform.unwrap_or(Affine::IDENTITY));
        self.shape(shape);
        let brush_transform = centred(&brush, transform, brush_transform);
        self.with(|p, at| p.stroke(style, at * transform, brush, brush_transform, shape));
    }

    fn fill<'a>(&mut self, style: Fill, transform: Affine, brush: impl Into<PaintRef<'a>>, brush_transform: Option<Affine>, shape: &impl Shape) {
        let brush = brush.into();
        self.hash.write_u8(14);
        self.hash.write_u8(style as u8);
        self.affine(transform);
        self.paint(&brush);
        self.affine(brush_transform.unwrap_or(Affine::IDENTITY));
        self.shape(shape);
        if let Paint::Custom(c) = &brush
            && let Some(t) = c.downcast_ref::<blitz_paint::TilePattern>()
        {
            return self.pattern(style, transform, t, shape);
        }
        let brush_transform = centred(&brush, transform, brush_transform);
        self.with(|p, at| p.fill(style, at * transform, brush, brush_transform, shape));
    }

    fn draw_glyphs<'a, 's: 'a>(
        &'s mut self,
        font: &'a FontData,
        font_size: f32,
        hint: bool,
        normalized_coords: &'a [NormalizedCoord],
        embolden: vello::kurbo::Vec2,
        style: impl Into<StyleRef<'a>>,
        brush: impl Into<PaintRef<'a>>,
        brush_alpha: f32,
        transform: Affine,
        glyph_transform: Option<Affine>,
        glyphs: impl Iterator<Item = Glyph> + Clone,
    ) {
        let (style, brush) = (style.into(), brush.into());
        self.hash.write_u8(15);
        self.hash.write_u64(font.data.id());
        self.hash.write_u32(font.index);
        self.hash.write_u32(font_size.to_bits());
        self.hash.write_u8(hint as u8);
        for c in normalized_coords {
            self.hash.write_i16(*c);
        }
        self.f64s(&[embolden.x, embolden.y]);
        match &style {
            StyleRef::Fill(f) => self.hash.write_u8(*f as u8),
            StyleRef::Stroke(s) => self.f64s(&[s.width, s.join as u8 as f64, s.start_cap as u8 as f64]),
        }
        self.paint(&brush);
        self.hash.write_u32(brush_alpha.to_bits());
        self.affine(transform);
        self.affine(glyph_transform.unwrap_or(Affine::IDENTITY));
        for g in glyphs.clone() {
            self.hash.write_u32(g.id);
            self.hash.write_u32(g.x.to_bits());
            self.hash.write_u32(g.y.to_bits());
        }
        let (scene, at) = self.top();
        VelloScenePainter::new(scene).draw_glyphs(font, font_size, hint, normalized_coords, embolden, style, brush, brush_alpha, at * transform, glyph_transform, glyphs);
    }

    fn draw_box_shadow(&mut self, transform: Affine, rect: Rect, brush: Color, radius: f64, std_dev: f64) {
        self.hash.write_u8(16);
        self.affine(transform);
        self.f64s(&[rect.x0, rect.y0, rect.x1, rect.y1, radius, std_dev]);
        self.color(brush);
        self.with(|p, at| p.draw_box_shadow(at * transform, rect, brush, radius, std_dev));
    }
}

/// A gradient brush moved half a device pixel up and left: Vello's fine shader samples gradients at each pixel's
/// top left corner (images at its centre), Chrome at its centre, so every gradient was half a pixel off and a hard
/// stop on a pixel centre fell the wrong way (desktop.tsx's 2 px scanlines over its 2 px checker split every square).
/// `transform` is the fill's, which the page's translation to a layer's pixels doesn't change. Upstream: vello's
/// fine.wgsl CMD_LIN_GRAD, CMD_RAD_GRAD and CMD_SWEEP_GRAD use `xy`, not `xy + 0.5`.
/// Row-major 3x3 homographies (on x, y, w), for warped layers.
fn mul3(a: &[f64; 9], b: &[f64; 9]) -> [f64; 9] {
    let mut m = [0.0; 9];
    for i in 0..3 {
        for j in 0..3 {
            m[i * 3 + j] = (0..3).map(|k| a[i * 3 + k] * b[k * 3 + j]).sum();
        }
    }
    m
}

fn inverse3(m: &[f64; 9]) -> Option<[f64; 9]> {
    let [a, b, c, d, e, f, g, h, i] = *m;
    let (x, y, z) = (e * i - f * h, f * g - d * i, d * h - e * g);
    let det = a * x + b * y + c * z;
    if det.abs() < 1e-12 {
        return None;
    }
    let r = 1.0 / det;
    Some([x * r, (c * h - b * i) * r, (b * f - c * e) * r, y * r, (a * i - c * g) * r, (c * d - a * f) * r, z * r, (b * g - a * h) * r, (a * e - b * d) * r])
}

fn centred(brush: &PaintRef<'_>, transform: Affine, brush_transform: Option<Affine>) -> Option<Affine> {
    if !matches!(brush, Paint::Gradient(_)) || transform.determinant().abs() < 1e-12 {
        return brush_transform;
    }
    Some(transform.inverse() * Affine::translate((-0.5, -0.5)) * transform * brush_transform.unwrap_or(Affine::IDENTITY))
}

/// A number that differs on each call (a custom brush's print).
fn rand_like() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    N.fetch_add(1, Ordering::Relaxed)
}

/// How far `filter` reaches past a box, in device px: its expansion scaled by the transform (anyrender's
/// `bounds_expansion` moves it by the transform's translation too).
fn reach(filter: &Filter, transform: Affine) -> Rect {
    let r = filter.expansion_rect();
    let s = transform.determinant().abs().sqrt();
    Rect::new(r.x0 * s, r.y0 * s, r.x1 * s, r.y1 * s)
}
