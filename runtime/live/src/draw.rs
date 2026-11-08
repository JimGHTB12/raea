//! A live frame drawn, snapshotted and presented, and its 3D scene opened.

use super::*;

impl Live {
    /// The page as it stands, drawn and read back as premultiplied RGBA (the playtest's --snapshot): what the stage
    /// shows, its 3D under it.
    pub fn snapshot(&mut self) -> R<(u32, u32, Vec<u8>)> {
        self.draw()?;
        Ok((self.w, self.h, self.page.frame()?))
    }

    /// Whether a frame was stepped (or the page made again) since the page was last drawn: a host drawing it only when
    /// it changes asks this after a step (void's shell, apps/legacy/shell/shell.feature).
    pub fn changed(&self) -> bool {
        self.dirty
    }

    /// A baked timeline's clips (its Sequences' boxes, data-sequence), for a timeline control (void's dock): each one's
    /// name, its first frame (data-raea-start) and the frame after its last (the commit that took its box away, after the
    /// last keyframe it's in; else the next keyframe, or the end), how deeply it's nested in others, and what media it
    /// holds (video, audio, img). Empty played live (a game).
    pub fn clips(&self) -> Vec<Value> {
        let Some(b) = self.baked.as_ref() else { return Vec::new() };
        struct Clip {
            name: String,
            start: i64,
            last_key: i64,
            /// its node at the last keyframe it's in (a keyframe's nodes are its own: ids differ between them, and a
            /// removed node's id is given to a later one)
            id: u64,
            depth: usize,
            holds: std::collections::BTreeSet<String>,
        }
        // by name and start, not node: a Sequence's node differs from keyframe to keyframe, and a removed one's id is
        // given to a later one (six Sequences one after another read as one clip, then as some twice)
        let mut clips: std::collections::BTreeMap<(i64, String), Clip> = Default::default();
        for (f, ops) in &b.keys {
            let (mut parent, mut kids, mut kind): (std::collections::HashMap<u64, u64>, std::collections::HashMap<u64, Vec<u64>>, std::collections::HashMap<u64, &str>) = Default::default();
            let mut boxes: Vec<(u64, String, i64)> = Vec::new();
            for op in ops {
                let id = |k: &str| op.get(k).and_then(|v| v.as_u64());
                match op.get("op").and_then(|o| o.as_str()) {
                    Some("create") => {
                        let Some(i) = id("id") else { continue };
                        kind.insert(i, op.get("kind").and_then(|k| k.as_str()).unwrap_or(""));
                        let props = op.get("props");
                        let start = props.and_then(|p| p.get("data-raea-start")).and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));
                        if let Some(start) = start {
                            let name = props.and_then(|p| p.get("data-sequence")).and_then(|v| v.as_str()).unwrap_or("Sequence").to_string();
                            boxes.push((i, name, start.round() as i64));
                        }
                    }
                    Some("append") => {
                        if let (Some(p), Some(c)) = (id("parent"), id("child")) {
                            parent.insert(c, p);
                            kids.entry(p).or_default().push(c);
                        }
                    }
                    _ => {}
                }
            }
            let is_box: std::collections::HashSet<u64> = boxes.iter().map(|x| x.0).collect();
            for (i, name, start) in boxes {
                let mut depth = 0;
                let mut at = i;
                while let Some(&p) = parent.get(&at) {
                    depth += is_box.contains(&p) as usize;
                    at = p;
                }
                let mut holds = std::collections::BTreeSet::new();
                let mut stack = vec![i];
                while let Some(n) = stack.pop() {
                    if let Some(k @ ("video" | "audio" | "img")) = kind.get(&n).copied() {
                        holds.insert(k.to_string());
                    }
                    stack.extend(kids.get(&n).into_iter().flatten().copied());
                }
                let c = clips.entry((start, name.clone())).or_insert(Clip { name, start, last_key: *f, id: i, depth, holds: Default::default() });
                (c.last_key, c.id) = (*f, i);
                c.holds.extend(holds);
            }
        }
        let gone = |id: u64, from: i64| -> i64 {
            let (child, until) = (format!("\"child\":{id}"), (from + raea_runtime_bake::KEY).min(b.frames));
            (from.max(0)..until).find(|&f| b.frame_ops.get(f as usize).is_some_and(|ops| ops.iter().any(|o| o.contains("\"remove") && o.contains(&child)))).unwrap_or(until)
        };
        clips.into_values().map(|c| json!({"name": c.name, "start": c.start, "end": gone(c.id, c.last_key + 1), "depth": c.depth, "holds": c.holds})).collect()
    }

    /// The page as it stands, written as a PNG (void's RAEA_STAGE_DUMP).
    pub fn save_snapshot(&mut self, path: &Path) -> R<()> {
        let (w, h, px) = self.snapshot()?;
        image::save_buffer(path, &px, w, h, image::ColorType::Rgba8).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// The mixer's drift samples (ms), for the playtest report.
    pub fn drift_ms(&self) -> &[f64] {
        self.mixer.as_ref().map(|m| m.drift_ms.as_slice()).unwrap_or(&[])
    }

    /// Paused or stopped: the sound fades out (it comes back with the next frame stepped).
    pub fn silence(&mut self) {
        if let Some(m) = self.mixer.as_mut() {
            m.stop_all();
        }
    }

    /// The sound device's stream stopped while paused (after silence and its fade-outs): no callback renders silence
    /// through the graph (lightspeed R6). The next frame's sound starts it again.
    pub fn rest(&mut self) {
        if let Some(m) = self.mixer.as_mut() {
            m.rest();
        }
    }

    /// The frame stepped last drawn on the page, on the shared device and read back nowhere: the 3D under it or in its
    /// layered slot (its engine opened when its rows arrive, each <Screen>'s picture as the page last drew it copied in,
    /// this frame drawn) and the elements holding the 3D see-through, as a render draws them. Its picture is `texture`.
    pub fn draw(&mut self) -> R<()> {
        self.open_scene();
        let slot = if self.scene.is_some() { raea_runtime_react::scene::scene_slot(self.app.world_mut()) } else { None };
        // a camera in a box of the page (docs/sdk.feature S3a; it was at the last frame): the page laid out first, the
        // view drawn into that box of the 3D's frame and the box's picture, the page's backgrounds kept
        if self.scene.is_some() && slot.is_none() && self.page.in_box {
            self.page.has_under(false);
            self.page.sync(self.app.world_mut(), &std::collections::HashMap::new(), self.brand.as_deref())?;
            self.page.draw();
            if let Some(b) = self.page.view_box() {
                if let Some(e) = self.scene.as_mut() {
                    for (name, t) in self.page.screen_textures() {
                        e.copy_screen(&name, &t);
                    }
                    e.set_view_box(Some(b));
                    let mats = self.page.set_ui_materials(e.world_mut())?;
                    e.draw(self.frame);
                    if mats {
                        self.page.place_ui_materials(e.world());
                    }
                    if let Some(err) = e.take_error() {
                        eprintln!("raea live: frame {}: the 3D: {err}", self.frame);
                    }
                    if let Some(t) = e.target_texture() {
                        self.page.set_view_texture(&t, b);
                    }
                }
                self.page.draw();
                self.dirty = false;
                return Ok(());
            }
            self.page.in_box = false;
        }
        if let Some(e) = self.scene.as_mut() {
            e.set_view_box(None);
        }
        // a page with <Screen>s: synced and drawn before the 3D, so each screen's picture is this frame's, as a
        // render's (they showed the frame before; synced again below with the 3D's holders)
        if self.scene.is_some() && !self.page.screen_textures().is_empty() {
            self.page.has_under(false);
            self.page.sync(self.app.world_mut(), &std::collections::HashMap::new(), self.brand.as_deref())?;
            self.page.draw();
        }
        let target = match self.scene.as_mut() {
            // RAEA_SHARED_DEVICE=0 puts Bevy's 3D on a device of its own, whose texture the page can't draw
            Some(_) if std::env::var("RAEA_SHARED_DEVICE").as_deref() == Ok("0") => return Err("live 3D draws under the page on the one shared GPU device; RAEA_SHARED_DEVICE=0 gives Bevy a device of its own, which the page can't draw over (unset it)".into()),
            Some(e) => {
                for (name, t) in self.page.screen_textures() {
                    e.copy_screen(&name, &t);
                }
                // the UiMaterial nodes the page last laid out (a frame behind, as the page syncs after the 3D here)
                self.page.set_ui_materials(e.world_mut())?;
                e.draw(self.frame);
                if let Some(err) = e.take_error() {
                    eprintln!("raea live: frame {}: the 3D: {err}", self.frame);
                }
                e.target_texture().map(|t| (t, e.size()))
            }
            None => None,
        };
        // the elements holding a 3D drawn under the whole page show it through: their backgrounds go
        let mut holders = std::collections::HashMap::new();
        if target.is_some() && slot.is_none() {
            let mut held = std::mem::take(&mut self.holders);
            raea_runtime_react::scene::holder_entities(self.app.world_mut(), &mut held);
            let world = self.app.world();
            for &e in &held {
                let (Some(n), Some(p)) = (world.get::<raea_runtime_react::ReactNode>(e), world.get::<raea_runtime_react::ReactProps>(e)) else { continue };
                let mut p = p.0.clone();
                raea_runtime_react::scene::see_through(&mut p);
                holders.insert(n.0, p);
            }
            self.holders = held;
        }
        self.page.has_under(target.is_some() && slot.is_none());
        self.page.sync(self.app.world_mut(), &holders, self.brand.as_deref())?;
        if let Some((t, (tw, th))) = &target {
            match slot {
                Some(_) => self.page.set_scene_texture(t, *tw, *th),
                None => self.page.set_under(t, *tw, *th),
            }
        }
        if let Some(e) = self.scene.as_ref() {
            self.page.place_ui_materials(e.world());
        }
        self.page.draw();
        // its camera's box came this frame: drawn again as a box
        if target.is_some() && slot.is_none() && self.page.view_box().is_some() {
            self.page.in_box = true;
            return self.draw();
        }
        self.dirty = false;
        Ok(())
    }

    /// The page as it stands, drawn and presented on `stage` (only when something changed since the last present, or
    /// `force`); true when a window's layer took the frame. Offscreen, the GPU is waited for, so a frame's time is its
    /// whole draw.
    pub fn present(&mut self, stage: &mut raea_gpu_device::present::Present, force: bool) -> R<bool> {
        if !self.dirty && !force {
            return Ok(false);
        }
        self.draw()?;
        let Some(t) = self.texture() else { return Ok(false) };
        if !stage.has_surface() {
            if let Ok((_, _, device, _)) = raea_gpu_device::device() {
                let _ = device.poll(wgpu::PollType::wait_indefinitely());
            }
            return Ok(false);
        }
        stage.show(&t, self.w, self.h)
    }

    /// The page drawn and shared as texture `id` for a window's chrome to draw the stage in its own frame
    /// (`<img data-raea-texture=id>`, page.rs share_texture). `fit` is the box it's shown in: the page keeps its own
    /// size and layout (video is fixed scale) and the chrome scales it. The texture's size.
    pub fn paint_texture_fit(&mut self, id: u64, fit: (u32, u32)) -> R<(u32, u32)> {
        let _ = fit;
        self.draw()?;
        let t = self.texture().ok_or("the stage's page has no picture")?;
        raea_runtime_page::share_texture(id, t, self.w, self.h);
        Ok((self.w, self.h))
    }

    /// The page's picture as last drawn (`draw`): a texture on the shared device, `w` x `h`, premultiplied.
    pub fn texture(&self) -> Option<wgpu::Texture> {
        self.page.texture()
    }

    /// The 3D's rows in from their thread: the engine opens here, once.
    pub(crate) fn open_scene(&mut self) {
        let Some(rx) = &self.scene_rx else { return };
        match rx.try_recv() {
            Ok(Ok(Some(prog))) => {
                match raea_render::engine::Engine::new_live(&prog, &raea_render::inserted_of(&prog), &self.dir) {
                    Ok(e) => self.scene = Some(e),
                    Err(e) => eprintln!("raea live: the 3D didn't open: {e}"),
                }
                self.scene_rx = None;
            }
            Ok(Ok(None)) | Err(std::sync::mpsc::TryRecvError::Disconnected) => self.scene_rx = None,
            Ok(Err(e)) => {
                eprintln!("raea live: the 3D's rows failed: {e}");
                self.scene_rx = None;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
        }
    }
}
