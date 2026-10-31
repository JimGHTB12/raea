//! raea's bundled faces: the Fontsource latin subsets in this folder (index.json), and the aliases that map a requested
//! family onto one (the composition format's deterministic fonts: Helvetica, Arial and SF Pro draw as Inter, Times and Georgia as EB
//! Garamond, Courier and Menlo as JetBrains Mono, ...), so a render looks the same on every machine. Each family is
//! under the SIL Open Font License 1.1 (licenses/).

/// Lower-cased family → canonical bundled family (the composition format's FONT_ALIAS_MAP).
pub const ALIASES: &[(&str, &str)] = &[
    ("inter", "inter"), ("montserrat", "montserrat"), ("outfit", "outfit"), ("nunito", "nunito"),
    ("oswald", "oswald"), ("league gothic", "league-gothic"), ("archivo black", "archivo-black"),
    ("space mono", "space-mono"), ("ibm plex mono", "ibm-plex-mono"), ("jetbrains mono", "jetbrains-mono"),
    ("eb garamond", "eb-garamond"), ("playfair display", "playfair-display"), ("source code pro", "source-code-pro"),
    ("noto sans jp", "noto-sans-jp"), ("roboto", "roboto"), ("open sans", "open-sans"), ("lato", "lato"),
    ("poppins", "poppins"),
    ("helvetica neue", "inter"), ("helvetica", "inter"), ("arial", "inter"), ("helvetica bold", "inter"),
    ("futura", "montserrat"), ("din alternate", "montserrat"), ("arial black", "montserrat"),
    ("bebas neue", "league-gothic"), ("courier new", "jetbrains-mono"), ("courier", "jetbrains-mono"),
    ("garamond", "eb-garamond"), ("noto sans japanese", "noto-sans-jp"), ("segoe ui", "roboto"),
    ("sf pro", "inter"), ("sf pro display", "inter"), ("sf pro text", "inter"), ("sf pro rounded", "inter"),
    ("avenir", "inter"), ("avenir next", "inter"), ("lucida grande", "inter"), ("geneva", "inter"),
    ("optima", "inter"), ("verdana", "inter"), ("tahoma", "inter"), ("trebuchet ms", "inter"),
    ("calibri", "inter"), ("candara", "inter"), ("corbel", "inter"), ("lucida sans", "inter"),
    ("lucida sans unicode", "inter"), ("noto sans", "inter"), ("dejavu sans", "inter"),
    ("liberation sans", "inter"), ("sf mono", "jetbrains-mono"), ("menlo", "jetbrains-mono"),
    ("monaco", "jetbrains-mono"), ("consolas", "jetbrains-mono"), ("lucida console", "jetbrains-mono"),
    ("lucida sans typewriter", "jetbrains-mono"), ("andale mono", "jetbrains-mono"),
    ("dejavu sans mono", "jetbrains-mono"), ("liberation mono", "jetbrains-mono"),
    ("georgia", "eb-garamond"), ("palatino", "eb-garamond"), ("palatino linotype", "eb-garamond"),
    ("book antiqua", "eb-garamond"), ("cambria", "eb-garamond"), ("times", "eb-garamond"),
    ("times new roman", "eb-garamond"), ("dejavu serif", "eb-garamond"), ("liberation serif", "eb-garamond"),
];

/// (canonical family, weight, style, woff2 bytes)
pub const BUNDLED: &[(&str, &str, &str, &[u8])] = &include!(concat!(env!("OUT_DIR"), "/bundled_fonts.rs"));

/// The canonical bundled family `family` draws as (aliases applied), if any.
pub fn canonical(family: &str) -> Option<&'static str> {
    let lower = family.trim().to_ascii_lowercase();
    ALIASES.iter().find(|(k, _)| *k == lower).map(|(_, v)| *v)
}

/// The bundled face (WOFF2) for `family`, aliases applied: the nearest weight, the matching style first. None when
/// no bundled family matches.
pub fn face(family: &str, weight: u16, italic: bool) -> Option<&'static [u8]> {
    let canonical = canonical(family)?;
    let style = if italic { "italic" } else { "normal" };
    BUNDLED
        .iter()
        .filter(|(f, _, _, _)| *f == canonical)
        .min_by_key(|(_, w, st, _)| {
            let w: i32 = w.parse().unwrap_or(400);
            ((*st != style) as i32) * 10_000 + (w - weight as i32).abs()
        })
        .map(|(_, _, _, bytes)| *bytes)
}

/// The bundled face for `family` as TrueType/OpenType bytes (WOFF2 decoded), for code that reads outlines itself
/// (raea-render's text3d). None when no bundled face matches.
pub fn face_ttf(family: &str, weight: u16, italic: bool) -> Option<Vec<u8>> {
    face(family, weight, italic).and_then(|w| wuff::decompress_woff2(w).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_pick_bundled_faces() {
        assert_eq!(canonical("Helvetica"), Some("inter"));
        assert_eq!(canonical("Times New Roman"), Some("eb-garamond"));
        assert_eq!(canonical("Comic Sans"), None);
        // the nearest weight: 600 is nearer 700 than 400
        assert_eq!(face("Inter", 600, false), face("inter", 700, false));
        assert!(face_ttf("Inter", 400, false).is_some_and(|b| b.len() > 1000));
    }
}
