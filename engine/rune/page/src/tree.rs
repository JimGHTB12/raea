//! The page as the wire's ops leave it (R1a): every element by its wire id (0 is the root), with its kind, props, text,
//! the events it listens for and its children in order. It applies a batch the way sdk/ui's strict test host does
//! (sdk/ui/test/host.ts): an op on an id it doesn't know, a second create, a listen twice or an empty update is an
//! error, so a driver's mistake fails a test instead of passing quietly. `view` (R2) draws this tree with Rune.

use std::collections::{HashMap, HashSet};

use raea_core_wire::{Ev, Op, decode};
use serde_json::{Map, Value, json};

/// The root's id, as the wire numbers it (sdk/ui wire.ts ROOT).
pub const ROOT: u64 = 0;

/// One element of the page.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub kind: String,
    pub props: Map<String, Value>,
    /// a text element's string
    pub text: Option<String>,
    /// the events it listens for
    pub on: HashSet<Ev>,
    pub children: Vec<u64>,
    pub parent: Option<u64>,
}

impl Node {
    fn new(kind: &str, props: Map<String, Value>, text: Option<String>) -> Self {
        Node { kind: kind.to_string(), props, text, on: HashSet::new(), children: Vec::new(), parent: None }
    }
}

/// The page's elements.
#[derive(Debug)]
pub struct Tree {
    nodes: HashMap<u64, Node>,
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

impl Tree {
    /// A page with only its root.
    pub fn new() -> Self {
        Tree { nodes: HashMap::from([(ROOT, Node::new("#root", Map::new(), None))]) }
    }

    /// Every element's wire id, the root's included.
    pub fn ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.nodes.keys().copied()
    }

    /// An element by its wire id.
    pub fn get(&self, id: u64) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// How many elements there are, the root's included.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the page holds only its root.
    pub fn is_empty(&self) -> bool {
        self.nodes.len() == 1
    }

    /// A batch as the JS flushed it (`{v, ops}`), applied in order. A batch of another version is refused whole; an op
    /// that can't apply stops the batch there with why.
    pub fn apply_json(&mut self, json: &str) -> Result<(), String> {
        for op in decode(json)? {
            self.apply(&op)?;
        }
        Ok(())
    }

    /// One op.
    pub fn apply(&mut self, op: &Op) -> Result<(), String> {
        match op {
            Op::Clear => {
                for c in std::mem::take(&mut self.node_mut(ROOT)?.children) {
                    self.node_mut(c)?.parent = None;
                    self.drop_subtree(c);
                }
            }
            Op::Create { id, kind, props, text } => {
                if self.nodes.contains_key(id) {
                    return Err(format!("element {id} made twice"));
                }
                self.nodes.insert(*id, Node::new(kind, props.clone(), text.clone()));
            }
            Op::Append { parent, child } => self.place(*parent, *child, None)?,
            Op::Insert { parent, child, before } => self.place(*parent, *child, Some(*before))?,
            Op::Remove { id } => {
                self.detach(*id)?;
                self.drop_subtree(*id);
            }
            Op::Update { id, set, unset, text } => {
                if set.is_none() && unset.is_none() && text.is_none() {
                    return Err(format!("an empty update of {id}"));
                }
                let n = self.node_mut(*id)?;
                for (k, v) in set.iter().flatten() {
                    n.props.insert(k.clone(), v.clone());
                }
                for k in unset.iter().flatten() {
                    n.props.remove(k);
                }
                if let Some(t) = text {
                    n.text = Some(t.clone());
                }
            }
            Op::Listen { id, events } => {
                let n = self.node_mut(*id)?;
                for e in events {
                    if !n.on.insert(*e) {
                        return Err(format!("{id} listens for {} twice", ev_name(*e)));
                    }
                }
            }
            Op::Unlisten { id, events } => {
                let n = self.node_mut(*id)?;
                for e in events {
                    if !n.on.remove(e) {
                        return Err(format!("{id} unlistens {}, which it didn't listen for", ev_name(*e)));
                    }
                }
            }
        }
        Ok(())
    }

    /// The tree from `id` as sdk/ui's test host's `shape()` dumps it: `[kind, props, text|null, events, children]`,
    /// events sorted by name, ids left out (two drivers number their elements differently).
    pub fn shape(&self, id: u64) -> Value {
        let Some(n) = self.nodes.get(&id) else { return Value::Null };
        let mut events: Vec<&str> = n.on.iter().map(|e| ev_name(*e)).collect();
        events.sort_unstable();
        let mut keys: Vec<&String> = n.props.keys().collect();
        keys.sort();
        let props: Map<String, Value> = keys.into_iter().map(|k| (k.clone(), n.props[k].clone())).collect();
        let kids: Vec<Value> = n.children.iter().map(|c| self.shape(*c)).collect();
        json!([n.kind, props, n.text, events, kids])
    }

    fn node_mut(&mut self, id: u64) -> Result<&mut Node, String> {
        self.nodes.get_mut(&id).ok_or_else(|| format!("no element {id}"))
    }

    /// `child` under `parent`, last or before `before`, moved there if it was elsewhere.
    fn place(&mut self, parent: u64, child: u64, before: Option<u64>) -> Result<(), String> {
        self.node_mut(parent)?;
        self.detach(child)?;
        let p = self.node_mut(parent)?;
        match before {
            None => p.children.push(child),
            Some(b) => {
                let at = p.children.iter().position(|c| *c == b).ok_or_else(|| format!("insert before {b}, which isn't under {parent}"))?;
                p.children.insert(at, child);
            }
        }
        self.node_mut(child)?.parent = Some(parent);
        Ok(())
    }

    /// `id` out of its parent's children (nothing when it has no parent).
    fn detach(&mut self, id: u64) -> Result<(), String> {
        let Some(parent) = self.node_mut(id)?.parent.take() else { return Ok(()) };
        let p = self.node_mut(parent)?;
        p.children.retain(|c| *c != id);
        Ok(())
    }

    /// `id` and everything in it gone.
    fn drop_subtree(&mut self, id: u64) {
        if let Some(n) = self.nodes.remove(&id) {
            for c in n.children {
                self.drop_subtree(c);
            }
        }
    }
}

/// An event's name on the wire ("pointerdown").
pub fn ev_name(e: Ev) -> &'static str {
    match e {
        Ev::PointerDown => "pointerdown",
        Ev::PointerMove => "pointermove",
        Ev::PointerUp => "pointerup",
        Ev::PointerEnter => "pointerenter",
        Ev::PointerLeave => "pointerleave",
        Ev::ContextMenu => "contextmenu",
        Ev::Wheel => "wheel",
        Ev::Scroll => "scroll",
        Ev::Change => "change",
        Ev::KeyDown => "keydown",
        Ev::Focus => "focus",
        Ev::Blur => "blur",
        Ev::Select => "select",
        Ev::Close => "close",
        Ev::KeyUp => "keyup",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(t: &mut Tree, ops: &str) -> Result<(), String> {
        t.apply_json(&format!(r#"{{"v":1,"ops":[{ops}]}}"#))
    }

    #[test]
    fn applies_each_op() {
        let mut t = Tree::new();
        run(
            &mut t,
            r#"{"op":"create","id":1,"kind":"div","props":{"style":{"width":3}}},{"op":"create","id":2,"kind":"text","text":"a"},
            {"op":"create","id":3,"kind":"text","text":"b"},{"op":"append","parent":0,"child":1},{"op":"append","parent":1,"child":2},
            {"op":"insert","parent":1,"child":3,"before":2},{"op":"listen","id":1,"events":["pointerdown","keydown"]}"#,
        )
        .unwrap();
        assert_eq!(t.get(1).unwrap().children, vec![3, 2]);
        // a move: 2 goes under the root, out of 1
        run(&mut t, r#"{"op":"append","parent":0,"child":2},{"op":"update","id":1,"set":{"title":"x"},"unset":["style"]},{"op":"update","id":3,"text":"c"}"#).unwrap();
        assert_eq!(t.get(1).unwrap().children, vec![3]);
        assert_eq!(t.get(0).unwrap().children, vec![1, 2]);
        assert_eq!(t.shape(1), json!(["div", {"title": "x"}, null, ["keydown", "pointerdown"], [["text", {}, "c", [], []]]]));
        run(&mut t, r#"{"op":"unlisten","id":1,"events":["keydown"]},{"op":"remove","id":1}"#).unwrap();
        assert!(t.get(3).is_none(), "a removed element's subtree goes with it");
        run(&mut t, r#"{"op":"clear"}"#).unwrap();
        assert!(t.is_empty());
    }

    #[test]
    fn strict_like_the_test_host() {
        let mut t = Tree::new();
        run(&mut t, r#"{"op":"create","id":1,"kind":"div"}"#).unwrap();
        assert!(run(&mut t, r#"{"op":"create","id":1,"kind":"div"}"#).unwrap_err().contains("twice"));
        assert!(run(&mut t, r#"{"op":"append","parent":0,"child":9}"#).unwrap_err().contains("no element 9"));
        assert!(run(&mut t, r#"{"op":"update","id":1}"#).unwrap_err().contains("empty update"));
        assert!(run(&mut t, r#"{"op":"insert","parent":0,"child":1,"before":5}"#).unwrap_err().contains("isn't under"));
        run(&mut t, r#"{"op":"listen","id":1,"events":["wheel"]}"#).unwrap();
        assert!(run(&mut t, r#"{"op":"listen","id":1,"events":["wheel"]}"#).unwrap_err().contains("twice"));
        assert!(run(&mut t, r#"{"op":"unlisten","id":1,"events":["blur"]}"#).unwrap_err().contains("didn't listen"));
    }

    #[test]
    fn refuses_another_version_whole() {
        let mut t = Tree::new();
        assert!(t.apply_json(r#"{"v":2,"ops":[{"op":"create","id":1,"kind":"div"}]}"#).unwrap_err().contains("version 2"));
        assert!(t.get(1).is_none());
    }
}
