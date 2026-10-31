//! A frame composited on the GPU: its clips from the pool, the overlay and page drawn over them, waited for or not.

use super::*;

impl GpuView {
    pub fn frame(&mut self, f: i64) -> Result<FrameTimes, String> {
        self.frame_wait(f, true)
    }

    /// As `frame`; without `wait` the composite is only submitted (the direct render keeps several frames in flight).
    pub(crate) fn frame_wait(&mut self, f: i64, wait: bool) -> Result<FrameTimes, String> {
        let t0 = Instant::now();
        self.tick += 1;
        let tick = self.tick;
        let mut ft = FrameTimes::default();
        let st = self.state(f)?.clone();
        ft.rules = t0.elapsed().as_secs_f64() * 1e3;

        // the overlay, rendered once per distinct state
        let t = Instant::now();
        // a page handed over on the GPU (render_nv12_fed) is this frame's overlay, under a key no state hashes to
        let fed = self.fed.take();
        let was_fed = fed.is_some();
        let dg = if was_fed { FED } else { self.overlay_key(&st, f) };
        ft.overlay_cached = fed.is_none() && self.overlays.contains_key(&dg);
        if let Some(b) = fed {
            self.overlays.insert(FED, b);
        } else if !ft.overlay_cached {
            let px = match self.ahead.take() {
                Some((af, adg, px)) if af == f && adg == dg => px,
                _ => self.render_overlay(&st, f)?,
            };
            let b = self.upload(&px);
            // a page that changes every frame would keep a full-frame buffer per frame: keep the last few states
            if self.overlays.len() >= 32 {
                self.overlays.clear();
            }
            self.overlays.insert(dg, b);
        }
        // the next frame's row overlay painted on the GPU while this frame's clips decode and composite (a frame
        // waiting on its own paint cost ~5 ms of GPU latency each)
        // (not for a fed page: the caller paints the next one, and a row paint between would share its compositor)
        if !was_fed && self.page.is_none() && self.ahead.is_none() && f + 1 < self.prog.frames {
            let st1 = self.state(f + 1)?.clone();
            let dg1 = Self::overlay_digest(&st1, f + 1);
            if dg1 != dg && !self.overlays.contains_key(&dg1) && self.page_image_rows(&st1, f + 1).is_none() {
                let px = self.render_overlay(&st1, f + 1)?;
                self.ahead = Some((f + 1, dg1, px));
            }
        }
        ft.overlay = t.elapsed().as_secs_f64() * 1e3;

        // the clips: decode, upload once, slot
        let t = Instant::now();
        let (fps, fw, fh) = (self.prog.fps, self.w as f64, self.h as f64);
        let mut clips: Vec<&Tup> = st.iter().filter(|t| t.kind == "clip").collect();
        clips.sort_by(|a, b| num(&a.vals, "z", 0.0).total_cmp(&num(&b.vals, "z", 0.0)).then(a.id.cmp(&b.id)));
        let mut slots: Vec<(PoolKey, raea_core_k::Slot)> = Vec::new();
        let mut masks: Vec<Option<String>> = Vec::new();
        let mut slots_of: HashMap<String, usize> = HashMap::new();
        for c in clips {
            let v = &c.vals;
            let from = num(v, "from", 0.0) as i64;
            let to = v.get("to").map(|x| x.num() as i64).unwrap_or(i64::MAX);
            let opacity = num(v, "opacity", 1.0) as f32 as f64;
            if f < from || f >= to || opacity <= 0.0 {
                continue;
            }
            let Some(V::S(src)) = v.get("src") else { return Err(format!("clip {}: no src", c.id)) };
            let slot = {
                let s = slots_of.entry(src.clone()).or_insert(0);
                *s += 1;
                *s - 1
            };
            let at = num(v, "src_in", 0.0) + (f - from) as f64 / fps;
            if !self.decoders.contains_key(&(src.clone(), slot)) {
                let p = raea_sys_frame::resolve_src(&self.rows, src, &self.dir)?;
                let d = self.backend.open_video(&p)?;
                self.decoders.insert((src.clone(), slot), d);
            }
            let dec = self.decoders.get_mut(&(src.clone(), slot)).unwrap();
            // the pool is keyed by the frame's index, known without decoding: a frame in the pool isn't decoded again
            let idx = dec.index_at(at);
            let (dw, dh) = (dec.info().width, dec.info().height);
            let hit = idx.map(|i| (src.clone(), i as i64)).filter(|k| self.pool.contains_key(k));
            let key: PoolKey = match hit {
                Some(k) => {
                    self.pool_used.insert(k.clone(), tick);
                    k
                }
                None => {
                    #[cfg(target_vendor = "apple")]
                    let zc_on = self.zc.is_some();
                    #[cfg(not(target_vendor = "apple"))]
                    let zc_on = false;
                    let (pts, pooled) = if zc_on {
                        #[cfg(target_vendor = "apple")]
                        {
                            let sf = dec.frame_surface_at(at)?.ok_or("RAEA_ZEROCOPY needs the apple backend's decoders")?;
                            let planes = self.zc.as_ref().unwrap().0.wrap(&self.gpu.device, sf.buffer, sf.width, sf.height)?;
                            (sf.pts, Pooled::Surf(planes))
                        }
                        #[cfg(not(target_vendor = "apple"))]
                        unreachable!()
                    } else {
                        let fr = dec.frame_nv12_at(at)?;
                        (fr.pts, Pooled::Buf(self.upload(&fr.rgba)))
                    };
                    let key = (src.clone(), idx.map(|i| i as i64).unwrap_or((pts * 1e6).round() as i64 + (1 << 40)));
                    self.pool_bytes += pooled.bytes();
                    self.pool.insert(key.clone(), pooled);
                    self.pool_used.insert(key.clone(), tick);
                    ft.uploads += 1;
                    // over budget: the least recently used frames not in this frame go
                    if self.pool_bytes > crate::device::pool_budget() {
                        let mut old: Vec<(PoolKey, u64)> = self.pool_used.iter().filter(|(_, t)| **t != tick).map(|(k, t)| (k.clone(), *t)).collect();
                        old.sort_by_key(|x| x.1);
                        for (k, _) in old {
                            if self.pool_bytes <= crate::device::pool_budget() {
                                break;
                            }
                            if let Some(b) = self.pool.remove(&k) {
                                self.pool_bytes -= b.bytes();
                            }
                            self.pool_used.remove(&k);
                        }
                    }
                    key
                }
            };
            let fr = (dw, dh);
            let (sw, sh) = (fr.0 as f64, fr.1 as f64);
            // the row's values through f32, as the engine's World holds them, so the slots (and frames) are the engine's
            let n32 = |k: &str, d: f64| num(v, k, d) as f32 as f64;
            let (x, y, w, h) = (n32("x", 0.0), n32("y", 0.0), n32("w", fw), n32("h", fh));
            let zoom = n32("zoom", 1.0).max(0.01);
            let (cx, cy, cw, ch) = raea_sys_frame::cover(sw, sh, w, h, zoom, (n32("pan_x", 0.0), n32("pan_y", 0.0)));
            let q = |v: f64| (v * 65536.0).round() as i32;
            masks.push(crate::masks::mask_of(v));
            slots.push((
                key,
                raea_core_k::Slot {
                    on: 1,
                    bx: x.round() as i32,
                    by: y.round() as i32,
                    ex: (x + w).round() as i32,
                    ey: (y + h).round() as i32,
                    sw: fr.0 as i32,
                    sh: fr.1 as i32,
                    cx: q(cx - cw / 2.0),
                    cy: q(cy - ch / 2.0),
                    dx: q(cw / w),
                    dy: q(ch / h),
                    al: (opacity.min(1.0) * 256.0).round() as i32,
                },
            ));
        }
        if slots.len() > 4 {
            return Err(format!("frame {f}: more than four clips at once"));
        }
        ft.decode_upload = t.elapsed().as_secs_f64() * 1e3;

        // one composite into the resident frame; clips with a mask (a solid in space and time) show its slice
        let t = Instant::now();
        let mut sl: Vec<raea_core_k::Slot> = (0..4).map(|s| slots.get(s).map(|x| x.1).unwrap_or_else(raea_core_k::Slot::off)).collect();
        let mask = crate::masks::slot_masks(&masks, &mut sl, f, self.w as usize, self.h as usize, self.prog.frames)?;
        let mask_buf = mask.map(|m| self.gpu.buffer(m.len() * 4, Some(bytemuck::cast_slice(&m))));
        let mbuf = mask_buf.as_ref().unwrap_or(&self.no_mask);
        let mut words: Vec<u8> = Vec::new();
        for s in sl.iter() {
            words.extend(s.words().iter().flat_map(|v| v.to_le_bytes()));
        }
        let params = self.gpu.buffer(words.len(), Some(&words));
        let o = &self.overlays[&dg];
        let n = (self.w * self.h) as usize;
        let ts = self.ts.as_ref().map(|q| (q, 2));
        #[cfg(target_vendor = "apple")]
        if let Some((_, comp_tex, dy, duv, _)) = &self.zc {
            // zero-copy: each slot's planes are the decoder's own buffer, as textures (composite_tex.k)
            let mut ys: Vec<&wgpu::TextureView> = Vec::new();
            let mut uvs: Vec<&wgpu::TextureView> = Vec::new();
            for s in 0..4 {
                match slots.get(s).map(|x| &self.pool[&x.0]) {
                    Some(Pooled::Surf(p)) => {
                        ys.push(&p.y);
                        uvs.push(&p.uv);
                    }
                    Some(Pooled::Buf(_)) => return Err("zero-copy: a pooled frame was copied".into()),
                    None => {
                        ys.push(dy);
                        uvs.push(duv);
                    }
                }
            }
            let tex: Vec<&wgpu::TextureView> = std::iter::once(&self.bg_view).chain(ys).chain(uvs).collect();
            self.gpu.dispatch_ts(comp_tex, &[o, &params, mbuf], &tex, &[&self.out], n, wait, ts);
            // RAEA_ZC_FLUSH=0 keeps CoreVideo's texture cache as it is (diagnostics); by default it's flushed each frame
            if std::env::var("RAEA_ZC_FLUSH").map(|v| v != "0").unwrap_or(true) {
                self.zc.as_ref().unwrap().0.flush();
            }
            ft.composite_gpu = t.elapsed().as_secs_f64() * 1e3;
            ft.total = t0.elapsed().as_secs_f64() * 1e3;
            return Ok(ft);
        }
        let planes: Vec<&wgpu::Buffer> = (0..4)
            .map(|s| match slots.get(s).map(|x| &self.pool[&x.0]) {
                Some(Pooled::Buf(b)) => b,
                _ => &self.dummy,
            })
            .collect();
        self.gpu.dispatch_ts(&self.comp, &[planes[0], planes[1], planes[2], planes[3], o, &params, mbuf], &[&self.bg_view], &[&self.out], n, wait, ts);
        ft.composite_gpu = t.elapsed().as_secs_f64() * 1e3;
        ft.total = t0.elapsed().as_secs_f64() * 1e3;
        Ok(ft)
    }

    /// The overlay at frame f drawn by bevy_ui (premultiplied RGBA, full size).
    pub(crate) fn render_overlay(&mut self, st: &[Tup], f: i64) -> Result<Vec<u8>, String> {
        if let Some(p) = self.page.as_mut() {
            p.seek(f as f64 / self.prog.fps.max(1.0))?;
            return p.frame();
        }
        if let Some(px) = self.page_image(st, f) {
            return Ok(px);
        }
        self.set_rows(st);
        let mut px = Vec::new();
        self.rw.render(f, &mut px)?;
        Ok(px)
    }

    /// A frame whose only 2D row showing is one image over the whole canvas, as raea's pages are: its picture
    /// is the overlay (transparent canvas, so the picture premultiplied, as vello takes it), decoded from its asset
    /// without painting a document around it. A page already painted once (on the GPU) was painted again here, for
    /// every frame (2026-10-17).
    pub(crate) fn page_image(&self, st: &[Tup], f: i64) -> Option<Vec<u8>> {
        let mut on = st.iter().filter(|t| crate::rows::is_2d(&t.kind)).filter(|t| {
            let v = &t.vals;
            f >= num(v, "from", f64::MIN) as i64 && v.get("to").map(|x| f < x.num() as i64).unwrap_or(true)
        });
        let t = on.next()?;
        if on.next().is_some() || t.kind != "image" {
            return None;
        }
        let v = &t.vals;
        let plain = v.keys().all(|k| matches!(k.as_str(), "src" | "x" | "y" | "w" | "h" | "from" | "to" | "z"));
        let whole = num(v, "x", 0.0) == 0.0 && num(v, "y", 0.0) == 0.0 && num(v, "w", -1.0) == self.w as f64 && num(v, "h", -1.0) == self.h as f64;
        let Some(V::S(src)) = v.get("src") else { return None };
        let hash = src.strip_prefix("asset:")?;
        if !(plain && whole) {
            return None;
        }
        let img = image::open(self.dir.join("assets").join(format!("{hash}.png"))).ok()?.into_rgba8();
        if img.width() != self.w || img.height() != self.h {
            return None;
        }
        let mut px = img.into_raw();
        for p in px.chunks_exact_mut(4) {
            let a = p[3] as u32;
            for c in &mut p[..3] {
                *c = ((*c as u32 * a + 127) / 255) as u8;
            }
        }
        Some(px)
    }

    /// Whether `page_image` would take frame `f` (cheap: the rows only, no decode).
    pub(crate) fn page_image_rows(&self, st: &[Tup], f: i64) -> Option<()> {
        let mut on = st.iter().filter(|t| crate::rows::is_2d(&t.kind)).filter(|t| {
            let v = &t.vals;
            f >= num(v, "from", f64::MIN) as i64 && v.get("to").map(|x| f < x.num() as i64).unwrap_or(true)
        });
        let t = on.next()?;
        (on.next().is_none() && t.kind == "image").then_some(())
    }

    /// Each 2D row's frame values onto its DOM node, where they changed (the engine's to_dom).
    pub(crate) fn set_rows(&mut self, st: &[Tup]) {
        for row in st.iter().filter(|t| crate::rows::is_2d(&t.kind)) {
            let id = format!("{}:{}", row.kind, row.id);
            let Some(base) = self.rw.base(&id) else { continue };
            let p = crate::rows::props_2d(&crate::rows::Base2d(base.clone()), &row.vals);
            self.rw.set(&id, p);
        }
    }
}
