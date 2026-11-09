//! A drawn thing's layer key (docs/review.feature R2a): where its JSX is written, its place in a list, and the uses of
//! the edit's components it was drawn in, as engine/bundle's source stamps give them. A 3D part carries it as this
//! component (runtime/scene bevy_parts makes it a component row of the part), never as a column its kind reads.

use bevy::prelude::*;

/// "file:line:col[#index]|use|use…": the part's stamp, then its use chain, nearest first
#[derive(Component, Reflect, Default, Clone, Debug, PartialEq, Eq)]
#[reflect(Component, Default)]
pub struct LayerKey(pub String);

/// The key a stamped element or part's props give it: `data-raea-loc`, `#data-raea-idx` when it has one, then
/// `|data-raea-use` when it was drawn inside the edit's components. None for an unstamped one.
pub fn key_of(loc: Option<&str>, idx: Option<&str>, uses: Option<&str>) -> Option<String> {
    let loc = loc.filter(|l| !l.is_empty())?;
    let mut k = loc.to_string();
    if let Some(i) = idx.filter(|i| !i.is_empty()) {
        k.push('#');
        k.push_str(i);
    }
    if let Some(u) = uses.filter(|u| !u.is_empty()) {
        k.push('|');
        k.push_str(u);
    }
    Some(k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_is_the_stamp_its_place_and_its_uses() {
        assert_eq!(key_of(Some("a.tsx:3:5"), None, None).as_deref(), Some("a.tsx:3:5"));
        assert_eq!(key_of(Some("card.tsx:2:10"), Some("2"), Some("comp.tsx:6:27#2")).as_deref(), Some("card.tsx:2:10#2|comp.tsx:6:27#2"));
        assert_eq!(key_of(None, Some("2"), Some("x")), None);
    }
}
