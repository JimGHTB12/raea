//! raea's command line, which the `raea` binary (main.rs) and a plugin build (bin/raea-bevy-sdk.rs) both run: the
//! dispatch and the help (cli.rs, usage.rs), the guides and a project's context (guide.rs), the small verbs (verbs.rs)
//! and the verbs of raea's app crates.
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use raea_runtime_react::scene::{self as motion, *};
use raea_runtime_export::{self as export, *};
use raea_app_tools as tools;
use serde_json::json;
use raea_neural as neural;

mod cli;
mod guide;
mod refused;
mod skill;
mod spec;
mod tests;
mod usage;
mod verbs;
pub use cli::cli;
use guide::*;
use usage::*;
use verbs::*;
