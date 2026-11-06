//! A canvas's measureText: each character's advance in the face the canvas would draw it with (text in draw.rs).
//! The page asks for the characters it hasn't measured yet in a font, and keeps the ones its own family has; a
//! character another family stood in for is asked again, so a face the page loads later takes over.

use serde_json::{json, Value};
use usvg::fontdb::{Family, Query, Style, Weight, ID};

/// `canvas.advances`: {font: {family, weight, style}, chars} -> {adv: [em or null], own: [bool]}, one per character:
/// its advance as a fraction of the font size, and whether the font's first family has it.
pub fn advances(args: &Value) -> Result<Value, String> {
    Ok(advances_in(&crate::page_options(), args))
}

fn advances_in(opt: &usvg::Options, args: &Value) -> Value {
    let db = &opt.fontdb;
    let font = &args["font"];
    let weight = font["weight"].as_str().and_then(|w| w.parse().ok()).unwrap_or(400u16);
    let style = if font["style"].as_str() == Some("italic") { Style::Italic } else { Style::Normal };
    let names: Vec<String> = font["family"].as_str().unwrap_or("sans-serif").split(',').map(|s| s.trim().trim_matches(['\'', '"']).to_string()).filter(|s| !s.is_empty()).collect();
    let face_of = |name: &str| -> Option<ID> {
        let family = match name.to_lowercase().as_str() {
            "serif" => Family::Serif,
            "sans-serif" | "system-ui" => Family::SansSerif,
            "monospace" => Family::Monospace,
            "cursive" => Family::Cursive,
            "fantasy" => Family::Fantasy,
            _ => Family::Name(name),
        };
        db.query(&Query { families: &[family], weight: Weight(weight), style, ..Default::default() })
    };
    // the faces in the order a character is looked for: a split first family's subsets (subsets.rs), the families as
    // listed, then the default
    let mut faces: Vec<(ID, bool)> = Vec::new();
    if let (Some(split), Some(first)) = (crate::split_families(db), names.first()) {
        faces.extend(split.get(&first.to_lowercase()).into_iter().flatten().map(|(_, id)| (*id, true)));
    }
    faces.extend(names.iter().enumerate().filter_map(|(k, n)| face_of(n).map(|id| (id, k == 0))));
    faces.extend(face_of("sans-serif").map(|id| (id, false)));
    let chars: Vec<char> = args["chars"].as_str().unwrap_or("").chars().collect();
    let mut out: Vec<Option<(f64, bool)>> = vec![None; chars.len()];
    for (id, own) in faces {
        if out.iter().all(Option::is_some) {
            break;
        }
        db.with_face_data(id, |data, index| {
            let Ok(f) = ttf_parser::Face::parse(data, index) else { return };
            let em = f64::from(f.units_per_em());
            for (c, slot) in chars.iter().zip(out.iter_mut()).filter(|(_, s)| s.is_none()) {
                if let Some(a) = f.glyph_index(*c).and_then(|g| f.glyph_hor_advance(g)) {
                    *slot = Some((f64::from(a) / em, own));
                }
            }
        });
    }
    json!({
        "adv": out.iter().map(|s| s.map(|(a, _)| a)).collect::<Vec<_>>(),
        "own": out.iter().map(|s| s.is_some_and(|(_, o)| o)).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advances_are_the_faces_and_say_whose() {
        let mut db = usvg::fontdb::Database::new();
        db.load_font_data(std::fs::read("/System/Library/Fonts/Supplemental/Arial.ttf").unwrap());
        db.set_sans_serif_family("Arial");
        let opt = usvg::Options { fontdb: std::sync::Arc::new(db), ..Default::default() };
        let r = advances_in(&opt, &json!({"font": {"family": "Arial", "weight": "400", "style": "normal"}, "chars": "im"}));
        let (i, m) = (r["adv"][0].as_f64().unwrap(), r["adv"][1].as_f64().unwrap());
        assert!(i < 0.3 && m > 0.8, "{r}");
        assert_eq!(r["own"], json!([true, true]));
        // a family the page doesn't have: the default stands in, and says so
        let r = advances_in(&opt, &json!({"font": {"family": "Nope, sans-serif", "weight": "400", "style": "normal"}, "chars": "i"}));
        assert_eq!(r["own"], json!([false]));
        assert!((r["adv"][0].as_f64().unwrap() - i).abs() < 1e-9);
    }
}
