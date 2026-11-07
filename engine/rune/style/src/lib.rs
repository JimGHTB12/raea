//! raea-rune-style: an element's CSS (inline styles and Tailwind classes, Tailwind 4.3.3's theme) resolved and mapped
//! to Rune's builder, with every prop that didn't map reported (docs/rune.feature R2, docs/targets.feature T3).

pub mod css;
pub mod motion;
pub mod report;
pub mod runeprops;
pub mod tailwind;
pub mod variants;
