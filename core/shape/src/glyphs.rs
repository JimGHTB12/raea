//! A word as a solid: the bundled face's glyph outlines, extruded.

use manifold_rust::cross_section::CrossSection;
use manifold_rust::linalg::{Vec2, Vec3};
use manifold_rust::manifold::Manifold;

/// Glyph outlines as closed polylines (curves flattened), in the face's units scaled to `size` per em.
struct Pen {
    polys: Vec<Vec<Vec2>>,
    cur: Vec<Vec2>,
    at: (f64, f64),
    dx: f64,
    k: f64,
}

impl Pen {
    fn pt(&self, x: f32, y: f32) -> (f64, f64) {
        (self.dx + x as f64 * self.k, y as f64 * self.k)
    }
    fn push(&mut self, p: (f64, f64)) {
        self.cur.push(Vec2::new(p.0, p.1));
        self.at = p;
    }
}

impl skrifa::outline::OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.close();
        let p = self.pt(x, y);
        self.push(p);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.pt(x, y);
        self.push(p);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let (a, c, b) = (self.at, self.pt(cx, cy), self.pt(x, y));
        for i in 1..=8 {
            let t = i as f64 / 8.0;
            let u = 1.0 - t;
            self.push((u * u * a.0 + 2.0 * u * t * c.0 + t * t * b.0, u * u * a.1 + 2.0 * u * t * c.1 + t * t * b.1));
        }
    }
    fn curve_to(&mut self, c0x: f32, c0y: f32, c1x: f32, c1y: f32, x: f32, y: f32) {
        let (a, c0, c1, b) = (self.at, self.pt(c0x, c0y), self.pt(c1x, c1y), self.pt(x, y));
        for i in 1..=10 {
            let t = i as f64 / 10.0;
            let u = 1.0 - t;
            let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            self.push((w0 * a.0 + w1 * c0.0 + w2 * c1.0 + w3 * b.0, w0 * a.1 + w1 * c0.1 + w2 * c1.1 + w3 * b.1));
        }
    }
    fn close(&mut self) {
        if self.cur.len() >= 3 {
            self.polys.push(std::mem::take(&mut self.cur));
        }
        self.cur.clear();
    }
}

fn area(p: &[Vec2]) -> f64 {
    (0..p.len()).map(|i| {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        a.x * b.y - b.x * a.y
    }).sum::<f64>() / 2.0
}

/// A word as a solid: the bundled face's outlines (the faces the renderer loads), `size` units an em, `depth` deep,
/// centred on the origin.
pub(crate) fn text3d(text: &str, size: f64, depth: f64, family: &str, weight: u16) -> Result<Manifold, String> {
    use skrifa::MetadataProvider;
    use skrifa::raw::TableProvider;
    let bytes = raea_core_fonts::face_ttf(family, weight, false).or_else(|| raea_core_fonts::face_ttf("Inter", weight, false)).ok_or("no bundled face for text3d")?;
    let font = skrifa::FontRef::new(&bytes).map_err(|e| format!("text3d's face: {e}"))?;
    let upem = font.head().map(|h| h.units_per_em()).unwrap_or(1000) as f64;
    let (charmap, outlines) = (font.charmap(), font.outline_glyphs());
    let metrics = font.glyph_metrics(skrifa::instance::Size::unscaled(), skrifa::instance::LocationRef::default());
    let mut pen = Pen { polys: Vec::new(), cur: Vec::new(), at: (0.0, 0.0), dx: 0.0, k: size / upem };
    for ch in text.chars() {
        let Some(gid) = charmap.map(ch) else { continue };
        if let Some(g) = outlines.get(gid) {
            let settings = skrifa::outline::DrawSettings::unhinted(skrifa::instance::Size::unscaled(), skrifa::instance::LocationRef::default());
            g.draw(settings, &mut pen).map_err(|e| format!("text3d: {e}"))?;
            skrifa::outline::OutlinePen::close(&mut pen);
        }
        pen.dx += metrics.advance_width(gid).unwrap_or(0.0) as f64 * size / upem;
    }
    if pen.polys.is_empty() {
        return Err(format!("text3d({text:?}): nothing to draw"));
    }
    // Manifold fills counter-clockwise contours: turn every glyph so its outer contours run that way (TrueType draws
    // them clockwise, CFF counter-clockwise), and its holes run against them
    let mut polys = pen.polys;
    let biggest = polys.iter().map(|p| area(p)).fold(0.0f64, |m, a| if a.abs() > m.abs() { a } else { m });
    if biggest < 0.0 {
        for p in &mut polys {
            p.reverse();
        }
    }
    let section = CrossSection::new(polys);
    let solid = Manifold::extrude(&section.to_polygons(), depth, 1, 0.0, Vec2::new(1.0, 1.0));
    let b = solid.bounding_box();
    Ok(solid.translate(Vec3::new(-(b.min.x + b.max.x) / 2.0, -(b.min.y + b.max.y) / 2.0, -(b.min.z + b.max.z) / 2.0)))
}
