//! raea: CSS border-image (css-backgrounds-3 §6), which bevy_ui's NodeImageMode::Sliced is: a picture cut into nine by
//! border-image-slice, its corners drawn into the border-image area's corners (border-image-width), its edges and
//! centre (with `fill`) stretched, repeated, rounded or spaced along them (border-image-repeat), over an area grown by
//! border-image-outset. It replaces the element's border styles once the picture has loaded.

use super::ElementCx;
use anyrender::PaintScene;
use blitz_dom::node::ImageData;
use kurbo::{Affine, Rect};
use style::values::computed::border::BorderImageSideWidth;
use style::values::computed::{NonNegativeLengthOrNumber, NumberOrPercentage};
use style::values::generics::border::GenericBorderImageSideWidth;
use style::values::generics::length::GenericLengthOrNumber;
use style::values::specified::border::BorderImageRepeatKeyword as Repeat;

/// One tile along an axis: where it starts and how long it is, in device px.
type Tile = (f64, f64);

/// bevy_ui's SliceScaleMode for one part (data-raea-bevy-slice): stretched, or tiled from its start every slice x
/// stretch_value px, the last tile cut (ui_texture_slice_pipeline's compute_tiled_subaxis)
#[derive(Clone, Copy)]
enum Bevy {
    Stretch,
    Tile(f64),
}

/// The tiles along one axis of a part `len` long from `start`, each `tile` long at its natural scale.
fn tiles(mode: Repeat, start: f64, len: f64, tile: f64) -> Vec<Tile> {
    if len <= 0.0 {
        return vec![];
    }
    if mode == Repeat::Stretch || tile <= 0.0 {
        return vec![(start, len)];
    }
    match mode {
        // whole tiles, scaled to fit a whole number of them
        Repeat::Round => {
            let n = (len / tile).round().max(1.0);
            (0..n as usize).map(|k| (start + k as f64 * len / n, len / n)).collect()
        }
        // whole tiles, the gaps between them (and at the ends) equal; none when not one fits
        Repeat::Space => {
            let n = (len / tile).floor();
            let gap = (len - n * tile) / (n + 1.0);
            (0..n as usize).map(|k| (start + gap + k as f64 * (tile + gap), tile)).collect()
        }
        // tiles centred on the part, cut at its ends (the caller clips to the part)
        _ => {
            let first = start + (len - tile) / 2.0;
            let before = ((first - start) / tile).ceil();
            let n = (len / tile).ceil() as usize + 2;
            (0..n).map(|k| (first + (k as f64 - before) * tile, tile)).filter(|(p, t)| p + t > start && *p < start + len).collect()
        }
    }
}

impl ElementCx<'_, '_> {
    /// Draws the border-image when it has a loaded picture; false when there's none, so the border styles draw.
    pub(super) fn draw_border_image(&self, scene: &mut impl PaintScene) -> bool {
        let Some(Some(res)) = self.element.border_images.first() else { return false };
        let ImageData::Raster(image) = &res.image else { return false };
        let border = self.style.get_border();
        let (iw, ih) = (image.width as f64, image.height as f64);
        if iw <= 0.0 || ih <= 0.0 {
            return false;
        }
        let s = self.scale;
        let bw = self.frame.border_width;
        // the slice offsets in image px, top right bottom left
        let slice = &border.border_image_slice;
        let cut = |v: &NumberOrPercentage, of: f64| match v {
            NumberOrPercentage::Number(n) => (*n as f64).clamp(0.0, of),
            NumberOrPercentage::Percentage(p) => (p.0 as f64 * of).clamp(0.0, of),
        };
        let o = &slice.offsets;
        let (st, sr, sb, sl) = (cut(&o.0.0, ih), cut(&o.1.0, iw), cut(&o.2.0, ih), cut(&o.3.0, iw));
        // the area: the border box grown by the outset (a number is that many border widths)
        let out = |v: &NonNegativeLengthOrNumber, w: f64| match v {
            GenericLengthOrNumber::Number(n) => n.0 as f64 * w,
            GenericLengthOrNumber::Length(l) => l.0.px() as f64 * s,
        };
        let oo = &border.border_image_outset;
        let bb = self.frame.border_box;
        let area = Rect::new(bb.x0 - out(&oo.3, bw.x0), bb.y0 - out(&oo.0, bw.y0), bb.x1 + out(&oo.1, bw.x1), bb.y1 + out(&oo.2, bw.y1));
        // the widths of its edges (a number is that many border widths, auto the slice), scaled down to fit
        let width = |v: &BorderImageSideWidth, w: f64, of: f64, sliced: f64| match v {
            GenericBorderImageSideWidth::Number(n) => n.0 as f64 * w,
            GenericBorderImageSideWidth::LengthPercentage(lp) => lp.0.resolve(style::values::computed::Length::new((of / s) as f32)).px() as f64 * s,
            GenericBorderImageSideWidth::Auto => sliced * s,
        };
        let ww = &border.border_image_width;
        let (aw, ah) = (area.width(), area.height());
        // bevy_ui's slicer (data-raea-bevy-slice="max_corner_scale sides center", each mode "s" or "t<stretch>"): the
        // edges are the slices times min(the node over the picture, max_corner_scale), and its own tiling
        let bevy = self.element.bevy_slice().map(|a| {
            let mut w = a.split_whitespace();
            let max: f64 = w.next().and_then(|x| x.parse().ok()).unwrap_or(1.0);
            let mode = |m: Option<&str>| match m.and_then(|m| m.strip_prefix('t')).and_then(|v| v.parse().ok()) {
                Some(v) => Bevy::Tile(v),
                None => Bevy::Stretch,
            };
            (max, mode(w.next()), mode(w.next()))
        });
        let (mut wt, mut wr, mut wb, mut wl) = match bevy {
            Some((max, _, _)) => {
                let k = (aw / s / iw).min(ah / s / ih).min(max) * s;
                (st * k, sr * k, sb * k, sl * k)
            }
            None => (width(&ww.0, bw.y0, ah, st), width(&ww.1, bw.x1, aw, sr), width(&ww.2, bw.y1, ah, sb), width(&ww.3, bw.x0, aw, sl)),
        };
        let f = (aw / (wl + wr)).min(ah / (wt + wb)).min(1.0);
        if bevy.is_none() && f.is_finite() && f < 1.0 {
            (wt, wr, wb, wl) = (wt * f, wr * f, wb * f, wl * f);
        }
        let quality = super::to_image_quality(self.style.clone_image_rendering());
        let brush = super::to_peniko_image(image, quality);
        // the image's [x0, x1] x [y0, y1] drawn into tiles along each axis, clipped to the part `to`
        let mut part = |src: Rect, to: Rect, tx: Vec<Tile>, ty: Vec<Tile>| {
            if src.width() <= 0.0 || src.height() <= 0.0 {
                return;
            }
            for &(y, h) in &ty {
                for &(x, w) in &tx {
                    let tile = Rect::new(x, y, x + w, y + h).intersect(to);
                    if tile.width() <= 0.0 || tile.height() <= 0.0 {
                        continue;
                    }
                    let map = Affine::translate((x, y)) * Affine::scale_non_uniform(w / src.width(), h / src.height()) * Affine::translate((-src.x0, -src.y0));
                    scene.fill(peniko::Fill::NonZero, self.transform, brush.as_ref(), Some(map), &tile);
                }
            }
        };
        let (x0, x1, x2, x3) = (area.x0, area.x0 + wl, area.x1 - wr, area.x1);
        let (y0, y1, y2, y3) = (area.y0, area.y0 + wt, area.y1 - wb, area.y1);
        let (sx1, sx2, sy1, sy2) = (sl, iw - sr, st, ih - sb);
        let one = |a: f64, b: f64| vec![(a, b - a)];
        // the corners, each its own slice into its own corner
        part(Rect::new(0.0, 0.0, sx1, sy1), Rect::new(x0, y0, x1, y1), one(x0, x1), one(y0, y1));
        part(Rect::new(sx2, 0.0, iw, sy1), Rect::new(x2, y0, x3, y1), one(x2, x3), one(y0, y1));
        part(Rect::new(0.0, sy2, sx1, ih), Rect::new(x0, y2, x1, y3), one(x0, x1), one(y2, y3));
        part(Rect::new(sx2, sy2, iw, ih), Rect::new(x2, y2, x3, y3), one(x2, x3), one(y2, y3));
        // the edges and centre: along each by border-image-repeat (a tile its slice at the edge's width), or Bevy's
        // mode (a tile the slice times stretch_value, from the part's start)
        let rep = border.border_image_repeat.clone();
        let (mw, mh) = (sx2 - sx1, sy2 - sy1);
        let along_x = |edge_w: f64, slice_h: f64| if slice_h > 0.0 { mw * edge_w / slice_h } else { mw };
        let along_y = |edge_w: f64, slice_w: f64| if slice_w > 0.0 { mh * edge_w / slice_w } else { mh };
        let run = |css: Repeat, own: Option<Bevy>, start: f64, len: f64, tile: f64, slice: f64| match own {
            Some(Bevy::Tile(v)) if slice * v > 0.0 => (0..(len / (slice * v * s)).ceil() as usize).map(|k| (start + k as f64 * slice * v * s, slice * v * s)).collect(),
            Some(_) => one(start, start + len),
            None => tiles(css, start, len, tile),
        };
        let (sides, centre) = (bevy.map(|b| b.1), bevy.map(|b| b.2));
        part(Rect::new(sx1, 0.0, sx2, sy1), Rect::new(x1, y0, x2, y1), run(rep.0, sides, x1, x2 - x1, along_x(wt, st), mw), one(y0, y1));
        part(Rect::new(sx1, sy2, sx2, ih), Rect::new(x1, y2, x2, y3), run(rep.0, sides, x1, x2 - x1, along_x(wb, sb), mw), one(y2, y3));
        part(Rect::new(0.0, sy1, sx1, sy2), Rect::new(x0, y1, x1, y2), one(x0, x1), run(rep.1, sides, y1, y2 - y1, along_y(wl, sl), mh));
        part(Rect::new(sx2, sy1, iw, sy2), Rect::new(x2, y1, x3, y2), one(x2, x3), run(rep.1, sides, y1, y2 - y1, along_y(wr, sr), mh));
        // the centre with `fill`: scaled as the top edge across and the left edge down (the bottom and right without them)
        if slice.fill {
            let tw = if st > 0.0 { along_x(wt, st) } else { along_x(wb, sb) };
            let th = if sl > 0.0 { along_y(wl, sl) } else { along_y(wr, sr) };
            part(Rect::new(sx1, sy1, sx2, sy2), Rect::new(x1, y1, x2, y2), run(rep.0, centre, x1, x2 - x1, tw, mw), run(rep.1, centre, y1, y2 - y1, th, mh));
        }
        true
    }
}
