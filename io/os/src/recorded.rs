//! T2d's recorded set of system calls, for each target's test (as io/net/calls serve.rs is for T2b): headless, with a
//! pick, a save and a dialog's answer queued, the script (recorded/os.js) opens two URLs, writes and reads the
//! clipboard, picks, reads, saves and writes files, and shows two dialogs; recorded/os.json is what both targets must
//! answer. A test runs `script` in its host and checks what it recorded with `check`.

use std::path::PathBuf;

use serde_json::Value;

use crate::{Scripted, opened, script, set_headless};

const SCRIPT: &str = include_str!("../recorded/os.js");
/// What the script records.
pub const RECORDED: &str = include_str!("../recorded/os.json");

/// The folder the picked and saved files are in.
pub struct Setup {
    pub dir: PathBuf,
}

/// Headless, a.txt ("abc") picked, b.txt saved, the dialog's second button pressed: all in `dir`.
pub fn setup(dir: PathBuf) -> Setup {
    set_headless();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.txt"), "abc").unwrap();
    script(Scripted::Pick(vec![dir.join("a.txt")]));
    script(Scripted::Save(dir.join("b.txt")));
    script(Scripted::Dialog(1));
    Setup { dir }
}

impl Setup {
    pub fn script(&self) -> &'static str {
        SCRIPT
    }

    /// What a host recorded, against RECORDED, and what os.open was asked for and b.txt: Ok, or how they differ.
    pub fn check(&self, recorded: &str) -> Result<(), String> {
        let got: Value = serde_json::from_str(recorded).map_err(|e| format!("recorded: {e}"))?;
        let want: Value = serde_json::from_str(RECORDED).unwrap();
        let (g, w) = (got.as_array().cloned().unwrap_or_default(), want.as_array().cloned().unwrap_or_default());
        for (i, (g, w)) in g.iter().zip(&w).enumerate() {
            if g != w {
                return Err(format!("step {}: got {g}, recorded {w}", i + 1));
            }
        }
        if g.len() != w.len() {
            return Err(format!("{} steps, recorded {}: {got}", g.len(), w.len()));
        }
        if opened() != ["https://raea.io/"] {
            return Err(format!("os.open was asked for {:?}", opened()));
        }
        match std::fs::read_to_string(self.dir.join("b.txt")) {
            Ok(t) if t == "xyz" => Ok(()),
            r => Err(format!("b.txt after fs.write: {r:?}")),
        }
    }
}
