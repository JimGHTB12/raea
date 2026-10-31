//! Manifold solids held as handles (`/3d`'s `manifold`) and 2D cross-sections by value.

use manifold_rust::manifold::Manifold;
use serde_json::{Value, json};

use super::{R, State, args, f, s, u};

pub(crate) struct Solid {
    m: Manifold,
    expr: String,
}

fn arg_text(a: &Value) -> String {
    match a {
        Value::String(s) => format!("{s:?}"),
        Value::Number(n) => n.to_string(),
        o => o.to_string(),
    }
}

impl State {
    fn put(&mut self, m: Manifold, expr: String) -> Value {
        self.next += 1;
        let id = self.next;
        self.solids.insert(id, Solid { m, expr: expr.clone() });
        json!({"id": id, "expr": expr})
    }

    fn solid(&self, id: u64) -> R<&Solid> {
        self.solids.get(&id).ok_or_else(|| format!("solid {id} is gone (freed?)"))
    }

}

/// A solid no expression describes (made from a mesh, a level set, a warp), written as OBJ under its content's hash; its
/// expression loads it (`obj("...")`), so the renderer rebuilds it like any other.
fn bake(m: &Manifold) -> R<String> {
    use std::hash::{Hash, Hasher};
    let (verts, tris) = raea_core_shape::to_mesh(m);
    let mut h = std::collections::hash_map::DefaultHasher::new();
    verts.iter().for_each(|x| x.to_bits().hash(&mut h));
    tris.hash(&mut h);
    let dir = std::env::temp_dir().join("raea-solids");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{:016x}.obj", h.finish()));
    if !path.exists() {
        raea_core_shape::export(m, &path.display().to_string(), "obj")?;
    }
    Ok(format!("obj({:?})", path.display().to_string()))
}

/// Polygons given as [[[x, y], ...], ...] in the polygon string raea-render's shape language reads.
fn polygons_text(v: &Value) -> R<String> {
    let polys = v.get("polygons").and_then(Value::as_array).ok_or("polygons: [[[x, y], ...], ...]")?;
    let mut out = Vec::new();
    for c in polys {
        let pts: Vec<String> = c.as_array().ok_or("a contour is [[x, y], ...]")?.iter().map(|p| {
            let x = p.get(0).and_then(Value::as_f64).unwrap_or(f64::NAN);
            let y = p.get(1).and_then(Value::as_f64).unwrap_or(f64::NAN);
            format!("{x} {y}")
        }).collect();
        out.push(pts.join(", "));
    }
    Ok(out.join("; "))
}

fn vec3_of(v: &Value, k: &str) -> R<[f64; 3]> {
    let a = v.get(k).and_then(Value::as_array).ok_or_else(|| format!("{k}: [x, y, z]"))?;
    Ok([a.first().and_then(Value::as_f64).unwrap_or(0.0), a.get(1).and_then(Value::as_f64).unwrap_or(0.0), a.get(2).and_then(Value::as_f64).unwrap_or(0.0)])
}

/// A 2D cross-section given as [[[x, y], ...], ...] (`key`), unioned by Manifold's positive fill rule.
fn section_of(v: &Value, key: &str) -> R<manifold_rust::cross_section::CrossSection> {
    use manifold_rust::linalg::Vec2;
    let polys = v.get(key).and_then(Value::as_array).ok_or_else(|| format!("{key}: [[[x, y], ...], ...]"))?;
    let mut out: Vec<Vec<Vec2>> = Vec::new();
    for c in polys {
        let pts: Vec<Vec2> = c.as_array().ok_or("a contour is [[x, y], ...]")?.iter().map(|p| Vec2::new(p.get(0).and_then(Value::as_f64).unwrap_or(f64::NAN), p.get(1).and_then(Value::as_f64).unwrap_or(f64::NAN))).collect();
        if pts.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
            return Err("a cross-section's points must be finite [x, y]".into());
        }
        out.push(pts);
    }
    Ok(manifold_rust::cross_section::CrossSection::new(out))
}

/// A cross-section as TypeScript's CrossSection holds it: its contours, area, contour count and 2D bounds.
fn section_json(cs: &manifold_rust::cross_section::CrossSection) -> Value {
    let polys: Vec<Vec<[f64; 2]>> = cs.to_polygons().iter().map(|p| p.iter().map(|q| [q.x, q.y]).collect()).collect();
    let (lo, hi) = if cs.is_empty() { ([0.0, 0.0], [0.0, 0.0]) } else { let b = cs.bounds(); ([b.min.x, b.min.y], [b.max.x, b.max.y]) };
    json!({"polygons": polys, "area": cs.area(), "contours": cs.num_contour(), "bounds": {"min": lo, "max": hi}})
}

pub(super) fn call(st: &mut State, name: &str, v: &Value) -> R<Value> {
    match name {
        "solid.expr" => {
            let expr = s(v, "expr")?.to_string();
            let m = raea_core_shape::solid(&expr)?;
            Ok(st.put(m, expr))
        }
        "solid.make" => {
            let fun = s(v, "fn")?;
            let expr = format!("{fun}({})", args(v).iter().map(arg_text).collect::<Vec<_>>().join(", "));
            let m = raea_core_shape::solid(&expr)?;
            Ok(st.put(m, expr))
        }
        "solid.combine" => {
            let fun = s(v, "fn")?;
            let ids: Vec<u64> = v.get("ids").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_u64).collect()).unwrap_or_default();
            if ids.len() < 2 && fun != "hull" {
                return Err(format!("{fun} takes two solids or more"));
            }
            let parts: Vec<&Solid> = ids.iter().map(|i| st.solid(*i)).collect::<R<_>>()?;
            let expr = format!("{fun}({})", parts.iter().map(|p| p.expr.as_str()).collect::<Vec<_>>().join(", "));
            let ms: Vec<Manifold> = parts.iter().map(|p| p.m.clone()).collect();
            let m = match fun {
                "union" => ms.into_iter().reduce(|a, b| a.union(&b)).unwrap(),
                "minus" => ms.into_iter().reduce(|a, b| a.difference(&b)).unwrap(),
                "intersect" => ms.into_iter().reduce(|a, b| a.intersection(&b)).unwrap(),
                "hull" => Manifold::hull_manifolds(&ms),
                o => return Err(format!("no boolean {o}: union, minus, intersect, hull")),
            };
            Ok(st.put(m, expr))
        }
        "solid.move" => {
            let base = st.solid(u(v, "id")?)?;
            let fun = s(v, "fn")?;
            let a: Vec<f64> = args(v).iter().filter_map(Value::as_f64).collect();
            let need = if fun == "scale" { 1 } else { 3 };
            if a.len() < need {
                return Err(format!("{fun} takes {need} numbers"));
            }
            let (m, expr) = match fun {
                "move" => (base.m.translate(manifold_rust::linalg::Vec3::new(a[0], a[1], a[2])), format!("move({}, {}, {}, {})", base.expr, a[0], a[1], a[2])),
                "rotate" => (base.m.rotate(a[0], a[1], a[2]), format!("rotate({}, {}, {}, {})", base.expr, a[0], a[1], a[2])),
                "scale" => {
                    let (x, y, z) = if a.len() >= 3 { (a[0], a[1], a[2]) } else { (a[0], a[0], a[0]) };
                    (base.m.scale(manifold_rust::linalg::Vec3::new(x, y, z)), format!("scale({}, {x}, {y}, {z})", base.expr))
                }
                o => return Err(format!("no transform {o}: move, rotate, scale")),
            };
            Ok(st.put(m, expr))
        }
        "solid.query" => {
            let sd = st.solid(u(v, "id")?)?;
            match s(v, "what")? {
                "volume" => Ok(json!(sd.m.volume())),
                "bounds" => {
                    let b = sd.m.bounding_box();
                    Ok(json!({"min": [b.min.x, b.min.y, b.min.z], "max": [b.max.x, b.max.y, b.max.z]}))
                }
                "triangles" => Ok(json!(sd.m.num_tri())),
                "empty" => Ok(json!(sd.m.is_empty())),
                "slice" => {
                    let cs = sd.m.slice(f(v, "z")?);
                    let polys: Vec<Vec<[f64; 2]>> = cs.to_polygons().iter().map(|p| p.iter().map(|q| [q.x, q.y]).collect()).collect();
                    Ok(json!({"area": cs.area(), "contours": cs.num_contour(), "polygons": polys}))
                }
                "area" => Ok(json!(sd.m.surface_area())),
                "genus" => Ok(json!(sd.m.genus())),
                "min_gap" => {
                    let o = st.solid(u(v, "other")?)?;
                    Ok(json!(sd.m.min_gap(&o.m, f(v, "z")?)))
                }
                "ray" => {
                    let a: Vec<f64> = args(v).iter().filter_map(Value::as_f64).collect();
                    if a.len() < 6 {
                        return Err("ray takes 6 numbers: from x y z, to x y z".into());
                    }
                    let hits = sd.m.ray_cast(manifold_rust::linalg::Vec3::new(a[0], a[1], a[2]), manifold_rust::linalg::Vec3::new(a[3], a[4], a[5]));
                    Ok(json!(hits.iter().map(|h| json!({"distance": h.distance, "position": [h.position.x, h.position.y, h.position.z], "normal": [h.normal.x, h.normal.y, h.normal.z]})).collect::<Vec<_>>()))
                }
                o => Err(format!("no query {o}: volume, bounds, triangles, empty, slice, area, genus, min_gap, ray")),
            }
        }
        // the rest of Manifold: each a new solid with the expression that rebuilds it (raea-render's shape language)
        "solid.op" => {
            let base = st.solid(u(v, "id")?)?;
            let fun = s(v, "fn")?;
            let a: Vec<f64> = args(v).iter().filter_map(Value::as_f64).collect();
            let need = |n: usize| if a.len() < n { Err(format!("{fun} takes {n} numbers")) } else { Ok(()) };
            let other = match v.get("other").and_then(Value::as_u64) {
                Some(o) => Some(st.solid(o)?),
                None => None,
            };
            let v3 = |i: usize| manifold_rust::linalg::Vec3::new(a[i], a[i + 1], a[i + 2]);
            let (m, expr) = match fun {
                "mirror" => {
                    need(3)?;
                    (base.m.mirror(v3(0)), format!("mirror({}, {}, {}, {})", base.expr, a[0], a[1], a[2]))
                }
                "transform" => {
                    need(12)?;
                    let mat = manifold_rust::linalg::Mat3x4::from_cols(v3(0), v3(3), v3(6), v3(9));
                    (base.m.transform(&mat), format!("transform({}, {})", base.expr, a.iter().take(12).map(|x| x.to_string()).collect::<Vec<_>>().join(", ")))
                }
                "trim" => {
                    need(4)?;
                    (base.m.trim_by_plane(v3(0), a[3]), format!("trim({}, {}, {}, {}, {})", base.expr, a[0], a[1], a[2], a[3]))
                }
                "refine" => {
                    need(1)?;
                    let n = a[0].max(1.0) as i32;
                    (base.m.refine(n), format!("refine({}, {n})", base.expr))
                }
                "smooth" => {
                    let (ang, sm, n) = (a.first().copied().unwrap_or(60.0), a.get(1).copied().unwrap_or(0.0), a.get(2).copied().unwrap_or(4.0).max(1.0) as i32);
                    (base.m.smooth_out(ang, sm).refine(n), format!("smooth({}, {ang}, {sm}, {n})", base.expr))
                }
                "minkowski" | "minkowski_diff" => {
                    let o = other.ok_or_else(|| format!("{fun} takes another solid"))?;
                    let m = if fun == "minkowski" { base.m.minkowski_sum(&o.m) } else { base.m.minkowski_difference(&o.m) };
                    (m, format!("{fun}({}, {})", base.expr, o.expr))
                }
                "simplify" | "refine_length" | "refine_tolerance" => {
                    need(1)?;
                    let m = match fun { "simplify" => base.m.simplify(a[0]), "refine_length" => base.m.refine_to_length(a[0]), _ => base.m.refine_to_tolerance(a[0]) };
                    (m, format!("{fun}({}, {})", base.expr, a[0]))
                }
                o => return Err(format!("no operation {o}: mirror, transform, trim, refine, refine_length, refine_tolerance, smooth, simplify, minkowski, minkowski_diff")),
            };
            Ok(st.put(m, expr))
        }
        "solid.extrude" | "solid.revolve" => {
            let polys = polygons_text(v)?;
            let a: Vec<f64> = args(v).iter().filter_map(Value::as_f64).collect();
            let expr = if name == "solid.extrude" {
                format!("extrude({polys:?}, {})", a.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))
            } else if a.is_empty() {
                format!("revolve({polys:?})")
            } else {
                format!("revolve({polys:?}, {})", a.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))
            };
            let m = raea_core_shape::solid(&expr)?;
            Ok(st.put(m, expr))
        }
        "solid.mesh" => {
            let sd = st.solid(u(v, "id")?)?;
            let (verts, tris) = raea_core_shape::to_mesh(&sd.m);
            Ok(json!({"verts": verts, "tris": tris}))
        }
        "solid.from_mesh" => {
            let verts: Vec<f32> = v.get("verts").and_then(Value::as_array).ok_or("verts: x y z ...")?.iter().filter_map(Value::as_f64).map(|x| x as f32).collect();
            let tris: Vec<u32> = v.get("tris").and_then(Value::as_array).ok_or("tris: i j k ...")?.iter().filter_map(Value::as_u64).map(|x| x as u32).collect();
            let m = raea_core_shape::from_mesh(&verts, &tris)?;
            let expr = bake(&m)?;
            Ok(st.put(m, expr))
        }
        "solid.level_set" => {
            // an SDF sampled on a grid (nx*ny*nz values over [min, max], x fastest), surfaced at `level`
            let dims: Vec<usize> = v.get("dims").and_then(Value::as_array).ok_or("dims: [nx, ny, nz]")?.iter().filter_map(Value::as_u64).map(|x| x as usize).collect();
            let grid: Vec<f64> = v.get("grid").and_then(Value::as_array).ok_or("grid: values")?.iter().filter_map(Value::as_f64).collect();
            let lo = vec3_of(v, "min")?;
            let hi = vec3_of(v, "max")?;
            if dims.len() != 3 || dims.iter().any(|d| *d < 2) || grid.len() != dims[0] * dims[1] * dims[2] {
                return Err("level_set: dims [nx, ny, nz] (each 2 or more) and nx*ny*nz grid values".into());
            }
            let level = v.get("level").and_then(Value::as_f64).unwrap_or(0.0);
            let edge = v.get("edge").and_then(Value::as_f64).unwrap_or(((hi[0] - lo[0]) / (dims[0] - 1) as f64).min((hi[1] - lo[1]) / (dims[1] - 1) as f64).min((hi[2] - lo[2]) / (dims[2] - 1) as f64));
            let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
            let sample = move |p: manifold_rust::linalg::Vec3| -> f64 {
                let at = |c: f64, l: f64, h: f64, n: usize| ((c - l) / (h - l) * (n - 1) as f64).clamp(0.0, (n - 1) as f64);
                let (fx, fy, fz) = (at(p.x, lo[0], hi[0], nx), at(p.y, lo[1], hi[1], ny), at(p.z, lo[2], hi[2], nz));
                let (x0, y0, z0) = (fx.floor() as usize, fy.floor() as usize, fz.floor() as usize);
                let (x1, y1, z1) = ((x0 + 1).min(nx - 1), (y0 + 1).min(ny - 1), (z0 + 1).min(nz - 1));
                let (tx, ty, tz) = (fx - x0 as f64, fy - y0 as f64, fz - z0 as f64);
                let g = |x: usize, y: usize, z: usize| grid[x + nx * (y + ny * z)];
                let l = |a: f64, b: f64, t: f64| a + (b - a) * t;
                // inside is negative in an SDF; Manifold's level set keeps where the function is above the level
                -l(l(l(g(x0, y0, z0), g(x1, y0, z0), tx), l(g(x0, y1, z0), g(x1, y1, z0), tx), ty), l(l(g(x0, y0, z1), g(x1, y0, z1), tx), l(g(x0, y1, z1), g(x1, y1, z1), tx), ty), tz)
            };
            let bounds = manifold_rust::types::Box { min: manifold_rust::linalg::Vec3::new(lo[0], lo[1], lo[2]), max: manifold_rust::linalg::Vec3::new(hi[0], hi[1], hi[2]) };
            let m = Manifold::level_set_with_level(sample, bounds, edge, -level);
            if m.is_empty() {
                return Err("level_set: no surface at that level inside the bounds".into());
            }
            let expr = bake(&m)?;
            Ok(st.put(m, expr))
        }
        "solid.export" => {
            let sd = st.solid(u(v, "id")?)?;
            let path = s(v, "path")?;
            let format = v.get("format").and_then(Value::as_str).map(str::to_string).unwrap_or_else(|| if path.ends_with(".glb") { "glb".into() } else { "obj".into() });
            Ok(json!({"bytes": raea_core_shape::export(&sd.m, path, &format)?}))
        }
        "solid.import" => {
            let path = s(v, "path")?;
            let f = if path.to_ascii_lowercase().ends_with(".glb") { "glb" } else { "obj" };
            let expr = format!("{f}({path:?})");
            let m = raea_core_shape::solid(&expr)?;
            Ok(st.put(m, expr))
        }
        // a solid's cross-section at z, or its silhouette on the x-y plane, as a 2D cross-section
        "solid.section" => {
            let sd = st.solid(u(v, "id")?)?;
            let cs = match v.get("z").and_then(Value::as_f64) { Some(z) => sd.m.slice(z), None => sd.m.project() };
            Ok(section_json(&cs))
        }
        // its disconnected pieces, each a solid (written under its hash, as a mesh's solid is)
        "solid.decompose" => {
            let parts = st.solid(u(v, "id")?)?.m.decompose();
            let mut out = Vec::new();
            for m in parts {
                let expr = bake(&m)?;
                out.push(st.put(m, expr));
            }
            Ok(Value::Array(out))
        }
        // solids that don't overlap, held as one (Manifold's compose: no boolean, so the union is exact)
        "solid.compose" => {
            let ids: Vec<u64> = v.get("ids").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_u64).collect()).unwrap_or_default();
            if ids.is_empty() {
                return Err("compose takes one solid or more".into());
            }
            let parts: Vec<&Solid> = ids.iter().map(|i| st.solid(*i)).collect::<R<_>>()?;
            let expr = if parts.len() == 1 { parts[0].expr.clone() } else { format!("union({})", parts.iter().map(|p| p.expr.as_str()).collect::<Vec<_>>().join(", ")) };
            let m = Manifold::compose(&parts.iter().map(|p| p.m.clone()).collect::<Vec<_>>());
            Ok(st.put(m, expr))
        }
        // the convex hull of points
        "solid.hull_points" => {
            let pts: Vec<manifold_rust::linalg::Vec3> = v.get("points").and_then(Value::as_array).ok_or("points: [[x, y, z], ...]")?.iter().map(|p| manifold_rust::linalg::Vec3::new(p.get(0).and_then(Value::as_f64).unwrap_or(0.0), p.get(1).and_then(Value::as_f64).unwrap_or(0.0), p.get(2).and_then(Value::as_f64).unwrap_or(0.0))).collect();
            if pts.len() < 4 {
                return Err("hull of points takes 4 points or more".into());
            }
            let m = Manifold::hull(&pts);
            let expr = bake(&m)?;
            Ok(st.put(m, expr))
        }
        // 2D cross-sections (Manifold's CrossSection), held by value: each answer is the new section
        "section.op" => {
            use manifold_rust::cross_section::CrossSection;
            use manifold_rust::linalg::Vec2;
            use manifold_rust::types::OpType;
            let fun = s(v, "fn")?;
            let a: Vec<f64> = args(v).iter().filter_map(Value::as_f64).collect();
            let need = |n: usize| if a.len() < n { Err(format!("section {fun} takes {n} numbers")) } else { Ok(()) };
            let others = || -> R<Vec<CrossSection>> {
                v.get("others").and_then(Value::as_array).map(|list| list.iter().map(|o| section_of(&json!({"p": o}), "p")).collect::<R<Vec<_>>>()).unwrap_or_else(|| Ok(Vec::new()))
            };
            let cs = match fun {
                "circle" => {
                    need(1)?;
                    CrossSection::circle(a[0], a.get(1).copied().unwrap_or(0.0) as i32)
                }
                "square" => {
                    need(2)?;
                    CrossSection::square_vec2(Vec2::new(a[0], a[1]), a.get(2).copied().unwrap_or(1.0) != 0.0)
                }
                "hull_points" => {
                    // every point of every contour, as given (no union first)
                    let raw: Vec<Vec2> = v.get("polygons").and_then(Value::as_array).into_iter().flatten().flat_map(|c| c.as_array().cloned().unwrap_or_default()).map(|p| Vec2::new(p.get(0).and_then(Value::as_f64).unwrap_or(0.0), p.get(1).and_then(Value::as_f64).unwrap_or(0.0))).collect();
                    if raw.len() < 3 {
                        return Err("section hull_points takes 3 points or more".into());
                    }
                    CrossSection::hull_points(&raw)
                }
                _ => {
                    let me = section_of(v, "polygons")?;
                    match fun {
                        "make" => me,
                        "union" | "minus" | "intersect" => {
                            let mut all = vec![me];
                            all.extend(others()?);
                            if all.len() < 2 {
                                return Err(format!("section {fun} takes another section"));
                            }
                            CrossSection::batch_boolean(&all, match fun { "union" => OpType::Add, "minus" => OpType::Subtract, _ => OpType::Intersect })
                        }
                        "hull" => {
                            let mut all = vec![me];
                            all.extend(others()?);
                            CrossSection::hull_cross_sections(&all)
                        }
                        "minkowski" => {
                            // Clipper's sum sweeps the outline, leaving the inside out: the inside comes back as this
                            // shape moved by one point of the other (the sum of a shape and a connected one)
                            let o = others()?.into_iter().next().ok_or("section minkowski takes another section")?;
                            let p0 = o.to_polygons().first().and_then(|c| c.first().copied()).ok_or("section minkowski: the other section is empty")?;
                            me.minkowski_sum(&o).union(&me.translate(p0))
                        }
                        "offset" => {
                            // delta, join (0 square, 1 round, 2 miter, 3 bevel), miter limit, round segments
                            need(1)?;
                            me.offset_with_params(a[0], a.get(1).copied().unwrap_or(1.0) as i32, a.get(2).copied().unwrap_or(2.0), a.get(3).copied().unwrap_or(0.0) as i32)
                        }
                        "simplify" => {
                            need(1)?;
                            me.simplify(a[0])
                        }
                        "translate" => {
                            need(2)?;
                            me.translate(Vec2::new(a[0], a[1]))
                        }
                        "rotate" => {
                            need(1)?;
                            me.rotate(a[0])
                        }
                        "scale" => {
                            need(1)?;
                            me.scale(Vec2::new(a[0], a.get(1).copied().unwrap_or(a[0])))
                        }
                        "mirror" => {
                            need(2)?;
                            me.mirror(Vec2::new(a[0], a[1]))
                        }
                        "decompose" => return Ok(Value::Array(me.decompose().iter().map(section_json).collect())),
                        o => return Err(format!("no section operation {o}: make, circle, square, hull_points, union, minus, intersect, hull, minkowski, offset, simplify, translate, rotate, scale, mirror, decompose")),
                    }
                }
            };
            Ok(section_json(&cs))
        }
        "solid.free" => {
            st.solids.remove(&u(v, "id")?);
            Ok(Value::Null)
        }
        o => Err(format!("no native call {o}")),
    }
}
