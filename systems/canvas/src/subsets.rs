//! Canvas text in a family split across files by unicode-range, as the web ships big fonts (Excalidraw's Excalifont
//! is seven subsets, all of Latin in one). fontdb ignores unicode-range: a query for the family gives one of its faces,
//! and glyphs that face lacks fall back to other families, so whole lines of a board drew in Helvetica. Each face of
//! such a family is also kept under an alias family of its own, and a text op names, run by run, the subset that has
//! the glyphs, as Chrome picks a face by unicode-range.

use std::collections::HashMap;

use usvg::fontdb::{Database, ID, Language, Source};

/// A split family's faces under their alias families, by the family's name in lower case.
pub type Subsets = HashMap<String, Vec<(String, ID)>>;

/// Adds an alias face for each face of a family that has several of the same style, weight and stretch (its
/// unicode-range subsets), and says which they are.
pub(crate) fn alias(db: &mut Database) -> Subsets {
    let mut groups: HashMap<String, Vec<ID>> = HashMap::new();
    for f in db.faces() {
        // only faces a page added from bytes (its @font-face faces): a system font file is never a subset
        if !matches!(f.source, Source::Binary(_)) {
            continue;
        }
        let Some((name, _)) = f.families.first() else { continue };
        groups.entry(format!("{}\u{0}{:?}{:?}{:?}", name.to_lowercase(), f.style, f.weight, f.stretch)).or_default().push(f.id);
    }
    let mut out = Subsets::new();
    for (key, ids) in groups {
        if ids.len() < 2 {
            continue;
        }
        let family = key.split('\u{0}').next().unwrap_or_default().to_string();
        // the files with the most glyphs first: a page's own subsets before the cut-down copy of the same family an
        // SVG image embeds (Excalidraw's exportToSvg keeps only the glyphs its picture uses, and blitz-dom adds those
        // faces too), which runs would otherwise pick for every glyph it happens to have
        let mut ids = ids;
        let glyphs = |id: ID| db.with_face_data(id, |data, index| ttf_parser::Face::parse(data, index).map(|f| f.number_of_glyphs()).unwrap_or(0)).unwrap_or(0);
        ids.sort_by_key(|id| std::cmp::Reverse(glyphs(*id)));
        for (k, id) in ids.into_iter().enumerate() {
            let Some(mut info) = db.face(id).cloned() else { continue };
            let name = format!("{family} raea-subset-{k}");
            info.families = vec![(name.clone(), Language::English_UnitedStates)];
            info.post_script_name = format!("{}-raea-subset-{k}", info.post_script_name);
            out.entry(family.clone()).or_default().push((name, db.push_face_info(info)));
        }
    }
    out
}

/// The text split into runs by which subset of its first family has each character, each run with that subset's
/// alias family (None: no subset has it, so the family list as given decides). None when the family isn't split.
pub(crate) fn runs(db: &Database, subsets: &Subsets, families: &str, text: &str) -> Option<Vec<(Option<String>, String)>> {
    let first = families.split(',').next()?.trim().trim_matches(['\'', '"']).to_lowercase();
    let faces = subsets.get(&first)?;
    let chars: Vec<char> = text.chars().collect();
    // which characters each subset has, read from its cmap once for the op
    let cover: Vec<Vec<bool>> = faces
        .iter()
        .map(|(_, id)| {
            db.with_face_data(*id, |data, index| {
                let f = ttf_parser::Face::parse(data, index).ok()?;
                Some(chars.iter().map(|c| f.glyph_index(*c).is_some()).collect())
            })
            .flatten()
            .unwrap_or_else(|| vec![false; chars.len()])
        })
        .collect();
    let mut out: Vec<(Option<String>, String)> = Vec::new();
    for (i, c) in chars.iter().enumerate() {
        // a space goes with the run it's in, whichever subset has it
        let pick = if c.is_whitespace() && !out.is_empty() {
            out.last().unwrap().0.clone()
        } else {
            cover.iter().position(|has| has[i]).map(|k| faces[k].0.clone())
        };
        match out.last_mut() {
            Some((f, s)) if *f == pick => s.push(*c),
            _ => out.push((pick, c.to_string())),
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FONT: &str = "/System/Library/Fonts/Supplemental/Arial.ttf";
    const OTHER: &str = "/System/Library/Fonts/Supplemental/Courier New.ttf";

    #[test]
    fn a_family_in_several_files_gets_an_alias_per_file_and_runs_name_the_one_with_the_glyphs() {
        let (Ok(bytes), Ok(other)) = (std::fs::read(FONT), std::fs::read(OTHER)) else { return };
        let mut db = Database::new();
        // two files of one family, as a page's unicode-range faces are, and a family in one file
        db.load_font_data(bytes.clone());
        db.load_font_data(bytes);
        db.load_font_data(other);
        let s = alias(&mut db);
        let faces = &s["arial"];
        assert_eq!(faces.len(), 2);
        assert_eq!(s.len(), 1);
        let r = runs(&db, &s, "Arial, sans-serif", "ab \u{4e2d}c").unwrap();
        let first = Some(faces[0].0.clone());
        // Arial has no CJK: that character is a run of its own that the family list decides
        assert_eq!(r, vec![(first.clone(), "ab ".to_string()), (None, "\u{4e2d}".to_string()), (first, "c".to_string())]);
        assert!(runs(&db, &s, "'No Such Family', sans-serif", "abc").is_none());
    }
}
