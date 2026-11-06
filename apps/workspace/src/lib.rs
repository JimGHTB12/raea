//! A workspace's folders and files (docs/projects.feature PJ1-PJ4): `raea workspace`, `raea project` (with split and
//! join) and `raea controller`. A workspace is a git repo raea changes by commits; its controllers are spec modules,
//! which the raea CLI evaluates on the engine and hands in as an `Eval`, so this crate needs no engine of its own.
use std::path::{Path, PathBuf};

use serde_json::json;

mod controllers;
pub mod doctor;
mod literal;
pub mod next;
mod storage;
mod workspace;

pub use controllers::{controller, module, summary};
pub use literal::{tidy, tidy_deep, ts_literal};
pub use workspace::{project, projects, root, slug, up, workspace};

pub type R<T> = Result<T, String>;

/// A spec module's default export as JSON: the raea CLI's spec_json, which runs it on the engine.
pub type Eval = fn(&Path) -> R<String>;

#[cfg(test)]
mod tests;
