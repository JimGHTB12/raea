//! A ComfyUI workflow as nodes with named inputs: API format (what /prompt takes: `{id: {class_type, inputs}}`,
//! links as `[id, slot]`) as it is, and UI format (what the editor saves: positional widgets, a link table, subgraphs,
//! reroutes, primitives, muted and bypassed nodes) brought to the same shape. Subgraphs are inlined with their ids
//! prefixed by the instance's (`105:23`); notes and muted nodes go; a bypassed node passes its input of the same type.

use std::collections::BTreeMap;

use serde_json::{Map, Value as J, json};

use crate::schema::{Schema, name_widgets};

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub class: String,
    /// a setting's value, or `[node id, output slot]` for a link
    pub inputs: Map<String, J>,
    pub title: Option<String>,
}

pub type Graph = BTreeMap<String, Node>;

/// `[id, slot]`: the node and output an input is linked to.
pub fn link(v: &J) -> Option<(&str, usize)> {
    let a = v.as_array().filter(|a| a.len() == 2)?;
    Some((a[0].as_str()?, a[1].as_u64()? as usize))
}

/// Either format, told apart by its shape. A PNG's or a wrapper's `{"prompt": {...}}` is opened too.
pub fn read(v: &J, schema: &Schema) -> Result<Graph, String> {
    if v["nodes"].is_array() {
        return from_ui(v, schema);
    }
    let api = if v["prompt"].is_object() { &v["prompt"] } else { v };
    let o = api.as_object().ok_or("not a ComfyUI workflow: neither UI format (nodes, links) nor API format ({id: {class_type, inputs}})")?;
    let mut g = Graph::new();
    for (id, n) in o {
        let Some(class) = n["class_type"].as_str() else { continue };
        let mut inputs = n["inputs"].as_object().cloned().unwrap_or_default();
        // API format writes link ids as strings or numbers depending on who saved it
        for v in inputs.values_mut() {
            if let Some(a) = v.as_array_mut().filter(|a| a.len() == 2 && a[1].is_u64()) {
                if let Some(n) = a[0].as_u64() {
                    a[0] = json!(n.to_string());
                }
            }
        }
        g.insert(id.clone(), Node { class: class.into(), inputs, title: n["_meta"]["title"].as_str().map(str::to_string) });
    }
    if g.is_empty() {
        return Err("no nodes with a class_type".into());
    }
    Ok(g)
}

/// Where an input's value comes from while a UI workflow is being flattened.
#[derive(Clone, Debug)]
enum Src {
    Link(String, usize),
    Lit(J),
}

struct Flat {
    nodes: Vec<(String, String, i64, J, Vec<(String, Option<Src>, String)>, Option<String>)>,
    /// (node, slot) that stand for something else: a subgraph's outputs, reroutes, primitives, bypassed nodes
    alias: BTreeMap<(String, usize), Option<Src>>,
}

pub(crate) fn ui_links(v: &J) -> BTreeMap<i64, (i64, usize)> {
    v.as_array().into_iter().flatten().filter_map(|l| match l {
        J::Array(a) => Some((a[0].as_i64()?, (a[1].as_i64()?, a[2].as_u64()? as usize))),
        J::Object(_) => Some((l["id"].as_i64()?, (l["origin_id"].as_i64()?, l["origin_slot"].as_u64()? as usize))),
        _ => None,
    }).collect()
}

fn flatten(nodes: &J, links: &J, prefix: &str, defs: &BTreeMap<String, J>, parent: &[Option<Src>], flat: &mut Flat, depth: usize) -> Result<Vec<Option<Src>>, String> {
    if depth > 16 {
        return Err("subgraphs nested more than 16 deep (a subgraph that contains itself?)".into());
    }
    let table = ui_links(links);
    let src_of = |link: &J| -> Option<Src> {
        let (origin, slot) = *table.get(&link.as_i64()?)?;
        if origin == -10 { parent.get(slot).cloned().flatten() } else { Some(Src::Link(format!("{prefix}{origin}"), slot)) }
    };
    for n in nodes.as_array().into_iter().flatten() {
        let id = format!("{prefix}{}", id_of(n));
        let class = n["type"].as_str().unwrap_or_default().to_string();
        let mode = n["mode"].as_i64().unwrap_or(0);
        let ins: Vec<(String, Option<Src>, String)> = n["inputs"].as_array().into_iter().flatten().map(|i| {
            (i["name"].as_str().unwrap_or_default().to_string(), src_of(&i["link"]), i["type"].as_str().unwrap_or("*").to_string())
        }).collect();
        if let Some(sub) = defs.get(&class) {
            // the instance's own widget values fill the subgraph's inputs that aren't linked, in widget order
            let wv = n["widgets_values"].as_array().cloned().unwrap_or_default();
            let mut wi = 0;
            let by_name: BTreeMap<String, Option<Src>> = n["inputs"].as_array().into_iter().flatten().map(|i| {
                let name = i["name"].as_str().unwrap_or_default().to_string();
                let mut s = src_of(&i["link"]);
                if i["widget"].is_object() {
                    if s.is_none() {
                        s = wv.get(wi).cloned().map(Src::Lit);
                    }
                    wi += 1;
                }
                (name, s)
            }).collect();
            let feed: Vec<Option<Src>> = sub["inputs"].as_array().into_iter().flatten().map(|si| by_name.get(si["name"].as_str().unwrap_or_default()).cloned().flatten()).collect();
            let outs = if mode == 2 { vec![] } else { flatten(&sub["nodes"], &sub["links"], &format!("{id}:"), defs, &feed, flat, depth + 1)? };
            for (k, o) in outs.into_iter().enumerate() {
                flat.alias.insert((id.clone(), k), o);
            }
            continue;
        }
        let first = || ins.first().and_then(|(_, v, _)| v.clone());
        match class.as_str() {
            "Note" | "MarkdownNote" | "Label (rgthree)" => continue,
            "Reroute" => { flat.alias.insert((id, 0), first()); continue; }
            "PrimitiveNode" => { flat.alias.insert((id, 0), n["widgets_values"].get(0).cloned().map(Src::Lit)); continue; }
            _ => {}
        }
        if mode == 2 {
            for k in 0..8 { flat.alias.insert((id.clone(), k), None); }
            continue;
        }
        if mode == 4 {
            // bypassed: each output passes the first linked input of its type
            for (k, o) in n["outputs"].as_array().into_iter().flatten().enumerate() {
                let ty = o["type"].as_str().unwrap_or("*");
                let pass = ins.iter().find(|i| i.1.is_some() && (i.2 == ty || ty == "*")).and_then(|i| i.1.clone());
                flat.alias.insert((id.clone(), k), pass);
            }
            continue;
        }
        flat.nodes.push((id, class, mode, n["widgets_values"].clone(), ins, n["title"].as_str().map(str::to_string)));
    }
    // what this level hands out: links into its output node (-20)
    let mut outs: Vec<Option<Src>> = Vec::new();
    for l in links.as_array().into_iter().flatten() {
        let (target, tslot, link_id) = match l {
            J::Array(a) => (a[3].as_i64(), a[4].as_u64(), a[0].clone()),
            _ => (l["target_id"].as_i64(), l["target_slot"].as_u64(), l["id"].clone()),
        };
        if target == Some(-20) {
            let k = tslot.unwrap_or(0) as usize;
            if outs.len() <= k {
                outs.resize(k + 1, None);
            }
            outs[k] = src_of(&link_id);
        }
    }
    Ok(outs)
}

fn from_ui(v: &J, schema: &Schema) -> Result<Graph, String> {
    let defs: BTreeMap<String, J> = v["definitions"]["subgraphs"].as_array().into_iter().flatten().filter_map(|s| Some((s["id"].as_str()?.to_string(), s.clone()))).collect();
    let mut flat = Flat { nodes: Vec::new(), alias: BTreeMap::new() };
    flatten(&v["nodes"], &v["links"], "", &defs, &[], &mut flat, 0)?;
    // follow aliases to a real node's output (or a literal)
    let resolve = |s: &Src| -> Option<Src> {
        let mut s = s.clone();
        for _ in 0..64 {
            match &s {
                Src::Link(id, slot) => match flat.alias.get(&(id.clone(), *slot)) {
                    Some(next) => s = next.clone()?,
                    None => return Some(s),
                },
                Src::Lit(_) => return Some(s),
            }
        }
        None
    };
    let mut g = Graph::new();
    for (id, class, _, wv, ins, title) in &flat.nodes {
        let c = schema.get(class);
        let mut inputs: Map<String, J> = name_widgets(c, wv).into_iter().filter(|(k, _)| c.is_none() || !k.starts_with("widget_")).collect();
        for (name, src, _) in ins {
            match src.as_ref().and_then(resolve) {
                Some(Src::Link(n, slot)) => { inputs.insert(name.clone(), json!([n, slot])); }
                Some(Src::Lit(l)) => { inputs.insert(name.clone(), l); }
                None => {}
            }
        }
        g.insert(id.clone(), Node { class: class.clone(), inputs, title: title.clone() });
    }
    // a link to a node that went (muted, a note) is dropped rather than left dangling
    let ids: std::collections::BTreeSet<String> = g.keys().cloned().collect();
    for n in g.values_mut() {
        n.inputs.retain(|_, v| link(v).is_none_or(|(id, _)| ids.contains(id)));
    }
    if g.is_empty() {
        return Err("the workflow has no nodes that run".into());
    }
    Ok(g)
}

/// API format, as /prompt takes it.
pub fn to_api(g: &Graph) -> J {
    J::Object(g.iter().map(|(id, n)| {
        let mut o = json!({"class_type": n.class, "inputs": n.inputs});
        if let Some(t) = &n.title {
            o["_meta"] = json!({"title": t});
        }
        (id.clone(), o)
    }).collect())
}

/// The nodes `id` reads from, nearest first, itself included.
pub fn upstream(g: &Graph, id: &str) -> Vec<String> {
    let (mut seen, mut queue) = (Vec::new(), vec![id.to_string()]);
    while let Some(n) = queue.pop() {
        if seen.contains(&n) {
            continue;
        }
        if let Some(node) = g.get(&n) {
            queue.extend(node.inputs.values().filter_map(link).map(|(s, _)| s.to_string()));
        }
        seen.push(n);
    }
    seen
}

/// A UI node's id as text (a number in most workflows, a string in some).
pub(crate) fn id_of(n: &J) -> String {
    match &n["id"] {
        J::Number(i) => i.to_string(),
        J::String(s) => s.clone(),
        _ => String::new(),
    }
}
