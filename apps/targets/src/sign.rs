//! A Mac app signed by its developer (docs/targets.feature T1d), on either target: `--identity NAME` (or
//! RAEA_SIGN_IDENTITY) is a codesign identity, e.g. "Developer ID Application: NAME (TEAM)", found in RAEA_SIGN_KEYCHAIN
//! when that's set (the caller unlocks it); `--notarize PROFILE` is a notarytool keychain profile (`xcrun notarytool
//! store-credentials PROFILE ...`, once). With an identity, each Mach-O inside the app and then the app are signed with
//! the hardened runtime, a timestamp and JavaScriptCore's JIT entitlement; with none it's signed ad hoc, and says so.
//! Nothing here names any one developer: raea's own builds pass theirs by the environment.

use std::path::{Path, PathBuf};
use std::process::Command;

/// How the app is signed, from the flags and the environment
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Signing {
    pub identity: Option<String>,
    pub notarize: Option<String>,
}

impl Signing {
    /// `--identity` over RAEA_SIGN_IDENTITY; `--notarize PROFILE` as given
    pub fn new(identity: Option<String>, notarize: Option<String>) -> Self {
        let env = std::env::var("RAEA_SIGN_IDENTITY").ok().filter(|v| !v.trim().is_empty());
        Signing { identity: identity.or(env), notarize }
    }
}

const ENTITLEMENTS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>com.apple.security.cs.allow-jit</key><true/>
</dict></plist>
"#;

fn run(cmd: &mut Command) -> Result<String, String> {
    let o = cmd.output().map_err(|e| format!("{:?}: {e}", cmd.get_program()))?;
    let text = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
    o.status.success().then(|| text.clone()).ok_or_else(|| format!("{} failed: {}", cmd.get_program().to_string_lossy(), text.trim()))
}

/// The Mach-O files under `dir` (by their magic), deepest first, so each is signed before what holds it
fn machos(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(machos(&p));
        } else if let Ok(b) = std::fs::read(&p).map(|b| b.get(..4).map(<[u8]>::to_vec).unwrap_or_default()) {
            if matches!(b.as_slice(), [0xcf, 0xfa, 0xed, 0xfe] | [0xca, 0xfe, 0xba, 0xbe]) {
                out.push(p);
            }
        }
    }
    out
}

/// `app` signed as `s` says, notarized and stapled when it names a profile: what each step said
pub fn sign(app: &Path, s: &Signing) -> Result<Vec<String>, String> {
    if !cfg!(target_os = "macos") {
        return Ok(vec!["not signed: codesign is the Mac's".into()]);
    }
    let Some(identity) = &s.identity else {
        if s.notarize.is_some() {
            return Err("--notarize needs a Developer ID: --identity NAME or RAEA_SIGN_IDENTITY".into());
        }
        run(Command::new("codesign").args(["--force", "--deep", "--sign", "-"]).arg(app))?;
        return Ok(vec!["signed ad hoc (no --identity or RAEA_SIGN_IDENTITY): it opens on the Mac that made it".into()]);
    };
    let ent = std::env::temp_dir().join(format!("raea-entitlements-{}.plist", std::process::id()));
    std::fs::write(&ent, ENTITLEMENTS).map_err(|e| e.to_string())?;
    let keychain = std::env::var("RAEA_SIGN_KEYCHAIN").ok().filter(|v| !v.trim().is_empty());
    let one = |p: &Path| {
        let mut c = Command::new("codesign");
        c.args(["--force", "--options", "runtime", "--timestamp", "--entitlements"]).arg(&ent).args(["-s", identity]);
        if let Some(k) = &keychain {
            c.args(["--keychain", k]);
        }
        run(c.arg(p))
    };
    let signed = machos(&app.join("Contents")).iter().try_for_each(|p| one(p).map(drop)).and_then(|_| one(app));
    let _ = std::fs::remove_file(&ent);
    signed?;
    let mut said = vec![format!("codesign --verify --deep --strict: {}", run(Command::new("codesign").args(["--verify", "--deep", "--strict", "--verbose=2"]).arg(app))?.trim())];
    if let Some(profile) = &s.notarize {
        said.push(notarized(app, profile)?);
        said.push(format!("spctl: {}", run(Command::new("spctl").args(["--assess", "--type", "execute", "--verbose=2"]).arg(app))?.trim()));
    }
    Ok(said)
}

/// `app` sent to Apple's notary service with the notarytool keychain profile `profile`, and its ticket stapled
fn notarized(app: &Path, profile: &str) -> Result<String, String> {
    let zip = std::env::temp_dir().join(format!("raea-notarize-{}.zip", std::process::id()));
    run(Command::new("ditto").args(["-c", "-k", "--keepParent"]).arg(app).arg(&zip))?;
    let r = run(Command::new("xcrun").args(["notarytool", "submit"]).arg(&zip).args(["--keychain-profile", profile, "--wait"]));
    let _ = std::fs::remove_file(&zip);
    let r = r?;
    if !r.contains("status: Accepted") {
        return Err(format!("notarytool: {}", r.trim()));
    }
    run(Command::new("xcrun").args(["stapler", "staple"]).arg(app))?;
    Ok("notarytool: Accepted, stapled".into())
}
