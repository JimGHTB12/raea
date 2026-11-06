//! A workflow as a whiteboard shows it (docs/whiteboards.feature CF1, apps/app src/whiteboard/flow.ts): each node
//! with its class, title, mode, place and settings by name; the links between them; its groups. And a board's edits
//! written back (CF2): settings, bypass and notes. The workflow stays the source of truth; the board is a view of it.

use serde_json::{Map, Value as J, json};

use crate::read::{id_of, link, ui_links};
use crate::schema::{Schema, slot_value, widget_slots};

const NOTES: [&str; 2] = ["Note", "MarkdownNote"];

fn mode_name(m: i64) -> &'static str {
    match m {
        2 => "mute",
        4 => "bypass",
        _ => "on",
    }
}

/// A node's settings by name, with where each is kept (`at`: an index into widgets_values, or an input's name).
fn ui_widgets(n: &J, schema: &Schema) -> Vec<J> {
    let class = n["type"].as_str().unwrap_or_default();
    let wv = &n["widgets_values"];
    if NOTES.contains(&class) {
        return vec![json!({"name": "text", "value": wv[0].clone(), "at": 0})];
    }
    widget_slots(schema.get(class), wv).into_iter().map(|(name, at)| {
        let value = slot_value(wv, &at);
        json!({"name": name, "value": value, "at": at})
    }).collect()
}

/// The workflow (UI or API format) as nodes, links and groups. A UI workflow's own layout comes along (pos, size);
/// an API one has none, and the board lays it out by its links. Subgraphs show as the node that holds them.
pub fn view(v: &J, schema: &Schema) -> Result<J, String> {
    if v["nodes"].is_array() {
        let table = ui_links(&v["links"]);
        let mut nodes = Vec::new();
        let mut links = Vec::new();
        for n in v["nodes"].as_array().into_iter().flatten() {
            let id = id_of(n);
            let mut inputs = Vec::new();
            for i in n["inputs"].as_array().into_iter().flatten() {
                let name = i["name"].as_str().unwrap_or_default();
                if let Some((from, slot)) = i["link"].as_i64().and_then(|l| table.get(&l)) {
                    links.push(json!({"from": from.to_string(), "slot": slot, "to": id, "input": name, "type": i["type"]}));
                }
                inputs.push(json!({"name": name, "type": i["type"], "linked": i["link"].is_i64()}));
            }
            let outputs: Vec<J> = n["outputs"].as_array().into_iter().flatten().map(|o| json!({"name": o["name"], "type": o["type"]})).collect();
            nodes.push(json!({
                "id": id, "class": n["type"], "title": n["title"], "mode": mode_name(n["mode"].as_i64().unwrap_or(0)),
                "pos": n["pos"], "size": n["size"], "widgets": ui_widgets(n, schema), "inputs": inputs, "outputs": outputs,
                "note": NOTES.contains(&n["type"].as_str().unwrap_or_default()),
            }));
        }
        let groups: Vec<J> = v["groups"].as_array().into_iter().flatten().map(|g| json!({"title": g["title"], "box": g["bounding"], "color": g["color"]})).collect();
        return Ok(json!({"format": "ui", "nodes": nodes, "links": links, "groups": groups}));
    }
    let api = if v["prompt"].is_object() { &v["prompt"] } else { v };
    let o = api.as_object().ok_or("not a ComfyUI workflow: neither UI format (nodes, links) nor API format ({id: {class_type, inputs}})")?;
    let mut ids: Vec<&String> = o.keys().filter(|k| o[k.as_str()]["class_type"].is_string()).collect();
    ids.sort_by_key(|k| (k.parse::<i64>().unwrap_or(i64::MAX), k.to_string()));
    let mut nodes = Vec::new();
    let mut links = Vec::new();
    for id in ids {
        let n = &o[id.as_str()];
        let mut widgets = Vec::new();
        let mut inputs = Vec::new();
        for (name, val) in n["inputs"].as_object().into_iter().flatten() {
            match link(val) {
                Some((from, slot)) => {
                    links.push(json!({"from": from, "slot": slot, "to": id, "input": name}));
                    inputs.push(json!({"name": name, "linked": true}));
                }
                None => widgets.push(json!({"name": name, "value": val, "at": name})),
            }
        }
        nodes.push(json!({"id": id, "class": n["class_type"], "title": n["_meta"]["title"], "mode": "on", "widgets": widgets, "inputs": inputs, "outputs": [], "note": false}));
    }
    if nodes.is_empty() {
        return Err("no nodes with a class_type".into());
    }
    Ok(json!({"format": "api", "nodes": nodes, "links": links, "groups": []}))
}

/// A board's value for a setting in the setting's own type: "30" for a number is 30, "true" for a flag is true.
fn coerce(old: &J, new: &J) -> Result<J, String> {
    let Some(s) = new.as_str() else { return Ok(new.clone()) };
    Ok(match old {
        J::Number(n) if n.is_i64() || n.is_u64() => J::from(s.trim().parse::<i64>().map_err(|_| format!("{s:?} isn't a whole number"))?),
        J::Number(_) => json!(s.trim().parse::<f64>().map_err(|_| format!("{s:?} isn't a number"))?),
        J::Bool(_) => J::Bool(match s.trim() { "true" | "on" | "yes" => true, "false" | "off" | "no" => false, _ => return Err(format!("{s:?} isn't true or false")) }),
        _ => new.clone(),
    })
}

/// A board's edits written into the workflow: `{widgets: [{node, name, value}], modes: [{node, mode: on|bypass|mute}],
/// notes: [{node?, text}]}` (a note with no node is a new Note node). Returns how many changes it made.
pub fn set(v: &mut J, changes: &J, schema: &Schema) -> Result<usize, String> {
    let mut n_changed = 0;
    if v["nodes"].is_array() {
        let find = |v: &mut J, id: &str| -> Result<usize, String> {
            v["nodes"].as_array().unwrap().iter().position(|n| id_of(n) == id).ok_or(format!("the workflow has no node {id}"))
        };
        for c in changes["widgets"].as_array().into_iter().flatten() {
            let (id, name) = (c["node"].as_str().unwrap_or_default(), c["name"].as_str().unwrap_or_default());
            let k = find(v, id)?;
            let n = &mut v["nodes"][k];
            let at = ui_widgets(n, schema).into_iter().find(|w| w["name"] == name).map(|w| w["at"].clone()).ok_or(format!("node {id} has no setting {name}"))?;
            let slot = match &at { J::String(key) => &mut n["widgets_values"][key.as_str()], _ => &mut n["widgets_values"][at.as_u64().unwrap_or(0) as usize] };
            let new = coerce(slot, &c["value"])?;
            if *slot != new {
                *slot = new;
                n_changed += 1;
            }
        }
        for c in changes["modes"].as_array().into_iter().flatten() {
            let id = c["node"].as_str().unwrap_or_default();
            let m = match c["mode"].as_str() { Some("on") => 0, Some("mute") => 2, Some("bypass") => 4, other => return Err(format!("mode {other:?}: on, bypass or mute")) };
            let k = find(v, id)?;
            if v["nodes"][k]["mode"].as_i64().unwrap_or(0) != m {
                v["nodes"][k]["mode"] = J::from(m);
                n_changed += 1;
            }
        }
        for c in changes["notes"].as_array().into_iter().flatten() {
            let text = c["text"].as_str().ok_or("a note needs its text")?;
            if let Some(id) = c["node"].as_str() {
                let k = find(v, id)?;
                if v["nodes"][k]["widgets_values"][0] != text {
                    v["nodes"][k]["widgets_values"] = json!([text]);
                    n_changed += 1;
                }
                continue;
            }
            // a new note goes right of everything, so it covers nothing
            let nodes = v["nodes"].as_array().unwrap();
            let right = nodes.iter().map(|n| n["pos"][0].as_f64().unwrap_or(0.0) + n["size"][0].as_f64().unwrap_or(0.0)).fold(0.0, f64::max);
            let top = nodes.iter().filter_map(|n| n["pos"][1].as_f64()).fold(f64::MAX, f64::min);
            let id = v["last_node_id"].as_i64().unwrap_or(0).max(nodes.iter().filter_map(|n| n["id"].as_i64()).max().unwrap_or(0)) + 1;
            let mut note = Map::new();
            for (k, val) in [("id", json!(id)), ("type", json!("Note")), ("pos", json!([right + 60.0, if top == f64::MAX { 0.0 } else { top }])), ("size", json!([320, 120])), ("flags", json!({})), ("order", json!(nodes.len())), ("mode", json!(0)), ("inputs", json!([])), ("outputs", json!([])), ("properties", json!({})), ("widgets_values", json!([text]))] {
                note.insert(k.into(), val);
            }
            v["nodes"].as_array_mut().unwrap().push(J::Object(note));
            v["last_node_id"] = json!(id);
            n_changed += 1;
        }
        return Ok(n_changed);
    }
    if changes["modes"].as_array().is_some_and(|a| !a.is_empty()) || changes["notes"].as_array().is_some_and(|a| !a.is_empty()) {
        return Err("an API-format workflow keeps no bypass or notes: save the workflow from ComfyUI's editor (UI format) to have them".into());
    }
    let api = if v["prompt"].is_object() { &mut v["prompt"] } else { v };
    for c in changes["widgets"].as_array().into_iter().flatten() {
        let (id, name) = (c["node"].as_str().unwrap_or_default(), c["name"].as_str().unwrap_or_default());
        let slot = api.get_mut(id).and_then(|n| n["inputs"].get_mut(name)).ok_or(format!("node {id} has no setting {name}"))?;
        if link(slot).is_some() {
            return Err(format!("node {id}'s {name} is a link, not a setting"));
        }
        let new = coerce(slot, &c["value"])?;
        if *slot != new {
            *slot = new;
            n_changed += 1;
        }
    }
    Ok(n_changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ui() -> J {
        json!({
            "last_node_id": 3,
            "nodes": [
                {"id": 1, "type": "KSampler", "mode": 0, "pos": [0, 0], "size": [300, 260], "inputs": [{"name": "model", "type": "MODEL", "link": 7}], "outputs": [{"name": "LATENT", "type": "LATENT"}], "widgets_values": [42, "randomize", 20, 7.5, "euler", "normal", 1.0]},
                {"id": 2, "type": "CheckpointLoaderSimple", "mode": 4, "pos": [-400, 0], "size": [300, 100], "outputs": [{"name": "MODEL", "type": "MODEL"}], "widgets_values": ["sd.safetensors"]},
                {"id": 3, "type": "Note", "mode": 0, "pos": [0, 300], "size": [300, 100], "widgets_values": ["try 30 steps"]}
            ],
            "links": [[7, 2, 0, 1, 0, "MODEL"]],
            "groups": [{"title": "Sample", "bounding": [-20, -40, 360, 320]}]
        })
    }

    #[test]
    fn a_ui_workflow_viewed_names_settings_past_the_seed_control() {
        let s = crate::schema::load();
        let g = view(&ui(), &s).unwrap();
        let k = &g["nodes"][0];
        assert_eq!(k["widgets"][1], json!({"name": "steps", "value": 20, "at": 2}));
        assert_eq!(g["nodes"][1]["mode"], "bypass");
        assert_eq!(g["links"][0], json!({"from": "2", "slot": 0, "to": "1", "input": "model", "type": "MODEL"}));
        assert_eq!(g["groups"][0]["title"], "Sample");
        assert_eq!(g["nodes"][2]["note"], true);
    }

    #[test]
    fn edits_go_back_into_the_workflow_in_their_own_types() {
        let s = crate::schema::load();
        let mut w = ui();
        let n = set(&mut w, &json!({"widgets": [{"node": "1", "name": "steps", "value": "30"}, {"node": "1", "name": "cfg", "value": "6.5"}], "modes": [{"node": "2", "mode": "on"}], "notes": [{"node": "3", "text": "30 is better"}, {"text": "new idea"}]}), &s).unwrap();
        assert_eq!(n, 5);
        assert_eq!(w["nodes"][0]["widgets_values"], json!([42, "randomize", 30, 6.5, "euler", "normal", 1.0]));
        assert_eq!(w["nodes"][1]["mode"], 0);
        assert_eq!(w["nodes"][2]["widgets_values"], json!(["30 is better"]));
        assert_eq!(w["nodes"][3]["type"], "Note");
        assert_eq!(w["last_node_id"], 4);
        assert!(set(&mut w, &json!({"widgets": [{"node": "1", "name": "steps", "value": "lots"}]}), &s).is_err());
    }

    #[test]
    fn an_api_workflow_takes_settings_but_not_bypass() {
        let s = crate::schema::load();
        let mut w = json!({"1": {"class_type": "ImageScale", "inputs": {"image": ["2", 0], "width": 48}}, "2": {"class_type": "LoadImage", "inputs": {"image": "in.png"}}});
        let g = view(&w, &s).unwrap();
        assert_eq!(g["links"][0], json!({"from": "2", "slot": 0, "to": "1", "input": "image"}));
        assert_eq!(set(&mut w, &json!({"widgets": [{"node": "1", "name": "width", "value": "64"}]}), &s).unwrap(), 1);
        assert_eq!(w["1"]["inputs"]["width"], 64);
        assert!(set(&mut w, &json!({"widgets": [{"node": "1", "name": "image", "value": "x"}]}), &s).is_err());
        assert!(set(&mut w, &json!({"modes": [{"node": "1", "mode": "bypass"}]}), &s).is_err());
    }
}
