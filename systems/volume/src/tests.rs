use super::*;

fn cloud() -> Spec {
    Spec { res: 24, puffs: vec![[0.0, 3.0, 0.0, 2.0], [1.5, 3.5, 0.5, 1.5]], ..Spec::default() }
}

/// a time reached backward, from a mark, is the time reached forward from the start, to the bit
#[test]
fn back_from_a_mark_is_forward_from_the_start() {
    let size = Vec3::new(8.0, 6.0, 6.0);
    let mut a = Sim::new(cloud(), size);
    a.run_to(3.0, size);
    assert!(a.marks.len() >= 3, "marks kept going forward: {}", a.marks.len());
    a.run_to(1.6, size);
    assert_eq!(a.step, 48);
    let mut b = Sim::new(cloud(), size);
    b.run_to(1.6, size);
    assert!(a.d == b.d && a.u == b.u && a.v == b.v && a.w == b.w && a.t == b.t, "back from a mark differs from forward from the start");
    // and on from there, the same again
    a.run_to(2.2, size);
    b.run_to(2.2, size);
    assert!(a.d == b.d && a.t == b.t);
}

/// past the budget the marks thin out, the spacing doubling, and still start at step 0
#[test]
fn marks_thin_out_past_their_budget() {
    let size = Vec3::new(8.0, 6.0, 6.0);
    let mut a = Sim::new(cloud(), size);
    let bytes = a.d.len() * 5 * 4;
    let cap = MARK_BYTES / bytes;
    a.run_to(((cap as u32 + 4) * CHECK) as f32 / RATE, size);
    assert!(a.marks.len() <= cap && a.every > CHECK && a.marks[0].step == 0);
    assert!(a.marks.iter().all(|m| m.step % a.every == 0));
}

fn row(shadow: Option<f64>) -> BTreeMap<String, V> {
    let mut v = BTreeMap::new();
    v.insert("scale_x".to_string(), V::N(8.0));
    v.insert("scale_y".to_string(), V::N(4.0));
    v.insert("scale_z".to_string(), V::N(4.0));
    if let Some(s) = shadow {
        v.insert("shadow".to_string(), V::N(s));
    }
    v
}

#[test]
fn a_volume_casts_as_drawn_unless_told_otherwise() {
    assert_eq!(shadow_of(&row(None)), 1.0);
    assert_eq!(shadow_of(&row(Some(0.5))), 0.5);
    assert_eq!(shadow_of(&row(Some(0.0))), 0.0);
    // kept to what the shader's made for
    assert_eq!(shadow_of(&row(Some(9.0))), 4.0);
    assert_eq!(shadow_of(&row(Some(-1.0))), 0.0);
    let s = Spec::default();
    assert_eq!(params(&row(Some(0.7)), &s, 0.0, Vec3::Y).shadow, 0.7);
}

/// The field the shadow pass reads (density and light) at a time is the same however that time is reached: a
/// frame alone equals that frame in play.
#[test]
fn the_field_at_a_time_is_the_same_from_the_start_or_from_before() {
    let spec = Spec { res: 16, puffs: vec![[0.0, 2.0, 0.0, 1.5], [2.0, 2.2, 0.5, 1.2]], turbulence: 0.6, pressure_iters: 8, ..Spec::default() };
    let size = Vec3::new(8.0, 4.0, 4.0);
    let mut a = Sim::new(spec.clone(), size);
    a.run_to(0.5, size);
    a.run_to(1.0, size);
    let mut b = Sim::new(spec, size);
    b.run_to(1.0, size);
    let sun = Vec3::new(0.3, 0.6, -0.7);
    assert_eq!(a.texels(sun), b.texels(sun));
    // and from later, back
    a.run_to(1.5, size);
    a.run_to(1.0, size);
    assert_eq!(a.texels(sun), b.texels(sun));
}
