//! Fields of grass' rows (`<Grass>`, sdk/render/src/3d/grass.tsx): a `grass` row each, its spec (the grids the edit
//! sampled and the blades' look, as JSON) and whether it casts shadows; systems/grass makes its blades. A field is
//! the first frame's: it doesn't change over the shot (the wind that moves it is the paint look's).

use super::*;

pub(super) fn rows(tracks: &[Track], p: &mut Program) {
    for (k, (_, track)) in tracks.iter().enumerate() {
        let its = filled(track);
        let first = &its[0].props;
        let Some(spec) = first.get("spec").filter(|s| s.is_object()) else { continue };
        let id = format!("grass{k}");
        let mut row = BTreeMap::new();
        row.insert("id".into(), V::S(id));
        row.insert("grass".into(), V::S(spec.to_string()));
        if first.get("shadow").and_then(Value::as_bool) == Some(true) {
            row.insert("shadow".into(), V::N(1.0));
        }
        p.spawns.push(("grass".into(), vec![row]));
    }
}
