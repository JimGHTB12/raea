//! The check's frames drawn by Bevy (`--headless app.js --draw bevy`): the session's Runtime drawn into a Bevy Image
//! on Bevy's device (raea-rune-bevy's draw_runtime, what a layer does each frame), then read back. Against shot.rs's
//! Shooter (Rune's own headless device), the same app and script give the same PNGs: T6c's parity.

use std::path::Path;

use bevy::prelude::*;
use rune::Runtime;
use rune::render::Renderer;

use crate::page::Page;

pub struct BevyShooter {
    app: App,
    image: Handle<Image>,
    size: (u32, u32),
    renderer: Option<Renderer>,
}

impl BevyShooter {
    /// A headless Bevy app and a `w` x `h` Image on its device.
    pub fn new(w: u32, h: u32) -> BevyShooter {
        let mut app = super::headless();
        let image = raea_rune_bevy::image(&mut app.world_mut().resource_mut::<Assets<Image>>(), (w, h));
        // the Image reaches the GPU in the render world's first prepare
        app.update();
        BevyShooter { app, image, size: (w, h), renderer: None }
    }

    /// The runtime's last frame drawn into the Image: tightly packed sRGB RGBA8, `w * h * 4` bytes.
    pub fn pixels(&mut self, rt: &mut Runtime<Page>) -> Result<Vec<u8>, String> {
        let (device, queue) = raea_rune_bevy::readback::device(&self.app);
        let view = raea_rune_bevy::readback::view(&self.app, &self.image).ok_or("bevy: the Image isn't on the GPU")?;
        // a browser's canvas: white under a page that paints nothing there, as shot.rs
        raea_rune_bevy::draw_runtime(&mut self.renderer, rt, &device, &queue, &view, self.size, [1.0, 1.0, 1.0, 1.0]);
        raea_rune_bevy::readback::read(&self.app, &self.image).ok_or_else(|| "bevy: no Image to read".into())
    }

    /// The runtime's last frame as a PNG at `path`.
    pub fn png(&mut self, rt: &mut Runtime<Page>, path: &Path) -> Result<(), String> {
        let px = self.pixels(rt)?;
        crate::shot::write_png(path, self.size.0, self.size.1, &px)
    }
}
