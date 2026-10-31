use super::*;

/// The sandbox's WASI tools (RAEA_SANDBOX_TOOLS, else raea/sandbox/modules) as wsx.* calls, in the current folder.
pub fn register_sandbox() {
    let dir = std::env::var_os("RAEA_SANDBOX_TOOLS").map(PathBuf::from).unwrap_or_else(|| raea_io_home::home().join("engine/sandbox/modules"));
    if !dir.is_dir() {
        return;
    }
    match wsx::Sandbox::new(&dir) {
        Ok(sb) => wsx::register(sb, std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        Err(e) => eprintln!("raea: the sandbox's tools didn't load ({}): {e}", dir.display()),
    }
}

pub fn cli(args: &[String]) -> i32 {
    // the media verbs (`media probe|still|sheet|...`, `ffmpeg`, `ffprobe`: raea-media, no ffmpeg binary) and an HTML
    // composition's render are raea-app-media's (apps/media/src/command.rs)
    let html = args.first().map(String::as_str) == Some("render") && args.get(1).is_some_and(|e| e.ends_with(".html") || Path::new(e).join("index.html").is_file());
    if html || matches!(args.first().map(String::as_str), Some("media" | "ffmpeg" | "ffprobe" | "shots")) {
        return raea_app_media::command(args);
    }
    raea_runtime_react::native::register();
    raea_runtime_react::startup_mark("native calls registered");
    if matches!(args.first().map(String::as_str), Some("run" | "test" | "types")) {
        register_sandbox();
    }
    // a play or a render leaves its session to look back at (raea_io_trace: ~/Library/Logs/raea/raea/)
    let traced = matches!(args.first().map(String::as_str), Some("play" | "render"));
    if traced {
        raea_io_trace::start("raea");
        raea_runtime_react::startup_mark("trace started");
    }
    let code = match run(args) {
        Ok(msg) => {
            crate::refused::clear(args);
            // a still is an export like a video (whose encoder signs and marks it): Content Credentials saying raea drew
            // it, and a PNG's pixels carry its export id
            if !msg.is_empty() {
                println!("{msg}");
            }
            0
        }
        Err(e) => {
            raea_io_trace::event("refused", json!({"error": e}));
            // and what changed since this file was last refused, so a fix that strays is seen (refused.rs)
            println!("REFUSED  {e}{}", crate::refused::note(args, &e));
            1
        }
    };
    if traced {
        raea_io_trace::end();
    }
    code
}

/// `raea traces [show [N|last|DIR]]`: the sessions every raea app left (newest first: how each ended, its
/// glitches), or one told in full (why it crashed, its worst frames, its errors, its memory).
pub fn traces(args: &[String]) -> R<String> {
    let all = raea_io_trace::sessions();
    if args.get(1).map(String::as_str) == Some("show") {
        let which = args.get(2).map(String::as_str).unwrap_or("last");
        let dir = match which {
            "last" => all.first().map(|s| s.1.clone()),
            n if n.parse::<usize>().is_ok() => all.get(n.parse::<usize>().unwrap()).map(|s| s.1.clone()),
            p => Some(PathBuf::from(p)),
        }
        .ok_or("no such session (raea traces lists them)")?;
        return Ok(raea_io_trace::summarize(&dir));
    }
    let mut out = format!("{:<4} {:<12} {:<28} {:<18} {:>9} {:>8}  {}\n", "#", "app", "session", "ended", "glitches", "errors", "folder");
    for (i, (app, d)) in all.iter().enumerate().take(40) {
        let events = std::fs::read_to_string(d.join("events.jsonl")).unwrap_or_default();
        let glitches = events.lines().filter(|l| l.contains("\"kind\":\"glitch\"")).count();
        let errors = events.lines().filter(|l| l.contains("\"level\":\"ERROR\"") || l.contains("\"kind\":\"error\"") || l.contains("\"kind\":\"gpu_error\"")).count();
        out += &format!("{:<4} {:<12} {:<28} {:<18} {:>9} {:>8}  {}\n", i, app, d.file_name().unwrap_or_default().to_string_lossy(), raea_io_trace::status(d), glitches, errors, d.display());
    }
    if all.is_empty() {
        out += "(no sessions yet)\n";
    }
    Ok(out)
}

/// `raea run SCRIPT.ts`: a TypeScript program against the engine, headless (the native calls: solids, ECS
/// worlds, print). The script's default export, if a function, is called; what it prints goes to stdout; a throw
/// ends the run with exit 1.
pub fn run_script(args: &[String]) -> R<i32> {
    let ts = PathBuf::from(&args[1]).canonicalize().map_err(|e| format!("{}: {e}", args[1]))?;
    let out = std::env::temp_dir().join(format!("raea-cli-{}.js", std::process::id()));
    bundle_script(&ts, &out)?;
    let secs: f64 = flag(args, "--timeout").and_then(|t| t.parse().ok()).unwrap_or(120.0);
    let (code, _) = run_bundle(&out, secs)?;
    let _ = std::fs::remove_file(&out);
    Ok(code)
}

pub fn run(args: &[String]) -> R<String> {
    // no arguments, or asking for help: the usage, plainly (the render line's refusal is for a render missing its file)
    if args.is_empty() || matches!(args[0].as_str(), "help" | "--help" | "-h") {
        let was = IN_HELP.replace(true);
        let usage = run(&["render".to_string()]).err().unwrap_or_default();
        IN_HELP.set(was);
        let (export_game, scene, physics, splat) = (raea_app_game::USAGE, export::SCENE_USAGE, motion::physics::USAGE, tools::splat::USAGE);
        let apps = format!("       {}\n", raea_app_check::USAGE);
        return Ok(format!("raea: renders, plays and checks raea compositions (React and TypeScript) on the GPU.\n\nusage: {usage}\n       raea init   (a project's files where they're missing: edit.tsx, cuts.ts, components/, AGENTS.md)\n       raea cut TAKE \"line\" ... | --at A-B ... [--append]   (cuts.ts and words.json: a take under media/ cut in the pauses around each line it says, transcribed first if it has no TAKE.words.json; or at those seconds)\n       raea frame TAKE [--at S]   (how a take fits the edit: a talking take for <FaceFrame>, a screen for <ScreenFit>, from where its face is)\n       raea part list | raea part add NAME ... [--force]   (a ready part copied into components/, yours to change)\n       raea brand DOMAIN|URL   (the brand kit read from its website on this machine: brand/brand.css, brand/brand.json, BRAND.md, logos in media/brand; says when a plain fetch saw too little)\n       raea guide [NAME]   (craft, the floor read before any edit; the modes: pitch, explain, story, play, tool; motion, music, layout and the rest; bevy and bevy/NAME for 3D)\n       raea context PROJECT   (what to read before working on a project: taste, brief, look, the floor and its modes' guides)\n{apps}       raea play EDIT.tsx [--input | --input-script KEYS.json] [--size WxH] [--seconds N|end]   (a game, live; also: --input-log FILE --offscreen --snapshot T FILE --json FILE --seek F FILE --scrub N --baked --autoplay --no-vsync --master --3d)\n       {export_game}   (a game as a program of its own: a Mac app or a command)\n       {scene}   (what each 3D part rests on: floating, sunk and clipping parts; light: the scene's lighting)\n       {physics}   (the bakes <Physics> and <Goo> make)\n       {splat}   (footage trained into a Gaussian splat for <Splat>; setup installs its tools)\n       raea run SCRIPT.ts   (a TypeScript program on the engine, headless: its default export is called, what it prints is the answer; a check like a room's checkLayout; also: --timeout S)\n       raea mezz [--check] PATH...   (an intra-only ProRes copy of each long-GOP source over 1080p, for smooth scrubbing; --check says which would get one)\n       raea traces [show [N|last|DIR]]   (the sessions raea left, newest first: how each ended, its glitches; show tells one in full)"));
    }
    // an argument the verb doesn't take is refused with its usage, never dropped (zsh's "--frame 20", one word, rendered
    // every frame): the verb's flags and how many arguments it takes are read from its lines in the help, the one list
    if !IN_HELP.get() {
        if let Some(why) = stray(&run(&["help".to_string()])?, args) {
            return Err(why);
        }
    }
    // `guide [NAME]` (or `craft`, its old name): the craft guides the skill points to, from the checkout or the install (share/raea)
    if matches!(args[0].as_str(), "guide" | "craft") {
        return guide(args.get(1).map(String::as_str));
    }
    // `context PROJECT`: what an agent reads before any work on a project (docs/craft-system.md, A)
    if args[0] == "context" {
        return context(args.get(1).map(String::as_str));
    }
    if args.first().map(String::as_str) == Some("traces") {
        return traces(args);
    }
    if args.first().map(String::as_str) == Some("play") {
        return raea_app_play::play(args);
    }
    // `export-game EDIT.tsx -o OUT`: the game as a program of its own (game.rs, sdk/react/docs/game-export.md)
    if args.first().map(String::as_str) == Some("export-game") {
        return raea_app_game::export(args);
    }
    // `add PKG... [--dev] | remove NAME... | install [--offline]  [--in DIR]`: npm packages for the project (its
    // package.json and raea.lock, a pnpm-style node_modules from one verified store; install scripts never run), which
    // a composition, an app screen or a legacy-app panel imports (io/npm)
    if matches!(args.first().map(String::as_str), Some("add" | "remove" | "install")) {
        return raea_io_npm::cli(args);
    }
    // `mezz [--check] PATH...`: optimized media (raea_io_media::mezz, lightspeed F4): each long-GOP source over 1080p under
    // PATH (a file, or a folder's videos) gets its intra-only ProRes copy, one at a time; --check only says which would
    if args.first().map(String::as_str) == Some("mezz") {
        return mezz(&args[1..]);
    }
    // the sound tools (apps/tools sound.rs): `sao "PROMPT" -o OUT.wav` and `tokensynth "TIMBRE" --notes C3,C4 -o DIR` make
    // through the edit's asset store as its render would; `setup-audio` installs or checks both local models;
    // `sounds [WORDS]` and `sounds get PACK/INSTRUMENT` search and fetch the open sample libraries
    match args.first().map(String::as_str) {
        Some("sao") => return tools::sound::sao(args),
        Some("tokensynth") => return tools::sound::tokensynth(args),
        Some("setup-audio") => return tools::sound::setup_audio(args),
        Some("sounds") => return tools::sound::sounds(args),
        // `splat train VIDEO|DIR -o OUT.ply`: footage trained into a Gaussian splat for <Splat> (apps/tools splat.rs)
        Some("splat") => return tools::splat::splat(args),
        // `physics bake|goo|part ...`: the bakes <Physics> and <Goo> make (systems/bake, registered in motion/physics.rs)
        Some("physics") => return motion::physics::cli(args),
        // `scene lint|explain EDIT.tsx`: what each 3D part rests on; floating, sunk and clipping parts (raea-sys-contact, through motion/contact.rs)
        Some("scene") => return export::scene_cli(args),
        // `song FILE [-o OUT.song.json] [--plot OUT.png]`: the song's energy map for pacing (apps/tools/src/song.rs)
        Some("song") => return tools::song::song(args),
        // `track CLIP.mp4 [--subject X,Y,W,H] [--from S]`: the clip's camera solved for <Matchmove> (motion/track.rs)
        Some("track") => return motion::track::track(args),
        // the agent shell's own verbs ported here (`init`, `cut`, `frame`): engine/cli's handler, run by its native.ts (shell())
        Some(v) if SHELL.contains(&v) => return shell(args),
        _ => {}
    }
    // `master-gain BUNDLE.js DIR`: the gain and ceiling a composition's render masters its sound with, as JSON (live
    // play asks this of a child process, so its second React and the whole mix never sit in the game's memory)
    if args.first().map(String::as_str) == Some("master-gain") && args.len() >= 3 {
        let g = master_gain(Path::new(&args[1]), Path::new(&args[2]))?;
        return Ok(match g {
            Some((gain, ceiling)) => json!({"gain_db": gain, "ceiling_dbtp": ceiling}),
            None => json!({"none": true}),
        }
        .to_string());
    }
    if args.first().map(String::as_str) == Some("bundle") && args.len() >= 2 {
        let ts = PathBuf::from(&args[1]).canonicalize().map_err(|e| format!("{}: {e}", args[1]))?;
        let out = PathBuf::from(flag(args, "-o").ok_or("raea bundle SCRIPT.ts -o OUT.js")?);
        if args.iter().any(|a| a == "--composition-bundle") {
            let portable = flag(args, "--portable").map(PathBuf::from);
            bundle_composition(&ts, &out, &comp_opts(args)?, portable.as_deref())?;
            return Ok(format!("APPLIED  {} bundled as a composition to {}", ts.display(), out.display()));
        }
        bundle_script(&ts, &out)?;
        return Ok(format!("APPLIED  {} bundled to {}", ts.display(), out.display()));
    }
    if args.first().map(String::as_str) == Some("types") {
        // the native calls' TypeScript, from the registry (sdk/raea/src/native-calls.ts)
        raea_runtime_react::native::register();
        let ts = raea_native::typescript();
        match flag(args, "-o") {
            Some(o) => {
                std::fs::write(&o, &ts).map_err(|e| format!("{o}: {e}"))?;
                return Ok(format!("APPLIED  {} native calls' types written to {o}", raea_native::calls().len()));
            }
            None => return Ok(ts),
        }
    }
    if args.first().map(String::as_str) == Some("test") && args.len() >= 2 {
        // conformance tests on the engine's V8: raea test FILE... [--filter TEXT] [--timeout S] [--bundle-only OUT.js]
        // --define K=V, --alias SPEC=PATH and --node-stub a,b go to the bundler (each may repeat)
        let mut files = Vec::new();
        let mut owned: Vec<String> = Vec::new();
        let mut i = 1;
        while i < args.len() {
            if args[i].starts_with("--") {
                if matches!(args[i].as_str(), "--define" | "--alias" | "--node-stub") && i + 1 < args.len() {
                    owned.push(args[i].clone());
                    // an alias's path is the caller's, so it's made absolute here (the bundler runs elsewhere)
                    owned.push(match (args[i].as_str(), args[i + 1].split_once('=')) {
                        ("--alias", Some((k, v))) => format!("{k}={}", std::path::absolute(v).map(|p| p.display().to_string()).unwrap_or_else(|_| v.to_string())),
                        _ => args[i + 1].clone(),
                    });
                }
                i += 2;
                continue;
            }
            files.push(PathBuf::from(&args[i]).canonicalize().map_err(|e| format!("{}: {e}", args[i]))?);
            i += 1;
        }
        let extra: Vec<&str> = owned.iter().map(String::as_str).collect();
        if let Some(o) = flag(args, "--bundle-only") {
            // the bundler runs in its own folder, so the output path is made absolute here
            let o = std::path::absolute(&o).map_err(|e| format!("{o}: {e}"))?.display().to_string();
            bundle_tests(&files, Path::new(&o), flag(args, "--filter").as_deref(), &extra)?;
            return Ok(format!("APPLIED  {} test files bundled to {o}", files.len()));
        }
        // the repo root for fixtures (tests read RAEA_ROOT through host.env)
        if std::env::var_os("RAEA_ROOT").is_none() {
            unsafe { std::env::set_var("RAEA_ROOT", raea_io_home::home().canonicalize().unwrap_or_default()) };
        }
        let out = std::env::temp_dir().join(format!("raea-cli-t{}.js", std::process::id()));
        bundle_tests(&files, &out, flag(args, "--filter").as_deref(), &extra)?;
        let secs: f64 = flag(args, "--timeout").and_then(|t| t.parse().ok()).unwrap_or(300.0);
        let (code, _) = run_bundle(&out, secs)?;
        let _ = std::fs::remove_file(&out);
        if code != 0 {
            return Err(format!("{} test file(s): some failed", files.len()));
        }
        return Ok(String::new());
    }
    if args.first().map(String::as_str) == Some("run") && args.len() >= 2 {
        let code = run_script(args)?;
        if code != 0 {
            return Err(format!("{} ended with exit {code}", args[1]));
        }
        return Ok(String::new());
    }
    // a composition already bundled (NAME.comp.js, as the iPhone's probe renders it), its media under DIR; --frame N
    // draws the one frame the probe's raea_motion_frame draws, for comparing the phone's pixels with the Mac's
    if args.first().map(String::as_str) == Some("render") && args.get(1).map(String::as_str) == Some("--bundle") && args.len() >= 3 {
        let js = PathBuf::from(&args[2]);
        let dir = PathBuf::from(flag(args, "--dir").unwrap_or_else(|| js.parent().unwrap().to_string_lossy().into_owned()));
        let out = PathBuf::from(flag(args, "-o").ok_or("-o OUT.mp4|OUT.png|DIR")?);
        let wanted = flag(args, "--frame").map(|f| f.parse::<i64>().map(|f| vec![f]).map_err(|_| "--frame takes a frame number")).transpose()?;
        return render_bundle(&js, &dir, &out, wanted);
    }
    if args.first().map(String::as_str) != Some("render") || args.len() < 2 {
        return Err(format!(
            "raea render EDIT.tsx -o OUT.mp4|OUT.png|DIR [--frame N | --at A,B] [--composition ID] [--w W] [--h H] [--fps FPS] [--frames N] [--words W.json] [--perception P.json] [--input I.json|inputs.rrl] [--manifest render.json] [--stage WxH] [--10bit] [--kbps N | --quality draft|high|master|0..1]   (also: --lite --comps)\n       raea render PAGE.html -o OUT.mp4|DIR [--at A,B]   (an HTML composition, the HTML composition format format)\n       raea render --bundle NAME.comp.js -o OUT.png|DIR [--dir DIR] [--frame N]   (a bundle from raea bundle, as render_bundle draws it)
       raea media probe|still|sheet|audio|loudness|motion|cuts|freezes|proxy|trim|concat FILE ... | ffmpeg ARGS | ffprobe ARGS   (media work, native: media help)
       raea bundle EDIT.tsx --composition-bundle -o OUT.js [--portable DIR]   (a composition for render_bundle, on the Mac or the iPhone)\n       raea add PKG[@RANGE]... [--dev] | remove NAME... | install [--offline]  [--in DIR]   (npm packages for the project: raea.lock, node_modules)\n       {}   (foley, local)\n       {}   (an instrument's notes, local)\n       raea setup-audio [--check] [sao|tokensynth]   (install or check the local audio models)\n       {}   (the song's energy map: beats, sections, builds, drops, stops, hits, cuts per bar; also: --backend NAME)\n       raea sounds [WORDS] | sounds get PACK/INSTRUMENT [--notes ..|--all] [-o DIR]   (open sample libraries for a Sampler)\n       {}   (the clip's camera solved, for <Matchmove>)",
            tools::sound::SAO_USAGE, tools::sound::TOKENSYNTH_USAGE, tools::song::USAGE, motion::track::USAGE
        ));
    }
    raea_runtime_export::render(args)
}
