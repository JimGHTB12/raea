//! T0a: an app's `export const app`, read without running it.

use raea_app_targets::config::{Target, parse};

#[test]
fn no_export_or_no_target_is_web() {
    let c = parse("export default function App() { return <div /> }").unwrap();
    assert_eq!((c.target, c.declared), (Target::Web, false));
    let c = parse("export const app = { title: 'Notes' };").unwrap();
    assert_eq!((c.target, c.declared, c.title.as_deref()), (Target::Web, true, Some("Notes")));
}

#[test]
fn a_literal_is_read_with_its_type_and_comments() {
    let src = r#"
import type { AppConfig } from "@raea/ui/app";
// the ad's app, a phone's width
export const app: AppConfig = {
  target: "native", /* Rune */
  title: "Tally — split it",
  size: [1080, 1920],
  minSize: [420, 400,],
} satisfies AppConfig;
export default function Tally() { return null }
"#;
    let c = parse(src).unwrap();
    assert_eq!(c.target, Target::Native);
    assert_eq!(c.title.as_deref(), Some("Tally — split it"));
    assert_eq!((c.size, c.min_size), (Some((1080.0, 1920.0)), Some((420.0, 400.0))));
}

#[test]
fn anything_but_a_literal_is_refused_at_its_line() {
    for (src, line, says) in [
        ("const t = 'native';\nexport const app = { target: t };", 2, "`t` isn't a literal"),
        ("\n\nexport const app = makeApp();", 3, "`makeApp` isn't a literal"),
        ("export const app = { ...base, title: 'x' };", 1, "spread"),
        ("export const app = {\n  title: `T ${n}`,\n};", 2, "template"),
        ("const title = 'x';\nexport const app = { title };", 2, "shorthand"),
        ("export const app = { target: 'mobile' };", 1, "\"web\" or \"native\""),
        ("export const app = { targt: 'native' };", 1, "no field \"targt\""),
        ("export const app = { size: [0, 10] };", 1, "two positive numbers"),
        ("const app = {};\n\nexport { app };", 3, "not as `export const app"),
        ("export let app = { target: 'native' };", 1, "not as `export const app"),
    ] {
        let e = parse(src).expect_err(src);
        assert_eq!(e.line, line, "{src}: {}", e.why);
        assert!(e.why.contains(says), "{src}: {}", e.why);
    }
}

#[test]
fn a_comment_or_a_string_is_not_an_export() {
    let c = parse("// export const app = { target: 'native' }\nconst s = \"export const app = 1\";").unwrap();
    assert!(!c.declared);
}

#[test]
fn the_command_line_overrides_the_file() {
    let dir = std::env::temp_dir().join(format!("raea-target-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("app.tsx");
    std::fs::write(&f, "export const app = { target: 'native' };").unwrap();
    assert_eq!(raea_app_targets::read(&f, None).unwrap().target, Target::Native);
    assert_eq!(raea_app_targets::read(&f, Some(Target::Web)).unwrap().target, Target::Web);
    std::fs::write(&f, "\nexport const app = { target: t };").unwrap();
    let e = raea_app_targets::read(&f, None).unwrap_err();
    assert!(e.starts_with(&format!("{}:2: ", f.display())), "{e}");
    std::fs::remove_dir_all(&dir).unwrap();
}
