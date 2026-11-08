//! Pipelined updates: Bevy's PipelinedRenderingPlugin (which raea-render's engine turns off) done inside the engine's
//! own update. An update renders the frame the update before extracted, on a thread of its own, while the main world
//! runs this frame's schedule; then this frame is extracted into the render world, which waits there, home in the App,
//! for the next update (or `flush`). A frame's readback so comes back an update later, which the engine's `frames` and
//! `frame` take as they come, by its tag; and everything that reads the render world between updates (the target, the
//! device, the GPU pool) reads it as before. A live engine's `draw` renders what's pending, then its own frame in an
//! update of its own, so the frame it hands on is drawn as it returns; its catch-up steps (a plugin's frames up to it)
//! are pipelined. Off with RAEA_SERIAL (which times each frame's GPU work alone).

use bevy::app::{App, SubApp};
use bevy::render::RenderApp;
use std::sync::mpsc::{Receiver, Sender, channel};

/// Whether an engine's updates are pipelined, whether its render world holds a frame not yet rendered, and the thread
/// that renders it: one for the engine's life, so what the render world keeps per thread (CoreText's faces, which
/// Vello's glyph tops read) is kept from frame to frame.
#[derive(Default)]
pub struct Pipe {
    pub on: bool,
    pending: bool,
    worker: Option<(Sender<SubApp>, Receiver<std::thread::Result<SubApp>>)>,
}

impl Pipe {
    /// On unless RAEA_SERIAL times updates one by one, or RAEA_PIPELINE=0 asks for every update in one piece.
    pub fn new() -> Pipe {
        Pipe { on: std::env::var_os("RAEA_SERIAL").is_none() && std::env::var("RAEA_PIPELINE").as_deref() != Ok("0"), ..Default::default() }
    }

    /// One update: pipelined when `on`, `App::update` otherwise.
    pub fn update(&mut self, app: &mut App) {
        if !self.on || cfg!(target_arch = "wasm32") {
            self.flush(app);
            return app.update();
        }
        let mut render = app.remove_sub_app(RenderApp).expect("the engine's render app");
        if std::mem::take(&mut self.pending) {
            let (to, back) = self.worker.get_or_insert_with(|| {
                let ((to, from_main), (to_main, back)) = (channel::<SubApp>(), channel());
                std::thread::Builder::new()
                    .name("raea render".into())
                    .spawn(move || {
                        for mut r in from_main {
                            let _ = to_main.send(std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                                r.update();
                                r
                            })));
                        }
                    })
                    .expect("a render thread");
                (to, back)
            });
            to.send(render).expect("the render thread");
            app.main_mut().run_default_schedule();
            render = back.recv().expect("the render thread").unwrap_or_else(|p| std::panic::resume_unwind(p));
        } else {
            app.main_mut().run_default_schedule();
        }
        // as SubApps::update does after the main schedule: extract, then the main world's trackers cleared
        render.extract(app.world_mut());
        app.world_mut().clear_trackers();
        app.insert_sub_app(RenderApp, render);
        self.pending = true;
    }

    /// The frame extracted and not yet rendered, rendered now: before a plain `App::update` (a second extract over an
    /// unrendered one would draw both) and wherever a caller needs the last update's frame on the GPU.
    pub fn flush(&mut self, app: &mut App) {
        if std::mem::take(&mut self.pending) {
            app.sub_app_mut(RenderApp).update();
        }
    }
}

/// Wait for every piece of work the App's GPU has been given.
pub fn wait_gpu(app: &App) {
    let dev = app.sub_app(RenderApp).world().resource::<bevy::render::renderer::RenderDevice>().clone();
    let _ = dev.wgpu_device().poll(bevy::render::render_resource::PollType::wait_indefinitely());
}
