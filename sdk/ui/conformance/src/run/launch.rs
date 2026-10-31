//! A case's world launched and waited up.

use super::*;

pub(crate) fn launch(o: &Opts, c: &Case, inst: &str, logs: &Path) -> Result<World, String> {
    if let Some(port) = o.attach {
        let ws = o.workspace.clone().unwrap_or_else(|| base().join("attached-workspace"));
        return Ok(World { proc: None, dir: None, workspace: ws, log: None, mcp: Mcp::new(port) });
    }
    let fx = fixture(&o.fixture)?;
    let dir = base().join("runs").join(std::process::id().to_string()).join(inst.replace(['/', '@'], "-"));
    let _ = std::fs::remove_dir_all(&dir);
    let (home, ws, tmp) = (dir.join("home"), dir.join("workspace"), dir.join("tmp"));
    for d in [&home.join(".raea"), &ws, &tmp] {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    // what the world starts with: a copy of the fixture (times kept), then dated fresh
    // clones where the filesystem can: macOS's cp -c (APFS), GNU cp's --reflink=auto (which has no -c)
    let clone: &[&str] = if cfg!(target_vendor = "apple") { &["-cRp"] } else { &["-Rp", "--reflink=auto"] };
    let cp = Command::new("cp").args(clone).arg(format!("{}/.", fx.display())).arg(&ws).status().map_err(|e| e.to_string())?;
    if !cp.success() {
        return Err("copying the fixture failed".into());
    }
    let _ = std::fs::remove_file(ws.join(".done"));
    // the fixture's .home/ is the world's home (the play items' games, which a feed item names by ~/ path)
    if ws.join(".home").is_dir() {
        let _ = std::fs::remove_dir_all(&home);
        std::fs::rename(ws.join(".home"), &home).map_err(|e| format!("the fixture's .home: {e}"))?;
        std::fs::create_dir_all(home.join(".raea")).map_err(|e| e.to_string())?;
    }
    date_files(&ws, &o.fixture.join("times.txt"));
    // an empty settings file: no key from ~/.raea/env reaches this world
    std::fs::write(home.join(".raea/env"), "").map_err(|e| e.to_string())?;

    let port = free_port()?;
    let real_home = std::env::var("HOME").unwrap_or_default();
    let path = format!("{}:{real_home}/.bun/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin", std::env::var("PATH").unwrap_or_default());
    let page = is_page_host(&o.bin);
    let mut cmd = Command::new(&o.bin);
    if page {
        // the app's Blitz build at the case's route; its own-origin fetches from blitz/out, then public/
        let out = o.app.join("blitz/out");
        let js = std::fs::read_to_string(out.join("app.js")).map_err(|e| format!("{}: {e}", out.join("app.js").display()))?;
        let bundle = dir.join("app.js");
        std::fs::write(&bundle, format!("{}\n{js}", location(c.route.as_deref().unwrap_or(ROUTE)))).map_err(|e| e.to_string())?;
        let roots = std::env::join_paths([out.clone(), o.app.join("public")]).map_err(|e| e.to_string())?;
        cmd.arg(&bundle).arg("--css").arg(out.join("app.css")).arg("--root").arg(roots);
    } else {
        cmd.arg(&ws);
    }
    cmd.current_dir(&ws).env_clear().process_group(0);
    let port_env = o.port_env.clone().unwrap_or_else(|| if page { "RAEA_PAGE_MCP_PORT" } else { "RAEA_FEED_MCP_PORT" }.into());
    let mut env: Vec<(String, String)> = vec![
        ("PATH".into(), path),
        ("HOME".into(), home.display().to_string()),
        ("TMPDIR".into(), format!("{}/", tmp.display())),
        ("RAEA_FEED_HOME".into(), home.join(".raea-feed").display().to_string()),
        (port_env, port.to_string()),
        ("RAEA_FEED_HEADLESS".into(), "1".into()),
        ("RAEA_FEED_BACKGROUND".into(), "1".into()),
        ("RAEA_AUDIO_MUTE".into(), "1".into()),
        ("RAEA_FEED_NO_MIC".into(), "1".into()),
        ("RAEA_FEED_AHEAD".into(), "0".into()),
        // the chat's stub: echoes, never a model (conformance.feature, "A run's world is its own")
        ("RAEA_FEED_AGENT".into(), "echo".into()),
        // and should a substrate not know the stub yet: every model endpoint points at a closed local port
        ("RAEA_ENV_FILE".into(), home.join(".raea/env").display().to_string()),
        ("RAEA_LLM_PROVIDER".into(), "custom".into()),
        ("RAEA_LLM_BASE_URL".into(), "http://127.0.0.1:9".into()),
        ("RAEA_LLM_API_KEY".into(), "conformance-stub".into()),
        ("RAEA_LLM_MODEL".into(), "conformance-stub".into()),
        ("RAEA_API".into(), "http://127.0.0.1:9".into()),
    ];
    // the display a Linux host opens its window on (an X server such as Xvfb, or Wayland's socket); unset on a Mac
    for k in ["USER", "LOGNAME", "LANG", "LC_ALL", "SHELL", "DISPLAY", "WAYLAND_DISPLAY", "XDG_RUNTIME_DIR"] {
        if let Ok(v) = std::env::var(k) {
            env.push((k.into(), v));
        }
    }
    // local models (voice, embeddings) and bun's package cache are read from the real home, never written
    let models = std::env::var("RAEA_MODELS").unwrap_or(format!("{real_home}/.local/share/raea-models"));
    if Path::new(&models).is_dir() {
        env.push(("RAEA_MODELS".into(), models));
    }
    env.push(("BUN_INSTALL_CACHE_DIR".into(), format!("{real_home}/.bun/install/cache")));
    if let Some(s) = &c.space {
        env.push(("RAEA_FEED_SPACE".into(), s.clone()));
    }
    env.extend(c.env.iter().cloned());
    env.extend(o.env.iter().cloned());
    for (k, v) in &env {
        cmd.env(k, v);
    }
    std::fs::create_dir_all(logs).map_err(|e| e.to_string())?;
    let log = logs.join(format!("{}.log", inst.replace(['/', '@'], "-")));
    let f = std::fs::File::create(&log).map_err(|e| e.to_string())?;
    cmd.stdout(Stdio::from(f.try_clone().map_err(|e| e.to_string())?)).stderr(Stdio::from(f)).stdin(Stdio::null());
    let child = cmd.spawn().map_err(|e| format!("launching {}: {e}", o.bin.display()))?;
    PGID.store(child.id() as i32, Ordering::SeqCst);
    Ok(World { proc: Some(Proc { child }), dir: Some(dir), workspace: ws, log: Some(log), mcp: Mcp::new(port) })
}

/// Until the substrate's MCP answers (or it dies, or `secs` pass); then its tools.
pub(crate) fn up(w: &mut World, secs: u64) -> Result<Vec<String>, String> {
    let t = Instant::now();
    loop {
        if let Some(p) = &mut w.proc {
            if let Ok(Some(st)) = p.child.try_wait() {
                return Err(format!("the substrate exited ({st}) before its MCP answered"));
            }
        }
        match w.mcp.tools() {
            Ok(t) => return Ok(t),
            Err(e) if t.elapsed() > Duration::from_secs(secs) => return Err(format!("its MCP never answered in {secs} s: {e}")),
            Err(_) => std::thread::sleep(Duration::from_millis(250)),
        }
    }
}

// --- the steps
