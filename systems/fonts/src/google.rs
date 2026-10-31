//! Google Fonts by name: a family a page's styles name that no face here has is looked for on Google Fonts, and its
//! Latin faces (each weight and style it has) are fetched once into raea's cache (raea_io_net::cache, so a
//! render is the same frame for frame, and offline once cached) and given to the document as an @font-face would give
//! them. Chrome loads only what a stylesheet links; a composition here just names the family (craft-system.md phase
//! 8: the Director had to fetch and embed each family itself). Only the Latin subset: Blitz has no unicode-range, so
//! every subset of a weight under one name would be one face to it, and one without Latin could be the one picked.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use blitz_dom::BaseDocument;
use blitz_dom::net::{FontFaceOverrides, Resource, ResourceLoadResponse};
use blitz_traits::net::Bytes;
use parley::fontique::FontStyle;

/// A family's faces: (TrueType bytes, weight, italic)
type Faces = Arc<Vec<(Bytes, f32, bool)>>;

/// Each family looked for in this process: its faces, none when Google Fonts has no such family or it can't be
/// fetched (looked for once a process, not every frame)
static FAMILIES: Mutex<Option<HashMap<String, Faces>>> = Mutex::new(None);

/// Every weight, upright and italic: Google's CSS API answers with the ones the family has
const WEIGHTS: &str = "100,100i,200,200i,300,300i,400,400i,500,500i,600,600i,700,700i,800,800i,900,900i";

/// RAEA_PAGE_GOOGLE_FONTS=1 turns it on (off by default: Chrome loads only what a page links)
pub fn enabled() -> bool {
    static ON: std::sync::LazyLock<bool> = std::sync::LazyLock::new(|| std::env::var("RAEA_PAGE_GOOGLE_FONTS").as_deref() == Ok("1"));
    *ON
}

/// Gives `doc` the faces of each family its elements name that it has none of and Google Fonts has; true when one
/// was given (its text is laid out again).
pub fn load_missing(doc: &mut BaseDocument) -> bool {
    let mut loaded = false;
    for family in doc.missing_font_families() {
        let faces = FAMILIES.lock().unwrap().get_or_insert_default().get(&family).cloned();
        let faces = faces.unwrap_or_else(|| {
            let found = Arc::new(fetch(&family));
            FAMILIES.lock().unwrap().get_or_insert_default().insert(family.clone(), found.clone());
            found
        });
        for (ttf, weight, italic) in faces.iter() {
            let overrides = FontFaceOverrides { family_name: Some(family.clone()), weight: Some(*weight), style: italic.then_some(FontStyle::Italic) };
            doc.load_resource(ResourceLoadResponse { request_id: usize::MAX, node_id: None, resolved_url: None, result: Ok(Resource::Font(ttf.clone(), overrides)) });
            loaded = true;
        }
    }
    loaded
}

/// A family's Latin faces from Google Fonts, or none
fn fetch(family: &str) -> Vec<(Bytes, f32, bool)> {
    let url = format!("https://fonts.googleapis.com/css?family={}:{WEIGHTS}", family.replace(' ', "+"));
    let css = match raea_io_net::cache::read(&url).and_then(|b| String::from_utf8(b).map_err(|e| e.to_string())) {
        Ok(css) => css,
        Err(e) => {
            bevy::log::debug!("raea-page: no Google Fonts family {family:?} ({e})");
            return Vec::new();
        }
    };
    let faces: Vec<_> = css
        .split("/* latin */")
        .skip(1)
        .filter_map(|block| {
            let block = &block[..block.find('}')?];
            let src = block.split("url(").nth(1)?.split(')').next()?;
            let weight = block.split("font-weight:").nth(1)?.trim_start().split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()?;
            let italic = block.split("font-style:").nth(1)?.trim_start().starts_with("italic");
            let woff2 = raea_io_net::cache::read(src).ok()?;
            let ttf = wuff::decompress_woff2(&woff2).ok()?;
            Some((Bytes::from(ttf), weight, italic))
        })
        .collect();
    bevy::log::info!("raea-page: Google Fonts {family:?}, {} faces", faces.len());
    faces
}
