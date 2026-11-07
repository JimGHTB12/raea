//! A layer's tree: a Rune `Runtime` of any state, its `Renderer` (made on the device it's first drawn with), and its
//! viewport in points and scale. [`AnyTree`] is what the plugin calls, so layers of different states share [`Trees`].

use rune::Runtime;
use rune::render::Renderer;

use crate::input::Input;

/// What the plugin does with a layer's tree, whatever its state.
pub trait AnyTree {
    /// a frame `dt` seconds on: the tree's poll, its view if state changed, its springs and layout
    fn step(&mut self, dt: f64);
    /// the last frame drawn into `view` (a layer's Image, `pixels()` big) on `device` and `queue`, cleared to `clear`
    fn draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, clear: [f32; 4]);
    /// pointer, wheel or keys, in the tree's points
    fn input(&mut self, ev: Input);
    /// the viewport in points and its scale
    fn viewport(&self) -> ((f32, f32), f32);
    /// a new viewport (a window resized, a HUD laid out at another size)
    fn resize(&mut self, viewport: (f32, f32), scale: f32);
    /// the Image's size: the viewport times its scale, in whole pixels
    fn pixels(&self) -> (u32, u32) {
        let ((w, h), s) = self.viewport();
        (((w * s).round() as u32).max(1), ((h * s).round() as u32).max(1))
    }
    /// the tree's state, for a caller that knows its type (`downcast_ref` on it)
    fn state(&self) -> &dyn std::any::Any;
    fn state_mut(&mut self) -> &mut dyn std::any::Any;
}

/// A Rune Runtime as a layer's tree.
pub struct Tree<S: 'static> {
    pub rt: Runtime<S>,
    scale: f32,
    renderer: Option<Renderer>,
    /// run after each frame with the runtime (a host reading the frame's layout, a video's encoder)
    pub after_frame: Option<Box<dyn FnMut(&mut Runtime<S>)>>,
}

impl<S: 'static> Tree<S> {
    /// `rt` at `scale` pixels a point. Its viewport is the one it was made with.
    pub fn new(mut rt: Runtime<S>, scale: f32) -> Tree<S> {
        let viewport = rt.viewport;
        rt.set_viewport(viewport, scale);
        Tree { rt, scale, renderer: None, after_frame: None }
    }
}

impl<S: 'static> AnyTree for Tree<S> {
    fn step(&mut self, dt: f64) {
        self.rt.frame(dt);
        if let Some(f) = self.after_frame.as_mut() {
            f(&mut self.rt);
        }
    }

    fn draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, clear: [f32; 4]) {
        let px = self.pixels();
        draw(&mut self.renderer, &mut self.rt, device, queue, view, px, clear);
    }

    fn input(&mut self, ev: Input) {
        crate::input::apply(&mut self.rt, ev);
    }

    fn viewport(&self) -> ((f32, f32), f32) {
        (self.rt.viewport, self.scale)
    }

    fn resize(&mut self, viewport: (f32, f32), scale: f32) {
        self.scale = scale;
        self.rt.set_viewport(viewport, scale);
    }

    fn state(&self) -> &dyn std::any::Any {
        &self.rt.state
    }

    fn state_mut(&mut self) -> &mut dyn std::any::Any {
        &mut self.rt.state
    }
}

/// `rt`'s last frame drawn into `view` (`px` big, in [`crate::FORMAT`]) on `device` and `queue`, cleared to `clear`, by
/// `renderer` (made on `device` the first time): what a layer does each frame, for a host that keeps its own Runtime
/// (raea-rune-host's headless check draws its session this way to show it draws as Rune's own renderer does).
pub fn draw<S>(renderer: &mut Option<Renderer>, rt: &mut Runtime<S>, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, px: (u32, u32), clear: [f32; 4]) {
    let renderer = renderer.get_or_insert_with(|| Renderer::new(device, crate::FORMAT));
    renderer.upload_images(device, queue, &mut rt.images);
    renderer.prepare(device, queue, &mut rt.text.atlas, &rt.instances, px);
    renderer.prepare_surfaces(device, queue, &rt.surfaces, px);
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("raea-rune-bevy layer") });
    renderer.draw(&mut enc, view, clear);
    renderer.submit(queue, enc);
}
