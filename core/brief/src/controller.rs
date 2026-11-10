//! A workspace's controllers: settings its Director puts together with the person, out of a small set of parts. A
//! controller has groups, a group has typed fields, and every agent in the workspace works to the values (`describe`).
//! The agent writes one as `controllers/<id>.ts` (`export default controller({...})`, docs/projects.feature PJ2);
//! `normalize` checks it and fills in ids, with errors in a sentence the agent can act on. The same contract as
//! apps/app src/controllers/types.ts, whose page reads these types.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct Controller {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")] pub help: Option<String>,
    pub groups: Vec<Group>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct Group {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")] pub help: Option<String>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct Field {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")] pub help: Option<String>,
    #[serde(flatten)]
    #[ts(flatten)]
    pub kind: Kind,
}

/// One step of a workflow field: who does it (an agent's name, or "you" for the person) and where a failure goes back to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct Step {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")] pub by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub back: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "lowercase")]
#[ts(optional_fields)]
pub enum Kind {
    Toggle { value: bool },
    Choice { options: Vec<String>, value: String },
    Number { value: f64, #[serde(skip_serializing_if = "Option::is_none")] unit: Option<String>, #[serde(skip_serializing_if = "Option::is_none")] min: Option<f64>, #[serde(skip_serializing_if = "Option::is_none")] max: Option<f64>, #[serde(skip_serializing_if = "Option::is_none")] step: Option<f64> },
    Text { value: String, #[serde(skip_serializing_if = "Option::is_none")] placeholder: Option<String>, #[serde(skip_serializing_if = "Option::is_none")] multiline: Option<bool> },
    List { value: Vec<String>, #[serde(skip_serializing_if = "Option::is_none")] placeholder: Option<String> },
    Agents { value: Vec<String> },
    Channels { value: Vec<String> },
    /// steps in order; `channels` run it (all of them when unset)
    Workflow { value: Vec<Step>, #[serde(skip_serializing_if = "Option::is_none")] channels: Option<Vec<String>> },
    /// MCP servers: URLs, commands or GitHub repos, given to every agent's session
    Servers { value: Vec<String> },
    /// skills: links to SKILL.md, read into every agent's context
    Skills { value: Vec<String> },
    /// a rule the work is held to, its label the rule's words (docs/direction.feature DR4): `check` is one of raea's
    /// checks by name with its `values`, or a path to the project's own check (a .ts file, from the controller's
    /// folder); `level` is error (when unset), warn or note. `raea check` runs it.
    Rule {
        check: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(type = "Record<string, number | string | boolean>")]
        values: Option<Map<String, Value>>,
        #[serde(skip_serializing_if = "Option::is_none")] level: Option<String>,
    },
}

const TYPES: &str = "toggle, choice, number, text, list, agents, channels, workflow, servers, skills, rule";

fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let out: String = out.trim_matches('-').chars().take(48).collect();
    if out.is_empty() { "x".into() } else { out.trim_end_matches('-').into() }
}

fn is_id(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty() && b.len() <= 48 && b[0].is_ascii_lowercase() && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

fn text(o: &Map<String, Value>, k: &str) -> Option<String> {
    o.get(k).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(String::from)
}

fn id_of(o: &Map<String, Value>, fallback: &str) -> String {
    o.get("id").and_then(Value::as_str).filter(|s| is_id(s)).map(String::from).unwrap_or_else(|| slug(fallback))
}

fn strings(v: Option<&Value>, what: &str) -> Result<Vec<String>, String> {
    match v {
        None | Some(Value::Null) => Ok(vec![]),
        Some(Value::Array(a)) if a.iter().all(Value::is_string) => {
            Ok(a.iter().filter_map(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(String::from).collect())
        }
        _ => Err(format!("{what} must be a list of strings")),
    }
}

fn number(o: &Map<String, Value>, k: &str) -> Option<f64> {
    o.get(k).and_then(Value::as_f64).filter(|n| n.is_finite())
}

fn steps(v: Option<&Value>, at: &str) -> Result<Vec<Step>, String> {
    let Some(Value::Array(a)) = v else { return Err(format!("{at}: a workflow's value is a list of steps: {{ name, by?, back? }}")) };
    a.iter()
        .enumerate()
        .map(|(i, s)| {
            let o = s.as_object().ok_or_else(|| format!("{at}: step {} needs a name", i + 1))?;
            let name = text(o, "name").ok_or_else(|| format!("{at}: step {} needs a name", i + 1))?;
            Ok(Step { name, by: text(o, "by").map(|b| b.trim_start_matches('@').to_string()), back: text(o, "back") })
        })
        .collect()
}

/// A field as the agent wrote it, checked and filled in; an error names the field and what it needs.
pub fn normalize_field(raw: &Value, at: &str) -> Result<Field, String> {
    let o = raw.as_object().ok_or_else(|| format!("{at}: a field is an object"))?;
    let label = text(o, "label").ok_or_else(|| format!("{at}: a field needs a label"))?;
    let id = id_of(o, &label);
    let w = format!("{at}.{id}");
    let value = o.get("value");
    let kind = match o.get("type").and_then(Value::as_str) {
        Some("toggle") => Kind::Toggle { value: value == Some(&Value::Bool(true)) },
        Some("choice") => {
            let options = strings(o.get("options"), &format!("{w}: options"))?;
            if options.len() < 2 {
                return Err(format!("{w}: a choice needs at least two options"));
            }
            let value = value.and_then(Value::as_str).filter(|v| options.iter().any(|x| x == v)).map(String::from).unwrap_or_else(|| options[0].clone());
            Kind::Choice { options, value }
        }
        Some("number") => Kind::Number { value: number(o, "value").unwrap_or(0.0), unit: o.get("unit").and_then(Value::as_str).map(String::from), min: number(o, "min"), max: number(o, "max"), step: number(o, "step") },
        Some("text") => Kind::Text { value: value.and_then(Value::as_str).unwrap_or("").into(), placeholder: o.get("placeholder").and_then(Value::as_str).map(String::from), multiline: Some(o.get("multiline") == Some(&Value::Bool(true))) },
        Some("list") => Kind::List { value: strings(value, &format!("{w}: value"))?, placeholder: o.get("placeholder").and_then(Value::as_str).map(String::from) },
        Some("agents") => Kind::Agents { value: strings(value, &format!("{w}: value"))? },
        Some("channels") => Kind::Channels { value: strings(value, &format!("{w}: value"))? },
        Some("servers") => Kind::Servers { value: strings(value, &format!("{w}: value"))? },
        Some("skills") => Kind::Skills { value: strings(value, &format!("{w}: value"))? },
        Some("workflow") => Kind::Workflow { value: steps(value, &w)?, channels: o.get("channels").map(|c| strings(Some(c), &format!("{w}: channels"))).transpose()? },
        Some("rule") => {
            let check = text(o, "check").ok_or_else(|| format!("{w}: a rule needs a check: one of raea's by name, or a path to a .ts check"))?;
            let level = text(o, "level");
            if level.as_deref().is_some_and(|l| !matches!(l, "error" | "warn" | "note")) {
                return Err(format!("{w}: a rule's level is error, warn or note"));
            }
            Kind::Rule { check, values: o.get("values").and_then(Value::as_object).cloned(), level }
        }
        _ => return Err(format!("{w}: type must be one of {TYPES}")),
    };
    Ok(Field { id, label, help: text(o, "help"), kind })
}

/// A controller as the agent wrote it (a module's default export, as JSON), checked: ids filled in from titles, each
/// field's value of its type.
pub fn normalize(raw: &Value) -> Result<Controller, String> {
    let o = raw.as_object().filter(|o| text(o, "title").is_some()).ok_or("a controller needs a title")?;
    let title = text(o, "title").unwrap_or_default();
    let id = id_of(o, &title);
    let gs = o.get("groups").and_then(Value::as_array).filter(|g| !g.is_empty()).ok_or_else(|| format!("{id}: a controller needs groups: [{{ title, fields: [...] }}]"))?;
    let mut groups = vec![];
    for (i, g) in gs.iter().enumerate() {
        let g = g.as_object().filter(|g| text(g, "title").is_some()).ok_or_else(|| format!("{id}: group {} needs a title", i + 1))?;
        let gtitle = text(g, "title").unwrap_or_default();
        let gid = id_of(g, &gtitle);
        let at = format!("{id}.{gid}");
        let fs = g.get("fields").and_then(Value::as_array).ok_or_else(|| format!("{at}: a group needs fields"))?;
        let fields = fs.iter().map(|f| normalize_field(f, &at)).collect::<Result<Vec<_>, _>>()?;
        if let Some(d) = fields.iter().enumerate().find(|(j, f)| fields.iter().position(|x| x.id == f.id) != Some(*j)) {
            return Err(format!("{at}: two fields are called {}", d.1.id));
        }
        groups.push(Group { id: gid, title: gtitle, help: text(g, "help"), fields });
    }
    Ok(Controller { id, title, help: text(o, "help"), groups })
}

/// A controller from its module's default export as JSON (what `raea spec` prints), checked as `normalize` does.
pub fn of_json(json: &str) -> Result<Controller, String> {
    normalize(&serde_json::from_str(json).map_err(|e| e.to_string())?)
}

/// A value checked against its field: the same type, a choice among its options, a number within its range. The
/// value as it's kept (a workflow's steps filled in, a list's strings trimmed).
pub fn check_value(f: &Field, v: &Value) -> Result<Value, String> {
    let w = &f.label;
    match &f.kind {
        Kind::Toggle { .. } => v.as_bool().map(Value::Bool).ok_or_else(|| format!("{w} is on or off: true or false")),
        Kind::Choice { options, .. } => v.as_str().filter(|s| options.iter().any(|o| o == s)).map(|s| Value::String(s.into())).ok_or_else(|| format!("{w} is one of: {}", options.join(", "))),
        Kind::Number { min, max, .. } => {
            let n = v.as_f64().filter(|n| n.is_finite()).ok_or_else(|| format!("{w} is a number"))?;
            if min.is_some_and(|m| n < m) || max.is_some_and(|m| n > m) {
                let (lo, hi) = (min.map_or("-∞".into(), |m| m.to_string()), max.map_or("∞".into(), |m| m.to_string()));
                return Err(format!("{w} is between {lo} and {hi}"));
            }
            Ok(v.clone())
        }
        Kind::Text { .. } => v.as_str().map(|s| Value::String(s.into())).ok_or_else(|| format!("{w} is text")),
        Kind::Workflow { .. } => Ok(serde_json::to_value(steps(Some(v), w)?).unwrap_or_default()),
        _ => Ok(serde_json::to_value(strings(Some(v), w)?).unwrap_or_default()),
    }
}

/// The field at a path (controller.group.field) among the controllers.
pub fn field<'a>(cs: &'a [Controller], path: &str) -> Option<&'a Field> {
    let mut p = path.splitn(3, '.');
    let (c, g, f) = (p.next()?, p.next()?, p.next()?);
    cs.iter().find(|x| x.id == c)?.groups.iter().find(|x| x.id == g)?.fields.iter().find(|x| x.id == f)
}
