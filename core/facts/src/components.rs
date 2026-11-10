//! The components a frame's world holds (DR1), and the world itself as data. Each field is measured from the
//! composition, never typed by its author. Names are the ones a check finds by: `world.find({ with: ["Text"] })`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::{Config, TS};

/// An entity: its id in this world, its parent (ChildOf), and its components by name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub parent: Option<u32>,
    pub components: BTreeMap<String, Value>,
}

/// One frame of a composition as a world: what a check queries, and what `raea world` prints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Facts {
    pub frame: i64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub entities: Vec<Entity>,
}

impl Facts {
    pub fn new(frame: i64, width: u32, height: u32, fps: f64) -> Self {
        Facts { frame, width, height, fps, entities: Vec::new() }
    }
    /// A new entity with no components yet: its id.
    pub fn spawn(&mut self, parent: Option<u32>) -> u32 {
        let id = self.entities.len() as u32;
        self.entities.push(Entity { id, parent, components: BTreeMap::new() });
        id
    }
    /// A component put on an entity under its type's name (a second of the same name replaces the first).
    pub fn put<C: Component>(&mut self, id: u32, c: &C) {
        if let Some(e) = self.entities.get_mut(id as usize) {
            e.components.insert(C::NAME.to_string(), serde_json::to_value(c).unwrap_or(Value::Null));
        }
    }
    /// A relation to another entity (Rides, StandsOn, Masks ...), by its name.
    pub fn relate(&mut self, id: u32, name: &str, to: u32) {
        if let Some(e) = self.entities.get_mut(id as usize) {
            e.components.insert(name.to_string(), serde_json::json!({ "to": to }));
        }
    }
    /// The entities that have every one of `with`, in id order (raea/ecs's find, for Rust's own checks).
    pub fn find(&self, with: &[&str]) -> Vec<&Entity> {
        self.entities.iter().filter(|e| with.iter().all(|n| e.components.contains_key(*n))).collect()
    }
    /// A component of an entity, read back as its type.
    pub fn get<C: Component>(&self, id: u32) -> Option<C> {
        self.entities.get(id as usize)?.components.get(C::NAME).and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    /// An entity's children.
    pub fn children(&self, id: u32) -> Vec<&Entity> {
        self.entities.iter().filter(|e| e.parent == Some(id)).collect()
    }
}

/// A component: a type with the name it goes by in the world.
pub trait Component: Serialize + for<'a> Deserialize<'a> {
    const NAME: &'static str;
}
macro_rules! component {
    ($($t:ident),*) => { $(impl Component for $t { const NAME: &'static str = stringify!($t); })* };
}
component!(Source, Layer, Text, Part, Camera, Shot, Clip, Cue, Line);

/// Where a thing was written: its file and line, the component that drew it, and the chain of uses down to it (the
/// layer key of docs/review.feature R2a).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct Source {
    pub file: Option<String>,
    pub line: Option<u32>,
    pub component: Option<String>,
    /// the layer key as the engine has it
    pub key: Option<String>,
}

/// What a layer is made of. A person's word for an element ("2D text", "3D text") is checked against this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
pub enum Medium {
    #[serde(rename = "2d-type")]
    Type2d,
    #[serde(rename = "2d-graphic")]
    Graphic2d,
    #[serde(rename = "3d")]
    Scene3d,
    #[serde(rename = "footage")]
    Footage,
    #[serde(rename = "page")]
    Page,
}

/// A composited layer of the frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct Layer {
    pub medium: Medium,
    /// its place in the stack, 0 at the back
    pub order: u32,
    pub opacity: f64,
}

/// A run of type, 2D or 3D. `extruded` is its depth in metres (0: flat).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct Text {
    pub words: String,
    pub font: Option<String>,
    /// its letters' height in px of a frame 1080 on its short side (None: not laid out without a render)
    pub size: Option<f64>,
    /// its box in the frame, each 0 to 1: x0, y0, x1, y1 (None: not laid out without a render)
    #[serde(rename = "box")]
    #[ts(rename = "box")]
    pub box_: Option<[f64; 4]>,
    pub extruded: f64,
    /// what it needs before `size` and `box` are known ("render"), when they aren't
    pub needs: Option<String>,
}

/// A 3D mesh or model, placed in the world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct Part {
    /// what draws it: "mesh", "model", "text3d" ...
    pub kind: String,
    /// its name in the scene (`<Mesh3d name>`), or what it draws
    pub name: String,
    /// its taxonomy tags (a component's `meta`, DR11), none until it has one
    pub tags: Vec<String>,
    /// its world position, rotation (x, y, z, w) and scale
    pub at: [f64; 3],
    pub rotation: [f64; 4],
    pub scale: [f64; 3],
    /// its world box, low corner then high (None: a kind raea doesn't measure, or a file it can't read)
    pub min: Option<[f64; 3]>,
    pub max: Option<[f64; 3]>,
    pub triangles: Option<u32>,
    /// a model's file
    pub src: Option<String>,
}

/// The frame's camera.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct Camera {
    pub eye: [f64; 3],
    pub look: [f64; 3],
    /// vertical field of view in degrees (a perspective camera)
    pub fov: Option<f64>,
    /// the height it sees in metres (an orthographic camera)
    pub ortho: Option<f64>,
    pub near: f64,
    pub far: f64,
}

/// A stretch between two cuts, in frames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct Shot {
    pub from: i64,
    pub to: i64,
}

/// A piece of footage on the timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct Clip {
    pub source: String,
    /// seconds into the source this frame shows
    pub time: Option<f64>,
}

/// A sound placed on the timeline: its source, when it starts (seconds) and its gain in dB.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(optional_fields)]
pub struct Cue {
    pub source: String,
    pub at: f64,
    pub gain: Option<f64>,
    pub bus: Option<String>,
}

/// A sung or spoken line: its words with their times in seconds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct Line {
    pub words: Vec<(String, f64, f64)>,
}

/// The components' TypeScript declarations, for a check's types (sdk/raea/src/check.ts is generated from these).
pub fn declarations() -> Vec<String> {
    let cfg = Config::default();
    macro_rules! decls { ($($t:ty),*) => { vec![$(format!("export {}", <$t as TS>::decl(&cfg))),*] }; }
    decls!(Source, Medium, Layer, Text, Part, Camera, Shot, Clip, Cue, Line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_world_is_found_by_component_and_reads_back() {
        let mut w = Facts::new(12, 1920, 1080, 30.0);
        let layer = w.spawn(None);
        w.put(layer, &Layer { medium: Medium::Type2d, order: 1, opacity: 1.0 });
        let text = w.spawn(Some(layer));
        w.put(text, &Text { words: "OUT".into(), font: None, size: Some(96.0), box_: Some([0.6, 0.2, 0.9, 0.4]), extruded: 0.0, needs: None });
        let seat = w.spawn(None);
        let rider = w.spawn(None);
        w.relate(rider, "Rides", seat);
        assert_eq!(w.find(&["Text"]).len(), 1);
        assert_eq!(w.find(&["Text", "Layer"]).len(), 0, "a component on the parent isn't the child's");
        assert_eq!(w.children(layer)[0].id, text);
        assert_eq!(w.get::<Layer>(layer).unwrap().medium, Medium::Type2d);
        assert_eq!(w.find(&["Rides"])[0].components["Rides"]["to"], seat);
        // the JSON is raea/ecs's shape: components by name, a parent, an id
        let j = serde_json::to_value(&w).unwrap();
        assert_eq!(j["entities"][1]["components"]["Text"]["box"][0], 0.6);
        assert_eq!(j["entities"][0]["components"]["Layer"]["medium"], "2d-type");
        assert_eq!(serde_json::from_value::<Facts>(j).unwrap(), w);
    }

    #[test]
    fn every_component_has_a_typescript_declaration() {
        let d = declarations().join("\n");
        for name in ["Source", "Layer", "Text", "Part", "Camera", "Shot", "Clip", "Cue", "Line"] {
            assert!(d.contains(&format!("type {name} ")), "no declaration for {name}: {d}");
        }
        assert!(d.contains("\"2d-type\""), "the medium's words are the person's: {d}");
    }
}
