//! One finding, for every command (docs/direction.feature DR3): what the clipping check, `raea scene lint`, `raea check`
//! and the rest write, in one shape, so "everything found about this part" is one query and a count means the same
//! thing whichever command printed it. raea check's ranks and lint's words map onto its three levels.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// How bad: an error stops a render's exit (unless `--despite`), a warn is shown, a note is said. Ordered worst first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warn,
    Note,
}

impl Level {
    /// raea check's ranks: P0 and P1 are errors, P2 a warn, P3 (and anything past it) a note.
    pub fn of_rank(p: u8) -> Level {
        match p {
            0 | 1 => Level::Error,
            2 => Level::Warn,
            _ => Level::Note,
        }
    }

    /// lint's and the older commands' words: "error", "warning" or "warn"; anything else is a note.
    pub fn of_word(w: &str) -> Level {
        match w {
            "error" => Level::Error,
            "warning" | "warn" => Level::Warn,
            _ => Level::Note,
        }
    }

    fn word(self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warn => "warn",
            Level::Note => "note",
        }
    }
}

/// An entity a finding is about, by what names it across renders: its layer key (a Source's key, "file:line:col…")
/// when it has one, and the name it goes by (a part's label, a text's words).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct About {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[ts(optional)]
    pub key: Option<String>,
}

/// The number a finding turns on: what was measured, its value and unit, and the limit it was held to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct Measure {
    pub name: String,
    pub value: f64,
    pub unit: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[ts(optional)]
    pub limit: Option<f64>,
}

/// One thing found: by which check, how bad, about which entities, on which frames, in a sentence, with the number it
/// turns on and how to fix it, and what found it (a command, or the ask a note's check was written for, DR6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct Finding {
    pub check: String,
    pub level: Level,
    #[serde(default)]
    pub about: Vec<About>,
    /// from its first frame to the frame after its last; None when it is about the whole piece
    #[serde(default)]
    pub frames: Option<[i64; 2]>,
    pub says: String,
    #[serde(default)]
    pub measure: Option<Measure>,
    #[serde(default)]
    pub fix: Option<String>,
    pub by: String,
}

impl Finding {
    pub fn new(check: &str, level: Level, says: impl Into<String>, by: &str) -> Finding {
        Finding { check: check.into(), level, about: Vec::new(), frames: None, says: says.into(), measure: None, fix: None, by: by.into() }
    }

    pub fn about(mut self, name: impl Into<String>, key: Option<String>) -> Finding {
        self.about.push(About { name: name.into(), key });
        self
    }

    pub fn frames(mut self, from: i64, to: i64) -> Finding {
        self.frames = Some([from, to]);
        self
    }

    pub fn measure(mut self, name: &str, value: f64, unit: &str, limit: Option<f64>) -> Finding {
        self.measure = Some(Measure { name: name.into(), value, unit: unit.into(), limit });
        self
    }

    pub fn fix(mut self, fix: impl Into<String>) -> Finding {
        self.fix = Some(fix.into());
        self
    }

    /// One line: `error clips  frames 120-139  seat, rider: the rider is 4 cm into the seat (fix: …)`.
    pub fn line(&self) -> String {
        let mut s = format!("{:<5} {}", self.level.word(), self.check);
        if let Some([a, b]) = self.frames {
            s.push_str(&if b <= a + 1 { format!("  frame {a}") } else { format!("  frames {a}-{}", b - 1) });
        }
        if !self.about.is_empty() {
            s.push_str("  ");
            s.push_str(&self.about.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", "));
        }
        s.push_str(": ");
        s.push_str(&self.says);
        if let Some(f) = &self.fix {
            s.push_str(&format!(" (fix: {f})"));
        }
        s
    }
}

static SAID: std::sync::Mutex<Vec<Finding>> = std::sync::Mutex::new(Vec::new());

/// Findings a command has said as it went (a render's clipping, from its own thread), kept for its count at the end.
pub fn said(fs: &[Finding]) {
    SAID.lock().unwrap_or_else(|e| e.into_inner()).extend(fs.iter().cloned());
}

/// Every finding said so far, taken: what the command ends by counting.
pub fn taken() -> Vec<Finding> {
    std::mem::take(&mut *SAID.lock().unwrap_or_else(|e| e.into_inner()))
}

/// Findings worst first, each level in the order found.
pub fn ranked(mut fs: Vec<Finding>) -> Vec<Finding> {
    fs.sort_by_key(|f| f.level);
    fs
}

/// The count a command prints last: "2 errors, 1 warn, 3 notes".
pub fn count(fs: &[Finding]) -> String {
    let say = |l: Level| match fs.iter().filter(|f| f.level == l).count() {
        1 => format!("1 {}", l.word()),
        k => format!("{k} {}s", l.word()),
    };
    [Level::Error, Level::Warn, Level::Note].map(say).join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_old_ranks_and_words_map_onto_three_levels() {
        assert_eq!([0, 1, 2, 3].map(Level::of_rank), [Level::Error, Level::Error, Level::Warn, Level::Note]);
        assert_eq!(["error", "warning", "warn", "info"].map(Level::of_word), [Level::Error, Level::Warn, Level::Warn, Level::Note]);
    }

    #[test]
    fn a_finding_reads_as_one_line_and_as_json_and_back() {
        let f = Finding::new("clips", Level::Error, "the rider is 4 cm into the seat", "raea render")
            .about("rider", Some("scene.tsx:12:5".into()))
            .about("seat", None)
            .frames(120, 140)
            .measure("depth", 0.04, "m", Some(0.005))
            .fix("raise the rider 4 cm");
        assert_eq!(f.line(), "error clips  frames 120-139  rider, seat: the rider is 4 cm into the seat (fix: raise the rider 4 cm)");
        let j = serde_json::to_value(&f).unwrap();
        assert_eq!((j["level"].as_str(), j["about"][1].get("key")), (Some("error"), None));
        assert_eq!(serde_json::from_value::<Finding>(j).unwrap(), f);
    }

    #[test]
    fn findings_rank_worst_first_and_count() {
        let fs = vec![Finding::new("a", Level::Note, "n", "x"), Finding::new("b", Level::Error, "e", "x"), Finding::new("c", Level::Warn, "w", "x")];
        let fs = ranked(fs);
        assert_eq!(fs.iter().map(|f| f.check.as_str()).collect::<Vec<_>>(), ["b", "c", "a"]);
        assert_eq!(count(&fs), "1 error, 1 warn, 1 note");
    }
}
