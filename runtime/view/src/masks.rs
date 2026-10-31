//! Clips' masks (a row's `mask` column, a raea_core_shape space-time solid) as the coverage K's mask M reads.

use raea_core_program::ir::V;

/// The masked clips' coverage at frame `f`, one byte a slot in a word a pixel (kgpu's mask M), marking those slots
/// masked (on = 2). `masks[s]` is slot s's `mask` expression. None when no slot has one.
pub fn slot_masks(masks: &[Option<String>], slots: &mut [raea_core_k::Slot], f: i64, w: usize, h: usize, frames: i64) -> Result<Option<Vec<u32>>, String> {
    use raea_core_shape::{mask_coverage, mask_solid};
    if masks.iter().all(Option::is_none) {
        return Ok(None);
    }
    let mut words = vec![0u32; w * h];
    for (s, m) in masks.iter().enumerate().take(4) {
        let Some(expr) = m else { continue };
        let solid = mask_solid(expr, w as f64, h as f64, frames as f64).map_err(|e| format!("mask {expr}: {e}"))?;
        for (wd, c) in words.iter_mut().zip(mask_coverage(&solid, f, w, h)) {
            *wd |= (c as u32) << (8 * s);
        }
        slots[s].on = 2;
    }
    Ok(Some(words))
}

/// A row's mask expression, if it has one.
pub fn mask_of(v: &std::collections::BTreeMap<String, V>) -> Option<String> {
    match v.get("mask") {
        Some(V::S(s)) if !s.trim().is_empty() => Some(s.clone()),
        _ => None,
    }
}
