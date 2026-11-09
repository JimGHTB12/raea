//! The ID scene: blitz-paint's calls drawn with the layer index of the node each draws for (blitz_paint::painting) as
//! a flat colour, antialiasing off, so each pixel holds exactly one layer's index: low byte red, high byte green,
//! alpha 255 where something was drawn. Brushes, opacity, blends and filters are not drawn; clips are. A node in no
//! layer draws index 0, so it still covers the layers under it.

use std::collections::HashMap;
use std::sync::Arc;

use anyrender::{Glyph, NormalizedCoord, PaintRef, PaintScene, RenderContext};
use blitz_dom::NodeId;
use vello_cpu::kurbo::{Affine, BezPath, Rect, Shape, Stroke};
use vello_cpu::peniko::{BlendMode, Color, Fill, FontData, StyleRef};
use vello_cpu::{PaintType, Pixmap, Resources};

/// a pixel more than this covered (of 255) is drawn whole, a less covered one not at all: about where an edge changes
/// the picture visibly (bench/layers/oracle.sh counts a change past 10 of 255)
const COVERED: u8 = 12;
const TOLERANCE: f64 = 0.1;

pub(crate) struct IdScene<'a> {
    ctx: vello_cpu::RenderContext,
    resources: Resources,
    /// each node's layer index (nodes not here: 0)
    ids: &'a HashMap<NodeId, u16>,
}

impl<'a> IdScene<'a> {
    pub(crate) fn new(w: u16, h: u16, ids: &'a HashMap<NodeId, u16>) -> Self {
        let mut ctx = vello_cpu::RenderContext::new(w, h);
        ctx.set_aliasing_threshold(Some(COVERED));
        IdScene { ctx, resources: Resources::new(), ids }
    }

    /// Each pixel's layer index, row-major (NONE where nothing was drawn).
    pub(crate) fn finish(mut self) -> Vec<u16> {
        self.ctx.flush();
        let mut px = Pixmap::new(self.ctx.width(), self.ctx.height());
        self.ctx.render(&mut px, &mut self.resources);
        px.data().iter().map(|p| if p.a == 0 { crate::file::NONE } else { p.r as u16 | (p.g as u16) << 8 }).collect()
    }

    /// Ready to draw for the node being painted, at `transform`: false when its brush draws nothing.
    fn brush(&mut self, paint: PaintRef<'_>, transform: Affine) -> bool {
        if let PaintRef::Solid(c) = paint {
            if c.components[3] <= 0.0 {
                return false;
            }
        }
        let id = blitz_paint::painting().and_then(|n| self.ids.get(&n)).copied().unwrap_or(0);
        self.ctx.set_transform(transform);
        self.ctx.reset_paint_transform();
        self.ctx.set_paint(PaintType::Solid(Color::from_rgba8((id & 0xff) as u8, (id >> 8) as u8, 0, 255)));
        true
    }
}

impl RenderContext for IdScene<'_> {}

impl PaintScene for IdScene<'_> {
    fn reset(&mut self) {
        self.ctx.reset();
    }

    fn push_layer(&mut self, fill: Fill, _blend: impl Into<BlendMode>, alpha: f32, transform: Affine, clip: &impl Shape, _filter: Option<Arc<anyrender::Filter>>, _backdrop: Option<Arc<anyrender::Filter>>) {
        // a layer drawn at no opacity shows nothing of its content
        if alpha <= 0.0 {
            return self.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &Rect::ZERO);
        }
        self.push_clip_layer(fill, transform, clip);
    }

    fn push_clip_layer(&mut self, fill: Fill, transform: Affine, clip: &impl Shape) {
        self.ctx.set_transform(transform);
        self.ctx.set_fill_rule(fill);
        self.ctx.push_clip_layer(&clip.into_path(TOLERANCE));
    }

    fn pop_layer(&mut self) {
        self.ctx.pop_layer();
    }

    fn stroke<'b>(&mut self, style: &Stroke, transform: Affine, brush: impl Into<PaintRef<'b>>, _bt: Option<Affine>, shape: &impl Shape) {
        if self.brush(brush.into(), transform) {
            self.ctx.set_stroke(style.clone());
            self.ctx.stroke_path(&shape.into_path(TOLERANCE));
        }
    }

    fn fill<'b>(&mut self, style: Fill, transform: Affine, brush: impl Into<PaintRef<'b>>, _bt: Option<Affine>, shape: &impl Shape) {
        if self.brush(brush.into(), transform) {
            self.ctx.set_fill_rule(style);
            let path: BezPath = shape.into_path(TOLERANCE);
            self.ctx.fill_path(&path);
        }
    }

    fn draw_glyphs<'b, 's: 'b>(
        &'s mut self,
        font: &'b FontData,
        font_size: f32,
        hint: bool,
        coords: &'b [NormalizedCoord],
        embolden: vello_cpu::kurbo::Vec2,
        style: impl Into<StyleRef<'b>>,
        brush: impl Into<PaintRef<'b>>,
        brush_alpha: f32,
        transform: Affine,
        glyph_transform: Option<Affine>,
        glyphs: impl Iterator<Item = Glyph> + Clone,
    ) {
        if brush_alpha <= 0.0 || !self.brush(brush.into(), transform) {
            return;
        }
        let glyphs = glyphs.map(|g| vello_cpu::Glyph { id: g.id, x: g.x, y: g.y });
        let run = |ctx: &'s mut vello_cpu::RenderContext, res: &'s mut Resources| ctx.glyph_run(res, font).font_size(font_size).hint(hint).normalized_coords(coords).glyph_transform(glyph_transform.unwrap_or_default()).font_embolden(glifo::FontEmbolden::new(vello_cpu::kurbo::Diagonal2::new(embolden.x, embolden.y)));
        match style.into() {
            StyleRef::Fill(fill) => {
                self.ctx.set_fill_rule(fill);
                run(&mut self.ctx, &mut self.resources).fill_glyphs(glyphs);
            }
            StyleRef::Stroke(stroke) => {
                self.ctx.set_stroke(stroke.clone());
                run(&mut self.ctx, &mut self.resources).stroke_glyphs(glyphs);
            }
        }
    }

    /// a shadow is its element's (hiding the element hides it): drawn out to where its blur still shows, about 1.5
    /// deviations past its box
    fn draw_box_shadow(&mut self, transform: Affine, rect: Rect, brush: Color, radius: f64, std_dev: f64) {
        if self.brush(PaintRef::Solid(brush), transform) {
            let grow = 1.5 * std_dev;
            self.ctx.fill_path(&vello_cpu::kurbo::RoundedRect::from_rect(rect.inflate(grow, grow), radius + grow).into_path(TOLERANCE));
        }
    }
}
