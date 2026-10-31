//! styles.gen.ts's Tailwind compile on whichever JS engine the build has (docs/ports.feature PT2): JavaScriptCore on a
//! Mac, QuickJS on Linux or with `raea-native/portable`, the same CSS from each. RAEA_CSS_OUT=FILE writes the
//! file there, to compare one machine's against another's.

#![cfg(feature = "modules")]

#[test]
fn tailwind_compiles_the_classes_used() {
    let home = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if !home.join("sdk/react/tw/node_modules/tailwindcss").is_dir() {
        eprintln!("skipped: no Tailwind package (`bun install` in sdk/react/tw)");
        return;
    }
    // SAFETY: set before anything reads it, on the test's own thread
    unsafe { std::env::set_var("RAEA_HOME", &home) };
    let dir = std::env::temp_dir().join(format!("raea-styles-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("edit.tsx"), "import { css } from \"./styles.gen\";\nexport default () => <div className=\"flex p-4 text-red-500 hover:underline\" />;\n").unwrap();
    let made = raea_runtime_bundle::styles::ensure(&dir);
    let written = std::fs::read_to_string(dir.join("styles.gen.ts"));
    let _ = std::fs::remove_dir_all(&dir);
    made.unwrap();
    let written = written.unwrap();
    for class in [".flex", ".p-4", ".text-red-500", ".hover\\\\:underline"] {
        assert!(written.contains(class), "{class} missing from styles.gen.ts");
    }
    if let Some(out) = std::env::var_os("RAEA_CSS_OUT") {
        std::fs::write(out, &written).unwrap();
    }
}
