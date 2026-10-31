//! A person's taste: standing corrections kept across every project, read by every agent before any work. A rule is
//! words; a rule that's mechanical also carries a check, a typed value `raea check --taste` holds every export to.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct Taste {
    pub title: String,
    /// when this file is read ("Read for pitch, explain and story")
    pub note: Option<String>,
    #[serde(rename = "for")]
    #[ts(rename = "for")]
    pub for_: Option<String>,
    /// the kinds of work with their own taste file (motion, games, sound, ui)
    #[serde(default)]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub kinds: Vec<String>,
    pub sections: Vec<TasteSection>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct TasteSection {
    /// "Always", "Never", "Camera"
    pub heading: String,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct Rule {
    pub text: String,
    pub check: Option<Check>,
}

/// What a check measures on an export. Values keep the units the checker has always used.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "form", content = "value", rename_all = "kebab-case")]
pub enum Check {
    /// every text holds at least this many seconds
    TextHold(f64),
    /// every text's bottom edge sits above this percent of the frame's height
    TextAbove(f64),
    /// every text's top edge sits below this percent of the frame's height
    TextBelow(f64),
    /// every text keeps this many px (at 1080 on the short side) from each edge
    TextMargin(f64),
    /// integrated loudness within 1 LU of this many LUFS
    Loudness(f64),
    /// nothing holds still longer than this many seconds
    StillMax(f64),
}

impl Check {
    /// The form's name as `raea check` reports it.
    pub fn form(&self) -> &'static str {
        match self {
            Check::TextHold(_) => "text-hold",
            Check::TextAbove(_) => "text-above",
            Check::TextBelow(_) => "text-below",
            Check::TextMargin(_) => "text-margin",
            Check::Loudness(_) => "loudness",
            Check::StillMax(_) => "still-max",
        }
    }
    pub fn value(&self) -> f64 {
        match *self {
            Check::TextHold(v) | Check::TextAbove(v) | Check::TextBelow(v) | Check::TextMargin(v) | Check::Loudness(v) | Check::StillMax(v) => v,
        }
    }
}

impl Taste {
    /// Every rule that carries a check, with its words.
    pub fn checks(&self) -> impl Iterator<Item = (&str, Check)> {
        self.sections.iter().flat_map(|s| s.rules.iter()).filter_map(|r| r.check.map(|c| (r.text.as_str(), c)))
    }
}
