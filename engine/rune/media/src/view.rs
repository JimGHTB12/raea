//! The page's media elements for raea-rune-page's view (T3d): `<img>` and `<video>`, a `<style>` (not drawn: the
//! host maps inline styles and Tailwind classes, so a sheet's rules are reported, and so is an @font-face it brings
//! that didn't register at the start), and `fontFamily`'s family as Rune knows it.

use rune::{El, Family, SurfaceFit, el};
use serde_json::{Map, Value};

use raea_rune_style::report::Report;

use crate::fonts;
use crate::images::{Images, Load};
use crate::video::Videos;

/// objectFit as css.rs reads it: unset, or `Some(None)` for fill (stretch), `Some(Some(fit))` for contain or cover.
pub type Fit = Option<Option<SurfaceFit>>;

/// Element `id`'s image: its pixels once its source loads, sized by its style or its own aspect (objectFit fits or
/// crops it), else an empty box; one that can't load is reported.
pub fn img(id: u64, props: &Map<String, Value>, fit: Fit, path: &str, images: &Images, report: &mut Report) -> El {
    let src = props.get("src").and_then(Value::as_str).unwrap_or("");
    match images.show(id, src) {
        Load::Image(image) => match fit.flatten() {
            Some(f) => el().pixels_fit(image.pixels(), f),
            None => el().pixels(image.pixels()),
        },
        Load::Fetching => el(),
        Load::Failed(why) => {
            report.note(path, "img src", src, leak(why));
            el()
        }
    }
}

/// A video (element `id`, its objectFit) on Rune's video surface, at its natural size (Chrome's 300x150 until it
/// loads) unless its style sizes it; one that can't open is reported.
pub fn video(props: &Map<String, Value>, (id, fit): (u64, Fit), path: &str, videos: &Videos, report: &mut Report) -> El {
    match videos.player(id, props) {
        Ok(p) => {
            let (w, h) = p.natural_size().map_or((300.0, 150.0), |(w, h)| (w as f32, h as f32));
            // CSS's fill stretches; Rune's surface fits or crops, so fill is drawn contained (reported)
            if fit == Some(None) {
                report.note(path, "objectFit", "fill", "a video is drawn contained (Rune's surface fits or crops)");
            }
            el().video(&p, fit.flatten().unwrap_or(SurfaceFit::Fit)).w(w).h(h)
        }
        Err(why) => {
            report.note(path, "video src", props.get("src").and_then(Value::as_str).unwrap_or(""), leak(why));
            el().w(300.0).h(150.0)
        }
    }
}

/// A `<style>`'s text: each face it brings that didn't register at the start, and its other rules, reported.
pub fn sheet(css: &str, path: &str, report: &mut Report) {
    for f in fonts::late(css) {
        report.note(path, "@font-face", &f, "a face after the start: Rune registers fonts only when it starts (projects/rune/upstream/fonts-after-start.md)");
    }
    let rest: String = css.split("@font-face").enumerate().map(|(i, p)| if i == 0 { p } else { p.split_once('}').map_or("", |(_, r)| r) }).collect();
    if rest.contains('{') {
        report.note(path, "style", "rules", "a stylesheet's rules (the host maps inline styles and Tailwind classes)");
    }
}

/// `f` as Rune's family: its sans and mono slots, or a registered face's family (an @font-face name reaches the
/// face's own).
pub fn family(f: Option<&'static str>) -> Family {
    match f {
        None | Some("Inter") | Some("sans-serif") | Some("ui-sans-serif") | Some("system-ui") => Family::Sans,
        Some("monospace") | Some("ui-monospace") => Family::Mono,
        Some(f) => Family::Named(fonts::family(f).unwrap_or(f)),
    }
}

/// A fontFamily whose face didn't register, reported where it's named.
pub fn check_family(f: Option<&str>, path: &str, report: &mut Report) {
    if let Some(why) = f.and_then(fonts::missing) {
        report.note(path, "fontFamily", f.unwrap_or(""), why);
    }
}

/// A why as the report keeps it ('static): each one leaked once.
fn leak(why: String) -> &'static str {
    use std::sync::{Mutex, OnceLock};
    static WHYS: OnceLock<Mutex<Vec<&'static str>>> = OnceLock::new();
    let mut all = WHYS.get_or_init(Default::default).lock().unwrap();
    if let Some(w) = all.iter().find(|w| **w == why) {
        return w;
    }
    let w: &'static str = Box::leak(why.into_boxed_str());
    all.push(w);
    w
}
