//! Rigid bodies stepped by Rapier.

use std::path::PathBuf;

use rapier3d::glamx::glam;
use rapier3d::prelude::*;
use serde_json::{Value, json};

use super::{Place, R};
use super::glb::*;

fn f(v: &Value, k: &str, d: f64) -> f32 {
    v.get(k).and_then(Value::as_f64).unwrap_or(d) as f32
}

fn v3(v: &Value, k: &str, d: [f32; 3]) -> glam::Vec3 {
    v.get(k).and_then(Value::as_array).filter(|a| a.len() == 3).map(|a| glam::Vec3::new(a[0].as_f64().unwrap_or(0.0) as f32, a[1].as_f64().unwrap_or(0.0) as f32, a[2].as_f64().unwrap_or(0.0) as f32)).unwrap_or(glam::Vec3::from_array(d))
}

/// A body's shape and the point it turns about (its rest centre), from its request: a hull (or trimesh, for a fixed
/// body) of its glTF nodes' triangles, or a box, sphere or capsule given in metres.
fn shape_of(at: &Place, b: &Value, glbs: &mut std::collections::HashMap<PathBuf, Glb>) -> R<(ColliderBuilder, glam::Vec3)> {
    let name = b.get("name").and_then(Value::as_str).unwrap_or("?");
    let centre = v3(b, "centre", [0.0; 3]);
    let shape = b.get("shape").cloned().unwrap_or(json!("hull"));
    if let Some(s) = shape.as_object() {
        let c = if let Some(x) = s.get("box").and_then(Value::as_array) {
            ColliderBuilder::cuboid(x[0].as_f64().unwrap_or(0.1) as f32 / 2.0, x[1].as_f64().unwrap_or(0.1) as f32 / 2.0, x[2].as_f64().unwrap_or(0.1) as f32 / 2.0)
        } else if let Some(r) = s.get("sphere").and_then(Value::as_f64) {
            ColliderBuilder::ball(r as f32)
        } else if let Some(x) = s.get("capsule").and_then(Value::as_array) {
            // [radius, length]: along y
            ColliderBuilder::capsule_y(x[1].as_f64().unwrap_or(0.1) as f32 / 2.0, x[0].as_f64().unwrap_or(0.05) as f32)
        } else {
            return Err(format!("body {name}: shape is \"hull\", \"mesh\", {{box: [x, y, z]}}, {{sphere: r}} or {{capsule: [r, length]}}"));
        };
        return Ok((c, centre));
    }
    let src = b.get("src").and_then(Value::as_str).ok_or(format!("body {name}: a hull needs src and nodes (or give a shape)"))?;
    let path = (at.media)(src);
    if !glbs.contains_key(&path) {
        glbs.insert(path.clone(), Glb::read(&path)?);
    }
    let g = &glbs[&path];
    let nodes: Vec<String> = b.get("nodes").and_then(Value::as_array).into_iter().flatten().filter_map(|x| x.as_str().map(String::from)).collect();
    let ix = g.nodes_named(&nodes).map_err(|e| format!("body {name}: {e}"))?;
    let (pos, tri) = g.triangles(&ix)?;
    if pos.len() < 4 {
        return Err(format!("body {name}: its nodes have {} vertices; a hull needs 4", pos.len()));
    }
    let (mut lo, mut hi) = (glam::Vec3::splat(f32::MAX), glam::Vec3::splat(f32::MIN));
    for p in &pos {
        lo = lo.min(*p);
        hi = hi.max(*p);
    }
    let c = if b.get("centre").is_some() { centre } else { (lo + hi) / 2.0 };
    let local: Vec<glam::Vec3> = pos.iter().map(|p| *p - c).collect();
    let col = match shape.as_str() {
        Some("mesh") => ColliderBuilder::trimesh(local, tri).map_err(|e| format!("body {name}: {e:?}"))?,
        Some("box") => {
            let h = (hi - lo) / 2.0;
            ColliderBuilder::cuboid(h.x.max(1e-3), h.y.max(1e-3), h.z.max(1e-3))
        }
        _ => ColliderBuilder::convex_hull(&local).ok_or(format!("body {name}: no hull (its vertices are flat?): give shape \"box\""))?,
    };
    Ok((col, c))
}

/// The bake: every body stepped at `substeps` x fps from its rest pose, its pose kept each frame as the rigid motion
/// from rest (`p` the translation of the rest-space points, `q` the rotation), so a part drawn where it stood at rest
/// is placed by a group with that position and quaternion.
pub fn simulate(at: &Place, req: &Value) -> R<Value> {
    let fps = req.get("fps").and_then(Value::as_f64).unwrap_or(30.0);
    let frames = req.get("frames").and_then(Value::as_u64).ok_or("physics.bake: frames")? as usize;
    let sub = req.get("substeps").and_then(Value::as_u64).unwrap_or(8).clamp(1, 64) as usize;
    let mut w = PhysicsWorld::new();
    w.gravity = v3(req, "gravity", [0.0, -9.81, 0.0]);
    w.integration_parameters.dt = (1.0 / (fps * sub as f64)) as f32;
    let bodies = req.get("bodies").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut glbs = std::collections::HashMap::new();
    struct B {
        name: String,
        h: RigidBodyHandle,
        c: glam::Vec3,
        release: Option<usize>,
        kin: Option<(Vec<f32>, Vec<f32>)>,
        impulses: Vec<Value>,
    }
    let mut list: Vec<B> = Vec::new();
    for b in &bodies {
        let name = b.get("name").and_then(Value::as_str).ok_or("physics.bake: every body needs a name")?.to_string();
        let (col, c) = shape_of(at, b, &mut glbs)?;
        let fixed = b.get("fixed").and_then(Value::as_bool) == Some(true);
        let kin = b.get("path").filter(|p| p.is_object()).map(|p| {
            let arr = |k: &str| p.get(k).and_then(Value::as_array).map(|a| a.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32).collect::<Vec<_>>()).unwrap_or_default();
            (arr("p"), arr("q"))
        });
        let release = b.get("release").and_then(Value::as_f64).map(|r| r.max(0.0).round() as usize).filter(|r| *r > 0);
        let rb = if fixed {
            RigidBodyBuilder::fixed()
        } else if kin.is_some() || release.is_some() {
            RigidBodyBuilder::kinematic_position_based()
        } else {
            RigidBodyBuilder::dynamic()
        };
        let rb = rb
            .translation(c)
            .linvel(v3(b, "velocity", [0.0; 3]))
            .angvel(v3(b, "angularVelocity", [0.0; 3]))
            .linear_damping(f(b, "linearDamping", 0.0))
            .angular_damping(f(b, "angularDamping", 0.05))
            .ccd_enabled(b.get("ccd").and_then(Value::as_bool).unwrap_or(true))
            .can_sleep(false)
            .build();
        let h = w.bodies.insert(rb);
        let mut col = col.friction(f(b, "friction", 0.5)).restitution(f(b, "restitution", 0.2));
        col = match b.get("mass").and_then(Value::as_f64) {
            Some(m) => col.mass(m as f32),
            None => col.density(f(b, "density", 1000.0)),
        };
        w.colliders.insert_with_parent(col.build(), h, &mut w.bodies);
        let impulses = b.get("impulses").and_then(Value::as_array).cloned().unwrap_or_default();
        list.push(B { name, h, c, release, kin, impulses });
    }
    // static colliders: the floor and tables (boxes, world space)
    for c in req.get("colliders").and_then(Value::as_array).into_iter().flatten() {
        let lo = v3(c, "min", [-1.0, -1.0, -1.0]);
        let hi = v3(c, "max", [1.0, 0.0, 1.0]);
        let h = (hi - lo) / 2.0;
        w.colliders.insert(ColliderBuilder::cuboid(h.x, h.y, h.z).translation((lo + hi) / 2.0).friction(f(c, "friction", 0.6)).restitution(f(c, "restitution", 0.1)).build());
    }
    let mut out: Vec<(Vec<f32>, Vec<f32>)> = list.iter().map(|_| (Vec::with_capacity(frames * 3), Vec::with_capacity(frames * 4))).collect();
    let pose_of = |k: &(Vec<f32>, Vec<f32>), fr: usize, c: glam::Vec3| -> (glam::Vec3, glam::Quat) {
        let n = k.0.len() / 3;
        let i = fr.min(n.saturating_sub(1));
        let p = if n > 0 { glam::Vec3::new(k.0[i * 3], k.0[i * 3 + 1], k.0[i * 3 + 2]) } else { glam::Vec3::ZERO };
        let q = if k.1.len() >= (i + 1) * 4 { glam::Quat::from_xyzw(k.1[i * 4], k.1[i * 4 + 1], k.1[i * 4 + 2], k.1[i * 4 + 3]).normalize() } else { glam::Quat::IDENTITY };
        // the path is the part's rigid motion from rest: its centre goes to p + q c
        (p + q * c, q)
    };
    for fr in 0..frames {
        // this frame's events: releases, impulses, kinematic targets
        for b in &list {
            let body = &mut w.bodies[b.h];
            if b.release == Some(fr) && b.kin.is_none() {
                body.set_body_type(RigidBodyType::Dynamic, true);
            }
            for e in &b.impulses {
                if e.get("frame").and_then(Value::as_f64).map(|x| x.round() as usize) != Some(fr) {
                    continue;
                }
                let imp = v3(e, "impulse", [0.0; 3]);
                match e.get("at").filter(|a| a.is_array()) {
                    Some(_) => body.apply_impulse_at_point(imp, v3(e, "at", [0.0; 3]), true),
                    None => body.apply_impulse(imp, true),
                }
                let t = v3(e, "torque", [0.0; 3]);
                if t != glam::Vec3::ZERO {
                    body.apply_torque_impulse(t, true);
                }
            }
        }
        for s in 0..sub {
            for b in &list {
                if let Some(k) = &b.kin {
                    // between this frame's target and the next's
                    let (p0, q0) = pose_of(k, fr, b.c);
                    let (p1, q1) = pose_of(k, fr + 1, b.c);
                    let u = (s + 1) as f32 / sub as f32;
                    let body = &mut w.bodies[b.h];
                    body.set_next_kinematic_position(Pose::from_parts(p0.lerp(p1, u), q0.slerp(q1, u)));
                }
            }
            w.step();
        }
        for (b, o) in list.iter().zip(out.iter_mut()) {
            let body = &w.bodies[b.h];
            let q = *body.rotation();
            let t = body.translation();
            // rest point x goes to t + q (x - c)
            let p = t - q * b.c;
            o.0.extend([p.x, p.y, p.z]);
            o.1.extend([q.x, q.y, q.z, q.w]);
        }
    }
    let r6 = |x: f32| (x as f64 * 1e6).round() / 1e6;
    let mut res = serde_json::Map::new();
    for (b, o) in list.iter().zip(&out) {
        res.insert(b.name.clone(), json!({"p": o.0.iter().map(|x| r6(*x)).collect::<Vec<_>>(), "q": o.1.iter().map(|x| r6(*x)).collect::<Vec<_>>(), "centre": b.c.to_array()}));
    }
    Ok(json!({"fps": fps, "frames": frames, "bodies": res}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sim(req: &Value) -> R<Value> {
        fn here(src: &str) -> PathBuf {
            PathBuf::from(src)
        }
        simulate(&Place { store: PathBuf::new(), media: &here, goo_setup: PathBuf::new() }, req)
    }

    fn drop_request() -> Value {
        json!({"fps": 30, "frames": 45, "colliders": [{"min": [-2, -1, -2], "max": [2, 0, 2]}],
            "bodies": [
                {"name": "ball", "shape": {"sphere": 0.1}, "centre": [0, 1, 0], "restitution": 0.6},
                {"name": "crate", "shape": {"box": [0.2, 0.2, 0.2]}, "centre": [0.5, 0.5, 0], "angularVelocity": [0, 0, 3]},
                {"name": "held", "shape": {"box": [0.2, 0.2, 0.2]}, "centre": [-0.5, 0.5, 0], "release": 20, "impulses": [{"frame": 20, "impulse": [2.0, 0, 0]}]}
            ]})
    }

    #[test]
    fn a_bake_is_the_same_every_time() {
        let a = sim(&drop_request()).unwrap();
        let b = sim(&drop_request()).unwrap();
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn bodies_fall_bounce_and_wait_to_be_released() {
        let v = sim(&drop_request()).unwrap();
        let p = |n: &str, fr: usize, a: usize| v["bodies"][n]["p"][fr * 3 + a].as_f64().unwrap();
        // the ball falls from 1 m and lands on the floor (its centre 0.1 above it: moved down by ~0.9)
        assert!(p("ball", 0, 1) > -0.01);
        assert!((p("ball", 44, 1) + 0.9).abs() < 0.12, "{}", p("ball", 44, 1));
        // held still until frame 20, then falls and is pushed along x
        assert!(p("held", 19, 1).abs() < 1e-6 && p("held", 19, 0).abs() < 1e-6);
        assert!(p("held", 44, 1) < -0.1 && p("held", 44, 0) > 0.01, "held at 20..44: {:?}",
            (20..45).step_by(4).map(|fr| (p("held", fr, 0), p("held", fr, 1))).collect::<Vec<_>>());
    }
}
