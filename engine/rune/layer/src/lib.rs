//! Rune layers in a page (docs/targets.feature T6c, per layer): any element marked
//! `data-raea-renderer="rune"` (sdk/rune's `<RuneLayer>`) keeps its place in the page, drawn by Blitz as a slot, and
//! everything under it is drawn by Rune instead, a tree of its own on Bevy's device (raea-rune-bevy) shown in the slot.
//! The wire doesn't change: the app's JS sends one op stream, and [`Router`] splits it here, so the page applies only
//! what is Blitz's and each layer's Rune page (raea-rune-page) applies its own, its children under its root. A page
//! that doesn't route (or a host without Rune) draws the marked element as a plain element: the layer degrades to Blitz.

pub mod route;

pub use route::{MARK, Routed, Router, creates_slot};
