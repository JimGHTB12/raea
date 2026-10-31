//! conformance: the UI conformance suite (sdk/ui/conformance/conformance.feature). Reads the case files, launches the UI
//! substrate under test headless (its own HOME, MCP port and throwaway copy of the fixture workspace, a stub agent),
//! drives it only through MCP `input` and reads it only through MCP `ui`, and writes results.jsonl.
//!
//!   conformance [--bin PATH] [--substrate NAME] [--env K=V]... [--tier quick|full] [--only GLOB]...
//!                [--cases DIR] [--out DIR] [--keep] [--attach PORT --workspace DIR] [--list]
//!
//! Exit: 0 every case passed (or was only slow), 1 any failed, 2 the run couldn't start.

mod case;
mod mcp;
mod pred;
mod run;

#[cfg(test)]
mod selftest;

use std::path::PathBuf;

fn main() {
    run::signals();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let opts = match run::Opts::parse(&args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("conformance: {e}\n\n{}", run::USAGE);
            std::process::exit(2);
        }
    };
    std::process::exit(run::main(opts));
}

/// sdk/ui/conformance, where this crate's sources are.
pub fn here() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
