//! The page's half of the layer pass: every node's layer (the nearest stamped element up from it, engine/bundle
//! stamp.rs), the page painted again with those as colours (scene.rs), and the layers it has this frame.

use std::collections::HashMap;

use blitz_dom::{BaseDocument, LocalName, NodeId};

use crate::{Layer, key_of};

/// The page as layer keys: each pixel's (row-major, `w` x `h`; None where no layer shows) by its index in `layers`,
/// and the layers shown or laid out this frame, in document order. `scale`: page px to frame px.
pub fn page_ids(doc: &mut BaseDocument, scale: f64, w: u32, h: u32) -> (Vec<u16>, Vec<Layer>) {
    let mut ids: HashMap<NodeId, u16> = HashMap::new();
    let mut layers: Vec<Layer> = Vec::new();
    // (node, the layer it's in, as an index into `layers` plus one)
    let mut stack = vec![(doc.root_node().id, 0u16)];
    while let Some((n, mut at)) = stack.pop() {
        let Some(node) = doc.get_node(n) else { continue };
        if node.is_element() && matches!(node.element_data().map(|e| &*e.name.local), Some("style" | "script" | "head")) {
            continue;
        }
        let attr = |a: &str| node.attr(LocalName::from(a));
        let key = node.is_element().then(|| key_of(attr("data-raea-loc"), attr("data-raea-idx"), attr("data-raea-use"))).flatten();
        if let (Some(key), true) = (key, node.has_boxes() && layers.len() < u16::MAX as usize - 1) {
            let tag = node.element_data().map(|e| e.name.local.to_string()).unwrap_or_default();
            let text = node.children.iter().all(|c| doc.get_node(*c).is_some_and(|c| c.is_text_node())) && !node.text_content().trim().is_empty();
            let (kind, src) = match (tag.as_str(), attr("data-raea-src")) {
                (_, Some(src)) => ("video", Some(src.to_string())),
                ("img", _) => ("image", attr("src").map(str::to_string)),
                _ if text => ("text", None),
                _ => ("element", None),
            };
            let name = match kind {
                "text" => node.text_content().trim().chars().take(40).collect(),
                _ => tag,
            };
            let (loc, rest) = key.split_once('|').map_or((key.as_str(), None), |(l, r)| (l, Some(r)));
            let (loc, index) = loc.split_once('#').map_or((loc, None), |(l, i)| (l, Some(i.to_string())));
            layers.push(Layer {
                kind,
                name,
                parent: at.checked_sub(1).map(|p| layers[p as usize].key.clone()),
                loc: loc.to_string(),
                uses: rest.map(|r| r.split('|').map(str::to_string).collect()).unwrap_or_default(),
                index,
                src,
                key,
            });
            at = layers.len() as u16;
        }
        if at > 0 {
            ids.insert(n, at);
        }
        stack.extend(node.children.iter().rev().map(|c| (*c, at)));
    }
    let mut scene = crate::scene::IdScene::new(w.min(u16::MAX as u32) as u16, h.min(u16::MAX as u32) as u16, &ids);
    blitz_paint::paint_scene(&mut scene, doc, scale, w, h, 0, 0);
    (scene.finish(), layers)
}
