//! The display list's ops drawn: fills, strokes, clears, text, images and videos, each under its state's shadow,
//! alpha, blend and clip.

use super::*;

impl Ctx<'_> {
    pub(crate) fn op(&mut self, pm: &mut sk::Pixmap, op: &Value) {
        let Some(a) = op.as_array() else { return };
        let (w, h) = (pm.width(), pm.height());
        match a.first().and_then(Value::as_str) {
            Some("clear") => {
                let st = &a[2];
                let Some(p) = path_of(&a[1]) else { return };
                let mask = clip_of(st, w, h);
                let mut paint = sk::Paint::default();
                paint.set_color(sk::Color::TRANSPARENT);
                paint.blend_mode = sk::BlendMode::Clear;
                paint.anti_alias = false;
                pm.fill_path(&p, &paint, sk::FillRule::Winding, sk::Transform::identity(), mask.as_ref());
            }
            Some("fill") => {
                let st = &a[4];
                let Some(p) = path_of(&a[1]) else { return };
                let rule = if a[2].as_str() == Some("evenodd") { sk::FillRule::EvenOdd } else { sk::FillRule::Winding };
                let alpha = st["alpha"].as_f64().unwrap_or(1.0) as f32;
                // in the call's own space, so a gradient is in it too
                let ts = ts_of(&st["ts"]);
                let Some(inv) = ts.invert() else { return };
                let Some(local) = p.transform(inv) else { return };
                self.shape(pm, st, |layer, paint| {
                    let mut paint = paint.clone();
                    if let Some(s) = shader_of(&a[3], alpha) {
                        paint.shader = s;
                    }
                    layer.fill_path(&local, &paint, rule, ts, None);
                });
            }
            Some("stroke") => {
                let st = &a[4];
                let Some(p) = path_of(&a[1]) else { return };
                let alpha = st["alpha"].as_f64().unwrap_or(1.0) as f32;
                let ts = ts_of(&st["ts"]);
                let Some(inv) = ts.invert() else { return };
                let Some(local) = p.transform(inv) else { return };
                let stroke = stroke_of(&a[2]);
                self.shape(pm, st, |layer, paint| {
                    let mut paint = paint.clone();
                    if let Some(s) = shader_of(&a[3], alpha) {
                        paint.shader = s;
                    }
                    layer.stroke_path(&local, &paint, &stroke, ts, None);
                });
            }
            Some("text") => self.text(pm, a),
            Some("image") => {
                let Some(img) = a.get(1).and_then(Value::as_str).and_then(|src| self.decode(src)) else { return };
                self.image(pm, a, &img)
            }
            // ["video", src, sx, sy, sw, sh, dx, dy, dw, dh, state, t]: the frame showing at t
            Some("video") => {
                let Some(img) = a.get(1).and_then(Value::as_str).and_then(|src| video_frame(src, a.get(11).and_then(Value::as_f64).unwrap_or(0.0))) else { return };
                self.image(pm, a, &img)
            }
            _ => {}
        }
    }

    /// A shape drawn through the call's clip, alpha, compositing and shadow: `draw` paints it on a layer (or straight
    /// onto the bitmap when nothing needs a layer).
    pub(crate) fn shape(&mut self, pm: &mut sk::Pixmap, st: &Value, draw: impl Fn(&mut sk::Pixmap, &sk::Paint)) {
        let (w, h) = (pm.width(), pm.height());
        let mut paint = sk::Paint { anti_alias: true, ..Default::default() };
        let mode = blend(st["comp"].as_str().unwrap_or("source-over"));
        let clip = clip_of(st, w, h);
        let shadow = st.get("shadow").filter(|s| !s.is_null());
        let filters = st.get("filter").and_then(Value::as_array).filter(|l| !l.is_empty());
        if shadow.is_none() && filters.is_none() && clip.is_none() && matches!(mode, sk::BlendMode::SourceOver | sk::BlendMode::DestinationOut | sk::BlendMode::DestinationOver | sk::BlendMode::Plus | sk::BlendMode::Multiply | sk::BlendMode::Screen) {
            paint.blend_mode = mode;
            draw(pm, &paint);
            return;
        }
        let Some(mut layer) = sk::Pixmap::new(w, h) else { return };
        draw(&mut layer, &paint);
        if let Some(l) = filters {
            crate::filter::apply(&mut layer, l);
        }
        if let Some(sh) = shadow {
            let Some(mut s) = sk::Pixmap::new(w, h) else { return };
            let col = color_of(&sh["color"], 1.0).unwrap_or(sk::Color::BLACK);
            let (ox, oy) = (f(&sh["x"]), f(&sh["y"]));
            // the shape's alpha in the shadow's colour, offset, blurred (sigma: half the blur)
            if let Some(mask) = Some(sk::Mask::from_pixmap(layer.as_ref(), sk::MaskType::Alpha)) {
                let mut p = sk::Paint::default();
                p.set_color(col);
                s.fill_rect(sk::Rect::from_xywh(0.0, 0.0, w as f32, h as f32).unwrap(), &p, sk::Transform::identity(), Some(&mask));
            }
            let mut shifted = sk::Pixmap::new(w, h).unwrap();
            shifted.draw_pixmap(0, 0, s.as_ref(), &sk::PixmapPaint::default(), sk::Transform::from_translate(ox, oy), None);
            blur(&mut shifted, f(&sh["blur"]) / 2.0);
            pm.draw_pixmap(0, 0, shifted.as_ref(), &sk::PixmapPaint { blend_mode: mode, ..Default::default() }, sk::Transform::identity(), clip.as_ref());
        }
        pm.draw_pixmap(0, 0, layer.as_ref(), &sk::PixmapPaint { blend_mode: mode, ..Default::default() }, sk::Transform::identity(), clip.as_ref());
    }

    pub(crate) fn text(&mut self, pm: &mut sk::Pixmap, a: &[Value]) {
        // ["text", kind, text, x, y, font, align, baseline, maxWidth, paint, stroke, state]
        let (Some(kind), Some(text)) = (a.get(1).and_then(Value::as_str), a.get(2).and_then(Value::as_str)) else { return };
        let st = &a[11];
        let font = &a[5];
        let anchor = match a[6].as_str() {
            Some("center") => "middle",
            Some("right") => "end",
            _ => "start",
        };
        let base = match a[7].as_str() {
            Some("top") => "text-before-edge",
            Some("hanging") => "hanging",
            Some("middle") => "middle",
            Some("bottom") => "text-after-edge",
            Some("ideographic") => "ideographic",
            _ => "alphabetic",
        };
        let mut esc = String::new();
        for c in text.chars() {
            match c {
                '&' => esc.push_str("&amp;"),
                '<' => esc.push_str("&lt;"),
                _ => esc.push(c),
            }
        }
        let fam = font["family"].as_str().unwrap_or("sans-serif").replace('"', "'");
        let paint_attr = if kind == "fill" {
            "fill=\"#fff\"".to_string()
        } else {
            let s = &a[10];
            format!("fill=\"none\" stroke=\"#fff\" stroke-width=\"{}\" stroke-linejoin=\"{}\"", f(&s["width"]), s["join"].as_str().unwrap_or("miter"))
        };
        let (w, h) = (pm.width(), pm.height());
        let doc = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\"><text x=\"{}\" y=\"{}\" font-size=\"{}\" font-family=\"{fam}\" font-weight=\"{}\" font-style=\"{}\" text-anchor=\"{anchor}\" dominant-baseline=\"{base}\" {paint_attr}>{esc}</text></svg>",
            f(&a[3]),
            f(&a[4]),
            f(&font["size"]),
            font["weight"].as_str().unwrap_or("400"),
            font["style"].as_str().unwrap_or("normal"),
        );
        let Ok(tree) = usvg::Tree::from_str(&doc, self.opt) else { return };
        let ts = ts_of(&st["ts"]);
        // the glyphs' coverage in the call's space, then the paint through it
        let Some(mut cover) = sk::Pixmap::new(w, h) else { return };
        resvg::render(&tree, ts, &mut cover.as_mut());
        let mask = sk::Mask::from_pixmap(cover.as_ref(), sk::MaskType::Alpha);
        let alpha = st["alpha"].as_f64().unwrap_or(1.0) as f32;
        let shader = shader_of(&a[9], alpha);
        let Some(inv) = ts.invert() else { return };
        self.shape(pm, st, |layer, paint| {
            let mut paint = paint.clone();
            if let Some(s) = shader.clone() {
                paint.shader = s;
            }
            // a rect over the whole bitmap in the call's space, painted through the glyphs
            let r = sk::Rect::from_xywh(0.0, 0.0, w as f32, h as f32).unwrap();
            if let Some(p) = sk::PathBuilder::from_rect(r).transform(inv) {
                layer.fill_path(&p, &paint, sk::FillRule::Winding, ts, Some(&mask));
            }
        });
    }

    pub(crate) fn decode(&mut self, src: &str) -> Option<sk::Pixmap> {
        if let Some(p) = self.images.get(src) {
            return Some(p.clone());
        }
        let (mime, bytes) = raea_io_net::data_url(src)?;
        let mime = mime.as_str();
        let img = if mime == "image/svg+xml" {
            let size = crate::svg::natural_size(&bytes, self.opt)?;
            crate::svg::raster_doc(&bytes, size.as_uvec2().max(UVec2::ONE), self.opt)?
        } else {
            Image::from_buffer(&bytes, bevy::image::ImageType::MimeType(mime), bevy::image::CompressedImageFormats::NONE, true, bevy::image::ImageSampler::Default, RenderAssetUsages::RENDER_WORLD).ok()?
        };
        let img = img.convert(TextureFormat::Rgba8UnormSrgb)?;
        let (w, h) = (img.width(), img.height());
        let data = img.data?;
        let mut pm = sk::Pixmap::new(w, h)?;
        for (i, px) in pm.pixels_mut().iter_mut().enumerate() {
            let c = &data[i * 4..i * 4 + 4];
            *px = sk::ColorU8::from_rgba(c[0], c[1], c[2], c[3]).premultiply();
        }
        self.images.insert(src.to_string(), pm.clone());
        Some(pm)
    }

    pub(crate) fn image(&mut self, pm: &mut sk::Pixmap, a: &[Value], img: &sk::Pixmap) {
        // ["image", src, sx, sy, sw, sh, dx, dy, dw, dh, state]
        let n = |i: usize| a.get(i).and_then(Value::as_f64).map(|v| v as f32);
        let (iw, ih) = (img.width() as f32, img.height() as f32);
        let (sx, sy, sw, sh) = (n(2).unwrap_or(0.0), n(3).unwrap_or(0.0), n(4).unwrap_or(iw), n(5).unwrap_or(ih));
        let (dx, dy) = (n(6).unwrap_or(0.0), n(7).unwrap_or(0.0));
        let (dw, dh) = (n(8).unwrap_or(sw), n(9).unwrap_or(sh));
        if sw == 0.0 || sh == 0.0 {
            return;
        }
        let st = &a[10];
        let ts = ts_of(&st["ts"]);
        let alpha = st["alpha"].as_f64().unwrap_or(1.0) as f32;
        let smooth = st["smooth"].as_bool().unwrap_or(true);
        // the source rect mapped onto the destination, in the call's space
        let place = ts.pre_translate(dx, dy).pre_scale(dw / sw, dh / sh).pre_translate(-sx, -sy);
        let quality = if smooth { sk::FilterQuality::Bilinear } else { sk::FilterQuality::Nearest };
        let pattern = sk::Pattern::new(img.as_ref(), sk::SpreadMode::Pad, quality, alpha, place);
        let Some(inv) = ts.invert() else { return };
        let _ = inv;
        self.shape(pm, st, |layer, paint| {
            let mut paint = paint.clone();
            paint.shader = pattern.clone();
            if let Some(r) = sk::Rect::from_xywh(dx, dy, dw, dh) {
                layer.fill_rect(r, &paint, ts, None);
            }
        });
    }
}
