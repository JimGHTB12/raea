//! A page's UiMaterial nodes (sdk/render's MaterialNode) on raea-render's engine: each frame, before the 3D draws, the
//! nodes the sync met are handed to raea-sys-shader's ui.rs at their laid-out size, which draws each one's shader into
//! a texture of its own; after the draw, each texture is the picture of its element (on the GPU, or read back for the
//! hosts that read frames back). The page then draws again with them.

use super::*;

impl BevyPage {
    /// The UiMaterial elements the last sync met, each with its spec and its box as last laid out (its transforms not
    /// applied: the page draws the picture through them), in whole px; one not laid out yet is left out.
    fn ui_material_boxes(&mut self) -> R<Vec<raea_sys_shader::ui::UiMat>> {
        let mats = self.ui_mats.clone();
        let mut out = Vec::new();
        for (id, spec) in mats {
            let Some(b) = self.page.box_of(id) else { continue };
            let (w, h) = (b[2].round() as u32, b[3].round() as u32);
            if w == 0 || h == 0 {
                continue;
            }
            let mut spec: Value = serde_json::from_str(&spec).map_err(|e| format!("a MaterialNode's spec: {e}"))?;
            // its files (staticFile's /media/ URLs) found from the composition's folder, as a 3D part's are
            let file = |v: &mut Value| {
                if let Some(s) = v.as_str() {
                    *v = Value::String(raea_core_scene::media_path(&self.dir, &raea_core_scene::decoded(s).replacen("media/", "/media/", 1)).to_string_lossy().into_owned());
                }
            };
            if let Some(f) = spec.get_mut("fragment") {
                file(f);
            }
            if let Some(t) = spec.get_mut("textures").and_then(Value::as_object_mut) {
                t.values_mut().for_each(file);
            }
            out.push(raea_sys_shader::ui::UiMat { id, spec, size: (w, h) });
        }
        Ok(out)
    }

    /// Before the engine (`world`, raea-render's) draws: this frame's UiMaterial nodes handed to it. Whether the page has
    /// any (and so needs `place_ui_materials` after the draw).
    pub fn set_ui_materials(&mut self, world: &mut World) -> R<bool> {
        let mats = self.ui_material_boxes()?;
        raea_sys_shader::ui::set(world, &mats)?;
        Ok(!mats.is_empty())
    }

    /// After the engine drew: each UiMaterial node's texture its element's picture, on the GPU (premultiplied, as
    /// Bevy's blend into a clear target leaves it).
    pub fn place_ui_materials(&mut self, world: &World) {
        for (id, t, w, h) in raea_sys_shader::ui::textures(world) {
            self.page.place_texture_at(id, &t.into(), (0, 0), w, h, true);
        }
    }

    /// `place_ui_materials` for the hosts that read frames back: each picture read back, straight.
    pub fn read_ui_materials(&mut self, world: &World) -> R<()> {
        for (id, w, h, rgba) in raea_sys_shader::ui::read(world)? {
            self.page.place_picture(id, w, h, rgba);
        }
        Ok(())
    }
}
