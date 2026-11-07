use raea_core_wire::{Ev, Op};
use serde_json::{Map, json};

use super::*;

fn create(id: u64, kind: &str) -> Op {
    Op::Create { id, kind: kind.into(), props: Map::new(), text: None }
}

fn slot(id: u64) -> Op {
    let props = json!({"data-raea-renderer": "rune", "className": "w-40 h-20"}).as_object().unwrap().clone();
    Op::Create { id, kind: "div".into(), props, text: None }
}

fn text(id: u64, s: &str) -> Op {
    Op::Create { id, kind: "#text".into(), props: Map::new(), text: Some(s.into()) }
}

fn append(parent: u64, child: u64) -> Op {
    Op::Append { parent, child }
}

/// The page's first mount, as React sends it: built from the leaves up, then its top attached to the root.
fn mount() -> Vec<Op> {
    vec![create(1, "div"), slot(2), create(3, "span"), text(4, "Lights"), append(3, 4), append(2, 3), create(5, "p"), append(1, 5), append(1, 2), append(0, 1)]
}

#[test]
fn a_page_without_a_layer_is_all_blitz_in_order() {
    let mut r = Router::default();
    let ops = vec![create(1, "div"), text(2, "hi"), append(1, 2), append(0, 1)];
    let out = r.route(ops);
    assert_eq!(out.page, vec![create(1, "div"), text(2, "hi"), append(1, 2), append(0, 1)]);
    assert!(out.layers.is_empty() && out.gone.is_empty() && out.refused.is_empty());
    assert!(!r.has_layers());
}

#[test]
fn a_slots_subtree_goes_to_its_layer_under_the_layers_root() {
    let mut r = Router::default();
    let out = r.route(mount());
    assert_eq!(out.page, vec![create(1, "div"), slot(2), create(5, "p"), append(1, 5), append(1, 2), append(0, 1)]);
    assert_eq!(out.layers.len(), 1);
    assert_eq!(out.layers[&2], vec![create(3, "span"), text(4, "Lights"), append(3, 4), append(0, 3)]);
    assert!(out.refused.is_empty(), "{:?}", out.refused);
}

#[test]
fn later_ops_follow_their_elements_owner() {
    let mut r = Router::default();
    r.route(mount());
    let set = json!({"className": "text-red-500"}).as_object().unwrap().clone();
    let out = r.route(vec![
        Op::Update { id: 3, set: Some(set.clone()), unset: None, text: None },
        Op::Listen { id: 3, events: vec![Ev::PointerDown] },
        Op::Update { id: 5, set: Some(set.clone()), unset: None, text: None },
        // a new element straight into the layer's slot, and one into the layer's span
        text(6, "on"),
        append(2, 6),
        create(7, "b"),
        Op::Insert { parent: 3, child: 7, before: 4 },
    ]);
    assert_eq!(out.page, vec![Op::Update { id: 5, set: Some(set.clone()), unset: None, text: None }]);
    assert_eq!(
        out.layers[&2],
        vec![Op::Update { id: 3, set: Some(set), unset: None, text: None }, Op::Listen { id: 3, events: vec![Ev::PointerDown] }, text(6, "on"), append(0, 6), create(7, "b"), Op::Insert { parent: 3, child: 7, before: 4 }]
    );
}

#[test]
fn a_slot_removed_or_cleared_is_gone() {
    let mut r = Router::default();
    r.route(mount());
    let out = r.route(vec![Op::Remove { id: 2 }]);
    assert_eq!((out.page, out.gone), (vec![Op::Remove { id: 2 }], vec![2]));
    let mut r = Router::default();
    r.route(mount());
    let out = r.route(vec![Op::Clear]);
    assert_eq!((out.page, out.gone), (vec![Op::Clear], vec![2]));
    assert!(!r.has_layers());
}

#[test]
fn a_move_between_the_page_and_a_layer_is_refused_not_dropped() {
    let mut r = Router::default();
    r.route(mount());
    let out = r.route(vec![append(2, 5)]);
    assert!(out.page.is_empty() && out.layers.is_empty());
    assert_eq!(out.refused.len(), 1, "{:?}", out.refused);
}

#[test]
fn a_layer_inside_a_layer_is_the_outer_layers_element() {
    let mut r = Router::default();
    let out = r.route(vec![slot(1), slot(2), text(3, "x"), append(2, 3), append(1, 2), append(0, 1)]);
    assert_eq!(out.page, vec![slot(1), append(0, 1)]);
    assert_eq!(out.layers[&1], vec![slot(2), text(3, "x"), append(2, 3), append(0, 2)]);
}
