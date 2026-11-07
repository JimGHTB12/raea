//! The app's fonts (T3d), registered with Rune's text system when it's made: Rune takes its faces then and has no
//! call to add one later (projects/rune/upstream/fonts-after-start.md). So the host gathers them before it starts:
//! every font file under the app folder's `fonts/`, and every @font-face rule in the folder's CSS files (brand/brand.css
//! among them) and in the app's bundle (a `<style>` in its TSX), its first `url()` that loads read relative to the
//! file the rule is in, through raea's URL cache when it's http(s). WOFF and WOFF2 are decoded to the TrueType Rune
//! reads. A rule's `font-family` names its face for `fontFamily`, beside the face's own family name. Rune's Inter
//! stays the sans: it is registered first whenever anything else is, since faces given to Rune replace its own.
//!
//! What can't register is said, never dropped: a rule or file that doesn't load (`notes`), a family `fontFamily`
//! names that no face has (`missing`), and a face a `<style>` brings after the start (`late`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::src::{self, Src};

const INTER: [&[u8]; 3] = [include_bytes!("../assets/Inter-Regular.ttf"), include_bytes!("../assets/Inter-Medium.ttf"), include_bytes!("../assets/Inter-SemiBold.ttf")];

/// Families Rune draws without a face of the app's: its sans (Inter) and mono slots.
const GENERIC: [&str; 7] = ["inter", "sans-serif", "ui-sans-serif", "system-ui", "monospace", "ui-monospace", "-apple-system"];

/// One face: the names `fontFamily` reaches it by, its bytes (TrueType or OpenType) and where it came from.
#[derive(Clone, Debug)]
pub struct Face {
    pub names: Vec<String>,
    pub bytes: Vec<u8>,
    pub from: PathBuf,
}

#[derive(Debug, Default)]
pub struct Gathered {
    pub faces: Vec<Face>,
    /// (what, why) for each rule or file that didn't load
    pub notes: Vec<(String, String)>,
}

struct Registered {
    faces: &'static [&'static [u8]],
    /// a lowercase name → the family Rune knows the face by
    names: HashMap<String, &'static str>,
    notes: Vec<(String, String)>,
}

static REGISTERED: OnceLock<Registered> = OnceLock::new();

/// The app's faces in `dir` (its folder) and in `bundle` (its code), with what didn't load.
pub fn gather(dir: &Path, bundle: &str) -> Gathered {
    let mut g = Gathered::default();
    let mut files = Vec::new();
    walk(&dir.join("fonts"), 8, &mut files, &|p| has_ext(p, &["ttf", "otf", "woff", "woff2"]));
    for f in files {
        match std::fs::read(&f).map_err(|e| e.to_string()).and_then(|b| face(&b, Vec::new(), &f)) {
            Ok(face) => g.faces.push(face),
            Err(e) => g.notes.push((f.display().to_string(), e)),
        }
    }
    let mut sheets = Vec::new();
    walk(dir, 3, &mut sheets, &|p| has_ext(p, &["css"]));
    for s in sheets {
        let css = std::fs::read_to_string(&s).unwrap_or_default();
        rules(&css, s.parent().unwrap_or(dir), &s, &mut g);
    }
    // the bundle's rules: CSS inside JS strings, its escapes undone enough to read the rule
    let unescaped = bundle.replace("\\\"", "\"").replace("\\'", "'").replace("\\n", "\n");
    rules(&unescaped, dir, &dir.join("<bundle>"), &mut g);
    g
}

/// `g`'s faces given to Rune: what its text system (`TextSystem::new`, `AppSpec::fonts`) takes from `registered()`.
/// Once per process; a second call keeps the first's.
pub fn register(g: Gathered) {
    let mut names = HashMap::new();
    let mut faces: Vec<&'static [u8]> = Vec::new();
    if !g.faces.is_empty() {
        faces.extend(INTER);
    }
    for f in g.faces {
        let Some(own) = f.names.last().cloned() else { continue };
        let own: &'static str = Box::leak(own.into_boxed_str());
        for n in &f.names {
            names.entry(n.to_lowercase()).or_insert(own);
        }
        faces.push(Box::leak(f.bytes.into_boxed_slice()));
    }
    let _ = REGISTERED.set(Registered { faces: Box::leak(faces.into_boxed_slice()), names, notes: g.notes });
}

/// The faces for Rune's text system: empty (Rune's own Inter) until `register` gave it some.
pub fn registered() -> &'static [&'static [u8]] {
    REGISTERED.get().map_or(&[], |r| r.faces)
}

/// What didn't load at the start, (what, why).
pub fn notes() -> &'static [(String, String)] {
    REGISTERED.get().map_or(&[], |r| &r.notes)
}

/// The family Rune knows `css`'s face by (a rule's font-family, or the face's own name), when one registered.
pub fn family(css: &str) -> Option<&'static str> {
    REGISTERED.get()?.names.get(&css.trim().to_lowercase()).copied()
}

/// Why `css` (a fontFamily's first family) draws in another face, if it does: no face of that name registered.
pub fn missing(css: &str) -> Option<&'static str> {
    let l = css.trim().to_lowercase();
    (!GENERIC.contains(&l.as_str()) && family(&l).is_none()).then_some("a family with no face registered at the start (an @font-face rule or a file in fonts/)")
}

/// The families a stylesheet's @font-face rules name that didn't register at the start: Rune can't add them now.
pub fn late(css: &str) -> Vec<String> {
    blocks(css).filter_map(|b| decl(b, "font-family")).map(|f| unquote(&f)).filter(|f| family(f).is_none() && !f.contains(['$', '`', '{', '+'])).collect()
}

/// Each @font-face rule's body.
fn blocks(css: &str) -> impl Iterator<Item = &str> {
    css.match_indices("@font-face").filter_map(|(i, _)| {
        let rest = &css[i..];
        let open = rest.find('{')?;
        let close = rest[open..].find('}')?;
        Some(&rest[open + 1..open + close])
    })
}

fn rules(css: &str, base: &Path, file: &Path, g: &mut Gathered) {
    for b in blocks(css) {
        // a template's rule (@raea/ui's own FontFace sheet, in every bundle) isn't one of the app's
        let Some(family) = decl(b, "font-family").map(|f| unquote(&f)).filter(|f| !f.contains(['$', '`', '{', '+'])) else { continue };
        let what = format!("@font-face {family} in {}", file.display());
        let srcs = decl(b, "src").unwrap_or_default();
        let urls: Vec<String> = srcs.match_indices("url(").filter_map(|(i, _)| srcs[i + 4..].split_once(')').map(|(u, _)| unquote(u))).collect();
        if urls.is_empty() {
            g.notes.push((what, "no url() (local() faces aren't read: the host has no system fonts)".into()));
            continue;
        }
        let loaded = urls.iter().map(|u| (u, src::resolve(u, Some(base)).and_then(|s| src::bytes(&s).map(|b| (s, b))))).find_map(|(_, r)| r.ok());
        match loaded {
            Some((s, bytes)) => {
                let from = match s {
                    Src::File(p) => p,
                    Src::Url(u) => raea_io_net::cache::fetch(&u).unwrap_or_default(),
                    Src::Bytes(_) => file.to_path_buf(),
                };
                match face(&bytes, vec![family], &from) {
                    Ok(f) => g.faces.push(f),
                    Err(e) => g.notes.push((what, e)),
                }
            }
            None => g.notes.push((what, format!("none of its sources loads: {}", urls.join(", ")))),
        }
    }
}

/// A face from a file's bytes (WOFF and WOFF2 decoded), known by `names` and then its own family name.
fn face(bytes: &[u8], mut names: Vec<String>, from: &Path) -> Result<Face, String> {
    let bytes = match bytes.get(..4) {
        Some(b"wOF2") => wuff::decompress_woff2(bytes).map_err(|e| format!("its WOFF2: {e:?}"))?,
        Some(b"wOFF") => wuff::decompress_woff1(bytes).map_err(|e| format!("its WOFF: {e:?}"))?,
        _ => bytes.to_vec(),
    };
    let face = ttf_parser::Face::parse(&bytes, 0).map_err(|e| format!("not a font Rune reads: {e}"))?;
    let own = [ttf_parser::name_id::TYPOGRAPHIC_FAMILY, ttf_parser::name_id::FAMILY]
        .iter()
        .find_map(|id| face.names().into_iter().find(|n| n.name_id == *id && n.is_unicode()).and_then(|n| n.to_string()))
        .ok_or("no family name in it")?;
    names.push(own);
    Ok(Face { names, bytes, from: from.to_path_buf() })
}

/// A declaration's value in a rule's body.
fn decl(body: &str, prop: &str) -> Option<String> {
    body.split(';').find_map(|d| {
        let (k, v) = d.split_once(':')?;
        (k.trim().eq_ignore_ascii_case(prop)).then(|| v.trim().to_string())
    })
}

fn unquote(s: &str) -> String {
    s.trim().trim_matches(['"', '\'']).trim().to_string()
}

/// `p`'s extension is one of `exts` (lowercase), whatever its case.
fn has_ext(p: &Path, exts: &[&str]) -> bool {
    let e = p.extension().and_then(|e| e.to_str()).unwrap_or("");
    exts.iter().any(|x| e.eq_ignore_ascii_case(x))
}

/// Files under `d` (`depth` folders down at most) that `keep` takes, sorted, skipping dot folders and node_modules.
fn walk(d: &Path, depth: u32, out: &mut Vec<PathBuf>, keep: &dyn Fn(&Path) -> bool) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(d).into_iter().flatten().flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        if p.is_dir() {
            if depth > 0 && !name.starts_with('.') && name != "node_modules" {
                walk(&p, depth - 1, out, keep);
            }
        } else if keep(&p) {
            out.push(p);
        }
    }
}
