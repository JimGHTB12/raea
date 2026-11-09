use super::*;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Fullscreen, Window, WindowId};

pub(crate) struct Player {
    pub(crate) game: Game,
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) stage: Option<raea_gpu_device::present::Present>,
    pub(crate) live: Option<Live>,
    pub(crate) keys: Vec<String>,
    pub(crate) pointer: Option<(f64, f64, bool)>,
    /// the wheel turned since the last frame (CSS px, down positive): an app screen's scroll (runtime/page/src/screen.rs)
    pub(crate) wheel: f64,
    pub(crate) mods: ModifiersState,
    pub(crate) start: Option<Instant>,
    pub(crate) frame: i64,
    pub(crate) error: Option<String>,
    /// when the window was asked for, and when its first paint (the splash) went up (lightspeed F5)
    pub(crate) launched: Instant,
}

pub(crate) fn key_name(k: &Key) -> Option<String> {
    Some(match k {
        Key::Named(NamedKey::ArrowLeft) => "ArrowLeft".into(),
        Key::Named(NamedKey::ArrowRight) => "ArrowRight".into(),
        Key::Named(NamedKey::ArrowUp) => "ArrowUp".into(),
        Key::Named(NamedKey::ArrowDown) => "ArrowDown".into(),
        Key::Named(NamedKey::Space) => " ".into(),
        Key::Named(NamedKey::Enter) => "Enter".into(),
        Key::Named(NamedKey::Escape) => "Escape".into(),
        Key::Named(NamedKey::Shift) => "Shift".into(),
        Key::Named(NamedKey::Tab) => "Tab".into(),
        Key::Named(NamedKey::Backspace) => "Backspace".into(),
        Key::Character(c) => c.to_string(),
        _ => return None,
    })
}

impl Player {
    pub(crate) fn frame(&mut self) -> R<()> {
        // the game opened behind the splash, on the first redraw: a 3D game's first launch compiles its pipelines here
        // (seconds while the system's shader cache is cold). Its clock starts at the next frame, from nothing: keys and
        // pointer from the splash's time are let go, so frame 0 is a render's frame 0
        if self.live.is_none() {
            if self.stage.is_none() {
                let Some(window) = self.window.clone() else { return Ok(()) };
                let px = window.inner_size();
                self.stage = Some(raea_gpu_device::present::window(window, px.width, px.height, true)?);
            }
            let Some(stage) = self.stage.as_ref() else { return Ok(()) };
            let (w, h) = (stage.w, stage.h);
            let mut live = self.game.open((w, h))?;
            if self.game.has_3d {
                live.scene_3d();
            }
            live.open_mixer();
            self.live = Some(live);
            (self.start, self.frame) = (None, -1);
            self.keys.clear();
            self.pointer = None;
            if probing() {
                eprintln!("raea game: playable {:.0} ms after launch", self.launched.elapsed().as_secs_f64() * 1e3);
            }
            return Ok(());
        }
        let (Some(live), Some(stage)) = (self.live.as_mut(), self.stage.as_mut()) else { return Ok(()) };
        if self.start.is_none() {
            // input winit queued while the open blocked is delivered before this frame: let go of it too
            self.keys.clear();
            self.pointer = None;
        }
        let start = *self.start.get_or_insert_with(Instant::now);
        let elapsed = start.elapsed().as_secs_f64();
        // every frame due by now, each with this moment's input (a few at most when behind; past that the clock jumps)
        let due = (elapsed * live.fps).floor() as i64;
        let mut n = 0;
        while self.frame < due && n < 4 {
            self.frame += 1;
            let lag = elapsed - self.frame as f64 / live.fps;
            let keys = raea_runtime_live::keys_with_wheel(&self.keys, &mut self.wheel);
            live.step_at(self.frame, &keys, self.pointer, lag)?;
            n += 1;
        }
        if self.frame < due {
            self.frame = due;
        }
        // presented every refresh (vsync paces the loop), a new frame or not
        let _ = n;
        live.present(stage, true)?;
        Ok(())
    }

    pub(crate) fn toggle_fullscreen(&self) {
        if let Some(w) = &self.window {
            w.set_fullscreen(if w.fullscreen().is_some() { None } else { Some(Fullscreen::Borderless(None)) });
        }
    }
}

impl ApplicationHandler for Player {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        // the composition's shape, as large as fits in most of the screen
        let (cw, ch) = (self.game.w as f64, self.game.h as f64);
        let (sw, sh) = el.primary_monitor().map(|m| { let s = m.size().to_logical::<f64>(m.scale_factor()); (s.width, s.height) }).unwrap_or((1440.0, 900.0));
        let k = (sw * 0.85 / cw).min(sh * 0.85 / ch).min(1.0);
        let attrs = Window::default_attributes().with_title(&self.game.name).with_inner_size(winit::dpi::LogicalSize::new((cw * k).round(), (ch * k).round()));
        // RAEA_PLAYER_PROBE=1: the launch measured with no window on anyone's screen (hidden, never active), its first
        // paint and playable times printed, and the player gone 2 s after it's playable
        let attrs = if probing() { attrs.with_visible(false).with_active(false) } else { attrs };
        let window = match el.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                self.error = Some(format!("window: {e}"));
                el.exit();
                return;
            }
        };
        // the first paint at once, with nothing of ours compiled (lightspeed F5): the window's own background, dark, with
        // the game's name as its title; the stage (whose pipelines compile on a first launch) and the game open on the
        // first redraw, so the window never sits empty however long that takes
        raea_gpu_device::present::window_background(&*window, [0.05, 0.05, 0.063]);
        if probing() {
            eprintln!("raea game: first paint {:.0} ms after launch", self.launched.elapsed().as_secs_f64() * 1e3);
        }
        window.request_redraw();
        self.window = Some(window);
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(px) => {
                if let Some(s) = self.stage.as_mut() {
                    let _ = s.resize(px.width, px.height);
                }
                if let Some(l) = self.live.as_mut() {
                    l.resize(px.width, px.height);
                }
            }
            WindowEvent::ModifiersChanged(m) => self.mods = m.state(),
            WindowEvent::Focused(false) => self.keys.clear(),
            WindowEvent::KeyboardInput { event, .. } => {
                let cmd = self.mods.super_key();
                if event.state == ElementState::Pressed && !event.repeat {
                    let ch = match &event.logical_key {
                        Key::Character(c) => c.to_lowercase(),
                        _ => String::new(),
                    };
                    if cmd && (ch == "q" || ch == "w") {
                        el.exit();
                        return;
                    }
                    if matches!(event.logical_key, Key::Named(NamedKey::F11)) || (cmd && ch == "f") {
                        self.toggle_fullscreen();
                        return;
                    }
                }
                // a shortcut held with Cmd isn't the game's
                if cmd {
                    return;
                }
                if let Some(k) = key_name(&event.logical_key) {
                    match event.state {
                        ElementState::Pressed if !self.keys.contains(&k) => self.keys.push(k),
                        ElementState::Released => self.keys.retain(|x| *x != k),
                        _ => {}
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let (Some(w), Some(l)) = (self.window.as_ref(), self.live.as_ref()) {
                    // the page is shown whole in the window, centred (object-fit contain)
                    let s = w.inner_size();
                    let k = (s.width as f64 / l.w as f64).min(s.height as f64 / l.h as f64);
                    let (ox, oy) = ((s.width as f64 - l.w as f64 * k) / 2.0, (s.height as f64 - l.h as f64 * k) / 2.0);
                    let down = self.pointer.is_some_and(|p| p.2);
                    self.pointer = Some(((position.x - ox) / k, (position.y - oy) / k, down));
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scale = self.window.as_ref().map(|w| w.scale_factor()).unwrap_or(1.0);
                self.wheel += raea_runtime_live::wheel_px(&delta, scale);
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                if let Some(p) = self.pointer.as_mut() {
                    p.2 = state == ElementState::Pressed;
                }
            }
            WindowEvent::RedrawRequested => {
                if probing() && self.start.is_some_and(|s| s.elapsed().as_secs_f64() > 2.0) {
                    el.exit();
                    return;
                }
                if let Err(e) = self.frame() {
                    self.error = Some(e);
                    el.exit();
                }
                if let Some(w) = self.window.as_ref() {
                    w.request_redraw();
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn probing() -> bool {
    raea_io_home::env_on("RAEA_PLAYER_PROBE")
}

pub(crate) fn windowed(game: Game) -> R<String> {
    let el = EventLoop::new().map_err(|e| format!("event loop: {e}"))?;
    el.set_control_flow(ControlFlow::Poll);
    let mut p = Player { game, window: None, stage: None, live: None, keys: Vec::new(), pointer: None, wheel: 0.0, mods: ModifiersState::empty(), start: None, frame: -1, error: None, launched: Instant::now() };
    el.run_app(&mut p).map_err(|e| format!("event loop: {e}"))?;
    if let Some(l) = p.live.as_mut() {
        l.silence();
    }
    match p.error {
        Some(e) => Err(e),
        None => Ok(String::new()),
    }
}
