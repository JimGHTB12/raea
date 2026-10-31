//! The op wire between a page's script and the engine, typed: what a UI driver sends (a batch of ops) and what the
//! host sends back (events, host events, answers). No Bevy here, so a Bevy upgrade doesn't reach the protocol.
//! React through @raea/ui's adapter emits these ops (sdk/ui/src/wire.ts is the JS side, and says what each op means).
//! `tree` diffs a page's elements into ops; `raw` reads a batch's updates where they lie in its JSON.

pub mod raw;
pub mod tree;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

/// The op protocol's version: every batch carries it, and a batch of another is refused whole (sdk/ui OPS_VERSION).
pub const OPS_VERSION: u32 = 1;

/// One flush of ops.
#[derive(Deserialize, Debug)]
pub struct Batch {
    pub v: u32,
    pub ops: Vec<Op>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
pub enum Op {
    /// everything under the root goes (a fresh mount)
    Clear,
    /// an element; `text` is a text element's string
    Create {
        id: u64,
        kind: String,
        #[serde(default)]
        props: Map<String, Value>,
        #[serde(default)]
        text: Option<String>,
    },
    /// child last under parent, moved there if it was elsewhere
    Append { parent: u64, child: u64 },
    /// child before `before` under parent
    Insert { parent: u64, child: u64, before: u64 },
    /// the element and everything in it go
    Remove { id: u64 },
    /// props set (each replacing its whole value), props unset, a text element's string
    Update {
        id: u64,
        #[serde(default)]
        set: Option<Map<String, Value>>,
        #[serde(default)]
        unset: Option<Vec<String>>,
        #[serde(default)]
        text: Option<String>,
    },
    /// the element's events start coming back
    Listen { id: u64, events: Vec<Ev> },
    /// ...and stop
    Unlisten { id: u64, events: Vec<Ev> },
}

/// An element's event: the host's own names (DOM's event types), whatever a driver calls its handler props.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Ev {
    PointerDown,
    PointerMove,
    PointerUp,
    PointerEnter,
    PointerLeave,
    ContextMenu,
    Wheel,
    Scroll,
    Change,
    KeyDown,
    Focus,
    Blur,
    Select,
    Close,
    /// a key let go (DOM roots: sdk/ui's dom layer; src/host.rs sends it to the element with the focus then)
    KeyUp,
}

impl Ev {
    fn bit(self) -> u32 {
        1 << self as u16
    }
}

/// The events an element listens for.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Listening(u32);

impl Listening {
    pub fn has(self, e: Ev) -> bool {
        self.0 & e.bit() != 0
    }
    pub fn add(&mut self, e: Ev) {
        self.0 |= e.bit();
    }
    pub fn remove(&mut self, e: Ev) {
        self.0 &= !e.bit();
    }
}

/// A batch as the JS flushed it: its ops, or why it's refused (not JSON of a batch, another version).
pub fn decode(json: &str) -> Result<Vec<Op>, String> {
    let b: Batch = serde_json::from_str(json).map_err(|e| format!("a batch that isn't one: {e}"))?;
    if b.v != OPS_VERSION {
        return Err(format!("a batch of op version {}; this host speaks {OPS_VERSION} (rebuild the space's bundle against sdk/ui)", b.v));
    }
    Ok(b.ops)
}

/// An element's event, for its listener.
pub fn event(id: u64, e: Ev, payload: Value) -> Value {
    json!({"t": "event", "id": id, "event": e, "payload": payload})
}

/// A host event (resize, scan, agent, keydown, frame, action).
pub fn named(name: &str, value: Value) -> Value {
    json!({"t": "named", "name": name, "value": value})
}

/// The host's answer to a call.
pub fn answer(id: u64, r: Result<Value, String>) -> Value {
    match r {
        Ok(v) => json!({"t": "answer", "id": id, "ok": true, "value": v}),
        Err(e) => json!({"t": "answer", "id": id, "ok": false, "error": e}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_each_op() {
        let ops = decode(
            r#"{"v":1,"ops":[{"op":"clear"},{"op":"create","id":1,"kind":"text","props":{},"text":"hi"},{"op":"append","parent":0,"child":1},
            {"op":"insert","parent":0,"child":2,"before":1},{"op":"remove","id":2},{"op":"update","id":1,"set":{"style":{"width":3}},"unset":["className"],"text":"yo"},
            {"op":"listen","id":1,"events":["pointerdown","keydown"]},{"op":"unlisten","id":1,"events":["keydown"]}]}"#,
        )
        .unwrap();
        assert_eq!(ops.len(), 8);
        assert_eq!(ops[6], Op::Listen { id: 1, events: vec![Ev::PointerDown, Ev::KeyDown] });
        let mut l = Listening::default();
        l.add(Ev::Close);
        l.add(Ev::PointerDown);
        l.remove(Ev::PointerDown);
        assert!(l.has(Ev::Close) && !l.has(Ev::PointerDown));
    }

    #[test]
    fn refuses_another_version_and_old_ops() {
        assert!(decode(r#"{"v":2,"ops":[]}"#).unwrap_err().contains("version 2"));
        // bevy-react's wire (an array, updateText, createText) isn't this one
        assert!(decode(r#"[{"op":"createText","id":1,"text":"x"}]"#).is_err());
        assert!(decode(r#"{"v":1,"ops":[{"op":"updateText","id":1,"text":"x"}]}"#).is_err());
        assert!(decode(r#"{"v":1,"ops":[{"op":"remove","parent":0,"child":1}]}"#).is_err());
    }

    #[test]
    fn event_names_are_the_wire_s() {
        assert_eq!(event(3, Ev::PointerDown, Value::Null)["event"], "pointerdown");
        assert_eq!(serde_json::to_value(Ev::ContextMenu).unwrap(), "contextmenu");
    }
}
