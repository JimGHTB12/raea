use super::*;

#[test]
fn a_name_is_its_folder() {
    assert_eq!(slug("Shane's Studio"), "shane-s-studio");
    assert_eq!(slug("  CJ / Reel 2 "), "cj-reel-2");
}

#[test]
fn a_spec_module_is_tidy_typescript() {
    // keys unquoted where they can be, nulls and empty lists left out, whole numbers whole, short string lists inline
    let v = json!({"id": "delivery", "help": null, "tags": [], "n": 45.0, "options": ["9:16", "16:9"], "odd-key": 1});
    assert_eq!(ts_literal(&v, 0), "{\n  id: \"delivery\",\n  n: 45,\n  \"odd-key\": 1,\n  options: [\"9:16\", \"16:9\"],\n}");
}

#[test]
fn up_finds_the_nearest_marker() {
    let root = std::env::temp_dir().canonicalize().unwrap().join(format!("raea-ws-up-{}", std::process::id()));
    let deep = root.join("projects/launch/media");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::create_dir_all(root.join(".raea")).unwrap();
    std::fs::write(root.join(".raea/workspace"), "").unwrap();
    assert_eq!(up(&deep, ".raea/workspace").as_deref(), Some(root.as_path()));
    assert_eq!(up(&deep, ".raea/project"), None);
    let _ = std::fs::remove_dir_all(&root);
}

fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("raea-ws-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn next_walks_a_project_from_brief_to_show() {
    let d = scratch("next");
    let shaped = next::Brief { needs: vec![], modes: vec!["tool".into()] };
    let unshaped = next::Brief { needs: vec!["who it's for (audience)".into()], modes: vec!["pitch".into()] };
    let s = next::steps(&d, &unshaped, 0);
    assert!(s[0].starts_with("shape the brief: who it's for") && s[1].starts_with("plan it") && s[2].starts_with("make the first item"), "{s:?}");
    assert!(next::steps(&d, &shaped, 2)[0].starts_with("raea doctor: 2"));
    std::fs::write(d.join("piece.tsx"), "registerRoot(() => null)").unwrap();
    assert!(next::steps(&d, &shaped, 0)[0].starts_with("render piece.tsx"));
    std::fs::create_dir_all(d.join("out")).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(d.join("out/piece.mp4"), "").unwrap();
    assert!(next::steps(&d, &shaped, 0)[0].starts_with("check piece.mp4"));
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(d.join("out/piece.check.json"), r#"{"issues":[{"p":"P1","says":"text clipped at 2 s"}]}"#).unwrap();
    assert_eq!(next::steps(&d, &shaped, 0)[0], "polish piece.mp4: P1 text clipped at 2 s (`raea guide polish`)");
    std::fs::write(d.join("out/piece.check.json"), r#"{"issues":[{"p":"P3","says":"minor"}]}"#).unwrap();
    assert!(next::steps(&d, &shaped, 0)[0].starts_with("show piece.mp4"));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn doctor_finds_old_briefs_and_old_skills() {
    let d = scratch("doctor");
    std::fs::write(d.join("acme.brief"), "title: x").unwrap();
    std::fs::create_dir_all(d.join("skills/raea")).unwrap();
    std::fs::write(d.join("skills/raea/SKILL.md"), "---\nname: raea\nversion: 0.0.1\n---\n").unwrap();
    let skills = [("claude", d.join("skills"))];
    let found = doctor::findings(&d, &skills, "0.0.2");
    assert_eq!(found.len(), 2, "{:?}", found.iter().map(|f| &f.says).collect::<Vec<_>>());
    assert!(found[0].says.contains("acme.brief") && !found[0].fixable());
    assert!(found[1].says.contains("is 0.0.1, this raea is 0.0.2") && found[1].fixable());
    assert!(doctor::findings(&d, &skills, "0.0.1").len() == 1);
    let reinstalled = std::cell::Cell::new(false);
    let out = doctor::doctor(&["--fix".into(), d.display().to_string()], &skills, "0.0.2", &|a| { reinstalled.set(a == "claude"); Ok(String::new()) }).unwrap();
    assert!(reinstalled.get() && out.starts_with("APPLIED  1 of 2 fixed"), "{out}");
    let _ = std::fs::remove_dir_all(&d);
}
