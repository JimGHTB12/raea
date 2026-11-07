//! raea-app-targets: the app targets of docs/targets.feature. An app is React TSX on @raea/ui's wire, hosted by Blitz on
//! Bevy (the web target, the default) or by Rune (the native target); the app's own file says which (T0a), and
//! `raea app run|check|bundle` work on either (T0b: verbs.rs).

pub mod brp;
pub mod config;
pub mod literal;
pub mod macapp;
pub mod native;
pub mod sign;
pub mod steps;
pub mod verbs;
pub mod web;
pub mod webcheck;

pub use config::{AppConfig, Target, read};
pub use verbs::command;
