//! A predicate checked against the UI tree and its snapshots.

use super::*;

pub(crate) fn check(p: &Value, tree: &Value, snaps: &Snaps) -> Result<(), String> {
    let kind = p["check"].as_str().unwrap_or("");
    let bad = |mut v: Vec<String>, what: &str| -> Result<(), String> {
        if v.is_empty() {
            return Ok(());
        }
        let n = v.len();
        v.truncate(6);
        Err(format!("{what} ({n}): {}", v.join("; ")))
    };
    match kind {
        "no_overlap" => {
            let ns: Vec<(&Value, Rect)> = pick(tree, p)
                .into_iter()
                .filter_map(|n| {
                    let r = Rect::of(n)?;
                    match scroller_of(tree, n, &r) {
                        Some(s) => r.cut(&s, 0.0).map(|c| (n, c)),
                        None => Some((n, r)),
                    }
                })
                .collect();
            let mut out = vec![];
            for (i, (a, ra)) in ns.iter().enumerate() {
                for (b, rb) in &ns[i + 1..] {
                    if ra.contains(rb, 0.5) || rb.contains(ra, 0.5) {
                        continue;
                    }
                    if let Some(c) = ra.cut(rb, p["tolerance"].as_f64().unwrap_or(0.5)) {
                        out.push(format!("{} [{}] and {} [{}] overlap by {:.0}x{:.0}", a["id"].as_str().unwrap_or("?"), ra.show(), b["id"].as_str().unwrap_or("?"), rb.show(), c.w, c.h));
                    }
                }
            }
            bad(out, "overlapping")
        }
        "min_size" => {
            let min = p["min"].as_f64().unwrap_or(24.0);
            let out = pick(tree, p)
                .into_iter()
                .filter_map(|n| Rect::of(n).filter(|r| r.w < min - 0.01 || r.h < min - 0.01).map(|r| format!("{} is {:.1}x{:.1}", n["id"].as_str().unwrap_or("?"), r.w, r.h)))
                .collect();
            bad(out, &format!("smaller than {min} pt"))
        }
        "on_screen" => {
            let win = window(tree);
            let out = pick(tree, p)
                .into_iter()
                .filter_map(|n| {
                    let r = Rect::of(n)?;
                    let ok = match scroller_of(tree, n, &r) {
                        Some(_) => r.cut(&win, 0.0).is_some(),
                        None => win.contains(&r, 0.5),
                    };
                    (!ok).then(|| format!("{} at {} (window {:.0}x{:.0})", n["id"].as_str().unwrap_or("?"), r.show(), win.w, win.h))
                })
                .collect();
            bad(out, "off screen")
        }
        "text_width" => {
            let out = nodes(tree)
                .iter()
                .filter(|n| n["visible"].as_bool() == Some(true) && n["text"].as_str().is_some_and(|t| !t.trim().is_empty()))
                .filter_map(|n| {
                    let r = Rect::of(n);
                    (r.is_none_or(|r| r.w <= 0.0 || r.h <= 0.0)).then(|| format!("{} ({:?}) is {}", n["id"].as_str().unwrap_or("?"), crate::mcp::clip(n["text"].as_str().unwrap_or(""), 30), r.map(|r| r.show()).unwrap_or("without a rect".into())))
                })
                .collect();
            bad(out, "text cut to nothing")
        }
        "window" => {
            let win = window(tree);
            if let Some(w) = p["w"].as_f64() {
                if (win.w - w).abs() > 0.5 {
                    return Err(format!("the window is {:.0}x{:.0} (wanted w {w})", win.w, win.h));
                }
            }
            if let Some(h) = p["h"].as_f64() {
                if (win.h - h).abs() > 0.5 {
                    return Err(format!("the window is {:.0}x{:.0} (wanted h {h})", win.w, win.h));
                }
            }
            if let Some(s) = p["unchanged_since"].as_str() {
                let then = window(snaps.get(s).ok_or(format!("no snapshot {s:?}"))?);
                if (then.w - win.w).abs() > 0.5 || (then.h - win.h).abs() > 0.5 {
                    return Err(format!("the window went from {:.0}x{:.0} to {:.0}x{:.0}", then.w, then.h, win.w, win.h));
                }
            }
            Ok(())
        }
        "inside" => {
            let within = p["within"].as_str().unwrap_or("");
            let outer = node(tree, within).and_then(Rect::of).ok_or(format!("{within}: not in the tree"))?;
            let out = nodes(tree)
                .iter()
                .filter(|n| n["visible"].as_bool() == Some(true) && glob(p["id"].as_str().unwrap_or(""), n["id"].as_str().unwrap_or("")))
                .filter_map(|n| Rect::of(n).filter(|r| !within_axis(&outer, r, p["axis"].as_str().unwrap_or("both"))).map(|r| format!("{} at {} is outside {within} [{}]", n["id"].as_str().unwrap_or("?"), r.show(), outer.show())))
                .collect();
            bad(out, "outside")
        }
        // every shown panel presents a texture the size of its box, and none is stale (a substrate that composites panels
        // reports {name, shown, rect, texture: {w, h, scale?}, stale} under `panels`; texture in pixels, rect in points)
        "panels_fresh" => {
            let ps = tree["panels"].as_array().ok_or("the tree has no panels (this substrate doesn't report its panels' textures)")?;
            let names: Vec<&str> = p["names"].as_array().map(|a| a.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
            let tol = p["tolerance"].as_f64().unwrap_or(1.0);
            let mut out = vec![];
            let mut seen = 0;
            for q in ps.iter().filter(|q| q["shown"].as_bool() == Some(true)) {
                let name = q["name"].as_str().unwrap_or("?");
                if !names.is_empty() && !names.contains(&name) {
                    continue;
                }
                if q["stale"].as_bool() == Some(true) {
                    out.push(format!("{name} is stale"));
                }
                let (Some(r), Some(tw), Some(th)) = (Rect::of(q), q["texture"]["w"].as_f64(), q["texture"]["h"].as_f64()) else { continue };
                seen += 1;
                let s = q["texture"]["scale"].as_f64().or(tree["scale"].as_f64()).unwrap_or(1.0);
                if (tw / s - r.w).abs() > tol || (th / s - r.h).abs() > tol {
                    out.push(format!("{name} presents {:.0}x{:.0} in a {:.0}x{:.0} box", tw / s, th / s, r.w, r.h));
                }
            }
            if seen == 0 && out.is_empty() {
                return Err("no shown panel reports a texture size".into());
            }
            bad(out, "panels not presented at their size")
        }
        other => Err(format!("no check {other:?}")),
    }
}

/// Does `p` hold on `tree`? Err: what the tree showed instead.
pub fn holds(p: &Value, tree: &Value, snaps: &Snaps) -> Result<(), String> {
    if let Some(all) = p["all"].as_array() {
        for q in all {
            holds(q, tree, snaps)?;
        }
        return Ok(());
    }
    if let Some(any) = p["any"].as_array() {
        let errs: Vec<String> = any.iter().filter_map(|q| holds(q, tree, snaps).err()).collect();
        return if errs.len() < any.len() { Ok(()) } else { Err(format!("none held: {}", errs.join(" | "))) };
    }
    if !p["not"].is_null() {
        return match holds(&p["not"], tree, snaps) {
            Ok(()) => Err(format!("held, and shouldn't have: {}", p["not"])),
            Err(_) => Ok(()),
        };
    }
    if p.get("check").is_some() {
        return check(p, tree, snaps);
    }
    let ns = selected(p, tree);
    // exists is about the selection, not each node: false holds when no node matches the id (a glob included) and
    // `where`, true when one does (a glob that matches nothing has no node to fail on, so say so here)
    if let Some(e) = p["exists"].as_bool() {
        let there: Vec<&Value> = ns.iter().filter(|n| n["absent"].as_bool() != Some(true)).collect();
        let id = p["id"].as_str().unwrap_or("*");
        if !e {
            return if there.is_empty() { Ok(()) } else { Err(format!("{} there: {}", there.len(), there.iter().map(|n| show(n)).take(4).collect::<Vec<_>>().join("; "))) };
        }
        if there.is_empty() {
            return Err(format!("{id}: not in the tree"));
        }
    }
    let results: Vec<(&Value, Result<(), String>)> = ns.iter().map(|n| (n, node_holds(p, n, snaps))).collect();
    let ok = results.iter().filter(|(_, r)| r.is_ok()).count();
    let fails = || results.iter().filter_map(|(_, r)| r.as_ref().err().cloned()).take(4).collect::<Vec<_>>().join("; ");
    let id = p["id"].as_str().unwrap_or("*");
    if let Some(c) = p["count"].as_u64() {
        if ok as u64 == c {
            return Ok(());
        }
        let which: Vec<&str> = results.iter().filter(|(_, r)| r.is_ok()).filter_map(|(n, _)| n["id"].as_str()).take(6).collect();
        return Err(format!("{ok} of {id} hold it (wanted {c}){}", if which.is_empty() { String::new() } else { format!(": {}", which.join(", ")) }));
    }
    match p["quant"].as_str().unwrap_or("any") {
        "all" if ok == results.len() => Ok(()),
        "all" => Err(fails()),
        "none" if ok == 0 => Ok(()),
        "none" => Err(format!("{} hold it: {}", ok, results.iter().filter(|(_, r)| r.is_ok()).map(|(n, _)| show(n)).take(4).collect::<Vec<_>>().join("; "))),
        _ if ok > 0 => Ok(()),
        _ => Err(fails()),
    }
}
