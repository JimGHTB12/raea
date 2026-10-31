//! A frame's layout from its inputs: the panes, the split, the subject's zone, the text blocks.

use super::*;

/// How a frame is laid out, from what Vision saw in it.
pub fn layout(i: &Inputs) -> Layout {
    let (ww, hh, g) = (i.w, i.h, &i.g);
    let ratio = first_max([("9:16", 9.0 / 16.0), ("4:5", 0.8), ("1:1", 1.0), ("16:9", 16.0 / 9.0)], |r| -(r.1 - ww / hh).abs())
        .map(|r| r.0)
        .unwrap_or("9:16");
    let mut zones: Vec<Zone> = Vec::new();
    let mut split = Split { kind: "none".into(), angle: None };

    let text_px: Vec<Rect> =
        i.lines.iter().map(|l| Rect::new(l.0.x * g.w as f64, l.0.y * g.h as f64, l.0.w * g.w as f64, l.0.h * g.h as f64)).collect();
    let still = split_holds(g, &i.next, &text_px);
    let mut framed: Option<Rect> = None;
    let mut pane_zones: Vec<Rect> = Vec::new();
    if still {
        let (all, first) = panes(g, &text_px);
        let content: Vec<Rect> = all
            .iter()
            .filter(|r| is_picture(g, **r))
            .map(|r| Rect::new(r.0 / g.w as f64, r.1 / g.h as f64, (r.2 - r.0) / g.w as f64, (r.3 - r.1) / g.h as f64))
            .collect();
        let around: Vec<R4> = all.iter().filter(|r| !is_picture(g, **r)).copied().collect();
        if all.len() > 1 && content.len() == 1 && around.iter().all(|r| detail(g, *r) <= 0.006) {
            framed = Some(content[0]);
        } else if content.len() > 1 {
            split.kind = if content.len() == 2 { if first == Some("row") { "stacked" } else { "side-by-side" } } else { "grid" }.into();
            pane_zones = content;
        }
        if split.kind == "none" && framed.is_none() {
            if let Some(d) = diagonal(g) {
                split = Split { kind: "diagonal".into(), angle: Some(r3(-d.0)) };
            }
        }
    }
    if framed.is_none() && split.kind == "none" {
        if let Some(band) = framed_band(g) {
            framed = Some(Rect::new(0.0, band.0, 1.0, band.1 - band.0));
        }
    }

    // screens and slides
    let small: Vec<&(Rect, String)> = i.lines.iter().filter(|l| l.0.h < 0.022).collect();
    let mut screen: Option<Rect> = None;
    if small.len() >= 6 {
        let bx = small.iter().fold(small[0].0, |a, l| a.union(&l.0));
        let holder = first_max(i.rects.iter().copied().filter(|r| small.iter().filter(|l| r.contains(&l.0)).count() as f64 >= 0.6 * small.len() as f64), |r| r.area());
        screen = Some(holder.unwrap_or(Rect::new(bx.x - 0.03, bx.y - 0.03, bx.w + 0.06, bx.h + 0.06)).clamped());
    } else if let Some(slide) =
        first_max(i.rects.iter().copied().filter(|r| r.area() > 0.2 && r.area() < 0.97 && i.lines.iter().filter(|l| r.contains(&l.0)).count() >= 4), |r| r.area())
    {
        screen = Some(slide);
    } else if i.screenshot && pane_zones.is_empty() {
        screen = Some(first_max(i.rects.iter().copied().filter(|r| r.area() > 0.12), |r| r.area()).or(framed).unwrap_or(Rect::new(0.0, 0.0, 1.0, 1.0)));
    }
    if let Some(sc) = screen {
        zones.push(sc.zone("video", None, None, Some("screen".into())));
        if pane_zones.iter().filter(|p| sc.inter(p) > 0.5 * p.area()).count() as i64 >= pane_zones.len() as i64 - 1 {
            pane_zones.clear();
            split = Split { kind: "none".into(), angle: None };
        }
    }
    for p in &pane_zones {
        zones.push(p.zone("video", None, None, Some("pane".into())));
    }
    if let Some(f) = framed {
        if !screen.is_some_and(|s| s.inter(&f) > 0.7 * s.area().min(f.area())) {
            zones.push(f.zone("video", None, None, Some("framed video".into())));
        }
    }

    // people
    let mut people: Vec<Rect> = Vec::new();
    for f in &i.faces {
        let mut body = Rect::new(f.x - 1.2 * f.w, f.y - 0.6 * f.h, 3.4 * f.w, 4.6 * f.h).clamped();
        if let Some(m) = &i.mask {
            let (mut x0, mut y0, mut x1, mut y1) = (1.0f64, 1.0f64, 0.0f64, 0.0f64);
            for k in 0..i.mw * i.mh {
                if m[k] > 0.5 {
                    let x = ((k % i.mw) as f64 + 0.5) / i.mw as f64;
                    let y = ((k / i.mw) as f64 + 0.5) / i.mh as f64;
                    if x > body.x && x < body.x1() && y > body.y && y < body.y1() {
                        x0 = x0.min(x);
                        y0 = y0.min(y);
                        x1 = x1.max(x);
                        y1 = y1.max(y);
                    }
                }
            }
            if x1 > x0 {
                body = Rect::new(x0, y0, x1 - x0, y1 - y0).union(f).clamped();
            }
        }
        let (fx, fy) = (f.x + f.w / 2.0, f.y + f.h / 2.0);
        let holder = screen.or(framed.filter(|r| r.area() < 0.95));
        let on_screen = holder.is_some_and(|r| fx > r.x + 0.15 * r.w && fx < r.x1() - 0.15 * r.w && fy > r.y + 0.15 * r.h && fy < r.y1() - 0.15 * r.h);
        let head_and_shoulders = body.h < 3.6 * f.h && body.area() < 0.15;
        let cut = body.y1() > 0.97 || body.x < 0.03 || body.x1() > 0.97;
        let (gx, gy) = (g.w as f64, g.h as f64);
        let seams = [
            ((body.x * gx, body.y * gy), (body.x1() * gx, body.y * gy)),
            ((body.x * gx, body.y * gy), (body.x * gx, body.y1() * gy)),
            ((body.x1() * gx, body.y * gy), (body.x1() * gx, body.y1() * gy)),
        ]
        .iter()
        .filter(|s| coverage(g, s.0, s.1, 0.07, &[]) >= 0.7)
        .count();
        let inset = head_and_shoulders && !on_screen && (cut && (fx < 0.35 || fx > 0.65) || seams >= 2);
        let bubble = (body.area() < 0.12 && holder.is_some() && !on_screen) || inset;
        let label = if bubble { Some("face bubble".to_string()) } else if on_screen { Some("on screen".to_string()) } else { None };
        zones.push(body.zone("subject", Some(*f), None, label));
        people.push(body);
    }
    // people the face finder missed: parts of the person mask with no face in them
    if let Some(m) = &i.mask {
        let gw = 48usize;
        let gh = ((48.0 * hh / ww) as usize).max(1);
        let mut on = vec![false; gw * gh];
        let mut seen = vec![false; gw * gh];
        for k in 0..i.mw * i.mh {
            if m[k] > 0.5 {
                on[((k / i.mw) * gh / i.mh).min(gh - 1) * gw + ((k % i.mw) * gw / i.mw).min(gw - 1)] = true;
            }
        }
        for start in 0..gw * gh {
            if !on[start] || seen[start] {
                continue;
            }
            let mut stack = vec![start];
            let mut cells = Vec::new();
            seen[start] = true;
            while let Some(c) = stack.pop() {
                cells.push(c);
                let (x, y) = ((c % gw) as i64, (c / gw) as i64);
                for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                    if nx >= 0 && ny >= 0 && nx < gw as i64 && ny < gh as i64 {
                        let k = ny as usize * gw + nx as usize;
                        if on[k] && !seen[k] {
                            seen[k] = true;
                            stack.push(k);
                        }
                    }
                }
            }
            if cells.len() as f64 / ((gw * gh) as f64) <= 0.01 {
                continue;
            }
            let xs: Vec<usize> = cells.iter().map(|c| c % gw).collect();
            let ys: Vec<usize> = cells.iter().map(|c| c / gw).collect();
            let (xmin, xmax, ymin, ymax) = (*xs.iter().min().unwrap(), *xs.iter().max().unwrap(), *ys.iter().min().unwrap(), *ys.iter().max().unwrap());
            let r = Rect::new(xmin as f64 / gw as f64, ymin as f64 / gh as f64, (xmax - xmin + 1) as f64 / gw as f64, (ymax - ymin + 1) as f64 / gh as f64);
            if people.iter().any(|p| p.inter(&r) > 0.3 * r.area()) {
                continue;
            }
            zones.push(r.zone("subject", None, None, Some("person".into())));
        }
    }

    // captions and titles: text blocks outside the screen
    let outside: Vec<(Rect, String)> = i.lines.iter().filter(|l| !screen.is_some_and(|s| s.contains(&l.0))).cloned().collect();
    for b in text_blocks(&outside) {
        let bx = b.iter().fold(b[0].0, |a, l| a.union(&l.0));
        if bx.h <= 0.012 {
            continue;
        }
        let words = b.iter().map(|l| l.1.split(' ').filter(|w| !w.is_empty()).count()).sum();
        let mut sorted = b.clone();
        sorted.sort_by(|p, q| p.0.y.total_cmp(&q.0.y));
        let text: String = sorted.iter().map(|l| l.1.as_str()).collect::<Vec<_>>().join(" ");
        zones.push(bx.zone("text", None, Some(words), Some(text.chars().take(40).collect())));
    }
    Layout { ratio: ratio.into(), split, zones }
}

/// The inputs as `ugc-perceive layout-inputs` prints them (grey and mask values as bytes).
pub fn inputs_from_json(j: &serde_json::Value) -> Result<Inputs, String> {
    let rect = |v: &serde_json::Value| -> Rect {
        let a: Vec<f64> = v.as_array().map(|a| a.iter().filter_map(|x| x.as_f64()).collect()).unwrap_or_default();
        Rect::new(a[0], a[1], a[2], a[3])
    };
    let bytes = |v: &serde_json::Value| -> Vec<f32> { v.as_array().map(|a| a.iter().map(|x| x.as_f64().unwrap_or(0.0) as f32 / 255.0).collect()).unwrap_or_default() };
    let grey = |v: &serde_json::Value| Grey { w: v["w"].as_u64().unwrap_or(0) as usize, h: v["h"].as_u64().unwrap_or(0) as usize, v: bytes(&v["v"]) };
    let mask = bytes(&j["mask"]);
    Ok(Inputs {
        w: j["W"].as_f64().ok_or("W")?,
        h: j["H"].as_f64().ok_or("H")?,
        screenshot: j["screenshot"].as_bool().unwrap_or(false),
        faces: j["faces"].as_array().map(|a| a.iter().map(rect).collect()).unwrap_or_default(),
        lines: j["lines"].as_array().map(|a| a.iter().map(|l| (rect(&l["box"]), l["text"].as_str().unwrap_or("").to_string())).collect()).unwrap_or_default(),
        rects: j["rects"].as_array().map(|a| a.iter().map(rect).collect()).unwrap_or_default(),
        mask: if mask.is_empty() { None } else { Some(mask) },
        mw: j["mw"].as_u64().unwrap_or(0) as usize,
        mh: j["mh"].as_u64().unwrap_or(0) as usize,
        g: grey(&j["g"]),
        next: j["next"].as_array().map(|a| a.iter().map(grey).collect()).unwrap_or_default(),
    })
}
