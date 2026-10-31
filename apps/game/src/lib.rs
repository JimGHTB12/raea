//! Exported games (sdk/react/docs/game-export.md): a game made into a program that plays on its own, outside the feed and the
//! dev tree.
//!
//!   raea export-game EDIT.tsx -o OUT [--target mac-app|mac-cli] [--name NAME] [--icon PNG] [--keys SETS]
//!
//! bundles the game for live play's runtime ahead of time (bun runs here, never in the export), gathers every file the
//! bundle names (media, staticFile paths, glTF models and what they name, generated sounds, the brand's CSS and its
//! fonts) into the package with paths relative to it, works out what play would work out at open (the composition's
//! size and rate, its 3D, its sound master) and puts the player beside it: a macOS .app (Contents/MacOS, Resources/game,
//! Info.plist) or a folder (the player and game/), signed ad hoc.
//!
//! The player is `raea-game` (src/bin/raea-game.rs, feature `game`): it opens a window and plays the package's game at
//! its rate with keys and pointer, its sound on the one mixer core and its 3D on raea-render's engine. F11, Cmd+F or Ctrl+Cmd+F
//! toggles fullscreen; Cmd+Q or Cmd+W quits.
//!
//!   raea-game [--game DIR]                                     play (the package's own game by default)
//!   raea-game --selftest 5s [--snapshot S OUT.png]... [--input-log F] [--json F] [--keys SETS]
//!
//! --selftest plays offscreen (no window, nothing shown, the sound mixed and kept silent) with scripted keys and pointer,
//! then prints what it measured: frames presented and missed, input changes stepped into the game, sound mixed (peak and
//! RMS of the mixer's own output) and panics. Its last line is PASS or FAIL.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use raea_runtime_react::scene::flag;
use raea_runtime_live::Live;

mod export;
use export::*;
pub use export::{USAGE, export};
mod gather;
use gather::*;
mod player;
use player::*;
pub use player::player_main;
mod window;
use window::*;
#[cfg(test)]
mod tests;

type R<T> = Result<T, String>;

/// The package's manifest, beside the bundle.
const MANIFEST: &str = "game.json";
/// The page's fonts, packaged at export (Live::packaged_fonts_css): what the player reads in place of fetching any
const FONTS: &str = "fonts.css";

/// The packaged font sheet with each inlined face written once to fonts/ and named there: a page's family stack (Inter,
/// Segoe UI, Roboto, Noto Sans ...) inlines the same face under every name, 16 copies of 8 faces in pong (lightspeed R7).
/// fonts_in puts the same data URLs back, quoted as they were, so the page reads the sheet it was exported with.
fn fonts_out(css: &str, game: &Path) -> R<String> {
    use base64::Engine as _;
    let (mut out, mut rest) = (String::with_capacity(css.len() / 8), css);
    while let Some(i) = rest.find("url(") {
        out.push_str(&rest[..i + 4]);
        let after = &rest[i + 4..];
        let q = if after.starts_with('"') { "\"" } else { "" };
        let body = &after[q.len()..];
        let parsed = body.strip_prefix("data:").and_then(|d| {
            let m = d.find(";base64,")?;
            let end = d[m + 8..].find(|c: char| c == '"' || c == ')')? + m + 8;
            Some((&d[..m], &d[m + 8..end], q.len() + 5 + end))
        });
        let Some((mime, b64, end)) = parsed else {
            rest = after;
            continue;
        };
        let ext = match mime {
            "font/woff2" => "woff2",
            "font/woff" => "woff",
            "font/otf" => "otf",
            _ => "ttf",
        };
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).map_err(|e| format!("fonts.css: {e}"))?;
        // FNV-1a of the face: the same face under another family is the same file
        let h = bytes.iter().fold(0xcbf29ce484222325u64, |h, b| (h ^ *b as u64).wrapping_mul(0x100000001b3));
        let rel = format!("fonts/{h:016x}.{ext}");
        let to = game.join(&rel);
        if !to.exists() {
            std::fs::create_dir_all(game.join("fonts")).map_err(|e| e.to_string())?;
            std::fs::write(&to, &bytes).map_err(|e| format!("{}: {e}", to.display()))?;
        }
        out.push_str(&format!("{q}{rel}"));
        rest = &after[end..];
    }
    out.push_str(rest);
    Ok(out)
}

/// fonts_out undone: each url("fonts/NAME") or url(fonts/NAME) in the sheet as the data URL it was exported from.
fn fonts_in(css: &str, dir: &Path) -> String {
    use base64::Engine as _;
    let (mut out, mut rest) = (String::with_capacity(css.len() * 4), css);
    while let Some(i) = rest.find("url(") {
        out.push_str(&rest[..i + 4]);
        let after = &rest[i + 4..];
        let q = if after.starts_with('"') { 1 } else { 0 };
        let body = &after[q..];
        let named = body.starts_with("fonts/").then(|| body.find(|c: char| c == '"' || c == ')')).flatten();
        let Some(close) = named else {
            rest = after;
            continue;
        };
        let rel = &body[..close];
        let mime = match rel.rsplit('.').next() {
            Some("woff2") => "font/woff2",
            Some("woff") => "font/woff",
            Some("otf") => "font/otf",
            _ => "font/ttf",
        };
        out.push_str(&after[..q]);
        match std::fs::read(dir.join(rel)) {
            Ok(b) => out.push_str(&format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(&b))),
            Err(_) => out.push_str(rel),
        }
        rest = &body[close..];
    }
    out.push_str(rest);
    out
}
