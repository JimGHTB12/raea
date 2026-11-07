// A three.js ShaderMaterial or RawShaderMaterial as raea's <ThreeShaderMaterial> (docs/r3f.feature R3F11): its GLSL as
// three would compile it (the #includes expanded from three's ShaderChunk, its `defines` in front), its uniforms as
// numbers, its textures as pictures. raea's engine translates the GLSL with naga and runs it with three's built-ins
// (raea-sys-three-shader), so the shader runs as written.
//
// What three puts on screen is what raea's engine shows: a ShaderMaterial's colour is the canvas's value unless it
// includes three's tone mapping, so it goes as "display" (the engine takes it back through Bevy's tone mapping); one
// that includes <tonemapping_fragment> on a tone-mapped material goes as "linear", those includes dropped, and Bevy's
// tone mapping maps it as three's would.
import * as THREE from "three";
import { srcOf } from "./texture";
import type { Part, Props } from "./material";

const hex = (c: THREE.Color) => "#" + c.getHexString();
const INCLUDE = /^[ \t]*#include +<([\w\d./]+)>/gm;

/** a shader's #includes expanded from three's chunks, as three's WebGLProgram does */
export function expandIncludes(src: string, depth = 0): string {
  if (depth > 16) throw new Error("raea/r3f: #include nested too deep");
  return src.replace(INCLUDE, (_m, name: string) => {
    const chunk = (THREE.ShaderChunk as Record<string, string>)[name];
    if (chunk === undefined) throw new Error(`raea/r3f: the shader includes <${name}>, which isn't one of three's chunks`);
    return expandIncludes(chunk, depth + 1);
  });
}

/**
 * A shader's #ifdef blocks decided where they can be: a condition made only of defined(X) / X tests (#ifdef, #ifndef,
 * #if and #elif with !, && and ||) is settled by `defines` (and the source's own #defines met on the way), its dead
 * branch dropped; any other condition (#if NUM_CLIPPING_PLANES > 0) is kept as written for the engine's preprocessor.
 * three's chunks guard what a material doesn't use (morph targets' MORPHTARGETS_COUNT arrays), which the engine reads
 * as declarations before preprocessing.
 */
export function decideIfdefs(src: string, defines: Set<string> | Map<string, string>): string {
  // what's defined, and the value it was given ("" when none)
  const known = new Map<string, string>(defines instanceof Map ? defines : [...defines].map((k) => [k, ""]));
  type Frame = { decided: boolean; live: boolean; taken: boolean; parentLive: boolean };
  const stack: Frame[] = [];
  const live = () => stack.every((f) => !f.decided || f.live);
  // a condition of defined(X), numbers and macros that are numbers, with ! && || and comparisons; undefined otherwise
  const evaluate = (expr: string): boolean | undefined => {
    let e = expr.replace(/\/\/.*$|\/\*.*?\*\//g, "").trim();
    e = e.replace(/defined\s*\(\s*(\w+)\s*\)|defined\s+(\w+)/g, (_m, a: string, b: string) => (known.has(a ?? b) ? "1" : "0"));
    let ok = true;
    e = e.replace(/[A-Za-z_]\w*/g, (n) => {
      const v = known.get(n);
      if (v === undefined || !/^-?\d+(\.\d+)?$/.test(v.trim())) ok = false;
      return v?.trim() ?? "0";
    });
    if (!ok || !/^[\s\d.!()&|<>=+-]*$/.test(e)) return undefined;
    try {
      return Boolean(Function(`return (${e})`)());
    } catch {
      return undefined;
    }
  };
  const out: string[] = [];
  for (const line of src.split("\n")) {
    const m = /^\s*#\s*(ifdef|ifndef|if|elif|else|endif|define|undef)\b\s*(.*)$/.exec(line);
    if (!m) {
      if (live()) out.push(line);
      continue;
    }
    const [, kind, rest] = m;
    const top = stack[stack.length - 1];
    if (kind === "ifdef" || kind === "ifndef" || kind === "if") {
      const v = kind === "ifdef" ? known.has(rest.trim().split(/\s/)[0]) : kind === "ifndef" ? !known.has(rest.trim().split(/\s/)[0]) : evaluate(rest);
      const parentLive = live();
      if (v === undefined) {
        stack.push({ decided: false, live: true, taken: false, parentLive });
        if (parentLive) out.push(line);
      } else stack.push({ decided: true, live: v, taken: v, parentLive });
      continue;
    }
    if (kind === "elif" && top) {
      if (!top.decided) {
        if (top.parentLive) out.push(line);
        continue;
      }
      const v = evaluate(rest);
      if (v === undefined) {
        // undecidable after a decided start: keep the rest for the engine, as an #if of its own
        top.decided = false;
        top.live = true;
        if (top.parentLive && !top.taken) out.push(line.replace(/#\s*elif/, "#if"));
        else if (top.parentLive) out.push("#if 0");
        continue;
      }
      top.live = !top.taken && v;
      top.taken ||= v;
      continue;
    }
    if (kind === "else" && top) {
      if (!top.decided) {
        if (top.parentLive) out.push(line);
        continue;
      }
      top.live = !top.taken;
      top.taken = true;
      continue;
    }
    if (kind === "endif" && top) {
      stack.pop();
      if (!top.decided && top.parentLive) out.push(line);
      continue;
    }
    if (live()) {
      out.push(line);
      if (kind === "define") {
        const [name, ...value] = rest.trim().split(/\s+/);
        known.set(name, value.join(" "));
      }
      if (kind === "undef") known.delete(rest.trim().split(/\s/)[0]);
    }
  }
  return out.join("\n");
}

/** three's sRGB encoding, which <colorspace_fragment> calls (three declares it in front of every fragment shader) */
const OUTPUT_TEXEL = "vec4 linearToOutputTexel(vec4 v) { return vec4(mix(pow(v.rgb, vec3(0.41666)) * 1.055 - vec3(0.055), v.rgb * 12.92, vec3(lessThanEqual(v.rgb, vec3(0.0031308)))), v.a); }\n";

/** a uniform's value as numbers (a texture is listed apart); undefined for one with nothing raea can send */
function valueOf(v: unknown): number | boolean | number[] | undefined {
  if (typeof v === "number" || typeof v === "boolean") return v;
  const o = v as { isColor?: boolean; isMatrix3?: boolean; isMatrix4?: boolean; isVector2?: boolean; isVector3?: boolean; isVector4?: boolean; isQuaternion?: boolean; r: number; g: number; b: number; elements: number[]; toArray(): number[] } | null;
  if (o?.isColor) return [o.r, o.g, o.b];
  if (o?.isMatrix3 || o?.isMatrix4) return [...o.elements];
  if (o?.isVector2 || o?.isVector3 || o?.isVector4 || o?.isQuaternion) return o.toArray();
  if (Array.isArray(v)) {
    const out: number[] = [];
    for (const x of v) {
      const e = valueOf(x);
      if (e === undefined) return undefined;
      if (Array.isArray(e)) out.push(...e);
      else out.push(Number(e));
    }
    return out;
  }
  if (ArrayBuffer.isView(v)) return Array.from(v as unknown as ArrayLike<number>);
  return undefined;
}

/** a picture three has no file for (a render target's): one white pixel, so the shader still runs */
const WHITE = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/x8AAwMCAO+ip1sAAAAASUVORK5CYII=";

const warned = new Set<string>();
function warnOnce(k: string, why: string) {
  if (warned.has(k)) return;
  warned.add(k);
  console.warn(`raea/r3f: ${k} ${why}`);
}

/** A ShaderMaterial as a <ThreeShaderMaterial> part. `points`: it draws three.js Points (each a quad here, its size
 * from gl_PointSize). */
export function shaderPart(m: THREE.ShaderMaterial, points = false): Part {
  const raw = (m as THREE.RawShaderMaterial).isRawShaderMaterial === true;
  const defines = Object.entries(m.defines ?? {}).filter(([, v]) => v !== false).map(([k, v]) => `#define ${k} ${v === true ? "" : v}`).join("\n");
  let vertex = expandIncludes(m.vertexShader);
  let fragment = expandIncludes(m.fragmentShader);
  // three's tone mapping, when the shader asks for it, is Bevy's here: the shader's colour is then scene light
  const tones = /#include +<tonemapping_fragment>/.test(m.fragmentShader) && m.toneMapped;
  if (tones) fragment = expandIncludes(m.fragmentShader.replace(/^[ \t]*#include +<(tonemapping_fragment|colorspace_fragment)>/gm, ""));
  else if (/#include +<tonemapping_fragment>/.test(m.fragmentShader)) fragment = expandIncludes(m.fragmentShader.replace(/^[ \t]*#include +<tonemapping_fragment>/gm, ""));
  // three's prefix in front of a ShaderMaterial (not a raw one): no clipping planes (raea cuts the mesh, clip.ts)
  const front = (raw ? "" : "#define NUM_CLIPPING_PLANES 0\n#define UNION_CLIPPING_PLANES 0\n") + (defines ? defines + "\n" : "");
  const on = new Map<string, string>(Object.entries(m.defines ?? {}).filter(([, v]) => v !== false).map(([k, v]) => [k, v === true ? "" : String(v)]));
  // naga's GLSL reads a parameter's `in` but not `const in` (three's <common> declares its helpers so)
  const params = (src: string) => src.replace(/([(,]\s*)const\s+in\s+/g, "$1in ");
  vertex = params(decideIfdefs(front + vertex, on));
  fragment = params(decideIfdefs(front + (raw ? "" : OUTPUT_TEXEL) + fragment, on));
  const uniforms: Record<string, number | boolean | number[]> = {};
  const textures: Record<string, string | { src: string; colorSpace: "linear" }> = {};
  const flipY: string[] = [];
  let color: string | undefined;
  for (const [name, u] of Object.entries(m.uniforms ?? {})) {
    const v = (u as THREE.IUniform | undefined)?.value;
    if ((v as THREE.Texture | null)?.isTexture) {
      const t = v as THREE.Texture;
      const src = srcOf(t);
      if (!src) warnOnce(`${m.type} texture ${name}`, "has no picture raea can load (a render target's?): drawn as white");
      // three samples a picture without a colour space raw, an sRGB one decoded (as Bevy's sRGB image is)
      textures[name] = src && t.colorSpace !== THREE.SRGBColorSpace ? { src, colorSpace: "linear" } : src ?? WHITE;
      if (t.flipY && src) flipY.push(name);
      continue;
    }
    if (v instanceof THREE.Color && !color && /^(u?[cC]olou?r|diffuse)$/.test(name)) color = hex(v);
    const n = valueOf(v);
    if (n === undefined) {
      if (v !== null && v !== undefined) warnOnce(`${m.type} uniform ${name}`, "isn't a number, vector, colour or matrix: left at zero");
      continue;
    }
    uniforms[name] = n;
  }
  const props: Props = {
    glsl: { vertex, fragment, raw, vertexColors: m.vertexColors === true, ...(points ? { points: true } : {}) },
    ...(flipY.length ? { flipY } : {}),
    uniforms,
    textures,
    output: tones ? "linear" : "display",
    side: m.side === THREE.BackSide ? "back" : m.side === THREE.DoubleSide ? "double" : "front",
    depthWrite: m.depthWrite,
    depthTest: m.depthTest,
    color: color ?? "#ffffff",
  };
  if (m.transparent) props.transparent = true;
  if (m.blending === THREE.AdditiveBlending) props.blending = "additive";
  else if (m.blending === THREE.MultiplyBlending) props.blending = "multiply";
  if (m.alphaTest > 0) props.alphaTest = m.alphaTest;
  return { type: "ThreeShaderMaterial", props };
}

/** whether a material is a ShaderMaterial (or Raw), which shaderPart draws */
export const isShader = (m: THREE.Material): m is THREE.ShaderMaterial => (m as THREE.ShaderMaterial).isShaderMaterial === true;

/** a geometry's own attributes (all but position, normal, uv and color) as a BufferMesh's `attributes` */
export function ownAttributes(g: THREE.BufferGeometry): Record<string, { size: number; values: number[] }> | undefined {
  const out: Record<string, { size: number; values: number[] }> = {};
  for (const [name, a] of Object.entries(g.attributes)) {
    if (["position", "normal", "uv", "color"].includes(name)) continue;
    const attr = a as THREE.BufferAttribute;
    if (attr.itemSize > 4) continue;
    const values: number[] = new Array(attr.count * attr.itemSize);
    for (let i = 0; i < attr.count; i++) for (let c = 0; c < attr.itemSize; c++) values[i * attr.itemSize + c] = attr.getComponent(i, c);
    out[name] = { size: attr.itemSize, values };
  }
  return Object.keys(out).length ? out : undefined;
}

/**
 * A built-in material three draws with a program of its own that needs no lights (matcap, normal) as raea's
 * <ThreeShaderMaterial> of that same program: three's ShaderLib source, the defines its WebGLProgram would set for
 * this material (map, matcap, flat shading, sides) and its uniforms, so it looks as three draws it.
 */
export function builtinPart(m: THREE.MeshMatcapMaterial | THREE.MeshNormalMaterial): Part {
  const lib = m.type === "MeshMatcapMaterial" ? THREE.ShaderLib.matcap : THREE.ShaderLib.normal;
  const uniforms = THREE.UniformsUtils.clone(lib.uniforms) as Record<string, THREE.IUniform>;
  const defines: Record<string, string | boolean> = {};
  if (m.flatShading) defines.FLAT_SHADED = true;
  if (m.side === THREE.DoubleSide) defines.DOUBLE_SIDED = true;
  if (m.side === THREE.BackSide) defines.FLIP_SIDED = true;
  uniforms.opacity.value = m.opacity;
  if (m.type === "MeshMatcapMaterial") {
    const mc = m as THREE.MeshMatcapMaterial;
    uniforms.diffuse.value = mc.color.clone();
    if (mc.matcap) {
      defines.USE_MATCAP = true;
      uniforms.matcap.value = mc.matcap;
    }
    if (mc.map) {
      defines.USE_MAP = true;
      defines.MAP_UV = "uv";
      mc.map.updateMatrix();
      uniforms.map.value = mc.map;
      uniforms.mapTransform.value = mc.map.matrix.clone();
    }
  }
  const sm = new THREE.ShaderMaterial({ uniforms, vertexShader: lib.vertexShader, fragmentShader: lib.fragmentShader, defines, transparent: m.transparent, side: m.side, toneMapped: m.toneMapped });
  sm.opacity = m.opacity;
  return shaderPart(sm);
}
