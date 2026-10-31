// raea: CSS Text 4 § 6.4 `text-autospace`: 1/8 em of space where an ideograph meets a letter or a decimal digit
// (UTR #59's Wide against Narrow, parley_engine raea_autospace_class), as Chrome inserts it (its initial value is
// `normal`). The space follows the logically earlier atom, so that it falls between the two: in a right-to-left run it
// goes on the later one, and after one on the run's first atom (its right edge). It crosses an inline element's edges
// but not an atomic inline. Both sides' styles must ask for it, and it takes the later atom's font size. The
// line breaker adds it to the atom's advance (LayoutData::tab_extras), doesn't count it toward the line's fit and
// drops it at the end of a line that wraps after the atom (line_break.rs).

use alloc::collections::BTreeMap;

use parley_engine::shape::Character;
use parley_engine::{AutospaceClass, raea_autospace_class};

use crate::layout::data::{LayoutData, LayoutItemKind};
use crate::style::{Brush, TextAutospace};
use crate::{InlineBox, InlineBoxKind};

/// The space after each atom that gets one, by the atom's first shaped cluster.
pub(crate) fn extras<B: Brush>(data: &LayoutData<B>, text: &str) -> BTreeMap<u32, f32> {
    let mut out = BTreeMap::new();
    if !parley_engine::raea_autospace() || data.styles.iter().all(|s| s.text_autospace == TextAutospace::default()) {
        return out;
    }
    // UTR #59's Conditional is Narrow in a Chinese language and Other elsewhere
    let class = |c: &Character| {
        let class = text
            .get(c.text_byte_range())
            .and_then(|s| s.chars().next())
            .map_or(AutospaceClass::Other, raea_autospace_class);
        match class {
            AutospaceClass::Conditional if chinese(&data.styles[c.style_index as usize].locale) => {
                AutospaceClass::Letter
            }
            AutospaceClass::Conditional => AutospaceClass::Other,
            class => class,
        }
    };
    // the atom before: its class at its end, its style, its first cluster, and whether its run is right-to-left
    let mut prev: Option<(AutospaceClass, u16, u32, bool)> = None;
    // the first cluster of the right-to-left atoms just before, which ends up at their right edge
    let mut rtl_first: Option<u32> = None;
    for item in &data.items {
        match item.kind {
            LayoutItemKind::InlineBox => {
                let b = &data.inline_boxes[item.index].inline_box;
                if b.kind == InlineBoxKind::InFlow && b.id & InlineBox::EDGE == 0 {
                    prev = None;
                }
            }
            LayoutItemKind::TextRun => {
                let run = &data.shaped_text.runs()[item.index];
                let rtl = run.bidi_level.is_rtl();
                for atom in data.shaped_text.run_slice(item.index as u32).atoms_start() {
                    let chars = atom.characters();
                    let first = class(&chars[0]);
                    // a mark takes its base's class
                    let last = chars.iter().rev().find(|c| c.grapheme_start).map_or(first, class);
                    let style = chars[0].style_index;
                    let key = atom.shaped_clusters_range().start;
                    if let Some((before, before_style, before_key, before_rtl)) = prev
                        && let Some(numeric) = pair(before, first)
                    {
                        let wants = |s: u16| {
                            let a = data.styles[s as usize].text_autospace;
                            if numeric { a.ideograph_numeric } else { a.ideograph_alpha }
                        };
                        if wants(before_style) && wants(style) {
                            let at = match (before_rtl, rtl) {
                                (true, true) => key,
                                (true, false) => rtl_first.unwrap_or(before_key),
                                _ => before_key,
                            };
                            *out.entry(at).or_insert(0.) += run.font_size / 8.;
                        }
                    }
                    rtl_first = match rtl {
                        true => rtl_first.filter(|_| prev.is_some_and(|p| p.3)).or(Some(key)),
                        false => None,
                    };
                    prev = Some((last, style, key, rtl));
                }
            }
        }
    }
    out
}

/// Whether `locale` is Chinese: the macrolanguage zh or a language it encompasses (Blink's IsMacrolanguageChinese).
fn chinese(locale: &Option<fontique::Language>) -> bool {
    const ZH: [&str; 17] = [
        "zh", "cdo", "cjy", "cmn", "cnp", "cpx", "csp", "czh", "czo", "gan", "hak", "hsn", "lzh", "mnp", "nan", "wuu",
        "yue",
    ];
    locale.as_ref().is_some_and(|l| ZH.contains(&l.language()))
}

/// Whether `a` then `b` is an ideograph against a letter (`Some(false)`) or a digit (`Some(true)`).
fn pair(a: AutospaceClass, b: AutospaceClass) -> Option<bool> {
    use AutospaceClass::*;
    match (a, b) {
        (Ideograph, Letter) | (Letter, Ideograph) => Some(false),
        (Ideograph, Numeral) | (Numeral, Ideograph) => Some(true),
        _ => None,
    }
}
