//! A project's brief: who it's for, the one thing they should believe, what it delivers and the work to get there.
//! The facts an agent guessed are listed in `assumed`, for the person to confirm.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(optional_fields)]
pub struct Brief {
    pub title: String,
    pub logline: Option<String>,
    /// who the work is for (a person or a client)
    #[serde(rename = "for")]
    #[ts(rename = "for")]
    pub for_: Option<String>,
    /// YYYY-MM-DD
    pub due: Option<String>,
    /// who watches or plays it, and where
    pub audience: Option<String>,
    /// the one thing they should think afterwards
    pub believe: Option<String>,
    /// pitch, explain, story, play, tool: an open set, one guide each; a piece can mix two
    #[serde(default)]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub mode: Vec<String>,
    /// the narrator and the tone
    pub voice: Option<String>,
    pub sound: Option<String>,
    /// the project's committed world: palette by role, type, camera language, light, references
    pub look: Option<String>,
    /// the anti-goals
    #[serde(default)]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub not_doing: Vec<String>,
    /// facts an agent wrote and the person hasn't confirmed, by key ("voice", "audience")
    #[serde(default)]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub assumed: Vec<String>,
    /// prose sections beyond the fields above (Goal, Background), in order
    #[serde(default)]
    #[ts(as = "Option<Vec<Section>>", optional)]
    pub sections: Vec<Section>,
    #[serde(default)]
    #[ts(as = "Option<Vec<Deliverable>>", optional)]
    pub deliver: Vec<Deliverable>,
    #[serde(default)]
    #[ts(as = "Option<Vec<Reference>>", optional)]
    pub references: Vec<Reference>,
    /// the tasks, in stages ("Act one", "The cut")
    #[serde(default)]
    #[ts(as = "Option<Vec<Stage>>", optional)]
    pub stages: Vec<Stage>,
    #[serde(default)]
    #[ts(as = "Option<Vec<Agent>>", optional)]
    pub agents: Vec<Agent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub heading: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(optional_fields)]
pub struct Deliverable {
    pub name: String,
    /// "90 s", "16:9", "with sound"
    #[serde(default)]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub specs: Vec<String>,
    /// the feed item it lands as
    pub item: Option<String>,
    /// its own brief module, when it differs from the project in mode, look or audience
    pub brief: Option<String>,
    /// its script module (storyboard or screenplay)
    pub script: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct Reference {
    pub target: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    pub name: String,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum Mark {
    #[default]
    Todo,
    Done,
    Dropped,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(optional_fields)]
pub struct Task {
    pub text: String,
    /// a task linked to an item takes the item's state (the work is the truth); this is for unlinked ones
    #[serde(default)]
    #[ts(as = "Option<Mark>", optional)]
    pub mark: Mark,
    #[serde(default)]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub agents: Vec<String>,
    pub item: Option<String>,
    #[serde(default)]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub tags: Vec<String>,
    pub due: Option<String>,
    /// the craft verb it was written as ("quieter"): agents read it, the page shows the text
    pub verb: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(optional_fields)]
pub struct Agent {
    pub name: String,
    pub role: Option<String>,
}

/// What a brief still needs before work starts, each with what to do about it. Never an error: an unshaped brief
/// loads, and the page offers to shape it.
pub fn doctor(b: &Brief) -> Vec<String> {
    let mut out = Vec::new();
    if b.believe.is_none() {
        out.push("no `believe`: the one thing they should think afterwards; shape it before making anything".into());
    }
    if b.mode.is_empty() {
        out.push("no `mode`: pitch, explain, story, play or tool picks the guide".into());
    }
    if b.audience.is_none() {
        out.push("no `audience`: who watches or plays it, and where".into());
    }
    for d in &b.deliver {
        if d.script.is_none() && b.mode.iter().any(|m| m == "story" || m == "explain") {
            out.push(format!("`{}` has no script: a {} piece is storyboarded before it renders", d.name, b.mode.join(", ")));
        }
    }
    for a in &b.assumed {
        out.push(format!("`{a}` is assumed: confirm it with the person"));
    }
    out
}
