//! `raea check VIDEO.mp4 [--manifest render.json] [--words W.json] [--perception P.json] [--shape 9:16]
//! [--platform tiktok|reels|shorts] [-o check.json]`: the floor's measurable lines on an export (docs/craft-system.md, I; sdk/guides/craft/floor.md).
//! Each issue is ranked P0 (breaks the piece) to P3 (polish), with where it is and what to do. Rules on the file alone:
//! loudness and true peak, a steady sound that stops dead (a source run out, a cut with no fade), stretches where nothing
//! changes, encoder blotching in dark areas. With the transcript (`--words`): a stretch with no voice louder than the
//! voice, and a bed under the voice closer than 12 dB to it. With the render's
//! manifest (`raea render … --manifest render.json`): text clipped off the frame, text in the platform's UI zones,
//! text held too briefly to read, caption contrast against the pixels behind it. With the transcript too: captions
//! that come on away from the word they carry. With the face's place (`--perception`, the perception.json `raea cut`
//! writes): text over more than 8% of the face. With `--taste FILE|DIR`: the person's own mechanical rules, from the
//! `## Checks` section of their taste briefs (`- text-above 62%: Captions sit clear of the bottom.`), each a P1 when
//! broken, since the person's rule beats the floor's default (docs/craft-system.md, J). Exits 1 when a P0 or P1 is found.
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::process::Command;

mod sound;
use sound::*;
mod picture;
use picture::*;
mod text;
use text::*;
#[cfg(test)]
mod tests;

type R<T> = Result<T, String>;

#[derive(Clone)]
struct Issue {
    rule: &'static str,
    p: u8,
    at: Option<f64>,
    says: String,
}

pub fn cli(args: &[String]) -> i32 {
    match run(args) {
        Ok((report, bad)) => {
            println!("{report}");
            i32::from(bad)
        }
        Err(e) => {
            eprintln!("REFUSED  {e}");
            2
        }
    }
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(String::as_str)
}

pub const USAGE: &str = "raea check VIDEO.mp4 [--manifest render.json] [--words W.json] [--perception P.json] [--shape 16:9|9:16|4:5|1:1] [--platform tiktok|reels|shorts] [--taste FILE|DIR] [-o check.json]   (an export against the floor: clipped text, contrast, safe areas, freezes, cues, loudness, blotching, a placeholder brand kit; P0-P3)";

/// A rule from the person's taste: a form raea can measure, its value, and the taste line it stands for.
#[derive(Clone, Debug, PartialEq)]
struct Taste {
    form: String,
    value: f64,
    line: String,
}

/// the forms a taste check can take, and what each measures (apps/app/host/taste.ts offers the Director these)
pub const TASTE_FORMS: [(&str, &str); 6] = [
    ("text-hold", "S: every text holds at least S seconds"),
    ("text-above", "P%: every text's bottom edge sits above P% of the frame's height"),
    ("text-below", "P%: every text's top edge sits below P% of the frame's height"),
    ("text-margin", "N: every text keeps N px (at 1080 on the short side) from each edge"),
    ("loudness", "L: integrated loudness within 1 LU of L LUFS (instead of -14)"),
    ("still-max", "S: nothing holds still longer than S seconds (instead of 3)"),
];

/// The `## Checks` lines of a taste brief, or of every `.brief` in a folder. A form raea doesn't know is skipped and
/// named, so a hand-written line never breaks a check.
fn taste_rules(path: &str) -> R<(Vec<Taste>, Vec<String>)> {
    let p = std::path::Path::new(path);
    let files: Vec<std::path::PathBuf> = if p.is_dir() {
        let mut v: Vec<_> = std::fs::read_dir(p).map_err(|e| format!("{path}: {e}"))?.filter_map(|e| e.ok().map(|e| e.path())).filter(|f| f.extension().is_some_and(|x| x == "brief")).collect();
        v.sort();
        v
    } else {
        vec![p.to_path_buf()]
    };
    let (mut rules, mut skipped) = (Vec::new(), Vec::new());
    for f in files {
        let text = std::fs::read_to_string(&f).map_err(|e| format!("{}: {e}", f.display()))?;
        let (r, s) = parse_taste(&text);
        rules.extend(r);
        skipped.extend(s);
    }
    Ok((rules, skipped))
}

fn parse_taste(text: &str) -> (Vec<Taste>, Vec<String>) {
    let (mut rules, mut skipped, mut on) = (Vec::new(), Vec::new(), false);
    for l in text.lines() {
        let t = l.trim();
        if let Some(h) = t.strip_prefix("## ") {
            on = h.trim().eq_ignore_ascii_case("checks");
            continue;
        }
        let Some(item) = t.strip_prefix("- ").filter(|_| on) else { continue };
        let (rule, line) = item.split_once(':').map(|(a, b)| (a.trim(), b.trim())).unwrap_or((item, ""));
        let mut words = rule.split_whitespace();
        let form = words.next().unwrap_or("").to_lowercase();
        let value = words.next().map(|v| v.trim_end_matches(|c: char| c.is_alphabetic() || c == '%')).and_then(|v| v.parse::<f64>().ok());
        match value {
            Some(value) if TASTE_FORMS.iter().any(|(f, _)| *f == form) => rules.push(Taste { form, value, line: if line.is_empty() { rule.to_string() } else { line.to_string() } }),
            _ => skipped.push(rule.to_string()),
        }
    }
    (rules, skipped)
}

fn taste_of<'a>(rules: &'a [Taste], form: &str) -> Option<&'a Taste> {
    rules.iter().rev().find(|r| r.form == form)
}

/// the person's text rules against the render's text runs
fn taste_text(rules: &[Taste], runs: &[Run], w: f64, h: f64, issues: &mut Vec<Issue>, passed: &mut Vec<String>) {
    let s = w.min(h) / 1080.0;
    for r in rules {
        let mut broke = 0;
        for t in runs {
            let [x, y, bw, bh] = t.box_;
            let q = quote(&t.text);
            let why = match r.form.as_str() {
                "text-hold" if t.to - t.from + 0.05 < r.value => Some(format!("{q} is on for {:.1} s, under your {:.1} s", t.to - t.from, r.value)),
                "text-above" if y + bh > h * r.value / 100.0 + 1.0 => Some(format!("{q} reaches {:.0}% down the frame, below your {:.0}%", (y + bh) / h * 100.0, r.value)),
                "text-below" if y < h * r.value / 100.0 - 1.0 => Some(format!("{q} starts {:.0}% down the frame, above your {:.0}%", y / h * 100.0, r.value)),
                "text-margin" => {
                    let m = x.min(y).min(w - x - bw).min(h - y - bh) / s;
                    (m + 1.0 < r.value).then(|| format!("{q} is {m:.0} px from the edge, inside your {:.0}", r.value))
                }
                _ => None,
            };
            if let Some(why) = why {
                broke += 1;
                issues.push(Issue { rule: "taste", p: 1, at: Some(t.from), says: format!("{why} (\"{}\")", r.line) });
            }
        }
        if broke == 0 && r.form.starts_with("text-") {
            passed.push(format!("taste: {} ({} {})", r.line, r.form, r.value));
        }
    }
}

/// the flags `raea check` takes, each with a value
const TAKES: [&str; 7] = ["--manifest", "--words", "--perception", "--shape", "--platform", "--taste", "-o"];

fn run(args: &[String]) -> R<(String, bool)> {
    let video = args.iter().enumerate().find(|(i, a)| !a.starts_with('-') && !(*i > 0 && TAKES.contains(&args[i - 1].as_str()))).map(|(_, a)| a.clone()).ok_or(USAGE)?;
    if !std::path::Path::new(&video).exists() {
        return Err(format!("no file at {video}\n{USAGE}"));
    }
    let probe = Probe::of(&video)?;
    let mut issues = Vec::new();
    let mut passed = Vec::new();
    let (taste, skipped) = match flag(args, "--taste") {
        Some(t) if std::path::Path::new(t).exists() => taste_rules(t)?,
        _ => (Vec::new(), Vec::new()),
    };
    for s in skipped {
        passed.push(format!("taste: \"{s}\" isn't a form raea measures; kept as a line, not checked"));
    }

    let manifest: Option<Value> = match flag(args, "--manifest") {
        Some(m) => Some(serde_json::from_str(&std::fs::read_to_string(m).map_err(|e| format!("{m}: {e}"))?).map_err(|e| format!("{m}: {e}"))?),
        None => None,
    };
    let fps = manifest.as_ref().and_then(|m| m["fps"].as_f64()).unwrap_or(30.0);
    let words = flag(args, "--words").map(|p| transcript(p, fps)).transpose()?;
    if probe.audio {
        loudness(&video, taste_of(&taste, "loudness"), &mut issues, &mut passed)?;
        sound(&video, words.as_deref(), &mut issues, &mut passed)?;
    }
    freezes(&video, &probe, taste_of(&taste, "still-max"), &mut issues, &mut passed)?;
    blotching(&video, &probe, &mut issues, &mut passed)?;
    brand_kit(&[flag(args, "--manifest").unwrap_or(""), video.as_str()], &mut issues, &mut passed);
    if let Some(manifest) = &manifest {
        let runs = text_runs(manifest);
        let (w, h) = (manifest["width"].as_f64().unwrap_or(probe.w), manifest["height"].as_f64().unwrap_or(probe.h));
        text_rules(&runs, words.as_deref(), w, h, flag(args, "--shape"), flag(args, "--platform"), &mut issues, &mut passed);
        contrast(&video, &runs, w, h, &probe, &mut issues, &mut passed);
        taste_text(&taste, &runs, w, h, &mut issues, &mut passed);
        if let Some(words) = &words {
            cues(&runs, words, &mut issues, &mut passed);
        }
        if let Some(p) = flag(args, "--perception") {
            let faces = faces_of(&serde_json::from_str(&std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))?).map_err(|e| format!("{p}: {e}"))?, fps);
            face_cover(&runs, &faces, &mut issues, &mut passed);
        }
    }

    issues.sort_by(|a, b| a.p.cmp(&b.p).then(a.at.unwrap_or(0.0).total_cmp(&b.at.unwrap_or(0.0))));
    let bad = issues.iter().any(|i| i.p <= 1);
    if let Some(out) = flag(args, "-o") {
        let j = json!({
            "file": video,
            "issues": issues.iter().map(|i| json!({"rule": i.rule, "p": format!("P{}", i.p), "at": i.at, "says": i.says})).collect::<Vec<_>>(),
            "passed": passed,
        });
        std::fs::write(out, serde_json::to_string_pretty(&j).unwrap()).map_err(|e| format!("{out}: {e}"))?;
    }
    let mut by = BTreeMap::new();
    for i in &issues {
        *by.entry(i.p).or_insert(0) += 1;
    }
    let counts = by.iter().map(|(p, n)| format!("P{p} {n}")).collect::<Vec<_>>().join(", ");
    let mut s = format!("raea check {}: {}\n", video, if issues.is_empty() { "nothing to fix".to_string() } else { format!("{} to fix ({counts})", issues.len()) });
    for i in &issues {
        let at = i.at.map(|t| format!("{t:6.2} s")).unwrap_or_else(|| "       ".into());
        s.push_str(&format!("P{} {:<9} {at}  {}\n", i.p, i.rule, i.says));
    }
    for p in &passed {
        s.push_str(&format!("ok {p}\n"));
    }
    Ok((s.trim_end().to_string(), bad))
}

struct Probe {
    w: f64,
    h: f64,
    duration: f64,
    audio: bool,
}

impl Probe {
    fn of(video: &str) -> R<Probe> {
        let out = Command::new("ffprobe")
            .args(["-v", "error", "-show_entries", "stream=codec_type,width,height,r_frame_rate:format=duration", "-of", "json", video])
            .output()
            .map_err(|e| format!("ffprobe: {e}"))?;
        let v: Value = serde_json::from_slice(&out.stdout).map_err(|e| format!("ffprobe: {e}"))?;
        let streams = v["streams"].as_array().cloned().unwrap_or_default();
        let vid = streams.iter().find(|s| s["codec_type"] == "video").ok_or("no video stream")?;
        Ok(Probe {
            w: vid["width"].as_f64().unwrap_or(1920.0),
            h: vid["height"].as_f64().unwrap_or(1080.0),
            duration: v["format"]["duration"].as_str().and_then(|d| d.parse().ok()).unwrap_or(0.0),
            audio: streams.iter().any(|s| s["codec_type"] == "audio"),
        })
    }
}
