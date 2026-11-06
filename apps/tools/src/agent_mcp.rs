//! The MCP servers `raea skills install` offers an agent (docs/skills.feature SK11): today `blender`, which drives a
//! running Blender through the MCP for Blender add-on (`raea setup blender` installs it). Each agent keeps MCP servers
//! in its own place; a server already registered under the name is left as the person set it.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The blender MCP server: uv runs it, so nothing is installed until an agent starts it.
const NAME: &str = "blender";
const CMD: &str = "uvx";
const ARGS: [&str; 1] = ["mcp-for-blender"];

type R<T> = Result<T, String>;

/// The person's home folder (HOME, or USERPROFILE on Windows).
pub fn user_home() -> R<PathBuf> {
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).ok_or_else(|| "no home folder (HOME)".into())
}

/// The JSON file holding `mcpServers` for agents configured that way (Kimi Code, Gemini CLI, Claude Code's own file).
fn json_file(agent: &str) -> R<Option<PathBuf>> {
    let h = user_home()?;
    Ok(match agent {
        "claude" => Some(h.join(".claude.json")),
        "kimi" => Some(std::env::var_os("KIMI_CODE_HOME").filter(|v| !v.is_empty()).map(PathBuf::from).unwrap_or_else(|| h.join(".kimi-code")).join("mcp.json")),
        "gemini" => Some(h.join(".gemini/settings.json")),
        _ => None,
    })
}

/// Adds `blender` to a JSON config's `mcpServers` unless one is there; true when it added.
pub fn add_json(path: &Path) -> R<bool> {
    let mut doc: Value = match std::fs::read_to_string(path) {
        Ok(t) if !t.trim().is_empty() => serde_json::from_str(&t).map_err(|e| format!("{}: {e}", path.display()))?,
        _ => json!({}),
    };
    let servers = doc.as_object_mut().ok_or_else(|| format!("{} isn't a JSON object", path.display()))?.entry("mcpServers").or_insert_with(|| json!({}));
    let servers = servers.as_object_mut().ok_or_else(|| format!("{}: mcpServers isn't an object", path.display()))?;
    if servers.contains_key(NAME) {
        return Ok(false);
    }
    servers.insert(NAME.into(), json!({"command": CMD, "args": ARGS}));
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let tmp = path.with_extension("raea-tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(&doc).unwrap_or_default() + "\n").map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(true)
}

/// Adds a `[mcp_servers.blender]` table to Codex's config.toml unless one is there; true when it added.
pub fn add_toml(path: &Path) -> R<bool> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    if text.lines().any(|l| l.trim() == format!("[mcp_servers.{NAME}]")) {
        return Ok(false);
    }
    let args = ARGS.iter().map(|a| format!("{a:?}")).collect::<Vec<_>>().join(", ");
    let sep = if text.is_empty() || text.ends_with("\n\n") { "" } else if text.ends_with('\n') { "\n" } else { "\n\n" };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, format!("{text}{sep}[mcp_servers.{NAME}]\ncommand = {CMD:?}\nargs = [{args}]\n")).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(true)
}

/// Registers the blender MCP server for an agent; the line says what happened. Claude Code rewrites ~/.claude.json while
/// it runs, so its own `claude mcp add` is used when `claude` is on PATH.
pub fn offer(agent: &str) -> R<String> {
    let line = |added: bool, at: &Path| if added { format!("{}: MCP server {NAME} added", at.display()) } else { format!("{}: MCP server {NAME} already there", at.display()) };
    if agent == "codex" {
        let p = user_home()?.join(".codex/config.toml");
        return Ok(line(add_toml(&p)?, &p));
    }
    let p = json_file(agent)?.ok_or_else(|| format!("no agent {agent:?}"))?;
    if agent == "claude" {
        let listed = Command::new("claude").args(["mcp", "get", NAME]).output();
        match listed {
            Ok(o) if o.status.success() => return Ok(line(false, &p)),
            Ok(_) => {
                let o = Command::new("claude").args(["mcp", "add", "--scope", "user", NAME, "--", CMD]).args(ARGS).output().map_err(|e| e.to_string())?;
                if !o.status.success() {
                    return Err(format!("claude mcp add {NAME}: {}", String::from_utf8_lossy(&o.stderr).trim()));
                }
                return Ok(line(true, &p));
            }
            Err(_) => {} // no claude on PATH: write its file
        }
    }
    Ok(line(add_json(&p)?, &p))
}

/// A hook entry raea added (its command is `raea hook ...`).
fn ours(hook: &Value) -> bool {
    hook["command"].as_str().unwrap_or("").starts_with("raea hook")
}

/// Claude Code's PostToolUse hook running `raea hook claude` after a file is written, in its settings.json, unless one
/// is there; true when it added. Its findings reach the agent as the hook's additionalContext.
pub fn claude_hook(settings: &Path) -> R<bool> {
    let mut doc: Value = match std::fs::read_to_string(settings) {
        Ok(t) if !t.trim().is_empty() => serde_json::from_str(&t).map_err(|e| format!("{}: {e}", settings.display()))?,
        _ => json!({}),
    };
    let post = doc.as_object_mut().ok_or("settings.json isn't an object")?.entry("hooks").or_insert_with(|| json!({})).as_object_mut().ok_or("settings.json: hooks isn't an object")?.entry("PostToolUse").or_insert_with(|| json!([]));
    let list = post.as_array_mut().ok_or("settings.json: hooks.PostToolUse isn't a list")?;
    if list.iter().any(|m| m["hooks"].as_array().into_iter().flatten().any(ours)) {
        return Ok(false);
    }
    list.push(json!({"matcher": "Write|Edit|MultiEdit", "hooks": [{"type": "command", "command": "raea hook claude", "timeout": 60}]}));
    let tmp = settings.with_extension("raea-tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(&doc).unwrap_or_default() + "\n").map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, settings).map_err(|e| format!("{}: {e}", settings.display()))?;
    Ok(true)
}

/// A move's guide as a skill of its own, named for the move: its description from the guide's first paragraph.
pub fn pin(to: &Path, mv: &str, v: &str) -> R<String> {
    let guide = raea_io_home::home().join("sdk/guides/craft/verbs").join(format!("{mv}.md"));
    let no = format!("no move {mv:?}: `raea guide` lists them");
    if mv.is_empty() || !mv.chars().all(|c| c.is_ascii_lowercase()) {
        return Err(no);
    }
    let text = std::fs::read_to_string(&guide).map_err(|_| no)?;
    let first = text.split("\n\n").nth(1).unwrap_or_default().replace('\n', " ");
    let md = format!("---\nname: {mv}\ndescription: raea's {mv} move, for a note on a raea piece. {}\nversion: {v}\n---\n\n# {mv}\n\nRun `raea guide {mv}` and follow it on the part or moment the note names. Answer the person in their words, never with the move's name. The raea skill has the rest.\n", first.replace(':', " -"));
    let dir = to.join(mv);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    std::fs::write(dir.join("SKILL.md"), md).map_err(|e| e.to_string())?;
    Ok(format!("{}: /{mv} pinned\n", dir.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_once_and_keeps_what_was_there() {
        let d = std::env::temp_dir().join(format!("raea-agent-mcp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let j = d.join("kimi/mcp.json");
        assert!(add_json(&j).unwrap());
        assert!(!add_json(&j).unwrap());
        std::fs::write(&j, r#"{"theme":"dark","mcpServers":{"blender":{"command":"mine"}}}"#).unwrap();
        assert!(!add_json(&j).unwrap());
        let kept: Value = serde_json::from_str(&std::fs::read_to_string(&j).unwrap()).unwrap();
        assert_eq!(kept["mcpServers"]["blender"]["command"], "mine");
        let t = d.join("codex/config.toml");
        std::fs::create_dir_all(t.parent().unwrap()).unwrap();
        std::fs::write(&t, "model = \"x\"\n").unwrap();
        assert!(add_toml(&t).unwrap());
        assert!(!add_toml(&t).unwrap());
        assert_eq!(std::fs::read_to_string(&t).unwrap(), "model = \"x\"\n\n[mcp_servers.blender]\ncommand = \"uvx\"\nargs = [\"mcp-for-blender\"]\n");
        let h = d.join("claude/settings.json");
        std::fs::create_dir_all(h.parent().unwrap()).unwrap();
        std::fs::write(&h, r#"{"model":"x","hooks":{"PostToolUse":[{"matcher":"Bash","hooks":[{"type":"command","command":"other"}]}]}}"#).unwrap();
        assert!(claude_hook(&h).unwrap());
        assert!(!claude_hook(&h).unwrap());
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&h).unwrap()).unwrap();
        assert_eq!(v["hooks"]["PostToolUse"].as_array().unwrap().len(), 2);
        assert_eq!(v["model"], "x");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn a_pinned_move_is_a_skill_of_its_own() {
        let d = std::env::temp_dir().join(format!("raea-pin-{}", std::process::id()));
        assert!(pin(&d, "quieter", "9.9").unwrap().contains("/quieter pinned"));
        let md = std::fs::read_to_string(d.join("quieter/SKILL.md")).unwrap();
        assert!(md.starts_with("---\nname: quieter\ndescription: raea's quieter move") && md.contains("\nversion: 9.9\n---\n") && md.contains("raea guide quieter"), "{md}");
        assert!(pin(&d, "../x", "1").is_err() && pin(&d, "nomove", "1").is_err());
        let _ = std::fs::remove_dir_all(&d);
    }
}
