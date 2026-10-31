// Copyright 2025 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Parley Engine provides low level APIs for shaping paragraphs of text.
//!
//! ## Usage
//!
//! Use [`Analyzer`], [`Analysis`], [`Shaper`] and [`ShapedText`] to shape a paragraph of text into
//! glyphs. Correct reshaping of lines is in progress; in the meantime you can break text at
//! [`Atom`] or [`ShapedCluster`][crate::shape::ShapedCluster] boundaries.
//!
//! Text analysis is performed before shaping, and the same source string must be passed to each
//! stage.
//!
//! Higher-level users may prefer using [`parley`][parley], which uses this crate and implements
//! layout and styling.
//!
//! ```rust,no_run
//! # // We only compile this doctest because we don't have a font available.
//! # use parley_engine::{Analysis, AnalysisOptions, Analyzer, FontInstanceRef, FontSelector, ShapedText, ShapeOptions, Shaper};
//! # use parley_engine::shape::CharCluster;
//! # use parley_engine::itemize::{Item, Segment};
//! #
//! # struct NoFont;
//! # impl FontSelector for NoFont {
//! #     fn select_font(
//! #         &mut self,
//! #         _segment: &Segment,
//! #         _options: &ShapeOptions<'_>,
//! #         _cluster: &mut CharCluster,
//! #     ) -> Option<FontInstanceRef<'_>> {
//! #         unimplemented!()
//! #     }
//! # }
//! #
//! # let select_font = NoFont;
//! let mut analysis = Analysis::default();
//! let mut analyzer = Analyzer::default();
//! let mut shaped_text = ShapedText::default();
//! let mut shaper = Shaper::default();
//!
//! let text = "The quick brown ثعلب jumps over the lazy dog.";
//! let char_count = text.chars().count();
//! let char_style_indices = vec![0; char_count];
//!
//! analyzer.analyze(text, &AnalysisOptions::default(), &mut analysis);
//! shaper.shape_text(
//!     text,
//!     &analysis,
//!     &char_style_indices,
//!     [Item {
//!         char_end: char_count.try_into().unwrap(),
//!         options: ShapeOptions {
//!             font_size: 16.0,
//!             language: None,
//!             features: &[],
//!             variations: &[],
//!         },
//!      }],
//!      select_font, // Selects fonts covering each cluster.
//!      &mut shaped_text,
//! );
//!
//! for (run_idx, run) in shaped_text.runs().iter().enumerate() {
//!     let slice = shaped_text.run_slice(run_idx as u32);
//!     // You can, for example, measure grapheme advances for hit-testing or
//!     // placing carets.
//!     for atom in slice.atoms_start() {
//!         for grapheme in atom.graphemes_start() {
//!             std::dbg!(grapheme);
//!         }
//!     }
//!
//!     // Or get glyphs for rendering (for simplicity, this iterates clusters in
//!     // logical order, but for rendering you'd want to reorder runs and clusters
//!     // according to their `run.bidi_level`).
//!     for cluster in slice.shaped_clusters_range() {
//!         for glyph in slice.shaped_cluster_glyphs(cluster) {
//!             std::dbg!(glyph);
//!         }
//!     }
//! }
//! ```
//!
//! ## Features
//!
//! - `std` (enabled by default): This is currently unused and is provided for forward compatibility.
//! - `bytemuck`: Forwarded to [`parlance`], which then implements traits from `bytemuck` on its
//!   types.
//!
//! [parley]: https://docs.rs/parley

// LINEBENDER LINT SET - lib.rs - v3
// See https://linebender.org/wiki/canonical-lints/
// These lints shouldn't apply to examples or tests.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
// These lints shouldn't apply to examples.
#![warn(clippy::print_stdout, clippy::print_stderr)]
// Targeting e.g. 32-bit means structs containing usize can give false positives for 64-bit.
#![cfg_attr(target_pointer_width = "64", warn(clippy::trivially_copy_pass_by_ref))]
// END LINEBENDER LINT SET
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]
#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

mod analysis;
mod analyzer;
mod bidi;
pub mod break_overrides;
mod glyph;
pub mod itemize;
mod lru_cache;
pub mod shape;

pub use linebender_resource_handle::FontData;
pub use parlance::{BaseDirection, NormalizedCoord};

pub use analysis::{Analysis, AnalysisDataSources, CharInfo};
pub use analyzer::{AnalysisOptions, Analyzer, LineBreakConfig};
pub use glyph::Glyph;
pub use shape::atom::{Atom, Atoms, Grapheme, Graphemes, ShapedClusterGlyphs, ShapedSlice};
pub use shape::shaped_text::{FontMetrics, ShapedRun, ShapedText};

/// raea: the paragraph-at-a-time bidi resolution (bidi.rs) and UAX #9 L1 for a line's trailing whitespace (parley's
/// line_break.rs), on by default (ui-parity full 252/255 with it, the calib cases only); RAEA_PAGE_BIDI=0 turns it off.
#[doc(hidden)]
pub fn raea_bidi() -> bool {
    #[cfg(feature = "std")]
    {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ON.get_or_init(|| std::env::var("RAEA_PAGE_BIDI").as_deref() != Ok("0"))
    }
    #[cfg(not(feature = "std"))]
    true
}
/// raea: collapsible spaces at a line's start and end are removed rather than hung (parley line_break.rs
/// remove_edge_spaces), on by default; RAEA_PAGE_EDGE_SPACES=0 turns it off.
#[doc(hidden)]
pub fn raea_edge_spaces() -> bool {
    #[cfg(feature = "std")]
    {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ON.get_or_init(|| std::env::var("RAEA_PAGE_EDGE_SPACES").as_deref() != Ok("0"))
    }
    #[cfg(not(feature = "std"))]
    true
}
/// raea: CSS Text 4 `text-autospace` (parley's layout/autospace.rs), on by default; RAEA_PAGE_AUTOSPACE=0 turns it off.
#[doc(hidden)]
pub fn raea_autospace() -> bool {
    #[cfg(feature = "std")]
    {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ON.get_or_init(|| std::env::var("RAEA_PAGE_AUTOSPACE").as_deref() != Ok("0"))
    }
    #[cfg(not(feature = "std"))]
    true
}

/// raea: a soft wrap opportunity between nowrap text and wrapping text after it is taken (CSS Text 3 § 5: the nearest
/// common ancestor's white-space controls it, as Chrome breaks `<span nowrap>国</span>国`), on by default;
/// RAEA_PAGE_WRAP_AFTER_NOWRAP=0 keeps the earlier text's nowrap in control.
#[doc(hidden)]
pub fn raea_wrap_after_nowrap() -> bool {
    #[cfg(feature = "std")]
    {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ON.get_or_init(|| std::env::var("RAEA_PAGE_WRAP_AFTER_NOWRAP").as_deref() != Ok("0"))
    }
    #[cfg(not(feature = "std"))]
    true
}

/// raea: a character's East_Asian_Spacing (UTR #59 § 4, as Blink's text-autospace reads it since it moved off its own
/// classes), with Narrow split into letters and digits for `ideograph-alpha` and `ideograph-numeric`.
#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AutospaceClass {
    /// Wide: East Asian script characters outside punctuation and symbols.
    Ideograph,
    /// Narrow: a letter or mark that isn't wide.
    Letter,
    /// Narrow: a decimal digit that isn't wide.
    Numeral,
    /// Conditional: other punctuation such as `%` or `!`; Narrow (a letter) in Chinese, Other elsewhere.
    Conditional,
    /// Other: spaces, symbols, most punctuation, wide forms.
    Other,
}

/// raea: see [`AutospaceClass`].
#[doc(hidden)]
pub fn raea_autospace_class(ch: char) -> AutospaceClass {
    use icu_properties::CodePointMapData;
    use icu_properties::props::{EastAsianWidth as Eaw, GeneralCategory as Gc, GeneralCategoryGroup as Gcg, Script};
    use icu_properties::script::ScriptWithExtensions;
    const EAST_ASIAN: [Script; 9] = [
        Script::Bopomofo,
        Script::Han,
        Script::Hangul,
        Script::Hiragana,
        Script::Katakana,
        Script::KhitanSmallScript,
        Script::Nushu,
        Script::Tangut,
        Script::Yi,
    ];
    let gc = CodePointMapData::<Gc>::new().get(ch);
    let eaw = CodePointMapData::<Eaw>::new().get(ch);
    let sc = CodePointMapData::<Script>::new().get(ch);
    let scx = ScriptWithExtensions::new();
    let wide = (EAST_ASIAN.contains(&sc)
        || (!matches!(eaw, Eaw::Neutral | Eaw::Narrow) && EAST_ASIAN.iter().any(|&s| scx.has_script(ch, s))))
        && eaw != Eaw::Halfwidth
        && !Gcg::Punctuation.contains(gc)
        && gc != Gc::OtherNumber
        && !(Gcg::Symbol.contains(gc) && gc != Gc::ModifierSymbol);
    if wide || ch == '\u{3013}' {
        return AutospaceClass::Ideograph;
    }
    let wide_form = matches!(eaw, Eaw::Fullwidth | Eaw::Halfwidth | Eaw::Wide);
    if gc == Gc::OtherPunctuation
        && !wide_form
        && !matches!(ch, '"' | '\'' | '*' | '/' | '\u{B7}' | '\u{2020}' | '\u{2021}' | '\u{2026}')
    {
        return AutospaceClass::Conditional;
    }
    if wide_form {
        return AutospaceClass::Other;
    }
    if gc == Gc::DecimalNumber {
        return AutospaceClass::Numeral;
    }
    if Gcg::Letter.contains(gc) || Gcg::Mark.contains(gc) {
        return AutospaceClass::Letter;
    }
    AutospaceClass::Other
}
/// raea: the glyphs (id, advance) that draw `text` in `font` at `size`, by the font's character map with no shaping (a
/// hyphen at a line's end, parley's Line::hyphen); `None` when the font lacks one of its characters.
#[doc(hidden)]
pub fn raea_plain_glyphs(
    font: &FontInstance,
    size: f32,
    coords: &[NormalizedCoord],
    text: &str,
) -> Option<alloc::vec::Vec<(u32, f32)>> {
    use skrifa::MetadataProvider as _;
    let font_ref = skrifa::FontRef::from_index(font.font.data.as_ref(), font.font.index).ok()?;
    let charmap = font_ref.charmap();
    let coords: alloc::vec::Vec<skrifa::instance::NormalizedCoord> =
        coords.iter().map(|c| skrifa::instance::NormalizedCoord::from_bits(c.to_bits())).collect();
    let metrics = font_ref.glyph_metrics(skrifa::instance::Size::new(size), coords.as_slice());
    text.chars()
        .map(|ch| {
            let id = charmap.map(ch)?;
            Some((id.to_u32(), metrics.advance_width(id)?))
        })
        .collect()
}

pub use shape::shaper::{FontInstance, FontInstanceRef, FontSelector, ShapeOptions, Shaper};
