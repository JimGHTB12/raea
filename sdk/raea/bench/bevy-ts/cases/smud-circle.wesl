// plugin-smud's shape: a circle filling the shape's bounds
fn sdf(input: smud::SdfInput) -> f32 {
    return smud::sd_circle(input.pos, min(input.bounds.x, input.bounds.y));
}
