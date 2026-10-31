//! `raea play EDIT.tsx`: the live stage in a window of its own, and the playtest harness. The composition runs
//! at its rate on the live engine (live.rs), keys and pointer as its input, presented every display refresh on the GPU.
//! When it ends (the window closes, or --seconds pass) it reports what it measured: frames stepped and presented, frame
//! times (p50, p99, worst), frames missed against the display, and resident memory over the run.
//!
//!   raea play EDIT.tsx [--size WxH] [--seconds S|end] [--autoplay] [--no-vsync] [--json OUT]
//!
//! --seconds end plays to the composition's last frame (its frames and fps as it opened, however its length is written).
//!
//! --autoplay holds ArrowLeft and ArrowRight in turn (a run nobody plays); --no-vsync presents as fast as frames come,
//! to measure what a frame costs past the display's rate; --offscreen runs with no window (nothing shown, the same
//! work), for runs that shouldn't take anyone's screen.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use raea_runtime_live::Live;

mod run;
mod offscreen;
use offscreen::*;
#[cfg(test)]
mod tests;

type R<T> = Result<T, String>;

/// when the first frame finished presenting, in ms since the Unix epoch (lightspeed F5: a harness takes its spawn time
/// from it for spawn to first frame)
static FIRST_FRAME: std::sync::OnceLock<f64> = std::sync::OnceLock::new();

fn first_frame() {
    if FIRST_FRAME.get().is_none() {
        raea_runtime_react::startup_mark("first frame presented");
    }
    FIRST_FRAME.get_or_init(|| std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64() * 1e3).unwrap_or(0.0));
}

struct Run {
    tsx: PathBuf,
    size: (u32, u32),
    seconds: Option<f64>,
    autoplay: bool,
    vsync: bool,
    window: Option<Arc<Window>>,
    /// the version of the game's window settings last applied (apply_window)
    window_version: u64,
    stage: Option<raea_gpu_device::present::Present>,
    live: Option<Live>,
    keys: Vec<String>,
    pointer: Option<(f64, f64, bool)>,
    /// the wheel turned since the last frame (CSS px, down positive): an app screen's scroll (runtime/page/src/screen.rs)
    wheel: f64,
    start: Option<Instant>,
    frame: i64,
    last_present: Option<Instant>,
    /// per presented frame: the work (step + present) and the gap since the last present, in ms
    work: Vec<f64>,
    gaps: Vec<f64>,
    /// per frame: React's step (event to commit) and the present, in ms
    steps: Vec<f64>,
    presents: Vec<f64>,
    /// per frame: the JS thread's time, Bevy's apply, and the commit JSON, from raea-runtime-react's counters
    js: Vec<f64>,
    apply: Vec<f64>,
    bytes: Vec<f64>,
    /// Rust allocations a frame: applying ops on Bevy's side, and taking the batch on the JS thread (RAEA_ZEROHEAP=1)
    allocs: Vec<(f64, f64, f64)>,
    /// every thread's allocations and bytes a frame (RAEA_ZEROHEAP=1)
    allocs_all: Vec<(f64, f64)>,
    stepped: u64,
    rss: Vec<u64>,
    /// physical footprint each second (MB): what the process really holds (resident size counts reusable clean pages)
    footprint: Vec<f64>,
    /// of the resident size each second, the reusable pages (KB): freed, there until the kernel takes them
    reusable: Vec<u64>,
    last_rss: Option<Instant>,
    error: Option<String>,
}

fn rss_kb() -> u64 {
    vm_kb().0
}

/// This process's resident size and, of it, the reusable pages, in KB (task_info's TASK_VM_INFO, what `ps` reads; no
/// process spawned each second). Reusable pages are ones an allocator freed with MADV_FREE_REUSABLE, as JavaScriptCore's
/// libpas does when it returns its free pages: they stay resident, so counted in RSS, until the kernel wants them, and
/// are in no footprint. A live game's RSS crept 2-4 MB in 4 minutes with all of it there (vmmap: "WebKit Malloc").
fn vm_kb() -> (u64, u64) {
    #[cfg(target_os = "macos")]
    unsafe {
        unsafe extern "C" {
            static mach_task_self_: u32;
            fn task_info(task: u32, flavor: i32, info: *mut i32, count: *mut u32) -> i32;
        }
        // task_vm_info through phys_footprint: resident_size is [2], reusable [10]
        let (mut info, mut count) = ([0u64; 19], 38u32);
        if task_info(mach_task_self_, 22, info.as_mut_ptr() as *mut i32, &mut count) == 0 {
            return (info[2] / 1024, info[10] / 1024);
        }
    }
    (0, 0)
}

fn key_name(k: &Key) -> Option<String> {
    Some(match k {
        Key::Named(NamedKey::ArrowLeft) => "ArrowLeft".into(),
        Key::Named(NamedKey::ArrowRight) => "ArrowRight".into(),
        Key::Named(NamedKey::ArrowUp) => "ArrowUp".into(),
        Key::Named(NamedKey::ArrowDown) => "ArrowDown".into(),
        Key::Named(NamedKey::Space) => " ".into(),
        Key::Named(NamedKey::Enter) => "Enter".into(),
        Key::Named(NamedKey::Escape) => "Escape".into(),
        Key::Named(NamedKey::Shift) => "Shift".into(),
        Key::Character(c) => c.to_string(),
        _ => return None,
    })
}

/// One source drives a play's keys: `--input` presses them at random and `--input-script` from a file, and the random
/// thread would overwrite the script's keys every frame, so the two together are refused.
fn key_source(args: &[String]) -> R<()> {
    if args.iter().any(|a| a == "--input") && args.iter().any(|a| a == "--input-script") {
        return Err("--input and --input-script both drive the keys (the random presses would replace the script's every frame); give one".into());
    }
    Ok(())
}

pub fn play(args: &[String]) -> R<String> {
    key_source(args)?;
    raea_runtime_live::warm_audio();
    let flag = |n: &str| args.iter().position(|a| a == n).and_then(|i| args.get(i + 1)).cloned();
    let tsx = PathBuf::from(args.get(1).ok_or("raea play EDIT.tsx [--size WxH] [--seconds S] [--autoplay] [--no-vsync] [--json OUT]")?);
    let size = flag("--size").and_then(|s| s.split_once('x').and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))).unwrap_or((1080, 1920));
    let mut run = Run {
        window_version: 0,
        tsx,
        size,
        seconds: flag("--seconds").and_then(|s| s.parse().ok()),
        autoplay: args.iter().any(|a| a == "--autoplay"),
        vsync: !args.iter().any(|a| a == "--no-vsync"),
        window: None,
        stage: None,
        live: None,
        keys: Vec::new(),
        pointer: None,
        wheel: 0.0,
        start: None,
        frame: -1,
        last_present: None,
        work: Vec::new(),
        gaps: Vec::new(),
        steps: Vec::new(),
        presents: Vec::new(),
        js: Vec::new(),
        apply: Vec::new(),
        bytes: Vec::new(),
        allocs: Vec::new(),
        allocs_all: Vec::new(),
        stepped: 0,
        rss: Vec::new(),
        footprint: Vec::new(),
        reusable: Vec::new(),
        last_rss: None,
        error: None,
    };
    if args.iter().any(|a| a == "--offscreen") {
        // --snapshot S OUT.png (repeatable): the page read back at S seconds
        let snaps: Vec<(f64, String)> = args.windows(3).filter(|w| w[0] == "--snapshot").filter_map(|w| Some((w[1].parse().ok()?, w[2].clone()))).collect();
        return offscreen(run, flag("--json"), args.iter().any(|a| a == "--no-vsync"), snaps);
    }
    let el = EventLoop::new().map_err(|e| format!("event loop: {e}"))?;
    el.set_control_flow(ControlFlow::Poll);
    el.run_app(&mut run).map_err(|e| format!("event loop: {e}"))?;
    let report = run.report();
    if let Some(out) = flag("--json") {
        std::fs::write(&out, serde_json::to_string_pretty(&report).unwrap()).map_err(|e| format!("{out}: {e}"))?;
    }
    if let Some(e) = &run.error {
        return Err(format!("{e}\n{}", serde_json::to_string_pretty(&report).unwrap()));
    }
    Ok(format!("PLAYED  {}", serde_json::to_string_pretty(&report).unwrap()))
}
