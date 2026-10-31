//! raea-page-host: a bundle on a Blitz page in a headless Bevy app, for the benches.
//!
//!   raea-page-host BUNDLE [--css FILE[:FILE...]] [--root DIR[:DIR...]] [--size WxH]
//!
//! BUNDLE is built JS (an app's Blitz build). --css: the page's own CSS, laid out as Chrome lays out a page; without
//! it, bevy_ui's kinds. --root: where the page's own-origin fetches are read from (RAEA_PAGE_ROOT). --size: the
//! window, default 1440x900. RAEA_PAGE_BRP_PORT (default: a free one), RAEA_PAGE_MCP_PORT (default 7477; 0: none),
//! RAEA_PAGE_PICK=0: the page's pointers and keys dispatched by the host (page::input) instead of through raea-page's
//! picking route, the default. The page's `<img data-raea-stage>` plays a composition live (src/stage.rs).

use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::window::{ExitCondition, WindowResolution};
use raea_page_host::{HostPlugin, brp, camera, mcp, page, stage, tree};

struct Args {
    bundle: String,
    css: Option<Vec<String>>,
    size: UVec2,
}

fn args() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let (mut bundle, mut css, mut size) = (None, None, UVec2::new(1440, 900));
    while let Some(a) = it.next() {
        let mut val = || it.next().ok_or(format!("{a} needs a value"));
        match a.as_str() {
            "--css" => css = Some(val()?.split(':').map(str::to_string).collect()),
            // read by raea-page's net.rs (page_roots), a list as PATH is
            "--root" => unsafe { std::env::set_var("RAEA_PAGE_ROOT", val()?) },
            "--size" => {
                let v = val()?;
                let (w, h) = v.split_once('x').ok_or(format!("--size {v}: WxH"))?;
                size = UVec2::new(w.trim().parse().map_err(|_| format!("--size {v}"))?, h.trim().parse().map_err(|_| format!("--size {v}"))?);
            }
            s if s.starts_with("--") => return Err(format!("no option {s}")),
            s => bundle = Some(s.to_string()),
        }
    }
    let bundle = bundle.ok_or("usage: raea-page-host BUNDLE [--css FILE[:FILE]] [--root DIR[:DIR]] [--size WxH]")?;
    Ok(Args { bundle, css, size })
}

fn main() {
    let a = args().unwrap_or_else(|e| {
        eprintln!("raea-page-host: {e}");
        std::process::exit(2);
    });
    let code = std::fs::read_to_string(&a.bundle).unwrap_or_else(|e| {
        eprintln!("raea-page-host: {}: {e}", a.bundle);
        std::process::exit(1);
    });
    let picking = !std::env::var("RAEA_PAGE_PICK").is_ok_and(|v| v == "0");
    mcp::serve(mcp::port(), mcp::Server { name: "raea-page-host".into(), instructions: "A raea page. Drive it with input, read it with ui; brp reaches the whole ECS (Bevy Remote Protocol).".into(), prefix: "page".into() });

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window { title: "raea".into(), resolution: WindowResolution::new(a.size.x, a.size.y), ..default() }),
                exit_condition: ExitCondition::DontExit,
                close_when_requested: false,
                ..default()
            })
            .set(LogPlugin { level: Level::WARN, ..default() })
            // the shared device (raea_gpu_device), so a game on the stage (src/stage.rs), drawn there by the live engine, is a
            // texture this renderer copies with no trip across devices (RAEA_SHARED_DEVICE=0: Bevy's own, no stage)
            .set(bevy::render::RenderPlugin { render_creation: raea_runtime_view::render_creation(), ..default() })
            // each update draws its own frame: the frame `ui` reports is the one rendered
            .disable::<PipelinedRenderingPlugin>(),
    );
    if !app.is_plugin_added::<ScheduleRunnerPlugin>() {
        app.add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0)));
    }
    app.add_plugins(HostPlugin { size: a.size, double_click: Duration::from_millis(500), picking })
        .add_plugins(brp::plugins(brp::port(), |r| r))
        .add_plugins(stage::StagePlugin);
    raea_page_host::page_plugins(&mut app, picking);
    let (css, size) = (a.css, a.size);
    app.add_systems(
        Startup,
        (camera, move |world: &mut World| {
            let m = page::Mount { code: code.clone(), preamble: vec![], css: css.clone(), styles: None, picking, size };
            if let Err(e) = page::mount(world, m) {
                eprintln!("raea-page-host: {e}");
                std::process::exit(1);
            }
        })
            .chain(),
    );
    if picking {
        app.add_systems(PreUpdate, (page::read_input, page::frame_input, page::size).chain().before(bevy::picking::PickingSystems::ProcessInput))
            .add_systems(Update, (page::step_dropping_asks, page::show).chain());
    } else {
        app.add_systems(Update, (page::read_input, page::input, page::size, page::step_dropping_asks, page::show).chain());
    }
    app.add_systems(Last, page::tree.before(tree::answer));
    app.run();
}
