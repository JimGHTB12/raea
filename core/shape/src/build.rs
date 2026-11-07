//! Each shape call built as a Manifold solid, and the space-time shapes a clip's mask uses.

use manifold_rust::cross_section::CrossSection;
use manifold_rust::linalg::{Vec2, Vec3};
use manifold_rust::manifold::Manifold;

use crate::SEGMENTS;
use crate::glyphs::text3d;
use crate::mesh::{from_mesh, read_glb, read_obj};
use crate::parse::{Arg, nums, shapes};

thread_local! {
    /// The edit's (width, height, frames) while a space-time mask is built.
    pub(crate) static PICTURE: std::cell::Cell<Option<(f64, f64, f64)>> = const { std::cell::Cell::new(None) };
}

/// A prism over all of the edit's time (a frame before the first to one after the last).
fn always(section: CrossSection) -> Result<Manifold, String> {
    let (_, _, n) = PICTURE.with(|p| p.get()).ok_or("frame(), rect, disc, wipe, during and slide are for a clip's mask")?;
    Ok(Manifold::extrude(&section.to_polygons(), n + 2.0, 1, 0.0, Vec2::new(1.0, 1.0)).translate(Vec3::new(0.0, 0.0, -1.0)))
}

fn space_time(f: &str, a: &[Arg]) -> Option<Result<Manifold, String>> {
    if !matches!(f, "frame" | "rect" | "disc" | "wipe") {
        return None;
    }
    let pic = PICTURE.with(|p| p.get());
    let rect = |x: f64, y: f64, w: f64, h: f64| CrossSection::from_rect(&manifold_rust::types::Rect { min: Vec2::new(x, y), max: Vec2::new(x + w, y + h) });
    Some((|| -> Result<Manifold, String> {
        let (w, h, _) = pic.ok_or_else(|| format!("{f}() is for a clip's mask (space and time)"))?;
        Ok(match f {
            "frame" => always(rect(0.0, 0.0, w, h))?,
            "rect" => {
                let n = nums(f, a, 4)?;
                always(rect(n[0], n[1], n[2], n[3]))?
            }
            "disc" => {
                let n = nums(f, a, 3)?;
                always(CrossSection::circle(n[2], 96).translate(Vec2::new(n[0], n[1])))?
            }
            "wipe" => {
                let dir = a.iter().find_map(|x| if let Arg::Str(s) = x { Some(s.clone()) } else { None }).unwrap_or_else(|| "right".into());
                let n = nums(f, a, 2)?;
                let (f0, f1) = (n[0], n[1].max(n[0] + 1e-6));
                // revealed where the edge has passed: a plane tilted in (x or y, t); trim keeps n . p >= offset
                let (k, along) = match dir.as_str() { "right" | "left" => (w / (f1 - f0), w), _ => (h / (f1 - f0), h) };
                let (nrm, off) = match dir.as_str() {
                    "right" => (Vec3::new(-1.0, 0.0, k), k * f0),
                    "left" => (Vec3::new(1.0, 0.0, -k), along - k * f0),
                    "down" => (Vec3::new(0.0, -1.0, k), k * f0),
                    "up" => (Vec3::new(0.0, 1.0, -k), along - k * f0),
                    o => return Err(format!("wipe({o:?}): right, left, down or up")),
                };
                let l = (nrm.x * nrm.x + nrm.y * nrm.y + nrm.z * nrm.z).sqrt();
                always(rect(0.0, 0.0, w, h))?.trim_by_plane(Vec3::new(nrm.x / l, nrm.y / l, nrm.z / l), off / l)
            }
            _ => unreachable!(),
        })
    })())
}

pub(crate) fn call(f: &str, a: Vec<Arg>) -> Result<Manifold, String> {
    if let Some(r) = space_time(f, &a) {
        return r;
    }
    if let Some(r) = crate::grown::call(f, &a) {
        return r;
    }
    let fold = |v: Vec<Manifold>, op: fn(&Manifold, &Manifold) -> Manifold| v.into_iter().reduce(|x, y| op(&x, &y)).unwrap();
    Ok(match f {
        "box" | "cube" => {
            let n = nums(f, &a, 1)?;
            let (w, h, d) = if n.len() >= 3 { (n[0], n[1], n[2]) } else { (n[0], n[0], n[0]) };
            Manifold::cube(Vec3::new(w, h, d), true)
        }
        // Bevy's Plane3d (plane(nx, ny, nz, w, h)): drawn with one face by Bevy's builder (sphere.rs plane_of); for CSG
        // and contacts the box of no depth it lies in, w x h laid out as raea/mesh's <Plane3d> does
        "plane" => {
            let n = nums(f, &a, 5)?;
            let (w, h) = (n[3], n[4]);
            let size = if n[0] != 0.0 { Vec3::new(0.0, w, h) } else if n[2] != 0.0 { Vec3::new(w, h, 0.0) } else { Vec3::new(w, 0.0, h) };
            Manifold::cube(size, true)
        }
        "rounded_box" => {
            let n = nums(f, &a, 4)?;
            let r = n[3].clamp(0.0, n[0].min(n[1]).min(n[2]) / 2.0 - 1e-6);
            // a box shrunk by r, swept by a sphere of r: the hull of eight spheres at its corners
            let (hx, hy, hz) = (n[0] / 2.0 - r, n[1] / 2.0 - r, n[2] / 2.0 - r);
            let ball = Manifold::sphere(r, SEGMENTS / 2);
            let corners: Vec<Manifold> = [-1.0, 1.0]
                .iter()
                .flat_map(|x| [-1.0, 1.0].iter().flat_map(move |y| [-1.0, 1.0].iter().map(move |z| (*x, *y, *z))))
                .map(|(x, y, z)| ball.translate(Vec3::new(x * hx, y * hy, z * hz)))
                .collect();
            Manifold::hull_manifolds(&corners)
        }
        // sphere(r), or sphere(r, n) for Bevy's icosphere of n subdivisions (drawn by Bevy's builder: engine spawn.rs),
        // its segments about an icosphere's (4.5 x 2^n) up to 256 for CSG and contacts; uv_sphere(r, sectors, stacks)
        "sphere" => {
            let n = nums(f, &a, 1)?;
            let seg = n.get(1).map_or(SEGMENTS, |s| (4.5 * 2f64.powf(s.clamp(0.0, 6.0))).round().clamp(8.0, 256.0) as i32);
            Manifold::sphere(n[0], seg)
        }
        "uv_sphere" => {
            let n = nums(f, &a, 3)?;
            Manifold::sphere(n[0], (n[1].round() as i32).clamp(3, 512))
        }
        "cylinder" => {
            let n = nums(f, &a, 2)?;
            Manifold::cylinder_centered(n[1], n[0], n[0], SEGMENTS, true)
        }
        "cone" => {
            let n = nums(f, &a, 3)?;
            Manifold::cylinder_centered(n[2], n[0], n[1], SEGMENTS, true)
        }
        "torus" => {
            let n = nums(f, &a, 2)?;
            let ring = CrossSection::circle(n[1], SEGMENTS / 2).translate(Vec2::new(n[0], 0.0));
            Manifold::revolve(&ring.to_polygons(), SEGMENTS, 360.0)
        }
        "text3d" => {
            let text = a.iter().find_map(|x| if let Arg::Str(s) = x { Some(s.clone()) } else { None }).ok_or("text3d(\"WORD\", size, depth)")?;
            let n = nums(f, &a, 2)?;
            let family = a.iter().filter_map(|x| if let Arg::Str(s) = x { Some(s.clone()) } else { None }).nth(1).unwrap_or_else(|| "Inter".into());
            let weight = n.get(2).copied().unwrap_or(800.0) as u16;
            text3d(&text, n[0], n[1], &family, weight)?
        }
        "union" => fold(shapes(f, a)?, Manifold::union),
        "minus" | "difference" => fold(shapes(f, a)?, Manifold::difference),
        "intersect" | "intersection" => fold(shapes(f, a)?, Manifold::intersection),
        "hull" => Manifold::hull_manifolds(&shapes(f, a)?),
        "during" => {
            let n = nums(f, &a, 2)?;
            let m = shapes(f, a)?.remove(0);
            let b = m.bounding_box();
            let slab = Manifold::cube(Vec3::new(b.max.x - b.min.x + 2.0, b.max.y - b.min.y + 2.0, n[1] - n[0]), false).translate(Vec3::new(b.min.x - 1.0, b.min.y - 1.0, n[0]));
            m.intersection(&slab)
        }
        "slide" => {
            // the shape from f0 to f1, moved by (dx, dy) in proportion: a shear of its prism through time
            let n = nums(f, &a, 4)?;
            let (dx, dy, f0, f1) = (n[0], n[1], n[2], n[3].max(n[2] + 1e-6));
            let m = shapes(f, a)?.remove(0);
            let b = m.bounding_box();
            let slab = Manifold::cube(Vec3::new(b.max.x - b.min.x + 2.0, b.max.y - b.min.y + 2.0, f1 - f0), false).translate(Vec3::new(b.min.x - 1.0, b.min.y - 1.0, f0));
            m.intersection(&slab).warp(move |v| {
                let u = (v.z - f0) / (f1 - f0);
                v.x += u * dx;
                v.y += u * dy;
            })
        }
        "move" | "translate" => {
            let n = nums(f, &a, 3)?;
            shapes(f, a)?.remove(0).translate(Vec3::new(n[0], n[1], n[2]))
        }
        "rotate" => {
            let n = nums(f, &a, 3)?;
            shapes(f, a)?.remove(0).rotate(n[0], n[1], n[2])
        }
        "scale" => {
            let n = nums(f, &a, 1)?;
            let v = if n.len() >= 3 { Vec3::new(n[0], n[1], n[2]) } else { Vec3::new(n[0], n[0], n[0]) };
            shapes(f, a)?.remove(0).scale(v)
        }
        // cross-sections made solid: a polygon string is "x y, x y, ...; x y, ..." (contours by ';', points by ',')
        "extrude" => {
            let poly = polygons_arg(f, &a)?;
            let n = nums(f, &a, 1)?;
            let g = |i: usize, d: f64| n.get(i).copied().unwrap_or(d);
            Manifold::extrude(&poly, n[0], g(1, 1.0) as i32, g(2, 0.0), Vec2::new(g(3, 1.0), g(4, 1.0)))
        }
        "revolve" => {
            let poly = polygons_arg(f, &a)?;
            let n: Vec<f64> = a.iter().filter_map(|x| if let Arg::Num(v) = x { Some(*v) } else { None }).collect();
            Manifold::revolve(&poly, n.get(1).map(|s| *s as i32).unwrap_or(SEGMENTS), n.first().copied().unwrap_or(360.0))
        }
        "mirror" => {
            let n = nums(f, &a, 3)?;
            shapes(f, a)?.remove(0).mirror(Vec3::new(n[0], n[1], n[2]))
        }
        "transform" => {
            // a 3x4 matrix, column by column (x, y, z axes, then the translation)
            let n = nums(f, &a, 12)?;
            let c = |i: usize| Vec3::new(n[i], n[i + 1], n[i + 2]);
            shapes(f, a)?.remove(0).transform(&manifold_rust::linalg::Mat3x4::from_cols(c(0), c(3), c(6), c(9)))
        }
        "trim" => {
            // what's on the normal's side of the plane n . p = offset
            let n = nums(f, &a, 4)?;
            shapes(f, a)?.remove(0).trim_by_plane(Vec3::new(n[0], n[1], n[2]), n[3])
        }
        "minkowski" => {
            let mut v = shapes(f, a)?;
            if v.len() < 2 {
                return Err("minkowski(a, b): two shapes".into());
            }
            let b = v.remove(1);
            v.remove(0).minkowski_sum(&b)
        }
        "minkowski_diff" => {
            let mut v = shapes(f, a)?;
            if v.len() < 2 {
                return Err("minkowski_diff(a, b): two shapes".into());
            }
            let b = v.remove(1);
            v.remove(0).minkowski_difference(&b)
        }
        "refine" => {
            let n = nums(f, &a, 1)?;
            shapes(f, a)?.remove(0).refine(n[0].max(1.0) as i32)
        }
        "smooth" => {
            // sharp edges kept above min_sharp_angle (degrees), the rest rounded, then refined n times
            let n: Vec<f64> = a.iter().filter_map(|x| if let Arg::Num(v) = x { Some(*v) } else { None }).collect();
            let m = shapes(f, a)?.remove(0);
            m.smooth_out(n.first().copied().unwrap_or(60.0), n.get(1).copied().unwrap_or(0.0)).refine(n.get(2).copied().unwrap_or(4.0).max(1.0) as i32)
        }
        // the mesh coarsened (vertices closer than tol merged) or refined until edges are under a length, or the
        // smoothed surface is within a tolerance
        "simplify" => {
            let n = nums(f, &a, 1)?;
            shapes(f, a)?.remove(0).simplify(n[0])
        }
        "refine_length" => {
            let n = nums(f, &a, 1)?;
            shapes(f, a)?.remove(0).refine_to_length(n[0])
        }
        "refine_tolerance" => {
            let n = nums(f, &a, 1)?;
            shapes(f, a)?.remove(0).refine_to_tolerance(n[0])
        }
        "obj" | "glb" => {
            let path = a.iter().find_map(|x| if let Arg::Str(s) = x { Some(s.clone()) } else { None }).ok_or_else(|| format!("{f}(\"path\")"))?;
            let (verts, tris) = if f == "obj" { read_obj(&path)? } else { read_glb(&path)? };
            from_mesh(&verts, &tris)?
        }
        o => return Err(format!("no shape {o}: box, rounded_box, sphere, uv_sphere, cylinder, cone, torus, text3d, extrude, revolve, union, minus, intersect, hull, minkowski, minkowski_diff, move, rotate, scale, mirror, transform, trim, refine, smooth, simplify, refine_length, refine_tolerance, obj, glb, sdf, tree, loft, terrain, boulder, rosette; in a mask also frame, rect, disc, wipe, during, slide")),
    })
}

/// The polygon string argument: "x y, x y, ...; x y, ..." (contours by ';', points by ',', each counter-clockwise for
/// solid and clockwise for a hole, as Manifold winds them).
fn polygons_arg(f: &str, a: &[Arg]) -> Result<manifold_rust::types::Polygons, String> {
    let text = a.iter().find_map(|x| if let Arg::Str(s) = x { Some(s.clone()) } else { None }).ok_or_else(|| format!("{f}(\"x y, x y, ...\", ...): give the outline"))?;
    let mut out = Vec::new();
    for contour in text.split(';').filter(|c| !c.trim().is_empty()) {
        let mut pts = Vec::new();
        for pt in contour.split(',') {
            let xy: Vec<f64> = pt.split_whitespace().map(|v| v.parse::<f64>().map_err(|_| format!("{f}: bad number {v:?}"))).collect::<Result<_, _>>()?;
            if xy.len() != 2 {
                return Err(format!("{f}: a point is \"x y\", got {pt:?}"));
            }
            pts.push(Vec2::new(xy[0], xy[1]));
        }
        if pts.len() < 3 {
            return Err(format!("{f}: a contour needs 3 points"));
        }
        out.push(pts);
    }
    Ok(out)
}
