//! Placement by relations (the 11-06 research: the scene builders that hold up measure placement, never eyeball it;
//! Infinigen's and Holodeck's layouts are relations solved, not coordinates typed). Objects are boxes with a real size;
//! rules say how they relate (on, against a wall, beside, near, apart, facing, aligned, a clear zone in front); the
//! solver moves the free ones until the rules hold and nothing on the same support overlaps, then says what still
//! doesn't hold, in scene lint's shape. Deterministic: the same input, the same layout.
//!
//! Frame: y up, metres. An object's `at` is the centre of its bottom face; its front is local +z, turned by `yaw`
//! degrees about y. Walls are the room's sides: "-x", "+x", "-z", "+z".

mod rules;
mod solve;

pub use rules::{Obj, Rule, parse, xyz};
pub use solve::solve;

use serde_json::Value;

/// `place.solve`'s whole call: JSON in (objects, rules, room), JSON out (objects placed, found, errors).
pub fn solve_json(v: &Value) -> Result<Value, String> {
    let (objs, rules, room) = parse(v)?;
    Ok(solve(objs, &rules, room))
}

#[cfg(test)]
mod tests;
