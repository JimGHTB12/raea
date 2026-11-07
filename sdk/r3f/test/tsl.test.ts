// TSL node materials as raea's ThreeShaderMaterial (src/input/tsl.ts): three's WGSLNodeBuilder, headless, its WGSL
// rewritten to the engine's contract (systems/three-shader). Each case is checked as text, and, where naga is on this
// machine (`NAGA=/path/to/naga`, or `naga` on PATH), compiled by it against a stand-in of the engine's prelude.
import { describe, expect, test } from "bun:test";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import * as THREE from "three";
import { MeshBasicNodeMaterial, MeshStandardNodeMaterial } from "three/webgpu";
import * as TSLTyped from "three/tsl";
// untyped: @types/three's chained TSL types make tsc take minutes and gigabytes
const { attribute, color, mix, sin, texture, time, uv } = TSLTyped as unknown as Record<string, (...a: unknown[]) => any> & { time: any };
import { composeEffects, configureFontTexture, createText, parseFont, scramble, wipe } from "lettra/three";
import { materialOf } from "../src/input/material";
import { nodeMaterialOf, type NodeAt, type ShaderSpec } from "../src/input/tsl";
import { standInImages } from "../src/input/texture";

standInImages();

const font = parseFont(JSON.parse(readFileSync(resolve(import.meta.dir, "parity/fonts/Inter-400.json"), "utf8")));

function scene(mesh: THREE.Mesh, t = 0): NodeAt {
  const s = new THREE.Scene();
  s.add(mesh);
  const camera = new THREE.PerspectiveCamera(45, 16 / 9, 0.1, 100);
  camera.position.set(0, 0, 5);
  s.updateMatrixWorld();
  camera.updateMatrixWorld();
  return { object: mesh, scene: s, camera, time: t };
}

function lettra() {
  const map = configureFontTexture(new THREE.TextureLoader().load("fonts/Inter-400.png"));
  return createText({ font, map, text: "raea", material: { fill: "#f2cc60" } });
}

/** the engine's prelude, stood in (the contract: systems/three-shader's doc comment) */
/** a part's props (<ThreeShaderMaterial wgsl>) as the compiled spec: the stages, each uniform with its value, the textures */
function specOf(props: unknown): ShaderSpec & Record<string, unknown> {
  const p = props as { wgsl: { vertex: string; fragment: string; uniforms: { name: string; type: string; array?: number }[] }; uniforms: Record<string, number[]>; textures: Record<string, { src: string; colorSpace: string }> } & Record<string, unknown>;
  return { ...p, vertex: p.wgsl.vertex, fragment: p.wgsl.fragment, uniforms: p.wgsl.uniforms.map((u) => ({ ...u, value: p.uniforms[u.name] })), textures: Object.entries(p.textures).map(([name, t]) => ({ name, ...t })) } as unknown as ShaderSpec & Record<string, unknown>;
}

function prelude(spec: ShaderSpec): string {
  let p = "struct ThreeVertex { position: vec3<f32>, normal: vec3<f32>, uv: vec2<f32>, color: vec4<f32>, a0: vec4<f32>, a1: vec4<f32>, a2: vec4<f32>, a3: vec4<f32>, instance: u32 }\n";
  p += "fn three_model(i: u32) -> mat4x4<f32> { return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, 0.0), vec4<f32>(0.0, 1.0, 0.0, 0.0), vec4<f32>(0.0, 0.0, 1.0, 0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0)); }\n";
  p += "fn three_view() -> mat4x4<f32> { return three_model(0u); }\nfn three_projection() -> mat4x4<f32> { return three_model(0u); }\n";
  p += "fn three_camera_position() -> vec3<f32> { return vec3<f32>(0.0); }\nfn three_time() -> f32 { return 0.0; }\n";
  p += "struct Slots { s: array<vec4<f32>, 64> };\n@group(2) @binding(0) var<uniform> slots: Slots;\n";
  let k = 0;
  for (const u of spec.uniforms) {
    const s = (i: number) => (u.array ? `slots.s[${k + i}u + i]` : `slots.s[${k + i}]`);
    const body: Record<string, string> = {
      f32: `${s(0)}.x`, i32: `i32(${s(0)}.x)`, u32: `u32(${s(0)}.x)`, "vec2<f32>": `${s(0)}.xy`, "vec3<f32>": `${s(0)}.xyz`, "vec4<f32>": s(0),
      "mat3x3<f32>": `mat3x3<f32>(${s(0)}.xyz, ${s(1)}.xyz, ${s(2)}.xyz)`, "mat4x4<f32>": `mat4x4<f32>(${s(0)}, ${s(1)}, ${s(2)}, ${s(3)})`,
    };
    p += `fn u_${u.name}(${u.array ? "i: u32" : ""}) -> ${u.type} { return ${body[u.type]}; }\n`;
    k += (u.type.startsWith("mat4") ? 4 : u.type.startsWith("mat3") ? 3 : 1) * (u.array ?? 1);
  }
  spec.textures.forEach((t, i) => (p += `@group(2) @binding(${1 + 2 * i}) var tex_${t.name}: texture_2d<f32>;\n@group(2) @binding(${2 + 2 * i}) var samp_${t.name}: sampler;\n`));
  return p;
}

/** the stand-in images arrive a microtask after load(), as a loader's do */
const loaded = () => new Promise((r) => setTimeout(r, 0));

const naga = process.env.NAGA ?? Bun.which("naga");

/** each stage compiled by naga with the prelude and an entry point, as the engine adds them */
function compiles(spec: ShaderSpec) {
  if (!naga) return;
  const dir = mkdtempSync(join(tmpdir(), "raea-tsl-"));
  const vs = prelude(spec) + spec.vertex + "\n@vertex fn vs(@location(0) p: vec3<f32>, @location(1) n: vec3<f32>, @location(2) t: vec2<f32>, @builtin(instance_index) i: u32) -> ThreeVaryings { return three_vertex(ThreeVertex(p, n, t, vec4<f32>(1.0), vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0), i)); }\n";
  const fs = prelude(spec) + spec.fragment + "\n@fragment fn fs(v: ThreeVaryings, @builtin(front_facing) f: bool) -> @location(0) vec4<f32> { return three_fragment(v, f); }\n";
  for (const [name, src] of [["vertex", vs], ["fragment", fs]]) {
    const f = join(dir, `${name}.wgsl`);
    writeFileSync(f, src);
    const r = Bun.spawnSync([naga, f]);
    if (r.exitCode !== 0) throw new Error(`naga refused the ${name} stage:\n${r.stderr.toString()}\n--- source ---\n${src}`);
  }
}

describe("TSL node materials → raea's ThreeShaderMaterial", () => {
  test("lettra's wipe and scramble: its own attributes as a0..a3, its glyph pool as a uniform array", async () => {
    const map = configureFontTexture(new THREE.TextureLoader().load("fonts/Inter-400.png"));
    const json = JSON.parse(readFileSync(resolve(import.meta.dir, "parity/fonts/Inter-400.json"), "utf8"));
    const t = createText({ font, map, text: "raea", material: { fill: "#f2cc60", effect: composeEffects(wipe(), scramble({ font: json, chars: "abc" })) } });
    await loaded();
    const part = nodeMaterialOf(t.mesh.material as THREE.Material, scene(t.mesh))!;
    expect(part.type).toBe("ThreeShaderMaterial");
    const wgsl = (part.props as { wgsl: { attributes: string[]; uniforms: { name: string; array?: number }[] } }).wgsl;
    expect(wgsl.attributes).toEqual(expect.arrayContaining(["layoutX", "glyphIndex", "cellUv"]));
    const pool = wgsl.uniforms.find((u) => u.array)!;
    // the pool is the chars the bake has (its capacity at least one)
    expect(pool.array).toBeGreaterThanOrEqual(1);
    const spec = specOf(part.props);
    expect(spec.fragment + spec.vertex).toContain(`u_${pool.name}(u32(`);
    expect(spec.fragment + spec.vertex).not.toContain("NodeBuffer_");
    compiles(spec);
  });

  test("lettra's MSDF text: three's WGSL in the contract's shape", async () => {
    const t = lettra();
    await loaded();
    const part = nodeMaterialOf(t.mesh.material as THREE.Material, scene(t.mesh))!;
    expect(part.type).toBe("ThreeShaderMaterial");
    const spec = specOf(part.props);
    expect(spec.vertex).toContain("fn three_vertex(v: ThreeVertex) -> ThreeVaryings {");
    expect(spec.vertex).toContain("three_projection()");
    expect(spec.vertex).toContain("three_view()");
    expect(spec.vertex).toContain("three_model(tsl_instance)");
    expect(spec.fragment).toContain("fn three_fragment(in: ThreeVaryings, front: bool) -> vec4<f32> {");
    expect(spec.fragment).toMatch(/struct ThreeVaryings \{[^}]*@builtin\( position \) clip : vec4<f32>/);
    // the MSDF fill: the median of three channels, antialiased by its screen-space rate
    expect(spec.fragment).toMatch(/textureSample\( tex_\w+, samp_\w+, /);
    expect(spec.fragment).toContain("fwidth");
    for (const s of [spec.vertex, spec.fragment]) {
      expect(s).not.toMatch(/@group|@binding|@vertex|@fragment|diagnostic/);
      expect(s).not.toMatch(/\b(render|object)\./);
    }
    expect(spec.textures).toEqual([{ name: expect.any(String), src: "fonts/Inter-400.png", colorSpace: "linear" }]);
    // the fill colour, linear, as three keeps it
    const fill = new THREE.Color("#f2cc60");
    expect(spec.uniforms.find((u) => u.type === "vec3<f32>")!.value.map((x) => +x.toFixed(4))).toEqual([fill.r, fill.g, fill.b].map((x) => +x.toFixed(4)));
    expect(spec.output).toBe("linear");
    expect(spec.transparent).toBe(true);
    expect(spec.depthWrite).toBe(false);
    compiles(spec);
  });

  test("a graph's uniforms move with R3F's clock (TSL's time)", () => {
    const m = new MeshBasicNodeMaterial();
    m.colorNode = mix(color("#58a6ff"), color("#e05a2b"), uv().y).mul(sin(time.mul(2)).mul(0.25).add(0.75));
    const mesh = new THREE.Mesh(new THREE.BoxGeometry(), m);
    const at = (t: number) => specOf(nodeMaterialOf(m, scene(mesh, t))!.props).uniforms;
    const a = at(0), b = at(0.5);
    const timed = a.findIndex((u, i) => u.type === "f32" && u.value[0] !== b[i].value[0]);
    expect(timed).toBeGreaterThanOrEqual(0);
    expect(a[timed].value[0]).toBe(0);
    expect(b[timed].value[0]).toBe(0.5);
    compiles(specOf(nodeMaterialOf(m, scene(mesh, 0.5))!.props));
  });

  test("a flipped picture is sampled flipped (the graph's uvs are three's, unflipped)", async () => {
    const tex = new THREE.TextureLoader().load("pic.png");
    await loaded();
    expect(tex.flipY).toBe(true);
    const m = new MeshBasicNodeMaterial();
    m.colorNode = texture(tex, uv()).rgb;
    const spec = specOf(nodeMaterialOf(m, scene(new THREE.Mesh(new THREE.PlaneGeometry(), m)))!.props);
    expect(spec.fragment).toMatch(/textureSample\( tex_\w+, samp_\w+, tsl_flip_v\(/);
    expect(spec.textures[0].colorSpace).toBe("linear");
    compiles(spec);
  });

  test("a node material with no node of its own is its classic material, drawn natively", () => {
    const m = new MeshStandardNodeMaterial({ color: "#3fb950", roughness: 0.3, metalness: 0.2 });
    const part = materialOf(m, false, scene(new THREE.Mesh(new THREE.BoxGeometry(), m)));
    expect(part.type).toBe("MeshStandardMaterial");
    expect(part.props).toMatchObject({ color: "#3fb950", roughness: 0.3, metalness: 0.2 });
  });

  test("a lit node material with a graph is lit by three's own light nodes", () => {
    const m = new MeshStandardNodeMaterial();
    m.colorNode = mix(color("#ff0000"), color("#0000ff"), uv().x);
    const mesh = new THREE.Mesh(new THREE.SphereGeometry(), m);
    const at = scene(mesh);
    at.scene.add(new THREE.DirectionalLight("#ffffff", 2));
    const part = materialOf(m, false, at);
    expect(part.type).toBe("ThreeShaderMaterial");
    compiles(specOf(part.props));
  });

  test("a graph raea can't carry is refused by name and drawn flat", () => {
    const warn = console.warn;
    const said: string[] = [];
    console.warn = (s: string) => said.push(s);
    try {
      const m = new MeshBasicNodeMaterial();
      // five attributes of the mesh's own, one more than the engine carries
      const names = ["aOne", "aTwo", "aThree", "aFour", "aFive"];
      m.colorNode = names.map((n) => attribute(n, "float")).reduce((x, y) => x.add(y)).mul(0.1).toVec3();
      const g = new THREE.BufferGeometry();
      g.setAttribute("position", new THREE.Float32BufferAttribute([0, 0, 0, 1, 0, 0, 0, 1, 0], 3));
      for (const n of names) g.setAttribute(n, new THREE.Float32BufferAttribute([0, 1, 2], 1));
      const part = materialOf(m, false, scene(new THREE.Mesh(g, m)));
      expect(part.type).toBe("MeshBasicMaterial");
      expect(said.join("\n")).toContain("more than 4 vertex attributes");
    } finally {
      console.warn = warn;
    }
  });
});
