//! `raea render EDIT.tsx -o OUT.mp4|OUT.png|DIR|OUT.sql [--frame N | --at A,B]` and the rest of raea's command line
//! (lib.rs's `main`), with the allocator every raea process counts with.

/// ZeroHeap Z1: every allocation counted per thread while RAEA_ZEROHEAP=1 (one relaxed load more otherwise)
#[global_allocator]
static ALLOC: raea_core_alloc::Counting = raea_core_alloc::Counting;

fn main() {
    std::process::exit(raea_cli::main());
}
