//! The input: objects (id, size, an optional starting place and yaw, fixed or free) and rules, read from JSON.
//!
//!   {"objects": [{"id": "desk", "size": [1.4, 0.75, 0.7]}, {"id": "lamp", "size": [0.2, 0.5, 0.2]}],
//!    "room": {"min": [-2, -2], "max": [2, 2]},
//!    "rules": [{"against": "desk", "wall": "+z"}, {"on": "lamp", "of": "desk"}, {"near": "chair", "of": "desk", "dist": 0.5},
//!              {"facing": "chair", "of": "desk"}, {"beside": "shelf", "of": "desk", "side": "+x", "gap": 0.1},
//!              {"apart": "a", "of": "b", "min": 1}, {"align": "a", "of": "b", "axis": "x"}, {"clear": "desk", "depth": 0.8}]}

use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Obj {
    pub id: String,
    /// width (x), height (y), depth (z) before the yaw
    pub size: [f64; 3],
    pub at: [f64; 3],
    pub yaw: f64,
    pub fixed: bool,
    /// whether a rule set its yaw (else it stays as given)
    pub turned: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Rule {
    On { a: usize, b: usize },
    Against { a: usize, wall: [f64; 2] },
    Beside { a: usize, b: usize, side: [f64; 2], gap: f64 },
    Near { a: usize, b: usize, dist: f64 },
    Apart { a: usize, b: usize, min: f64 },
    Facing { a: usize, b: usize },
    Align { a: usize, b: usize, axis: usize },
    Clear { a: usize, depth: f64 },
}

fn dir(s: &str) -> Result<[f64; 2], String> {
    Ok(match s {
        "+x" => [1.0, 0.0],
        "-x" => [-1.0, 0.0],
        "+z" => [0.0, 1.0],
        "-z" => [0.0, -1.0],
        o => return Err(format!("place: {o:?} isn't a side: +x, -x, +z, -z")),
    })
}

/// An [x, y, z] point, or an error naming `what` (place's and solid info expect files').
pub fn xyz(v: &Value, what: &str) -> Result<[f64; 3], String> {
    serde_json::from_value(v.clone()).map_err(|_| format!("{what} is [x, y, z], got {v}"))
}

pub type Room = Option<([f64; 2], [f64; 2])>;

pub fn parse(v: &Value) -> Result<(Vec<Obj>, Vec<Rule>, Room), String> {
    let mut objs = Vec::new();
    for o in v["objects"].as_array().ok_or("place: objects is a list of {id, size}")? {
        let id = o["id"].as_str().ok_or("place: each object has an id")?.to_string();
        if objs.iter().any(|x: &Obj| x.id == id) {
            return Err(format!("place: two objects are called {id}"));
        }
        let size = xyz(&o["size"], &format!("place: {id}'s size"))?;
        if size.iter().any(|s| *s <= 0.0) {
            return Err(format!("place: {id}'s size is above 0 on every axis"));
        }
        let at = if o["at"].is_null() { [0.0; 3] } else { xyz(&o["at"], &format!("place: {id}'s at"))? };
        objs.push(Obj { id, size, at, yaw: o["yaw"].as_f64().unwrap_or(0.0), fixed: o["fixed"] == true, turned: false });
    }
    let idx = |name: &Value, role: &str| -> Result<usize, String> {
        let n = name.as_str().ok_or_else(|| format!("place: a rule's {role} names an object"))?;
        objs.iter().position(|o| o.id == n).ok_or_else(|| format!("place: no object {n} (the objects: {})", objs.iter().map(|o| o.id.as_str()).collect::<Vec<_>>().join(", ")))
    };
    let mut rules = Vec::new();
    for r in v["rules"].as_array().into_iter().flatten() {
        let num = |k: &str, d: f64| r[k].as_f64().unwrap_or(d);
        let rule = if !r["on"].is_null() {
            Rule::On { a: idx(&r["on"], "on")?, b: idx(&r["of"], "of")? }
        } else if !r["against"].is_null() {
            Rule::Against { a: idx(&r["against"], "against")?, wall: dir(r["wall"].as_str().ok_or("place: against takes a wall: +x, -x, +z, -z")?)? }
        } else if !r["beside"].is_null() {
            Rule::Beside { a: idx(&r["beside"], "beside")?, b: idx(&r["of"], "of")?, side: dir(r["side"].as_str().unwrap_or("+x"))?, gap: num("gap", 0.0) }
        } else if !r["near"].is_null() {
            Rule::Near { a: idx(&r["near"], "near")?, b: idx(&r["of"], "of")?, dist: num("dist", 0.5) }
        } else if !r["apart"].is_null() {
            Rule::Apart { a: idx(&r["apart"], "apart")?, b: idx(&r["of"], "of")?, min: num("min", 1.0) }
        } else if !r["facing"].is_null() {
            Rule::Facing { a: idx(&r["facing"], "facing")?, b: idx(&r["of"], "of")? }
        } else if !r["align"].is_null() {
            let axis = match r["axis"].as_str().unwrap_or("x") { "x" => 0, "z" => 2, o => return Err(format!("place: align's axis is x or z, got {o}")) };
            Rule::Align { a: idx(&r["align"], "align")?, b: idx(&r["of"], "of")?, axis }
        } else if !r["clear"].is_null() {
            Rule::Clear { a: idx(&r["clear"], "clear")?, depth: num("depth", 0.8) }
        } else {
            return Err(format!("place: no rule in {r}: on, against, beside, near, apart, facing, align, clear"));
        };
        rules.push(rule);
    }
    let room = if v["room"].is_null() {
        None
    } else {
        let p = |k: &str| -> Result<[f64; 2], String> {
            match v["room"][k].as_array().map(|a| a.iter().map(Value::as_f64).collect::<Option<Vec<f64>>>()) {
                Some(Some(a)) if a.len() == 2 => Ok([a[0], a[1]]),
                _ => Err(format!("place: room.{k} is [x, z]")),
            }
        };
        Some((p("min")?, p("max")?))
    };
    Ok((objs, rules, room))
}
