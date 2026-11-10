// Ink outlines (outline.rs): the frame as drawn, with a line where the depth steps or the surface turns.
import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;

struct InkOutline {
    color: vec4<f32>,
    depth: f32,
    normal: f32,
    width: f32,
    far: f32,
    skip: u32,
};

@group(0) @binding(0) var screen: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var<uniform> p: InkOutline;
@group(0) @binding(3) var depth_tex: texture_depth_2d;
@group(0) @binding(4) var normal_tex: texture_2d<f32>;

// distance from the camera, up to a factor (Bevy's reverse-Z: depth is near / distance; nothing drawn is 0, far away)
fn dist(c: vec2<i32>) -> f32 {
    let s = vec2<i32>(textureDimensions(depth_tex));
    let d = textureLoad(depth_tex, clamp(c, vec2<i32>(0), s - vec2<i32>(1)), 0);
    return 1.0 / max(d, 1e-7);
}

fn normal_at(c: vec2<i32>) -> vec3<f32> {
    let s = vec2<i32>(textureDimensions(normal_tex));
    return normalize(textureLoad(normal_tex, clamp(c, vec2<i32>(0), s - vec2<i32>(1)), 0).xyz * 2.0 - 1.0);
}

// a pixel in a composition pass the line skips (compose.rs: the pass is in the normal's alpha)
fn skipped(c: vec2<i32>) -> bool {
    if p.skip == 0u {
        return false;
    }
    let s = vec2<i32>(textureDimensions(normal_tex));
    let k = (u32(round(textureLoad(normal_tex, clamp(c, vec2<i32>(0), s - vec2<i32>(1)), 0).a * 3.0)) + 1u) % 4u;
    return ((p.skip >> k) & 1u) == 1u;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let here = textureSample(screen, screen_sampler, in.uv);
    let c = vec2<i32>(floor(in.position.xy));
    if skipped(c) {
        return here;
    }
    let z = dist(c);
    let n = normal_at(c);
    // past `far` no line: they fade out over its last third
    let fade = select(1.0, 1.0 - smoothstep(p.far * 0.66, p.far, z), p.far > 0.0);
    if fade <= 0.0 {
        return here;
    }
    let ink = vec4<f32>(mix(here.rgb, p.color.rgb, fade), here.a);
    if p.width > 1.5 {
        return wide(c, z, n, here, ink);
    }
    let offs = array<vec2<i32>, 4>(vec2<i32>(1, 0), vec2<i32>(-1, 0), vec2<i32>(0, 1), vec2<i32>(0, -1));
    for (var k = 0; k < 4; k++) {
        let o = c + offs[k];
        // a silhouette: the line on the nearer side, so a figure is ringed by it and not what's behind it
        if dist(o) - z > p.depth * z {
            return ink;
        }
        // a crease: one side of it (the right and lower neighbours only), so the line is one pixel wide
        if p.normal > 0.0 && k % 2 == 0 && !skipped(o) && dist(o) < 1e6 && z < 1e6 && 1.0 - dot(n, normal_at(o)) > p.normal {
            return ink;
        }
    }
    return here;
}

// A line `width` pixels wide and even (a cartoon's): a silhouette wherever something within the width's radius is farther by the threshold (so the line lies on the nearer side, ringing the figure), a crease wherever the
// surface within the radius on one side (right and down) turns by more than `normal`. 16 directions at three radii, in
// one loop: FXC, DX12's shader compiler on Windows, can't unroll two nested loops with a return inside them, and every
// 3D scene builds this pipeline, so its error stopped every 3D render on DX12 (#66).
fn wide(c: vec2<i32>, z: f32, n: vec3<f32>, here: vec4<f32>, ink: vec4<f32>) -> vec4<f32> {
    let r = p.width;
    for (var i = 0u; i < 48u; i++) {
        let a = f32(i / 3u) * 0.39269908;
        let d = vec2<f32>(cos(a), sin(a));
        let o = c + vec2<i32>(round(d * r * f32(i % 3u + 1u) / 3.0));
        let zo = dist(o);
        if zo - z > p.depth * z {
            return ink;
        }
        if p.normal > 0.0 && (d.x + d.y) > 0.0 && !skipped(o) && zo < 1e6 && z < 1e6 && abs(zo - z) < p.depth * z && 1.0 - dot(n, normal_at(o)) > p.normal {
            return ink;
        }
    }
    return here;
}
