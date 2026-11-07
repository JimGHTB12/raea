//! A frame drawn offscreen by Rune's own renderer and written as a PNG (R4b): what the window would show, at the
//! session's viewport and scale 1, without a window.

use std::path::Path;

use rune::Runtime;
use rune::render::{Gpu, OFFSCREEN_FORMAT, Offscreen, Renderer};

use crate::page::Page;

pub struct Shooter {
    gpu: Gpu,
    renderer: Renderer,
    target: Offscreen,
}

impl Shooter {
    /// A headless GPU and a `w` x `h` target.
    pub fn new(w: u32, h: u32) -> Shooter {
        let gpu = Gpu::headless();
        let renderer = Renderer::new(&gpu.device, OFFSCREEN_FORMAT);
        let target = Offscreen::with_format(&gpu.device, w, h, OFFSCREEN_FORMAT);
        Shooter { gpu, renderer, target }
    }

    /// The runtime's last frame drawn: tightly packed sRGB RGBA8, `w * h * 4` bytes.
    pub fn pixels(&mut self, rt: &mut Runtime<Page>) -> Vec<u8> {
        let (w, h) = (self.target.width, self.target.height);
        self.renderer.upload_images(&self.gpu.device, &self.gpu.queue, &mut rt.images);
        self.renderer.prepare(&self.gpu.device, &self.gpu.queue, &mut rt.text.atlas, &rt.instances, (w, h));
        let mut enc = self.gpu.device.create_command_encoder(&Default::default());
        // a browser's canvas: white under a page that paints nothing there (Chrome is R5's reference)
        self.renderer.draw(&mut enc, &self.target.view, [1.0, 1.0, 1.0, 1.0]);
        self.target.copy(&mut enc);
        self.renderer.submit(&self.gpu.queue, enc);
        self.target.read(&self.gpu.device)
    }

    /// The runtime's last frame as a PNG at `path`.
    pub fn png(&mut self, rt: &mut Runtime<Page>, path: &Path) -> Result<(), String> {
        let px = self.pixels(rt);
        write_png(path, self.target.width, self.target.height, &px)
    }
}

/// RGBA8 pixels as a PNG file.
pub fn write_png(path: &Path, w: u32, h: u32, px: &[u8]) -> Result<(), String> {
    let f = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let mut wr = enc.write_header().map_err(|e| format!("{}: {e}", path.display()))?;
    wr.write_image_data(px).map_err(|e| format!("{}: {e}", path.display()))
}
