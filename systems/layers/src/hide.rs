//! Layers hidden by key (RAEA_HIDE_LAYERS), as the layer pass's oracle needs them (bench/layers/oracle.sh): a page
//! element as CSS (this), a 3D part as its Visibility (runtime/scene).

use crate::Layer;

/// RAEA_HIDE_LAYERS: the layers keyed (a key a line or comma) drawn hidden, each element with its stamps and all
/// inside it, as page CSS. None when it names none.
pub fn hidden_css() -> Option<String> {
    let keys = std::env::var("RAEA_HIDE_LAYERS").ok()?;
    let rules: Vec<String> = keys.split([',', '\n']).filter(|k| !k.trim().is_empty()).map(|k| layer_selector(k.trim())).collect();
    (!rules.is_empty()).then(|| format!("{} {{ visibility: hidden !important; }}", rules.join(", ")))
}

/// The CSS selector of the element a layer key names, and of everything inside it: "loc[#index][|use|use…]" as its
/// data-raea-loc, data-raea-idx and data-raea-use (engine/bundle stamp.rs)
pub fn layer_selector(key: &str) -> String {
    let l = Layer::of(key.to_string(), "element", String::new(), None, None);
    let q = |v: &str| serde_json::to_string(v).unwrap_or_default();
    let mut sel = format!("[data-raea-loc={}]", q(&l.loc));
    sel.push_str(&l.index.map_or_else(|| ":not([data-raea-idx])".to_string(), |i| format!("[data-raea-idx={}]", q(&i))));
    sel.push_str(&Some(l.uses.join("|")).filter(|u| !u.is_empty()).map_or_else(|| ":not([data-raea-use])".to_string(), |u| format!("[data-raea-use={}]", q(&u))));
    format!("{sel}, {sel} *")
}
