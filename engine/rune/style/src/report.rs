//! What the host couldn't map (R2b), reported once per page: the first element it was seen on (by path) and why. A
//! page's report is kept across rebuilds, so a prop that stays unmapped is said once, not every frame.

use std::collections::BTreeMap;

use crate::css::Unmapped;

/// One unmapped prop or value, where it was first seen.
#[derive(Clone, Debug, PartialEq)]
pub struct Seen {
    pub path: String,
    pub value: String,
    pub why: &'static str,
}

/// The page's unmapped props: one entry per (prop, why).
#[derive(Default, Debug)]
pub struct Report {
    seen: BTreeMap<(String, &'static str), Seen>,
    fresh: Vec<String>,
    /// what the SDK marked as having no web form (`data-rune-web-ignored`, T4b), by element path
    web: BTreeMap<String, String>,
}

impl Report {
    /// `u` seen on the element at `path`; a line for the log the first time this (prop, why) is seen.
    pub fn add(&mut self, path: &str, u: Unmapped) {
        let key = (u.prop.clone(), u.why);
        if self.seen.contains_key(&key) {
            return;
        }
        self.fresh.push(format!("[rune] unmapped {}: {} at {} ({})", u.prop, u.value, path, u.why));
        self.seen.insert(key, Seen { path: path.to_string(), value: u.value, why: u.why });
    }

    /// The same, for something that isn't a style prop (a kind, an inline layout the host approximates).
    pub fn note(&mut self, path: &str, what: &str, value: &str, why: &'static str) {
        self.add(path, Unmapped { prop: what.to_string(), value: value.to_string(), why });
    }

    /// Lines not yet logged.
    pub fn take_fresh(&mut self) -> Vec<String> {
        std::mem::take(&mut self.fresh)
    }

    /// Everything seen, by (prop, why).
    pub fn all(&self) -> impl Iterator<Item = (&str, &Seen)> {
        self.seen.iter().map(|((p, _), s)| (p.as_str(), s))
    }

    /// Element `path`'s Rune props that the web target ignores, as @raea/rune named them (T4b).
    pub fn web_ignored(&mut self, path: &str, what: &str) {
        self.web.insert(path.to_string(), what.to_string());
    }

    pub fn web(&self) -> impl Iterator<Item = (&str, &str)> {
        self.web.iter().map(|(p, w)| (p.as_str(), w.as_str()))
    }

    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}
