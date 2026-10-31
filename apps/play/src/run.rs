use super::*;

impl Run {
    /// The game's window settings (raea/window), applied when they change (docs/bevy-ts.feature G19).
    pub(crate) fn apply_window(&mut self) {
        let (version, set) = raea_runtime_plugins::window_settings();
        if version == self.window_version {
            return;
        }
        self.window_version = version;
        let (Some(w), Some(s)) = (self.window.as_ref(), set) else { return };
        if let Some(t) = s.get("title").and_then(|t| t.as_str()) {
            w.set_title(t);
        }
        if let Some(a) = s.get("size").and_then(|a| a.as_array()) {
            if let (Some(x), Some(y)) = (a.first().and_then(|x| x.as_f64()), a.get(1).and_then(|y| y.as_f64())) {
                let _ = w.request_inner_size(winit::dpi::PhysicalSize::new(x as u32, y as u32));
            }
        }
        if let Some(r) = s.get("resizable").and_then(|r| r.as_bool()) {
            w.set_resizable(r);
        }
        if let Some(d) = s.get("decorations").and_then(|d| d.as_bool()) {
            w.set_decorations(d);
        }
        if let Some(f) = s.get("fullscreen").and_then(|f| f.as_bool()) {
            w.set_fullscreen(f.then(|| winit::window::Fullscreen::Borderless(None)));
        }
        if let Some(c) = s.get("cursor").and_then(|c| c.as_str()) {
            use winit::window::CursorGrabMode;
            w.set_cursor_visible(c == "visible" || c == "confined");
            let grab = match c {
                "locked" => w.set_cursor_grab(CursorGrabMode::Locked).or_else(|_| w.set_cursor_grab(CursorGrabMode::Confined)),
                "confined" => w.set_cursor_grab(CursorGrabMode::Confined).or_else(|_| w.set_cursor_grab(CursorGrabMode::Locked)),
                _ => w.set_cursor_grab(CursorGrabMode::None),
            };
            if let Err(e) = grab {
                eprintln!("raea play: cursor {c}: {e}");
            }
        }
    }

    pub(crate) fn frame(&mut self, el: &ActiveEventLoop) -> R<()> {
        self.apply_window();
        let (Some(live), Some(stage)) = (self.live.as_mut(), self.stage.as_mut()) else { return Ok(()) };
        let start = *self.start.get_or_insert_with(Instant::now);
        let t0 = Instant::now();
        let elapsed = start.elapsed().as_secs_f64();
        if self.seconds.is_some_and(|s| elapsed >= s) {
            el.exit();
            return Ok(());
        }
        if self.autoplay {
            self.keys = vec![if (elapsed * 1.5) as i64 % 2 == 0 { "ArrowLeft".into() } else { "ArrowRight".into() }];
        }
        // the composition's clock: every frame due by now, each with this moment's input (a few at most when behind)
        let due = (elapsed * live.fps).floor() as i64;
        let mut n = 0;
        while self.frame < due && n < 4 {
            self.frame += 1;
            let keys = raea_runtime_live::keys_with_wheel(&self.keys, &mut self.wheel);
            live.step(self.frame, &keys, self.pointer)?;
            self.stepped += 1;
            n += 1;
        }
        if self.frame < due {
            self.frame = due;
        }
        live.present(stage, true)?;
        let now = Instant::now();
        let work = (now - t0).as_secs_f64() * 1e3;
        first_frame();
        self.work.push(work);
        let gap = self.last_present.map(|l| (now - l).as_secs_f64() * 1e3);
        if let Some(g) = gap {
            self.gaps.push(g);
        }
        let budget = 1000.0 / live.fps.max(1.0);
        let frame = self.frame;
        raea_io_trace::frame(work, gap.is_some_and(|g| g > budget * 1.5), budget, || serde_json::json!({"frame": frame, "gap_ms": gap, "stepped": n}));
        self.last_present = Some(now);
        if self.last_rss.is_none_or(|t| t.elapsed() > Duration::from_secs(1)) {
            let (rss, reusable) = vm_kb();
            self.rss.push(rss);
            self.reusable.push(reusable);
            self.footprint.push(raea_io_trace::footprint_mb());
            self.last_rss = Some(Instant::now());
        }
        Ok(())
    }

    pub(crate) fn report(&self) -> serde_json::Value {
        let pct = |v: &[f64], p: f64| {
            if v.is_empty() {
                return 0.0;
            }
            let mut s = v.to_vec();
            s.sort_by(|a, b| a.total_cmp(b));
            s[((s.len() - 1) as f64 * p).round() as usize]
        };
        let fps = self.live.as_ref().map(|l| l.fps).unwrap_or(0.0);
        let refresh = self.window.as_ref().and_then(|w| w.current_monitor()).and_then(|m| m.refresh_rate_millihertz()).map(|m| m as f64 / 1000.0).unwrap_or(60.0);
        // a frame is due every display refresh, or every composition frame when the composition is slower (a 30 fps edit
        // on a 60 Hz display presents every other refresh by design)
        let budget = 1000.0 / if fps > 0.0 { refresh.min(fps) } else { refresh };
        let missed = self.gaps.iter().filter(|g| **g > budget * 1.5).count();
        // ZeroHeap Z1 (RAEA_ZEROHEAP=1): Rust allocations a frame after the first second's warmup, means over the rest
        let after = self.allocs.get(fps.max(1.0) as usize..).unwrap_or(&[]);
        let mean = |f: fn(&(f64, f64, f64)) -> f64| if after.is_empty() { 0.0 } else { after.iter().map(f).sum::<f64>() / after.len() as f64 };
        let every = self.allocs_all.get(fps.max(1.0) as usize..).unwrap_or(&[]);
        let mean_all = |f: fn(&(f64, f64)) -> f64| if every.is_empty() { 0.0 } else { every.iter().map(f).sum::<f64>() / every.len() as f64 };
        let zh = serde_json::json!({"apply_per_frame": mean(|a| a.0), "apply_bytes_per_frame": mean(|a| a.1), "flush_per_frame": mean(|a| a.2),
            "all_threads_per_frame": mean_all(|a| a.0), "all_threads_bytes_per_frame": mean_all(|a| a.1),
            // by thread over the whole run (warmup included): allocations and KB a frame
            "sites": raea_core_alloc::sites().into_iter().take(40).map(|(k, n)| serde_json::json!({"site": k, "samples": n})).collect::<Vec<_>>(),
            "threads": raea_core_alloc::threads().into_iter().filter(|t| t.1 > 0).map(|(n, c, b)| serde_json::json!({"thread": n, "per_frame": c as f64 / self.work.len().max(1) as f64, "kb_per_frame": b as f64 / 1024.0 / self.work.len().max(1) as f64})).collect::<Vec<_>>()});
        let secs = self.start.map(|s| s.elapsed().as_secs_f64()).unwrap_or(0.0);
        let mut order: Vec<usize> = (0..self.work.len()).collect();
        order.sort_by(|a, b| self.work[*b].total_cmp(&self.work[*a]));
        // first-use hitches (lightspeed F6): frames past the first second whose work went over 2 ms, the time a new
        // material, shader, glyph or sound costs when it's met mid-play
        let warm = fps.max(1.0) as usize;
        let hitch: Vec<usize> = (warm..self.work.len()).filter(|&i| self.work[i] > 2.0).collect();
        let hitches: Vec<serde_json::Value> = hitch.iter().take(16).map(|&i| serde_json::json!({"frame": i, "at_s": (i as f64 / fps.max(1.0) * 100.0).round() / 100.0, "work": self.work[i], "step": self.steps.get(i), "present": self.presents.get(i)})).collect();
        let slowest: Vec<serde_json::Value> = order.iter().take(8).map(|&i| serde_json::json!({"frame": i, "at_s": (i as f64 / fps.max(1.0) * 100.0).round() / 100.0, "work": self.work[i], "step": self.steps.get(i), "present": self.presents.get(i)})).collect();
        serde_json::json!({
            "seconds": (secs * 10.0).round() / 10.0,
            "composition_fps": fps,
            "display_hz": refresh,
            "page": self.live.as_ref().map(|l| [l.w, l.h]),
            "stepped": self.stepped,
            "presented": self.work.len(),
            "presented_fps": if secs > 0.0 { (self.work.len() as f64 / secs * 10.0).round() / 10.0 } else { 0.0 },
            "work_ms": {"p50": pct(&self.work, 0.5), "p99": pct(&self.work, 0.99), "p999": pct(&self.work, 0.999), "max": pct(&self.work, 1.0)},
            "first_frame_unix_ms": FIRST_FRAME.get(),
            "gpu_mb": raea_gpu_device::device().ok().and_then(|(_, _, d, _)| d.generate_allocator_report()).map(|r| r.total_allocated_bytes as f64 / 1e6),
            "hitches_over_2ms": {"count": hitch.len(), "max": hitch.iter().map(|&i| self.work[i]).fold(0.0, f64::max), "first": hitches},
            "step_ms": {"p50": pct(&self.steps, 0.5), "p99": pct(&self.steps, 0.99)},
            "present_ms": {"p50": pct(&self.presents, 0.5), "p99": pct(&self.presents, 0.99)},
            "js_ms": {"p50": pct(&self.js, 0.5), "p99": pct(&self.js, 0.99)},
            "apply_ms": {"p50": pct(&self.apply, 0.5), "p99": pct(&self.apply, 0.99)},
            "commit_kb": {"p50": pct(&self.bytes, 0.5) / 1024.0, "p99": pct(&self.bytes, 0.99) / 1024.0},
            "rust_allocs": zh,
            "gap_ms": {"p50": pct(&self.gaps, 0.5), "p99": pct(&self.gaps, 0.99), "max": pct(&self.gaps, 1.0)},
            "missed_vs_display": missed,
            "slowest": slowest,
            "rss_mb": {"first": self.rss.first().map(|k| k / 1024), "last": self.rss.last().map(|k| k / 1024), "max": self.rss.iter().max().map(|k| k / 1024), "each_second": self.rss.iter().map(|k| k / 1024).collect::<Vec<_>>()},
            "reusable_mb": {"each_second": self.reusable.iter().map(|k| (*k as f64 / 102.4).round() / 10.0).collect::<Vec<_>>()},
            "footprint_mb": {"each_second": self.footprint.iter().map(|m| (m * 10.0).round() / 10.0).collect::<Vec<_>>()},
            "error": self.error,
            // the window settings the game asked for (raea/window): applied to a window, reported here
            "window": raea_runtime_plugins::window_settings().1,
        })
    }
}

impl ApplicationHandler for Run {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let (w, h) = self.size;
        let attrs = Window::default_attributes().with_title("raea play").with_inner_size(winit::dpi::PhysicalSize::new(w, h));
        let window = match el.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                self.error = Some(format!("window: {e}"));
                el.exit();
                return;
            }
        };
        let px = window.inner_size();
        let made = raea_gpu_device::present::window(window.clone(), px.width, px.height, self.vsync).and_then(|stage| Live::open(&self.tsx, (px.width, px.height)).map(|live| (stage, live)));
        match made {
            Ok((stage, live)) => {
                self.stage = Some(stage);
                self.live = Some(live);
            }
            Err(e) => {
                self.error = Some(e);
                el.exit();
                return;
            }
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
            WindowEvent::KeyboardInput { event, .. } => {
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
            // the other buttons held, as marked keys a game's useInput() reads as `buttons` (games.ts wireKeys)
            WindowEvent::MouseInput { state, button, .. } => {
                let k = format!("Mouse:{}", match button {
                    MouseButton::Right => "Right".to_string(),
                    MouseButton::Middle => "Middle".into(),
                    MouseButton::Back => "Back".into(),
                    MouseButton::Forward => "Forward".into(),
                    MouseButton::Other(n) => format!("Other{n}"),
                    MouseButton::Left => "Left".into(),
                });
                match state {
                    ElementState::Pressed if !self.keys.contains(&k) => self.keys.push(k),
                    ElementState::Released => self.keys.retain(|x| *x != k),
                    _ => {}
                }
            }
            // fingers, as marked keys "Touch:ID:X:Y" in composition px (useInput().touches)
            WindowEvent::Touch(t) => {
                let prefix = format!("Touch:{}:", t.id);
                self.keys.retain(|x| !x.starts_with(&prefix));
                if matches!(t.phase, winit::event::TouchPhase::Started | winit::event::TouchPhase::Moved)
                    && let (Some(w), Some(l)) = (self.window.as_ref(), self.live.as_ref())
                {
                    let s = w.inner_size();
                    let k = (s.width as f64 / l.w as f64).min(s.height as f64 / l.h as f64);
                    let (ox, oy) = ((s.width as f64 - l.w as f64 * k) / 2.0, (s.height as f64 - l.h as f64 * k) / 2.0);
                    self.keys.push(format!("{prefix}{}:{}", (t.location.x - ox) / k, (t.location.y - oy) / k));
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.frame(el) {
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
