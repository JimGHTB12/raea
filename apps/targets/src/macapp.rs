//! `raea app bundle` on the web target: a Mac app that opens the page in raea-page-host's window. App.app/Contents has
//! the page host, the app's script and stylesheet in Resources/, and a launcher in MacOS/ that starts the host on them
//! at the app's title and size, signed by the developer (sign.rs, T1d).

use std::path::PathBuf;

use crate::verbs::Ask;

fn plist(name: &str, id: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>{name}</string>
  <key>CFBundleDisplayName</key><string>{name}</string>
  <key>CFBundleIdentifier</key><string>{id}</string>
  <key>CFBundleExecutable</key><string>{name}</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>1.0</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
"#,
        name = xml(name)
    )
}

fn xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// A shell word, quoted
fn sh(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// The bundle id from the title: ascii letters and digits, under app.raea.local
fn bundle_id(title: &str) -> String {
    let id: String = title.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_lowercase();
    format!("local.raea.app.{}", if id.is_empty() { "app" } else { &id })
}

pub fn web(a: &Ask) -> Result<String, String> {
    let out = a.out.clone().unwrap_or_else(|| PathBuf::from(format!("{}.app", crate::verbs::stem(&a.app))));
    if out.exists() {
        std::fs::remove_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    let contents = out.join("Contents");
    let (macos, res) = (contents.join("MacOS"), contents.join("Resources"));
    for d in [&macos, &res] {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    crate::web::build(&a.app, &res)?;
    let host = crate::web::bin()?;
    std::fs::copy(&host, res.join("raea-page-host")).map_err(|e| format!("{}: {e}", host.display()))?;
    let name = &a.title;
    std::fs::write(contents.join("Info.plist"), plist(name, &bundle_id(name))).map_err(|e| e.to_string())?;
    let launcher = macos.join(name);
    let script = format!(
        "#!/bin/sh\n# made by raea app bundle: the page in raea-page-host's window\nR=\"$(cd \"$(dirname \"$0\")/../Resources\" && pwd)\"\nexec \"$R/raea-page-host\" \"$R/app.js\" --css \"$R/app.css\" --window --title {} --size {}x{}\n",
        sh(name),
        a.size.0,
        a.size.1
    );
    std::fs::write(&launcher, script).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&launcher, std::fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
    }
    let mut said = crate::sign::sign(&out, &a.sign)?;
    said.push(format!("{} (web)", out.display()));
    Ok(said.join("\n"))
}
