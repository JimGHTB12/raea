@group(0) @binding(0) var src: texture_2d<f32>;
@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
    let p = vec2f(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4f(p * 2.0 - 1.0, 0.0, 1.0);
}
fn linear(c: vec3f) -> vec3f {
    return select(pow((c + 0.055) / 1.055, vec3f(2.4)), c / 12.92, c <= vec3f(0.04045));
}
// premultiplied in the encoded values (as bevy_ui's page holds them); the target's sRGB view encodes what's written
@fragment fn fs(@builtin(position) p: vec4f) -> @location(0) vec4f {
    let c = textureLoad(src, vec2i(p.xy), 0);
    return vec4f(linear(c.rgb * c.a), c.a);
}
@fragment fn fs_straight(@builtin(position) p: vec4f) -> @location(0) vec4f {
    let c = textureLoad(src, vec2i(p.xy), 0);
    return vec4f(linear(c.rgb), c.a);
}
fn encoded(c: vec3f) -> vec3f {
    return select(1.055 * pow(c, vec3f(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3f(0.0031308));
}
// a camera's picture (premultiplied in its stored, sRGB-encoded values) made straight as raea's render makes its
// layered 3D (sdk/react motion.rs scene_straight): its alpha the largest of its four values, so light left in the
// clear (a sky, a glow) keeps; read decoded through an sRGB view, written through one
@fragment fn fs_unpremul(@builtin(position) p: vec4f) -> @location(0) vec4f {
    let c = textureLoad(src, vec2i(p.xy), 0);
    let e = round(encoded(c.rgb) * 255.0);
    let a = max(max(e.r, e.g), max(e.b, round(c.a * 255.0)));
    if c.a >= 1.0 || a == 0.0 {
        return c;
    }
    return vec4f(linear(min(floor((e * 255.0 + floor(a / 2.0)) / a), vec3f(255.0)) / 255.0), a / 255.0);
}
// the 3D under a whole page (live play's, which a window would show): its colour as stored, opaque, as a window
// presents it whatever alpha a material wrote (a Material's own shader may write any)
@fragment fn fs_opaque(@builtin(position) p: vec4f) -> @location(0) vec4f {
    return vec4f(textureLoad(src, vec2i(p.xy), 0).rgb, 1.0);
}
