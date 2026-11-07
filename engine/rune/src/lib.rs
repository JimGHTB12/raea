//! raea-rune-host: applies raea's op wire (core/wire, the Rust half of sdk/ui's wire.ts) to a tree and draws that
//! tree with Rune. docs/rune.feature in raea is the plan.

// the page and its styles, from their crates (raea-rune-page, raea-rune-style), at the paths they had here
pub use raea_rune_page::{css, dump, events, images, input, kit, layout, measure, motion, page, report, runeprops, session, state, tailwind, tree, variants, view};

pub mod js;
pub mod macapp;
pub mod script;
pub mod shot;
pub mod video;
pub mod window;

/// The op protocol version this host speaks: core/wire's, so a wire bump reaches this crate as a build error.
pub const OPS_VERSION: u32 = raea_core_wire::OPS_VERSION;

#[cfg(test)]
mod tests {
    #[test]
    fn links_rune_and_the_wire() {
        // Both dependencies resolve and link: Rune's view API and the wire's version.
        let _ = rune::col();
        assert_eq!(super::OPS_VERSION, 1);
    }
}
