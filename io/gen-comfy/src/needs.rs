//! What a workflow needs from a ComfyUI server beyond what it has: the node packs its classes come from and the model
//! files its loaders read. A UI-format workflow says both itself (each node's `properties.cnr_id`, the ComfyUI
//! registry's id for its pack, or `aux_id`, a GitHub repo; `properties.models`, each file with its download URL and
//! models/ folder). Classes it doesn't attribute, and API-format graphs, are looked up in ComfyUI-Manager's
//! extension-node-map (which repo provides which class), kept in ~/.cache/raea-gen, then the registry's node search. tools/comfy/setup.sh nodes|models
//! installs them on this machine's server, tools/comfy/modal_comfy.py on the hosted one.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value as J, json};

use crate::schema::Schema;

/// Classes neither the map nor the registry search knows, found by hand (10-29): (class, repo).
const KNOWN: &[(&str, &str)] = &[
    ("ImageConstant", "https://github.com/spacepxl/ComfyUI-Image-Filters"),
    // utility-audioseparation's own note names this pack; its __init__.py registers the class
    ("AudioStemSeparate", "https://github.com/lum3on/ComfyUI_AudioTools"),
];

/// The ComfyUI registry's pack for a class, by its node search (api.comfy.org); None when it finds none or several.
fn registry_pack(class: &str) -> Option<String> {
    let q: String = class.bytes().map(|b| if b.is_ascii_alphanumeric() { (b as char).to_string() } else { format!("%{b:02X}") }).collect();
    let r = raea_io_gen_core::http::call("GET", &format!("https://api.comfy.org/nodes/search?comfy_node_search={q}&limit=2"), &[], raea_io_gen_core::http::Body::None, None, 30).ok()?;
    let v = r.json().ok()?;
    match v["nodes"].as_array()?.as_slice() {
        [one] => one["id"].as_str().map(str::to_string),
        _ => None,
    }
}

const NODE_MAP: &str = "https://raw.githubusercontent.com/ltdrdata/ComfyUI-Manager/main/extension-node-map.json";

/// ComfyUI-Manager's class → repo map (the first repo listing a class; several packs reuse some names).
fn node_map() -> BTreeMap<String, String> {
    let f = raea_io_gen_core::made::cached("comfy-extension-node-map.json");
    if !f.is_file() {
        let _ = std::fs::create_dir_all(f.parent().unwrap_or(&f));
        let _ = raea_io_gen_core::http::call("GET", NODE_MAP, &[], raea_io_gen_core::http::Body::None, Some(&f), 120);
    }
    let v: J = std::fs::read(&f).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or(J::Null);
    let mut m = BTreeMap::new();
    for (repo, entry) in v.as_object().into_iter().flatten() {
        for class in entry[0].as_array().into_iter().flatten().filter_map(|c| c.as_str()) {
            m.entry(class.to_string()).or_insert_with(|| repo.clone());
        }
    }
    m
}

/// Every node of a UI-format workflow, its subgraphs' too.
fn ui_nodes(v: &J) -> Vec<&J> {
    let mut out: Vec<&J> = v["nodes"].as_array().into_iter().flatten().collect();
    for s in v["definitions"]["subgraphs"].as_array().into_iter().flatten() {
        out.extend(s["nodes"].as_array().into_iter().flatten());
    }
    out
}

/// What `workflow` needs that a server with `schema` lacks: {"packs": registry ids, "repos": git URLs, "models":
/// [{name, url, directory}], "unknown": classes nobody is known to provide}.
pub fn needs(workflow: &J, schema: &Schema) -> J {
    let (mut packs, mut repos, mut unknown) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    let mut models: BTreeMap<String, J> = BTreeMap::new();
    let mut loose = BTreeSet::new();
    let ui = workflow["nodes"].is_array();
    let classes: Vec<(String, Option<&J>)> = if ui {
        ui_nodes(workflow).into_iter().map(|n| (n["type"].as_str().unwrap_or("").to_string(), Some(&n["properties"]))).collect()
    } else {
        workflow.as_object().into_iter().flatten().map(|(_, n)| (n["class_type"].as_str().unwrap_or("").to_string(), None)).collect()
    };
    for (class, props) in classes {
        for m in props.and_then(|p| p["models"].as_array()).into_iter().flatten() {
            if let Some(name) = m["name"].as_str() {
                models.insert(name.to_string(), json!({"name": name, "url": m["url"], "directory": m["directory"]}));
            }
        }
        // a subgraph's own node (its type is the subgraph's id), a note, or a class the server has
        if class.is_empty() || schema.contains_key(&class) || class.len() == 36 && class.matches('-').count() == 4 || matches!(class.as_str(), "Note" | "MarkdownNote" | "Reroute" | "PrimitiveNode") {
            continue;
        }
        match props.map(|p| (p["cnr_id"].as_str(), p["aux_id"].as_str())) {
            Some((Some(id), _)) if id != "comfy-core" => { packs.insert(id.to_string()); }
            Some((_, Some(aux))) => { repos.insert(format!("https://github.com/{aux}")); }
            _ => { loose.insert(class); }
        }
    }
    if !loose.is_empty() {
        let map = node_map();
        for class in loose {
            if let Some(repo) = map.get(&class).cloned().or_else(|| KNOWN.iter().find(|k| k.0 == class).map(|k| k.1.to_string())) {
                repos.insert(repo);
            } else if let Some(id) = registry_pack(&class) {
                packs.insert(id);
            } else {
                unknown.insert(class);
            }
        }
    }
    json!({"packs": packs, "repos": repos, "models": models.into_values().collect::<Vec<_>>(), "unknown": unknown})
}
