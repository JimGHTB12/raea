// shadow_only.rs: a mesh that casts shadows without being drawn. The shadow passes draw it depth only (it's opaque);
// here, in the camera's colour pass, every pixel is thrown away.

@fragment
fn fragment() -> @location(0) vec4<f32> {
    discard;
    return vec4<f32>(0.0);
}
