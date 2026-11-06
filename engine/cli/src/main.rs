//! `raea render EDIT.tsx -o OUT.mp4|OUT.png|DIR|OUT.sql [--frame N | --at A,B]` and the rest of raea's command line
//! (lib.rs), with the allocator every raea process counts with.
use raea_runtime_react::{CLI, start, startup_mark};

/// ZeroHeap Z1: every allocation counted per thread while RAEA_ZEROHEAP=1 (one relaxed load more otherwise)
#[global_allocator]
static ALLOC: raea_core_alloc::Counting = raea_core_alloc::Counting;

fn main() {
    raea_core_alloc::init();
    raea_core_alloc::init_sites();
    startup_mark("main");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        // `check VIDEO.mp4 …`: the floor's measurable lines on an export, ranked P0-P3
        // (with --taste or --quick, through the engine, which evaluates the taste.ts files or renders stills: raea_cli's `check`)
        Some("check") if !args.iter().any(|a| a == "--taste" || a == "--quick") => raea_app_check::cli_with(&args[1..], Vec::new()),
        // `ci …`: the gates' sound measurements and lightspeed's process measurements
        Some("ci") => raea_app_ci::cli(&args[1..]),
        _ => {
            CLI.get_or_init(|| raea_cli::cli);
            start();
            raea_cli::cli(&args)
        }
    };
    std::process::exit(code);
}
