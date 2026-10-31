#![cfg(test)]
use super::key_source;

fn args(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| s.to_string()).collect()
}

#[test]
fn one_source_of_keys() {
    assert!(key_source(&args(&["play", "g.tsx", "--input"])).is_ok());
    assert!(key_source(&args(&["play", "g.tsx", "--input-script", "s.json"])).is_ok());
    let both = key_source(&args(&["play", "g.tsx", "--input", "--input-script", "s.json"])).unwrap_err();
    assert!(both.contains("give one"), "{both}");
}
