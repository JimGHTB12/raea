//! fontique's own generic families under names of their own (`raea-bevy sans-serif`, ...), for bevy_ui's generic
//! FontSources on a page (sdk/render bevy3d/ui-font.ts): Bevy resolves a generic family with fontique as it is, where a
//! page's CSS generics are mapped as Chrome's (sans-serif Inter, monospace Menlo) and Times, Helvetica and Courier get
//! Chrome's taller ascent.

use blitz_dom::FontContext;
use parley::fontique::{FontInfoOverride, GenericFamily};

/// Each generic family bevy_text has, by its CSS name. Emoji stays the page's own: Apple Color Emoji alone is about
/// 180 MB to read again, and its colour glyphs don't differ between the two.
const GENERICS: [(GenericFamily, &str); 10] = [
    (GenericFamily::Serif, "serif"),
    (GenericFamily::SansSerif, "sans-serif"),
    (GenericFamily::Monospace, "monospace"),
    (GenericFamily::Cursive, "cursive"),
    (GenericFamily::Fantasy, "fantasy"),
    (GenericFamily::SystemUi, "system-ui"),
    (GenericFamily::UiSerif, "ui-serif"),
    (GenericFamily::UiSansSerif, "ui-sans-serif"),
    (GenericFamily::UiMonospace, "ui-monospace"),
    (GenericFamily::UiRounded, "ui-rounded"),
];

/// The family a generic's alias is registered as.
pub fn alias(css: &str) -> String {
    format!("raea-bevy {css}")
}

/// Each generic family's first family, as fontique has it now, registered again as `alias(css)`: called before raea
/// maps the generics and boosts its three faces, so the alias holds the system's faces as Bevy draws them.
pub fn register(cx: &mut FontContext) {
    for (generic, css) in GENERICS {
        let Some(id) = cx.collection.generic_families(generic).next() else { continue };
        let Some(family) = cx.collection.family(id) else { continue };
        let name = alias(css);
        let mut seen = Vec::new();
        for font in family.fonts() {
            if seen.contains(&font.source().id()) {
                continue;
            }
            seen.push(font.source().id());
            let Some(blob) = font.load(None) else { continue };
            cx.collection.register_fonts(blob, Some(FontInfoOverride { family_name: Some(name.as_str()), ..Default::default() }));
        }
    }
}
