//! What a refused render changed since the last refusal of the same file. A writer fixing one error often rewrites
//! more than it names and breaks something else; the next refusal says which lines changed and which of them are away
//! from the error it was fixing, so it can put those back (whoever writes: an agent over ACP or the feed's own loop).

use std::path::{Path, PathBuf};

/// lines this close to the last error count as its fix
const NEAR: usize = 3;

/// The file `raea render FILE` was given, when it's a composition's source
fn source_of(args: &[String]) -> Option<PathBuf> {
    if args.first().map(String::as_str) != Some("render") {
        return None;
    }
    let f = Path::new(args.get(1)?);
    let ext = f.extension()?.to_str()?;
    (matches!(ext, "tsx" | "ts" | "jsx" | "js") && f.is_file()).then(|| f.canonicalize().unwrap_or_else(|_| f.to_path_buf()))
}

/// raea's caches' refused/HASH.json (raea_io_home::caches) for that file (RAEA_REFUSED_CACHE moves it; "0" turns it off)
fn kept(file: &Path) -> Option<PathBuf> {
    let dir = match std::env::var("RAEA_REFUSED_CACHE") {
        Ok(d) if d == "0" => return None,
        Ok(d) => PathBuf::from(d),
        Err(_) => raea_io_home::caches()?.join("refused"),
    };
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in file.to_string_lossy().bytes() {
        h = (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3);
    }
    Some(dir.join(format!("{h:016x}.json")))
}

/// The refusal `error` of `raea render FILE` with what changed since the file was last refused, and this one kept
/// for the next
pub fn note(args: &[String], error: &str) -> String {
    let Some(file) = source_of(args) else { return String::new() };
    let Some(at) = kept(&file) else { return String::new() };
    let Ok(now) = std::fs::read_to_string(&file) else { return String::new() };
    let said = std::fs::read_to_string(&at).ok().and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()).and_then(|v| {
        Some((v.get("source")?.as_str()?.to_string(), v.get("error")?.as_str()?.to_string()))
    });
    let _ = std::fs::create_dir_all(at.parent().unwrap_or(Path::new(".")));
    let _ = std::fs::write(&at, serde_json::json!({"source": now, "error": error}).to_string());
    match said {
        Some((before, last)) => since(&before, &last, &now),
        None => String::new(),
    }
}

/// A render that worked: the file's last refusal is over
pub fn clear(args: &[String]) {
    if let Some(at) = source_of(args).as_deref().and_then(kept) {
        let _ = std::fs::remove_file(at);
    }
}

/// What changed from `before` (refused with `last`) to `now`, in `now`'s line numbers
fn since(before: &str, last: &str, now: &str) -> String {
    let first = last.lines().next().unwrap_or("");
    let first = if first.chars().count() > 160 { format!("{}…", first.chars().take(160).collect::<String>()) } else { first.to_string() };
    let (old, new): (Vec<&str>, Vec<&str>) = (before.lines().collect(), now.lines().collect());
    let Some(hunks) = hunks(&old, &new) else { return String::new() };
    if hunks.is_empty() {
        return format!("\nThe file is unchanged since raea last refused it ({first}).");
    }
    let lines = |h: &Hunk| match h.new.1 - h.new.0 {
        0 => format!("removed after line {}", h.new.0),
        1 => format!("{}", h.new.0 + 1),
        _ => format!("{}-{}", h.new.0 + 1, h.new.1),
    };
    let all = hunks.iter().map(lines).collect::<Vec<_>>().join(", ");
    let mut s = format!("\nSince raea last refused this file ({first}), these lines changed: {all}.");
    let near = where_(last, &old);
    if near.is_empty() {
        return s;
    }
    let away: Vec<String> = hunks.iter().filter(|h| !near.iter().any(|&l| h.old.0 <= l + NEAR && l < h.old.1 + NEAR)).map(lines).collect();
    if !away.is_empty() {
        let shown = near.iter().map(|l| (l + 1).to_string()).collect::<Vec<_>>().join(", ");
        s += &format!(" That error was at line {shown} (before your change); {} away from it. If this error is in a change the last one didn't ask for, put those lines back as they were.", if away.len() == 1 { format!("{} is", away[0]) } else { format!("{} are", away.join(", ")) });
    }
    s
}

/// The 0-based lines of `old` the error points at: its "at line N", else the lines holding a "quoted" bit of it or one
/// of the [keyframe, lists] it shows (interpolate's)
fn where_(error: &str, old: &[&str]) -> Vec<usize> {
    if let Some(n) = error.split(" at line ").nth(1).and_then(|r| r.split(|c: char| !c.is_ascii_digit()).next()).and_then(|n| n.parse::<usize>().ok()) {
        return vec![n.saturating_sub(1)];
    }
    let lists = error.split('[').skip(1).filter_map(|r| r.split_once(']')).map(|(l, _)| l).filter(|l| l.contains(','));
    let quoted: Vec<String> = error.split('"').skip(1).step_by(2).filter(|q| q.len() >= 3).map(str::to_string).chain(lists.map(|l| format!("[{l}]"))).collect();
    let hit: Vec<usize> = (0..old.len()).filter(|&i| quoted.iter().any(|q| old[i].contains(q.as_str()))).collect();
    if hit.len() <= 3 { hit } else { Vec::new() }
}

/// A changed stretch: lines old.0..old.1 became new.0..new.1
#[derive(Debug, PartialEq)]
struct Hunk {
    old: (usize, usize),
    new: (usize, usize),
}

/// The changed stretches between two files' lines (a longest common subsequence; None past ~16M cells)
fn hunks(old: &[&str], new: &[&str]) -> Option<Vec<Hunk>> {
    let (n, m) = (old.len(), new.len());
    if n.saturating_mul(m) > 1 << 24 {
        return None;
    }
    // lcs[i][j]: the common lines of old[i..] and new[j..]
    let mut lcs = vec![0u32; (n + 1) * (m + 1)];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i * (m + 1) + j] = if old[i] == new[j] { lcs[(i + 1) * (m + 1) + j + 1] + 1 } else { lcs[(i + 1) * (m + 1) + j].max(lcs[i * (m + 1) + j + 1]) };
        }
    }
    let (mut i, mut j, mut out) = (0, 0, Vec::new());
    while i < n || j < m {
        if i < n && j < m && old[i] == new[j] {
            i += 1;
            j += 1;
            continue;
        }
        let (oi, nj) = (i, j);
        while (i < n || j < m) && !(i < n && j < m && old[i] == new[j]) {
            if j < m && (i == n || lcs[i * (m + 1) + j + 1] >= lcs[(i + 1) * (m + 1) + j]) {
                j += 1;
            } else {
                i += 1;
            }
        }
        out.push(Hunk { old: (oi, i), new: (nj, j) });
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BEFORE: &str = "a\nb\nc = interpolate(f, [0, 30], [0, undefined])\nd\ne\nf\ng\nh\ni\nj\n";

    #[test]
    fn finds_the_changed_stretches() {
        let old: Vec<&str> = "a\nb\nc\nd".lines().collect();
        let new: Vec<&str> = "a\nB\nc\nd\ne".lines().collect();
        assert_eq!(hunks(&old, &new).unwrap(), vec![Hunk { old: (1, 2), new: (1, 2) }, Hunk { old: (4, 4), new: (4, 5) }]);
    }

    #[test]
    fn names_changes_away_from_the_error() {
        let now = "a\nb\nc = interpolate(f, [0, 30], [0, 1])\nd\ne\nf\ng\nh\nI\nj\n";
        let s = since(BEFORE, "interpolate: output[1] is not a number (undefined) in interpolate(…, [0, 30], [0, undefined])", now);
        assert!(s.contains("these lines changed: 3, 9."), "{s}");
        assert!(s.contains("That error was at line 3 (before your change); 9 is away from it."), "{s}");
    }

    #[test]
    fn a_fix_only_where_the_error_was_says_no_more() {
        let now = "a\nb\nc = interpolate(f, [0, 30], [0, 1])\nd\ne\nf\ng\nh\ni\nj\n";
        let s = since(BEFORE, "Expected `>` but found `<` at line 3, column 9:", now);
        assert!(s.contains("these lines changed: 3.") && !s.contains("away"), "{s}");
    }

    #[test]
    fn says_when_nothing_changed() {
        assert!(since(BEFORE, "boom", BEFORE).contains("unchanged since raea last refused it (boom)"));
    }
}
