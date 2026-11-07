//! Tailwind's variants (docs/targets.feature T3b): `hover:`, `focus:`, `focus-visible:`, `active:`, `enabled:`,
//! `disabled:`, `sm:` to `2xl:` (the window's width against Tailwind 4.3.3's breakpoints) and `dark:` (the system's
//! appearance, Tailwind 4's default media strategy), chained in any order. Each holds or not for an element's `State`,
//! and a utility under variants lands where Tailwind's CSS puts it: after the plain utilities, ordered by its variants
//! as Tailwind 4.3.3 orders them (a bit per variant in the order below, a chain being the sum, so `md:hover:` comes
//! after `md:` and before `lg:`, and `dark:hover:` after `dark:`). Any other variant (`group-hover:`, `first:`,
//! `placeholder:`, an arbitrary one) leaves its utility unknown, to be reported.

use crate::tailwind::{px, var};

/// What an element's variants test: its own state and the page's.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct State {
    /// the pointer is over it or something inside it (CSS's :hover)
    pub hover: bool,
    /// a press began on it or inside it and hasn't been released (:active)
    pub active: bool,
    /// it has the page's focus (:focus)
    pub focus: bool,
    /// its focus is one the page shows (:focus-visible: from a key, or a text field however it was focused)
    pub focus_visible: bool,
    /// a form control (button, input, select, textarea): only these are :enabled or :disabled
    pub control: bool,
    /// a disabled control
    pub disabled: bool,
    /// the system's appearance is dark (prefers-color-scheme)
    pub dark: bool,
    /// the window's width in points (the viewport's, for min-width breakpoints)
    pub width: f32,
}

const BREAKPOINTS: [&str; 5] = ["sm", "md", "lg", "xl", "2xl"];

/// Variant `v`'s bit in Tailwind 4.3.3's order, and whether it holds for `s`; None for a variant this doesn't map.
fn variant(v: &str, s: &State) -> Option<(u32, bool)> {
    Some(match v {
        "hover" => (0, s.hover),
        "focus" => (1, s.focus),
        "focus-visible" => (2, s.focus && s.focus_visible),
        "active" => (3, s.active),
        "enabled" => (4, s.control && !s.disabled),
        "disabled" => (5, s.control && s.disabled),
        "dark" => (11, s.dark),
        _ => {
            let i = BREAKPOINTS.iter().position(|b| *b == v)?;
            let min = px(var(&format!("breakpoint-{v}"))?)?;
            (6 + i as u32, f64::from(s.width) >= min)
        }
    })
}

/// A class as its utility, its variants' place in the cascade (0 for none) and whether they all hold for `s`; None when
/// a variant isn't one this maps.
pub fn split<'a>(class: &'a str, s: &State) -> Option<(&'a str, u64, bool)> {
    // a colon inside an arbitrary value's brackets (`bg-[url(a:b)]`) isn't a variant's
    let mut parts = Vec::new();
    let (mut depth, mut start) = (0i32, 0);
    for (i, c) in class.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => depth -= 1,
            ':' if depth == 0 => {
                parts.push(&class[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    let util = &class[start..];
    let (mut key, mut holds) = (0u64, true);
    for v in parts {
        let (bit, on) = variant(v, s)?;
        key |= 1 << bit;
        holds &= on;
    }
    Some((util, key, holds))
}

/// The variants this maps, for the README's table.
pub const MAPPED: [&str; 12] = ["hover", "focus", "focus-visible", "active", "enabled", "disabled", "sm", "md", "lg", "xl", "2xl", "dark"];
