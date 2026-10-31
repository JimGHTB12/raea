// The SVG primitives anyrender's structs carry too little of (primitives.rs): appended to passes.wgsl, the same P.
// Values premultiplied unless said; p.m[2..4] hold, where a pass needs it, the texel -> user space affine.

fn straight(c: vec4f) -> vec4f {
    return select(vec4f(0.0), vec4f(c.rgb / c.a, c.a), c.a > 0.0);
}

fn premul(c: vec4f) -> vec4f {
    let k = clamp(c, vec4f(0.0), vec4f(1.0));
    return vec4f(k.rgb * k.a, k.a);
}

// --- feBlend: src (in) over other (in2), mixed by p.a.x (peniko's Mix order) ---

fn lum(c: vec3f) -> f32 { return dot(c, vec3f(0.3, 0.59, 0.11)); }

fn clip_color(c: vec3f) -> vec3f {
    let l = lum(c);
    let n = min(c.r, min(c.g, c.b));
    let x = max(c.r, max(c.g, c.b));
    var o = c;
    if n < 0.0 { o = l + (o - l) * l / (l - n); }
    if x > 1.0 { o = l + (o - l) * (1.0 - l) / (x - l); }
    return o;
}

fn set_lum(c: vec3f, l: f32) -> vec3f { return clip_color(c + (l - lum(c))); }

fn sat(c: vec3f) -> f32 { return max(c.r, max(c.g, c.b)) - min(c.r, min(c.g, c.b)); }

fn set_sat(c: vec3f, s: f32) -> vec3f {
    let n = min(c.r, min(c.g, c.b));
    let x = max(c.r, max(c.g, c.b));
    if x <= n { return vec3f(0.0); }
    return (c - n) * s / (x - n);
}

fn blend_soft(b: f32, s: f32) -> f32 {
    if s <= 0.5 { return b - (1.0 - 2.0 * s) * b * (1.0 - b); }
    let d = select(sqrt(b), ((16.0 * b - 12.0) * b + 4.0) * b, b <= 0.25);
    return b + (2.0 * s - 1.0) * (d - b);
}

fn blend_hard(b: f32, s: f32) -> f32 {
    if s <= 0.5 { return b * 2.0 * s; }
    let t = 2.0 * s - 1.0;
    return b + t - b * t;
}

fn blend_dodge(b: f32, s: f32) -> f32 {
    if b <= 0.0 { return 0.0; }
    if s >= 1.0 { return 1.0; }
    return min(1.0, b / (1.0 - s));
}

fn blend_burn(b: f32, s: f32) -> f32 {
    if b >= 1.0 { return 1.0; }
    if s <= 0.0 { return 0.0; }
    return 1.0 - min(1.0, (1.0 - b) / s);
}

// B(cb, cs): backdrop and source, straight
fn mixed(m: i32, b: vec3f, s: vec3f) -> vec3f {
    switch m {
        case 1: { return b * s; }
        case 2: { return b + s - b * s; }
        case 3: { return vec3f(blend_hard(s.r, b.r), blend_hard(s.g, b.g), blend_hard(s.b, b.b)); }
        case 4: { return min(b, s); }
        case 5: { return max(b, s); }
        case 6: { return vec3f(blend_dodge(b.r, s.r), blend_dodge(b.g, s.g), blend_dodge(b.b, s.b)); }
        case 7: { return vec3f(blend_burn(b.r, s.r), blend_burn(b.g, s.g), blend_burn(b.b, s.b)); }
        case 8: { return vec3f(blend_hard(b.r, s.r), blend_hard(b.g, s.g), blend_hard(b.b, s.b)); }
        case 9: { return vec3f(blend_soft(b.r, s.r), blend_soft(b.g, s.g), blend_soft(b.b, s.b)); }
        case 10: { return abs(b - s); }
        case 11: { return b + s - 2.0 * b * s; }
        case 12: { return set_lum(set_sat(s, sat(b)), lum(b)); }
        case 13: { return set_lum(set_sat(b, sat(s)), lum(b)); }
        case 14: { return set_lum(s, lum(b)); }
        case 15: { return set_lum(b, lum(s)); }
        default: { return s; }
    }
}

@fragment fn feblend(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let s = at(src, vec2i(f.xy));
    let b = at(other, vec2i(f.xy));
    let m = mixed(i32(p.a.x), straight(b).rgb, straight(s).rgb);
    let rgb = (1.0 - b.a) * s.rgb + (1.0 - s.a) * b.rgb + s.a * b.a * clamp(m, vec3f(0.0), vec3f(1.0));
    return vec4f(rgb, s.a + b.a - s.a * b.a);
}

// --- feMorphology: one axis (p.a.xy), radius p.a.z texels, p.a.w 1 dilate (max) or 0 erode (min) ---

@fragment fn morph(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let q = vec2i(f.xy);
    let d = vec2i(p.a.xy);
    let r = i32(p.a.z);
    let dilate = p.a.w > 0.5;
    var o = at(src, q);
    for (var i = -r; i <= r; i++) {
        let c = at(src, q + d * i);
        o = select(min(o, c), max(o, c), dilate);
    }
    return o;
}

// --- feConvolveMatrix: p.a = columns, rows, target x, y; p.b = divisor, bias, edge (0 none 1 duplicate 2 wrap),
// preserveAlpha; the kernel row-major in p.t ---

fn kernel(i: i32) -> f32 { return p.t[i / 4][i % 4]; }

fn conv_at(q: vec2i) -> vec4f {
    let e = i32(p.b.z);
    let s = vec2i(textureDimensions(src));
    if e == 1 { return edge(src, q); }
    if e == 2 { return textureLoad(src, ((q % s) + s) % s, 0); }
    return at(src, q);
}

@fragment fn convolve(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let q = vec2i(f.xy);
    let cols = i32(p.a.x);
    let rows = i32(p.a.y);
    let tg = vec2i(p.a.zw);
    let keep = p.b.w > 0.5;
    var sum = vec4f(0.0);
    for (var i = 0; i < rows; i++) {
        for (var j = 0; j < cols; j++) {
            sum += conv_at(q - tg + vec2i(j, i)) * kernel((rows - i - 1) * cols + (cols - j - 1));
        }
    }
    let here = at(src, q);
    if keep {
        // as Skia: premultiplied colour convolved, clamped to the pixel's own alpha
        let a = here.a;
        return vec4f(clamp(sum.rgb / p.b.x + p.b.y * a, vec3f(0.0), vec3f(a)), a);
    }
    let o = clamp(sum / p.b.x + p.b.y, vec4f(0.0), vec4f(1.0));
    return vec4f(min(o.rgb, vec3f(o.a)), o.a);
}

// --- feDisplacementMap: src displaced by other's (straight) channels p.a.y and p.a.z, p.a.x texels a unit ---

@fragment fn displace(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let q = vec2i(f.xy);
    var m = straight(at(other, q));
    let d = p.a.x * (vec2f(m[i32(p.a.y)], m[i32(p.a.z)]) - 0.5);
    return at(src, vec2i(floor(f.xy + d)));
}

// --- feComponentTransfer: per channel p.m[c] = (type, n | slope | amplitude, intercept | exponent, offset), tables
// in p.t (entry i of channel c at p.t[i][c]); on straight values ---

fn transfer(c: i32, v: f32) -> f32 {
    let k = p.m[c];
    let n = i32(k.y);
    switch i32(k.x) {
        case 1: {
            if n == 1 { return p.t[0][c]; }
            let x = v * f32(n - 1);
            let i = clamp(i32(floor(x)), 0, n - 2);
            return mix(p.t[i][c], p.t[i + 1][c], x - f32(i));
        }
        case 2: { return p.t[clamp(i32(floor(v * f32(n))), 0, n - 1)][c]; }
        case 3: { return k.y * v + k.z; }
        case 4: { return k.y * pow(v, k.z) + k.w; }
        default: { return v; }
    }
}

@fragment fn transfer_fn(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let s = straight(at(src, vec2i(f.xy)));
    return premul(vec4f(transfer(0, s.r), transfer(1, s.g), transfer(2, s.b), transfer(3, s.a)));
}

// --- feTurbulence: the spec's reference noise. `other` holds the lattice: texel (i, channel) = gradient x, y and the
// lattice selector at i. p.a = base frequency x, y, octaves, fractal; p.b = stitch: width, wrap x, height, wrap y
// (width 0: none); p.m[2..4] the texel -> user affine (rows) ---

fn lattice(i: i32) -> i32 { return i32(textureLoad(other, vec2i(i, 0), 0).z); }

fn gradient(ch: i32, i: i32) -> vec2f { return textureLoad(other, vec2i(i, ch), 0).xy; }

fn noise2(ch: i32, v: vec2f, st: vec4i) -> f32 {
    let t = v + 4096.0;
    var b0 = vec2i(t) & vec2i(255);
    var b1 = (b0 + 1) & vec2i(255);
    let r0 = t - floor(t);
    let r1 = r0 - 1.0;
    if st.x != 0 {
        if b0.x >= st.y { b0.x -= st.x; }
        if b1.x >= st.y { b1.x -= st.x; }
        if b0.y >= st.w { b0.y -= st.z; }
        if b1.y >= st.w { b1.y -= st.z; }
    }
    b0 &= vec2i(255);
    b1 &= vec2i(255);
    let i = lattice(b0.x);
    let j = lattice(b1.x);
    let b00 = lattice(i + b0.y);
    let b10 = lattice(j + b0.y);
    let b01 = lattice(i + b1.y);
    let b11 = lattice(j + b1.y);
    let s = r0 * r0 * (3.0 - 2.0 * r0);
    let a = mix(dot(r0, gradient(ch, b00)), dot(vec2f(r1.x, r0.y), gradient(ch, b10)), s.x);
    let b = mix(dot(vec2f(r0.x, r1.y), gradient(ch, b01)), dot(r1, gradient(ch, b11)), s.x);
    return mix(a, b, s.y);
}

@fragment fn turbulence(@builtin(position) f: vec4f) -> @location(0) vec4f {
    // sampled at the pixel's centre, as Skia's GPU noise
    let u = vec2f(dot(p.m[2].xyz, vec3f(f.xy, 1.0)), dot(p.m[3].xyz, vec3f(f.xy, 1.0)));
    let fractal = p.a.w > 0.5;
    var o = vec4f(0.0);
    for (var ch = 0; ch < 4; ch++) {
        var v = u * p.a.xy;
        var st = vec4i(p.b);
        var ratio = 1.0;
        var sum = 0.0;
        for (var k = 0; k < i32(p.a.z); k++) {
            let n = noise2(ch, v, st);
            sum += select(abs(n), n, fractal) / ratio;
            v *= 2.0;
            ratio *= 2.0;
            st = vec4i(st.x * 2, 2 * st.y - 4096, st.z * 2, 2 * st.w - 4096);
        }
        o[ch] = select(sum, (sum + 1.0) / 2.0, fractal);
    }
    return premul(o);
}

// --- feDiffuseLighting / feSpecularLighting on src's alpha: p.a = surface scale, constant, exponent, specular;
// p.b = light kind (0 distant 1 point 2 spot), spot exponent, cos outer cone (or -2: none), unused; p.m[0] the
// light's position in texels (or the distant light's direction), p.m[1] a spot's direction, p.c[0] the colour ---

fn height(q: vec2i) -> f32 { return edge(src, q).a; }

@fragment fn lighting(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let q = vec2i(f.xy);
    let ss = p.a.x;
    let nx = -ss / 4.0 * ((height(q + vec2i(1, -1)) + 2.0 * height(q + vec2i(1, 0)) + height(q + vec2i(1, 1)))
        - (height(q + vec2i(-1, -1)) + 2.0 * height(q + vec2i(-1, 0)) + height(q + vec2i(-1, 1))));
    let ny = -ss / 4.0 * ((height(q + vec2i(-1, 1)) + 2.0 * height(q + vec2i(0, 1)) + height(q + vec2i(1, 1)))
        - (height(q + vec2i(-1, -1)) + 2.0 * height(q + vec2i(0, -1)) + height(q + vec2i(1, -1))));
    let n = normalize(vec3f(nx, ny, 1.0));
    let kind = i32(p.b.x);
    let surface = vec3f(f.xy, ss * height(q));
    let l = select(normalize(p.m[0].xyz - surface), p.m[0].xyz, kind == 0);
    var color = p.c[0].rgb;
    if kind == 2 {
        let cosine = -dot(l, p.m[1].xyz);
        let outer = p.b.z;
        if cosine < outer {
            color = vec3f(0.0);
        } else {
            // Skia's antialiased cone edge
            let fade = select(1.0, clamp((cosine - outer) / 0.016, 0.0, 1.0), outer > -1.5);
            color *= pow(max(cosine, 0.0), p.b.y) * fade;
        }
    }
    if p.a.w < 0.5 {
        let rgb = clamp(p.a.y * max(dot(n, l), 0.0) * color, vec3f(0.0), vec3f(1.0));
        return vec4f(rgb, 1.0);
    }
    let h = normalize(l + vec3f(0.0, 0.0, 1.0));
    let rgb = clamp(p.a.y * pow(max(dot(n, h), 0.0), p.a.z) * color, vec3f(0.0), vec3f(1.0));
    let a = max(rgb.r, max(rgb.g, rgb.b));
    return vec4f(rgb, a);
}

// --- feTile: p.a the input's subregion in user space (x, y, w, h), repeated; p.m[0..2] user -> texel rows, p.m[2..4]
// texel -> user rows ---

@fragment fn tile(@builtin(position) f: vec4f) -> @location(0) vec4f {
    let u = vec2f(dot(p.m[2].xyz, vec3f(f.xy, 1.0)), dot(p.m[3].xyz, vec3f(f.xy, 1.0)));
    let w = p.a.xy + (u - p.a.xy) - p.a.zw * floor((u - p.a.xy) / p.a.zw);
    let q = vec2f(dot(p.m[0].xyz, vec3f(w, 1.0)), dot(p.m[1].xyz, vec3f(w, 1.0)));
    return at(src, vec2i(floor(q)));
}
