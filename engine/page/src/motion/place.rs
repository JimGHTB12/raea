//! What the page shows that isn't its own DOM: sub-pages, videos, frames, pictures and textures placed on elements.

use super::*;

impl BlitzPage {
    /// A <Screen>'s nested composition in a document of its own at the screen's size, drawn into its own Image (made
    /// again when the size changes).
    pub fn sync_sub(&mut self, key: &str, w: u32, h: u32, want: &[Want], order: &HashMap<u64, Vec<u64>>) -> R<()> {
        if self.subs.get(key).is_some_and(|s| (s.w, s.h) != (w, h)) {
            self.drop_sub(key);
        }
        if !self.subs.contains_key(key) {
            let (page, nodes) = raea_sys_page::open_doc(self.app.world_mut(), w, h, self.fonts.clone());
            self.subs.insert(key.to_string(), Sub { page, w, h, tree: Tree::default(), nodes });
            // its Image is made by the next update
            self.app.update();
        }
        let sub = self.subs.get_mut(key).unwrap();
        let (ops, _gone) = sub.tree.diff(want, order);
        if !ops.is_empty() {
            let mut docs = self.app.world_mut().non_send_mut::<Docs>();
            ops::apply(docs.0.get_mut(&sub.page).expect("the screen's document"), &mut sub.nodes, ops);
        }
        Ok(())
    }

    /// The <Screen>s not in `keep` gone.
    pub fn retain_subs(&mut self, keep: &[String]) {
        let gone: Vec<String> = self.subs.keys().filter(|k| !keep.contains(k)).cloned().collect();
        for k in gone {
            self.drop_sub(&k);
        }
    }

    pub(crate) fn drop_sub(&mut self, key: &str) {
        if let Some(s) = self.subs.remove(key) {
            self.app.world_mut().non_send_mut::<Docs>().0.remove(&s.page);
            self.app.world_mut().entity_mut(s.page).despawn();
        }
    }

    /// Each video element's frame at its time, decoded and set as its picture (`place_picture`). A frame already
    /// shown isn't decoded again.
    pub fn place_videos(&mut self, videos: Vec<raea_core_wire::tree::VideoAt>) -> R<()> {
        let trace = std::env::var_os("RAEA_PAGE_VIDEOS").is_some();
        // a framed video crops by its box, which a new element doesn't have until the document is resolved: resolve it
        // now, at this frame's time (the draw resolves again at the same time), so the first frame is framed too
        if videos.iter().any(|v| v.fit.is_some() && self.box_of(v.id).is_none_or(|b| b[2] < 1.0 || b[3] < 1.0)) {
            if let Some((now, _, _)) = self.clock {
                let mut docs = self.app.world_mut().non_send_mut::<Docs>();
                if let Some(doc) = docs.0.get_mut(&self.page) {
                    doc.resolve(now);
                }
            }
        }
        for v in videos {
            match self.videos.frame(&v)? {
                Some(f) => {
                    // RAEA_PAGE_VIDEOS=1: each video element's frame as placed (its mean, a black one stands out), or kept
                    if trace {
                        let mean = f.rgba.chunks_exact(4).map(|p| p[0] as u64 + p[1] as u64 + p[2] as u64).sum::<u64>() as f64 / (f.rgba.len() as f64 / 4.0 * 3.0);
                        eprintln!("raea-page: video {} {} at {:.3} s: {}x{}, mean {mean:.1}{}", v.id, v.file.display(), v.at, f.width, f.height, if self.locate(v.id).is_none() { " (no element)" } else { "" });
                    }
                    let crop = v.fit.and_then(|fit| self.box_of(v.id).and_then(|b| fit_rect(b[2], b[3], f.width, f.height, &fit)));
                    self.place_frame_in(v.id, f.width, f.height, &f.rgba, crop);
                    self.videos.shows(v);
                }
                None if trace => eprintln!("raea-page: video {} {} at {:.3} s: kept", v.id, v.file.display(), v.at),
                None => {}
            }
        }
        Ok(())
    }

    /// Element `id`'s picture is `rgba` (straight, `w` x `h`: a video's frame) through a texture of its own on the page's
    /// device, written again each frame it changes. Its placeholder keeps one slot in Vello's image atlas: a new blob a
    /// frame (place_picture) leaves the last frame's in the atlas until it's two draws old, and a page of many 1080p
    /// videos (P26 reel/v/platform/cli's strip, 19 of them) then fills its 8192 x 8192 and draws some of them blank.
    pub fn place_frame(&mut self, id: u64, w: u32, h: u32, rgba: &[u8]) {
        self.place_frame_in(id, w, h, rgba, None);
    }

    /// `place_frame`, the element showing only `crop` ([x, y, w, h] in the frame's pixels: a framed video's
    /// `fit_rect`); None: all of it.
    pub fn place_frame_in(&mut self, id: u64, w: u32, h: u32, rgba: &[u8], crop: Option<[u32; 4]>) {
        let Some(render) = self.app.get_sub_app(bevy::render::RenderApp) else { return };
        let device = render.world().resource::<bevy::render::renderer::RenderDevice>();
        let queue = render.world().resource::<bevy::render::renderer::RenderQueue>();
        let tex = match self.frames.get(&id) {
            Some(t) if (t.width(), t.height()) == (w, h) => t.clone(),
            _ => device.create_texture(&bevy::render::render_resource::TextureDescriptor {
                label: Some("raea page video frame"),
                size: bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: bevy::render::render_resource::TextureDimension::D2,
                format: bevy::render::render_resource::TextureFormat::Rgba8Unorm,
                usage: bevy::render::render_resource::TextureUsages::COPY_SRC | bevy::render::render_resource::TextureUsages::COPY_DST | bevy::render::render_resource::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }),
        };
        queue.write_texture(
            tex.as_image_copy(),
            rgba,
            bevy::render::render_resource::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(w * 4), rows_per_image: Some(h) },
            bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        self.frames.insert(id, tex.clone());
        let [x, y, cw, ch] = crop.unwrap_or([0, 0, w, h]);
        self.place_texture_at(id, &tex, (x, y), cw, ch, false);
    }

    /// Element `id`'s picture: `rgba` straight, `w` x `h` (a video's frame, a layered 3D scene's), uploaded by Vello
    /// when the page is drawn, and drawn by blitz-paint as an image's (object-fit and object-position hold).
    pub fn place_picture(&mut self, id: u64, w: u32, h: u32, rgba: Vec<u8>) {
        self.videos.forget(id);
        self.set_image(id, w, h, vello::peniko::Blob::new(Arc::new(rgba)));
        if let Some(old) = self.views.remove(&id) {
            raea_sys_page::view(&mut self.app, old, None);
        }
    }

    /// Element `id` without the picture `place_picture` or `place_texture_at` gave it: drawn as its own element again (a
    /// camera's holder once the camera has left it, which kept the view an earlier frame set).
    pub fn unplace(&mut self, id: u64) {
        self.videos.forget(id);
        self.frames.remove(&id);
        if let Some(old) = self.views.remove(&id) {
            raea_sys_page::view(&mut self.app, old, None);
        }
        let Some((page, n)) = self.locate(id) else { return };
        let mut docs = self.app.world_mut().non_send_mut::<Docs>();
        let Some(node) = docs.0.get_mut(&page).and_then(|doc| doc.get_node_mut(n)) else { return };
        if let Some(el) = node.element_data_mut() {
            el.special_data = blitz_dom::node::SpecialElementData::None;
        }
        node.clear_layout_cache();
        node.insert_damage(raea_sys_page::ALL_DAMAGE);
        node.mark_ancestors_dirty();
    }

    /// Element `id`'s picture is the top left `w` x `h` of `src` (a texture on the page's device: live play's 3D under
    /// the page or in its layered slot), copied by Vello on the GPU each frame the page is drawn; no readback (H7).
    pub fn place_texture(&mut self, id: u64, src: &bevy::render::render_resource::Texture, w: u32, h: u32) {
        self.place_texture_at(id, src, (0, 0), w, h, false)
    }

    /// `place_texture` for the box of `src` at `origin` (a camera's view in its element's box: docs/sdk.feature S3a),
    /// and for a camera's picture, premultiplied in its stored values, made straight on the way as raea-runtime-js's page
    /// makes it (B5: `place_premultiplied`). The box is kept inside the texture.
    pub fn place_texture_at(&mut self, id: u64, src: &bevy::render::render_resource::Texture, origin: (u32, u32), w: u32, h: u32, premultiplied: bool) {
        let (x, y) = (origin.0.min(src.width().saturating_sub(1)), origin.1.min(src.height().saturating_sub(1)));
        let (w, h) = (w.min(src.width() - x).max(1), h.min(src.height() - y).max(1));
        let image = match self.views.get(&id) {
            Some(i) if (i.width, i.height) == (w, h) => i.clone(),
            _ => {
                // the element holds a placeholder of the texture's size, whose blob is Vello's key for the override
                let blob = vello::peniko::Blob::new(Arc::new(vec![0u8; (w * h * 4) as usize]));
                self.set_image(id, w, h, blob.clone());
                vello::peniko::ImageData { data: blob, format: vello::peniko::ImageFormat::Rgba8, alpha_type: vello::peniko::ImageAlphaType::Alpha, width: w, height: h }
            }
        };
        self.views.insert(id, image.clone());
        raea_sys_page::view_at(&mut self.app, image, Some(raea_sys_page::Placed { texture: (**src).clone(), origin: (x, y), premultiplied }));
    }

    /// Element `id` drawn as an image: `data` (RGBA8) as its raster, its layout made again for the picture's size.
    pub(crate) fn set_image(&mut self, id: u64, w: u32, h: u32, data: vello::peniko::Blob<u8>) {
        let Some((page, n)) = self.locate(id) else { return };
        let mut docs = self.app.world_mut().non_send_mut::<Docs>();
        if let Some(doc) = docs.0.get_mut(&page) {
            picture(doc, n, w, h, data);
        }
    }

    /// Each <Screen>'s picture as last drawn, on the page's device.
    pub fn sub_textures(&self) -> Vec<(String, bevy::render::render_resource::Texture, u32, u32)> {
        self.subs.iter().filter_map(|(k, s)| Some((k.clone(), self.gpu(&self.image_of(s.page)?)?, s.w, s.h))).collect()
    }
}
