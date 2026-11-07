//! The op stream split between the page (Blitz) and its Rune layers. An element is Blitz's or a layer's, by where it's
//! attached: under a marked element (the layer's slot, itself Blitz's) or under one of the layer's own. React builds a
//! subtree before it attaches it (creates, appends inside it, then one append of its top), so ops for an element not yet
//! attached wait with its subtree's until it is; then each goes to its element's owner, in the order the JS sent them.
//! A layer's ops have the slot as their parent rewritten to the layer page's root (raea-rune-page's ROOT).

use std::collections::{BTreeMap, HashMap};

use raea_core_wire::Op;
use serde_json::Value;

/// The prop that marks an element whose subtree Rune draws, and its value.
pub const MARK: (&str, &str) = ("data-raea-renderer", "rune");

/// The root's id in a layer's page (raea-rune-page's tree::ROOT, sdk/ui wire.ts ROOT).
const ROOT: u64 = 0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Owner {
    Page,
    /// the layer whose slot is this element id
    Layer(u64),
}

/// What one batch becomes.
#[derive(Default, Debug)]
pub struct Routed {
    /// the page's own ops, for Blitz
    pub page: Vec<Op>,
    /// each layer's ops by its slot's id, for its Rune page; a layer's first ops make it
    pub layers: BTreeMap<u64, Vec<Op>>,
    /// layers whose slot went (removed, or the page cleared)
    pub gone: Vec<u64>,
    /// what can't be routed, said and not dropped: an element moved between the page and a layer
    pub refused: Vec<String>,
}

/// One page's routing, kept across its batches.
#[derive(Default)]
pub struct Router {
    /// attached elements' owners
    owner: HashMap<u64, Owner>,
    /// the slots (marked elements), by id
    slots: std::collections::HashSet<u64>,
    /// unattached subtrees: each element's top, and each top's ops waiting
    top: HashMap<u64, u64>,
    waiting: HashMap<u64, Vec<(u64, Op)>>,
    /// each op's place in the stream, so a subtree's ops leave in the order the JS sent them
    seq: u64,
}

fn marked(props: &serde_json::Map<String, Value>) -> bool {
    props.get(MARK.0).and_then(Value::as_str) == Some(MARK.1)
}

/// Whether `op` makes a slot: a page whose batches never have one needn't be routed.
pub fn creates_slot(op: &Op) -> bool {
    matches!(op, Op::Create { props, .. } if marked(props))
}

/// The element an op is about (its child, for a move).
fn subject(op: &Op) -> Option<u64> {
    match op {
        Op::Clear => None,
        Op::Create { id, .. } | Op::Remove { id } | Op::Update { id, .. } | Op::Listen { id, .. } | Op::Unlisten { id, .. } => Some(*id),
        Op::Append { child, .. } | Op::Insert { child, .. } => Some(*child),
    }
}

fn parent_of(op: &Op) -> Option<u64> {
    match op {
        Op::Append { parent, .. } | Op::Insert { parent, .. } => Some(*parent),
        _ => None,
    }
}

impl Router {
    /// Whether any layer has been seen: a page without one routes nothing (all of it is Blitz's).
    pub fn has_layers(&self) -> bool {
        !self.slots.is_empty()
    }

    /// `ops` split.
    pub fn route(&mut self, ops: Vec<Op>) -> Routed {
        let mut out = Routed::default();
        for op in ops {
            self.one(op, &mut out);
        }
        out
    }

    /// Whom an attached element's children belong to: one the router never saw is the page's (made before its
    /// first slot, when its batches went to Blitz unrouted).
    fn under(&self, parent: u64) -> Owner {
        if self.slots.contains(&parent) && self.owner.get(&parent) == Some(&Owner::Page) {
            return Owner::Layer(parent);
        }
        self.owner.get(&parent).copied().unwrap_or(Owner::Page)
    }

    fn one(&mut self, op: Op, out: &mut Routed) {
        self.seq += 1;
        let n = self.seq;
        if let Op::Clear = op {
            out.gone.extend(self.slots.iter().copied().filter(|s| self.owner.get(s) == Some(&Owner::Page)));
            out.gone.sort_unstable();
            *self = Router { seq: n, ..Router::default() };
            out.page.push(Op::Clear);
            return;
        }
        let Some(id) = subject(&op) else { return };
        if let Op::Create { props, .. } = &op
            && marked(props)
        {
            self.slots.insert(id);
        }
        if let Some(parent) = parent_of(&op) {
            return self.attach(n, op, id, parent, out);
        }
        if let Op::Create { .. } = op {
            self.top.insert(id, id);
            self.waiting.insert(id, vec![(n, op)]);
            return;
        }
        // an op on an element: its owner's, or waiting with its subtree
        if let Some(t) = self.top.get(&id) {
            self.waiting.get_mut(t).expect("a waiting subtree").push((n, op));
            return;
        }
        let owner = self.owner.get(&id).copied().unwrap_or(Owner::Page);
        if let Op::Remove { .. } = op {
            if self.slots.remove(&id) && owner == Owner::Page {
                out.gone.push(id);
            }
            self.owner.remove(&id);
        }
        send(owner, op, out);
    }

    /// An append or insert of `child` under `parent`.
    fn attach(&mut self, n: u64, op: Op, child: u64, parent: u64, out: &mut Routed) {
        // into a subtree that isn't attached yet: it waits with it
        if let Some(&t) = self.top.get(&parent) {
            let mut ops = self.waiting.remove(&child).unwrap_or_default();
            for v in self.top.values_mut().filter(|v| **v == child) {
                *v = t;
            }
            let w = self.waiting.get_mut(&t).expect("a waiting subtree");
            w.append(&mut ops);
            w.push((n, op));
            self.top.insert(child, t);
            return;
        }
        let to = self.under(parent);
        match self.waiting.remove(&child) {
            // a subtree attached: each of its ops to its element's owner, in order
            Some(mut ops) => {
                ops.push((n, op));
                ops.sort_by_key(|(n, _)| *n);
                let ops: Vec<Op> = ops.into_iter().map(|(_, op)| op).collect();
                self.top.retain(|_, t| *t != child);
                self.flush(ops, child, to, out);
            }
            // an attached element moved
            None => {
                let from = self.owner.get(&child).copied().unwrap_or(Owner::Page);
                if from != to {
                    out.refused.push(format!("element {child} moved between the page and a Rune layer (it stays where it was drawn)"));
                    return;
                }
                send(to, op, out);
            }
        }
    }

    /// A subtree's waiting ops, now that its top `top` is attached under `to`'s.
    fn flush(&mut self, ops: Vec<Op>, top: u64, to: Owner, out: &mut Routed) {
        // the subtree's shape from its appends, then each element's owner from its top down
        let mut parent: HashMap<u64, u64> = HashMap::new();
        for op in &ops {
            if let (Some(c), Some(p)) = (subject(op), parent_of(op))
                && c != top
            {
                parent.insert(c, p);
            }
        }
        self.owner.insert(top, to);
        let mut members: Vec<u64> = ops.iter().filter_map(subject).collect();
        members.dedup();
        for m in members {
            let mut chain = vec![m];
            while let Some(&p) = parent.get(chain.last().unwrap()) {
                if self.owner.contains_key(&p) || p == top {
                    chain.push(p);
                    break;
                }
                chain.push(p);
            }
            // owners down the chain from the first one known
            for w in chain.windows(2).rev() {
                let (c, p) = (w[0], w[1]);
                if !self.owner.contains_key(&c) {
                    let o = self.under(p);
                    self.owner.insert(c, o);
                }
            }
        }
        for op in ops {
            let id = subject(&op).expect("a subtree's op names its element");
            let owner = self.owner.get(&id).copied().unwrap_or(to);
            send(owner, op, out);
        }
    }
}

/// `op` to `owner`, a layer's with its slot as parent made its root.
fn send(owner: Owner, op: Op, out: &mut Routed) {
    match owner {
        Owner::Page => out.page.push(op),
        Owner::Layer(slot) => {
            let op = match op {
                Op::Append { parent, child } if parent == slot => Op::Append { parent: ROOT, child },
                Op::Insert { parent, child, before } if parent == slot => Op::Insert { parent: ROOT, child, before },
                op => op,
            };
            out.layers.entry(slot).or_default().push(op);
        }
    }
}

#[cfg(test)]
#[path = "route_tests.rs"]
mod tests;
