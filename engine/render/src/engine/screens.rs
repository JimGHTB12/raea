//! The Engine's screens (raea-sys-screen): their textures for the host, the screen under a point, a page's picture.

use super::*;

impl Engine {
    /// Every screen's texture on the paint device by name, with its size (raea-sys-screen's screen_targets).
    pub fn screen_targets(&self) -> Vec<(String, wgpu::Texture, u32, u32)> {
        screen::screen_targets(&self.app)
    }

    /// The screen under a point of the frame and where the point lands on its picture (screen_at).
    pub fn screen_at(&mut self, nx: f64, ny: f64) -> Option<(String, f64, f64)> {
        screen::screen_at(&mut self.app, nx, ny)
    }

    /// Screen `name`'s picture for the frame about to draw, copied from a texture on the paint device (copy_screen).
    pub fn copy_screen(&self, name: &str, src: &wgpu::Texture) -> bool {
        screen::copy_screen(&self.app, name, src)
    }

    /// Screen `name`'s picture for the next frame drawn (set_screen_picture).
    pub fn set_screen_picture(&mut self, name: &str, w: u32, h: u32, rgba: Vec<u8>) {
        screen::screen_page::set_screen_picture(&mut self.app, name, w, h, rgba)
    }
}
