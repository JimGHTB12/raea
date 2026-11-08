// A TSL node material (three/webgpu: MeshBasicNodeMaterial with a colorNode, lettra's MSDF text, any library built on
// TSL) as raea's ThreeShaderMaterial: three compiles the node graph to WGSL with its own WGSLNodeBuilder, run headless
// against a stand-in WebGPURenderer (it is asked for features and a node library, never to draw), and the WGSL is
// rewritten to the engine's contract (systems/three-shader):
//   the vertex stage   fn three_vertex(v: ThreeVertex) -> ThreeVaryings
//   the fragment stage fn three_fragment(in: ThreeVaryings, front: bool) -> vec4<f32>   (linear: `output: "linear"`)
//   both declare       struct ThreeVaryings { @builtin(position) clip: vec4<f32>, @location(0) ... }
// three's uniform groups and textures become the prelude's u_NAME() accessors (254 vec4 slots of the 256: three-shader module.rs) and tex_NAME/samp_NAME;
// the camera's projection and view and the model's world matrix become three_projection(), three_view() and
// three_model(instance), so Bevy's own camera and transform place it (Bevy's projection is reverse-Z). Every other
// uniform takes the value three's own node updates give it this frame (time from R3F's clock, a light's colour), so a
// uniform the scene animates in useFrame moves with it. A uniformArray is packed element by element and read as
// u_NAME(i); a vertex attribute of the mesh's own is one of the contract's a0..a3. A graph this can't carry (more than 4
// textures or 4 attributes of the mesh's own, a storage buffer, a render target) is refused with the reason, and the
// material is drawn flat as before.
import * as THREE from "three";
import { ModelNode, NodeFrame, WebGPURenderer } from "three/webgpu";
import type { Part, Props } from "./material";
import { srcOf } from "./texture";

/** what a node material is drawn for this frame: the object it's on, its scene and camera, R3F's clock */
export interface NodeAt {
  object: THREE.Object3D;
  scene: THREE.Scene;
  camera: THREE.Camera;
  /** R3F's clock, seconds (TSL's `time`) */
  time: number;
  /** seconds since the last frame (TSL's `deltaTime`) */
  delta?: number;
}

export interface ShaderUniform {
  name: string;
  type: string;
  value: number[];
  array?: number;
}
export interface ShaderTexture {
  name: string;
  src: string;
  colorSpace: "srgb" | "linear";
}
export interface ShaderSpec {
  vertex: string;
  fragment: string;
  uniforms: ShaderUniform[];
  textures: ShaderTexture[];
}

/** the engine's limits (systems/three-shader) */
export const SLOTS = 254;
export const TEXTURES = 4;

type AnyNode = { isNode?: boolean; name?: string; value?: unknown; uniformNode?: AnyNode; scope?: string };
interface NodeUniformLike { name: string; nodeUniform?: { node: AnyNode; value: unknown } }
interface BindingLike { name: string; isNodeUniformsGroup?: boolean; isSampledTexture?: boolean; isSampler?: boolean; isStorageBuffer?: boolean; isBuffer?: boolean; uniforms?: NodeUniformLike[]; texture?: THREE.Texture; isSampledCubeTexture?: boolean; isSampledTexture3D?: boolean }
interface Builder {
  scene: THREE.Scene;
  camera: THREE.Camera;
  material: THREE.Material;
  context: { material?: THREE.Material };
  lightsNode?: unknown;
  vertexShader: string;
  fragmentShader: string;
  updateNodes: AnyNode[];
  updateBeforeNodes: AnyNode[];
  updateAfterNodes: AnyNode[];
  build(): void;
  getBindings(): { name: string; bindings: BindingLike[] }[];
}

/** a refusal: what the graph needs that the engine's contract doesn't carry */
export class Unsupported extends Error {}

// ---- the stand-in renderer ----

let renderer: WebGPURenderer | undefined;

/** three's WebGPURenderer, never initialised: WGSLNodeBuilder reads its node library and asks it what the GPU can do */
function standIn(fresh = false): WebGPURenderer {
  if (renderer && !fresh) return renderer;
  const noop = () => {};
  const canvas = { style: {}, width: 1, height: 1, addEventListener: noop, removeEventListener: noop, getContext: () => null };
  const r = new WebGPURenderer({ canvas: canvas as unknown as HTMLCanvasElement });
  const own = r as unknown as Record<string, unknown>;
  // a builder asks before it is built; nothing here runs on a GPU, and Bevy has what these name
  own._initialized = true;
  own.hasFeature = () => false;
  own.hasCompatibility = () => true;
  // no shadow maps: a shadow in three's graph would be a depth texture raea doesn't fill (raea's own lights cast)
  r.shadowMap.enabled = false;
  if (fresh) return r;
  return (renderer = r);
}

// ---- compiling a graph ----

interface Compiled {
  key: string;
  builder: Builder;
  vertex: string;
  fragment: string;
  /** each packed uniform: its accessor name, type, slot count and how its value is read */
  uniforms: { name: string; type: string; array?: number; read: () => unknown; stored: "f32" | "u32" | "bool" }[];
  textures: { name: string; texture: THREE.Texture }[];
  /** the mesh's own attributes it reads, in order (the contract's a0..a3) */
  attributes: string[];
  frame: NodeFrame;
}

const compiled = new WeakMap<THREE.Material, Compiled>();

/** what a compile depends on beyond the material: the geometry's attributes (a graph reads the ones it has) */
function keyOf(m: THREE.Material, o: THREE.Object3D): string {
  const g = (o as THREE.Mesh).geometry as THREE.BufferGeometry | undefined;
  return `${m.version}|${g ? Object.keys(g.attributes).sort().join(",") : ""}|${(o as THREE.InstancedMesh).isInstancedMesh ? "i" : ""}`;
}

function compile(m: THREE.Material, at: NodeAt): Compiled {
  const key = keyOf(m, at.object);
  const had = compiled.get(m);
  if (had && had.key === key) return had;
  // a renderer of its own: three caches a shared group's binding (render, frame) per renderer, with the uniforms of
  // whichever material built it first
  const r = standIn(true);
  const b = (r as unknown as { backend: { createNodeBuilder(o: THREE.Object3D, r: WebGPURenderer): Builder } }).backend.createNodeBuilder(at.object, r);
  b.scene = at.scene;
  b.camera = at.camera;
  b.material = m;
  b.context.material = m;
  // a lit node material is lit by three's own light nodes, their colours and places this frame's
  if ((m as unknown as { lights?: boolean }).lights) b.lightsNode = (r as unknown as { lighting: { getNode(s: THREE.Scene, c: THREE.Camera): unknown } }).lighting.getNode(at.scene, at.camera);
  const errors: string[] = [];
  const err = console.error;
  console.error = (...a: unknown[]) => errors.push(a.map(String).join(" "));
  try {
    b.build();
  } finally {
    console.error = err;
  }
  if (errors.length) throw new Unsupported(`three couldn't build its graph: ${errors[0]}`);
  if (b.updateBeforeNodes.length || b.updateAfterNodes.length) throw new Unsupported("it renders the scene again first (a reflector, a render target, a pass): raea draws one scene");
  const c = rewrite(b);
  const out: Compiled = { key, builder: b, ...c, frame: new NodeFrame() };
  compiled.set(m, out);
  return out;
}

// ---- the WGSL, rewritten to the contract ----

/** the struct member types of every `struct NAME { ... }` in a source */
function structs(src: string): Map<string, Map<string, string>> {
  const out = new Map<string, Map<string, string>>();
  for (const s of src.matchAll(/struct\s+(\w+)\s*\{([^}]*)\}/g)) {
    const fields = new Map<string, string>();
    for (const f of s[2].split(",")) {
      const m = f.replace(/@\w+\s*\([^)]*\)/g, "").trim().match(/^(\w+)\s*:\s*(.+)$/);
      if (m) fields.set(m[1], m[2].trim());
    }
    out.set(s[1], fields);
  }
  return out;
}

/** `name` as a whole word in `src`, replaced (WGSL words: letters, digits, underscore) */
function word(src: string, name: string, to: string): string {
  return src.replace(new RegExp(`(?<![A-Za-z0-9_])${name}(?![A-Za-z0-9_])`, "g"), to);
}

/** each call of `fn` whose first argument is one of `textures`, its coordinate (third argument) flipped in v */
function flipSamples(src: string, flipped: Set<string>): string {
  if (!flipped.size) return src;
  let out = "";
  let i = 0;
  const re = /\b(textureSample\w*)\s*\(/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(src))) {
    const open = m.index + m[0].length;
    // the call's arguments, split at its own top-level commas
    const args: [number, number][] = [];
    let depth = 0, start = open, j = open;
    for (; j < src.length; j++) {
      const ch = src[j];
      if (ch === "(" || ch === "[") depth++;
      else if (ch === ")" || ch === "]") {
        if (depth === 0) break;
        depth--;
      } else if (ch === "," && depth === 0) {
        args.push([start, j]);
        start = j + 1;
      }
    }
    args.push([start, j]);
    const tex = src.slice(args[0][0], args[0][1]).trim();
    if (args.length < 3 || !flipped.has(tex)) continue;
    const [a, z] = args[2];
    out += src.slice(i, a) + ` tsl_flip_v( ${src.slice(a, z).trim()} )`;
    i = z;
  }
  return out + src.slice(i);
}

const FLIP = "fn tsl_flip_v(c: vec2<f32>) -> vec2<f32> { return vec2<f32>(c.x, 1.0 - c.y); }\n";

/** the contract's vertex attributes (ThreeVertex), by three's attribute name */
const ATTRIBUTES: Record<string, { field: string; type: string }> = {
  position: { field: "position", type: "vec3<f32>" },
  normal: { field: "normal", type: "vec3<f32>" },
  uv: { field: "uv", type: "vec2<f32>" },
  color: { field: "color", type: "vec4<f32>" },
};

/** a value of `type` read as `field` (a vec4 colour as a vec3 three declared, say) */
function fit(field: string, from: string, to: string): string {
  if (from === to) return field;
  const n = (t: string) => Number(t.match(/^vec(\d)/)?.[1] ?? 1);
  if (to.startsWith("vec") && from.startsWith("vec") && n(to) < n(from)) return `${field}.${"xyzw".slice(0, n(to))}`;
  if (from.startsWith("vec") && (to === "f32" || to === "i32" || to === "u32")) return to === "f32" ? `${field}.x` : `${to}(${field}.x)`;
  throw new Unsupported(`an attribute declared ${to} where raea gives ${from}`);
}

/** a uniform's slot count in vec4s */
const slotsOf = (type: string, array = 1) => (type.startsWith("mat4") ? 4 : type.startsWith("mat3") ? 3 : 1) * array;

/** a uniformArray's element type (three's name) as WGSL's */
const ELEMENT: Record<string, string> = { float: "f32", vec2: "vec2<f32>", vec3: "vec3<f32>", vec4: "vec4<f32>", color: "vec3<f32>" };
/** the mesh's own attributes the engine carries (raea-sys-three-shader's ATTRIBUTES) */
const ATTRIBUTES_OWN = 4;

const STORED: Record<string, string> = {
  f32: "f32", i32: "i32", u32: "u32", bool: "u32",
  "vec2<f32>": "vec2<f32>", "vec3<f32>": "vec3<f32>", "vec4<f32>": "vec4<f32>",
  "mat3x3<f32>": "mat3x3<f32>", "mat4x4<f32>": "mat4x4<f32>",
};

function rewrite(b: Builder): Omit<Compiled, "key" | "builder" | "frame"> {
  let vertex = b.vertexShader;
  let fragment = b.fragmentShader;
  const groups = b.getBindings();
  // the model's world matrix: the uniform behind ModelNode's world-matrix scope
  const modelWorld = new Set<AnyNode>();
  for (const n of b.updateNodes) if (n instanceof ModelNode && (n as AnyNode).scope === "worldMatrix" && (n as AnyNode).uniformNode) modelWorld.add((n as AnyNode).uniformNode!);
  // each group's members: where the engine's own value stands in, or the packed uniform's accessor
  const types = new Map([...structs(vertex), ...structs(fragment)]);
  const members = new Map<string, Map<string, { vertex: string; fragment: string }>>();
  const uniforms: Compiled["uniforms"] = [];
  const textures: Compiled["textures"] = [];
  const flipped = new Set<string>();
  /** a uniformArray's buffer: three's WGSL name for it, and the packed uniform it reads as */
  const buffers: { wgsl: string; name: string }[] = [];
  let slots = 0;
  for (const g of groups) {
    for (const x of g.bindings) {
      if (x.isNodeUniformsGroup && x.uniforms) {
        const decl = [...vertex.matchAll(new RegExp(`var<uniform>\\s+${x.name}\\s*:\\s*(\\w+)`, "g")), ...fragment.matchAll(new RegExp(`var<uniform>\\s+${x.name}\\s*:\\s*(\\w+)`, "g"))][0];
        const fields = decl ? types.get(decl[1]) : undefined;
        const map = new Map<string, { vertex: string; fragment: string }>();
        for (const u of x.uniforms) {
          const node = u.nodeUniform?.node;
          const t = fields?.get(u.name);
          // a shared group (render, frame) carries every uniform built against this renderer: one this shader
          // neither declares nor reads is another material's
          if (!t && !new RegExp(`(?<![A-Za-z0-9_])${x.name}\\.${u.name}(?![A-Za-z0-9_])`).test(vertex + fragment)) continue;
          if (!node || !t) throw new Unsupported(`uniform ${x.name}.${u.name} has no declaration three wrote`);
          const named = node.name;
          if (named === "cameraProjectionMatrix") map.set(u.name, { vertex: "three_projection()", fragment: "three_projection()" });
          else if (named === "cameraViewMatrix") map.set(u.name, { vertex: "three_view()", fragment: "three_view()" });
          else if (named === "cameraPosition") map.set(u.name, { vertex: "three_camera_position()", fragment: "three_camera_position()" });
          else {
            if (/^array</.test(t)) throw new Unsupported(`uniform ${u.name} is an array in three's uniform group (${t})`);
            const type = t;
            const stored = STORED[type];
            if (!stored) throw new Unsupported(`uniform ${u.name} is a ${t}, which raea's uniforms don't carry`);
            const name = u.name;
            const access = type === "bool" ? `(u_${name}() != 0u)` : `u_${name}()`;
            uniforms.push({ name, type: stored, read: () => u.nodeUniform!.value, stored: type === "bool" ? "bool" : stored === "u32" || stored === "i32" ? "u32" : "f32" });
            slots += slotsOf(stored);
            // in the vertex stage the model's matrix is Bevy's (the instance's); the fragment reads three's
            map.set(u.name, modelWorld.has(node) ? { vertex: "three_model(tsl_instance)", fragment: access } : { vertex: access, fragment: access });
          }
        }
        members.set(x.name, map);
      } else if (x.isSampledTexture) {
        if (x.isSampledCubeTexture || x.isSampledTexture3D) throw new Unsupported(`texture ${x.name} is a cube or 3D texture (raea's shaders take 2D pictures)`);
        const t = x.texture;
        const src = srcOf(t);
        if (!t || !src) throw new Unsupported(`texture ${x.name} has no picture raea can load (a render target, a video, a canvas it can't read)`);
        textures.push({ name: x.name, texture: t });
        if (t.flipY) flipped.add(`tex_${x.name}`);
      } else if (x.isSampler) {
        // its texture's: samp_NAME
      } else if (x.isBuffer && (x as unknown as { nodeUniform?: { array?: unknown[] } }).nodeUniform?.array) {
        // a uniformArray (lettra's scramble pool): its elements packed one after another, read by index (u_NAME(i))
        const n = (x as unknown as { nodeUniform: { id: number; array: unknown[]; elementType: string } }).nodeUniform;
        const type = ELEMENT[n.elementType];
        if (!type) throw new Unsupported(`uniform array ${x.name} holds ${n.elementType}s, which raea's uniforms don't carry`);
        const name = `buf${n.id}`;
        uniforms.push({ name, type, array: n.array.length, read: () => n.array, stored: "f32" });
        slots += slotsOf(type, n.array.length);
        buffers.push({ wgsl: `NodeBuffer_${n.id}`, name });
      } else throw new Unsupported(`binding ${x.name} is a ${x.isStorageBuffer ? "storage buffer" : x.isBuffer ? "uniform array (uniformArray)" : (x as object).constructor.name}, which raea's shaders don't carry`);
    }
  }
  if (slots > SLOTS) throw new Unsupported(`its uniforms take ${slots} vec4 slots, more than raea's ${SLOTS}`);
  if (textures.length > TEXTURES) throw new Unsupported(`it samples ${textures.length} textures, more than raea's ${TEXTURES}`);

  const common = (src: string, stage: "vertex" | "fragment") => {
    let s = src.replace(/^\s*diagnostic\s*\([^)]*\)\s*;\s*$/gm, "");
    // three's bindings go: the prelude binds the engine's
    const groupTypes = new Set<string>();
    s = s.replace(/@binding\(\s*\d+\s*\)\s*@group\(\s*\d+\s*\)\s*var(?:<[^>]*>)?\s+(\w+)\s*:\s*([^;]+);/g, (_all, name: string, type: string) => {
      if (members.has(name)) groupTypes.add(type.trim());
      return "";
    });
    for (const b of buffers) {
      s = s.replace(new RegExp(`struct\\s+${b.wgsl}Struct\\s*\\{[^}]*\\};?`, "g"), "");
      s = s.replace(new RegExp(`var<uniform>\\s+${b.wgsl}\\s*:[^;]*;`, "g"), "");
      s = indexed(s, `${b.wgsl}.value`, (i) => `u_${b.name}(u32( ${i} ))`);
    }
    for (const t of groupTypes) s = s.replace(new RegExp(`struct\\s+${t}\\s*\\{[^}]*\\};?`, "g"), "");
    for (const [group, map] of members) s = s.replace(new RegExp(`(?<![A-Za-z0-9_.])${group}\\.(\\w+)(?![A-Za-z0-9_])`, "g"), (all, m: string) => map.get(m)?.[stage] ?? all);
    for (const { name } of textures) {
      s = word(s, `${name}_sampler`, `samp_${name}`);
      s = word(s, name, `tex_${name}`);
    }
    s = flipSamples(s, flipped);
    s = word(s, "VaryingsStruct", "ThreeVaryings");
    return word(s, "builtinClipSpace", "clip");
  };

  // ---- the vertex stage ----
  vertex = common(vertex, "vertex");
  const ve = vertex.match(/@vertex\s*fn\s+main\s*\(([\s\S]*?)\)\s*->\s*ThreeVaryings\s*\{/);
  if (!ve) throw new Unsupported("three's vertex stage has no main returning its varyings");
  const vlets: string[] = ["\ttsl_instance = v.instance;"];
  const own: string[] = [];
  for (const p of params(ve[1])) {
    if (p.builtin === "instance_index") vlets.push(`\tlet ${p.name} = v.instance;`);
    else if (p.location !== undefined) {
      const a = ATTRIBUTES[p.name];
      if (a) vlets.push(`\tlet ${p.name} = ${fit(`v.${a.field}`, a.type, p.type)};`);
      else {
        // one of the mesh's own (lettra's layoutX, glyphIndex, cellUv): a0..a3, four floats each
        if (own.length === ATTRIBUTES_OWN) throw new Unsupported(`it reads more than ${ATTRIBUTES_OWN} vertex attributes of the mesh's own (${[...own, p.name].join(", ")})`);
        vlets.push(`\tlet ${p.name} = ${fit(`v.a${own.length}`, "vec4<f32>", p.type)};`);
        own.push(p.name);
      }
    } else throw new Unsupported(`its vertex stage reads the builtin ${p.builtin}`);
  }
  vertex = vertex.replace(ve[0], `fn three_vertex(v: ThreeVertex) -> ThreeVaryings {\n${vlets.join("\n")}\n`);
  vertex = "var<private> tsl_instance : u32;\n" + (flipped.size ? FLIP : "") + vertex;
  const varyings = vertex.match(/struct\s+ThreeVaryings\s*\{[^}]*\};?/)?.[0];
  if (!varyings) throw new Unsupported("three's vertex stage declares no varyings");

  // ---- the fragment stage ----
  fragment = common(fragment, "fragment");
  const fe = fragment.match(/@fragment\s*fn\s+main\s*\(([\s\S]*?)\)\s*->\s*(\w+)\s*\{/);
  if (!fe) throw new Unsupported("three's fragment stage has no main");
  const outs = structs(fragment).get(fe[2]);
  const colour = fragment.match(new RegExp(`struct\\s+${fe[2]}\\s*\\{[^}]*?@location\\(\\s*0\\s*\\)\\s*(\\w+)`))?.[1];
  if (!outs || !colour) throw new Unsupported("three's fragment stage writes no colour");
  const flets: string[] = [];
  for (const p of params(fe[1])) {
    if (p.location !== undefined) flets.push(`\tlet ${p.name} = in.${p.name};`);
    else if (p.builtin === "front_facing") flets.push(`\tlet ${p.name} = front;`);
    else if (p.builtin === "position") flets.push(`\tlet ${p.name} = in.clip;`);
    else throw new Unsupported(`its fragment stage reads the builtin ${p.builtin}`);
  }
  fragment = fragment.replace(fe[0], `fn three_fragment(in: ThreeVaryings, front: bool) -> vec4<f32> {\n${flets.join("\n")}\n`);
  fragment = fragment.replace(/return\s+(\w+)\s*;/g, (all, v: string) => (fragment.includes(`var<private> ${v} : ${fe[2]}`) ? `return ${v}.${colour};` : all));
  fragment = varyings + "\n" + (flipped.size ? FLIP : "") + fragment;

  return { vertex, fragment, uniforms, textures, attributes: own };
}

/** each `name[ EXPR ]` in `src` (EXPR to its own closing bracket) as `to(EXPR)` */
function indexed(src: string, name: string, to: (i: string) => string): string {
  let out = "";
  let i = 0;
  const re = new RegExp(`(?<![A-Za-z0-9_])${name.replace(/[.]/g, "\\.")}\\s*\\[`, "g");
  let m: RegExpExecArray | null;
  while ((m = re.exec(src))) {
    let depth = 0, j = m.index + m[0].length;
    for (; j < src.length; j++) {
      if (src[j] === "[" || src[j] === "(") depth++;
      else if (src[j] === "]" || src[j] === ")") {
        if (depth === 0) break;
        depth--;
      }
    }
    out += src.slice(i, m.index) + to(src.slice(m.index + m[0].length, j).trim());
    i = j + 1;
    re.lastIndex = i;
  }
  return out + src.slice(i);
}

/** an entry point's parameters: `@location( 0 ) uv : vec2<f32>`, `@builtin( front_facing ) isFront : bool` */
function params(list: string): { name: string; type: string; location?: number; builtin?: string }[] {
  return list
    .split(/,(?=\s*@)/)
    .map((p) => p.trim())
    .filter(Boolean)
    .map((p) => {
      const m = p.match(/^((?:@\w+\s*\([^)]*\)\s*)+)(\w+)\s*:\s*(.+)$/);
      if (!m) throw new Unsupported(`an entry parameter three wrote as ${JSON.stringify(p)}`);
      const loc = m[1].match(/@location\(\s*(\d+)\s*\)/);
      const bi = m[1].match(/@builtin\(\s*(\w+)\s*\)/);
      return { name: m[2], type: m[3].trim(), location: loc ? Number(loc[1]) : undefined, builtin: bi?.[1] };
    });
}

// ---- the values, this frame ----

function flat(v: unknown, type: string): number[] {
  if (typeof v === "number") return [v];
  if (typeof v === "boolean") return [v ? 1 : 0];
  if (v instanceof THREE.Color) return type === "vec4<f32>" ? [v.r, v.g, v.b, 1] : [v.r, v.g, v.b];
  if (v instanceof THREE.Matrix4 || v instanceof THREE.Matrix3) return [...v.elements];
  if (v && typeof (v as { toArray?: unknown }).toArray === "function") return (v as THREE.Vector4).toArray();
  if (Array.isArray(v)) return v.flatMap((x) => flat(x, type));
  throw new Unsupported(`a uniform's value ${String(v)} isn't a number, vector, colour or matrix`);
}

/** the graph's uniforms as three's node updates leave them for this object, camera and time */
function values(c: Compiled, m: THREE.Material, at: NodeAt): ShaderUniform[] {
  const f = c.frame as unknown as Record<string, unknown> & { updateNode(n: unknown): void };
  f.renderer = standIn();
  f.scene = at.scene;
  f.camera = at.camera;
  f.object = at.object;
  f.material = m;
  f.time = at.time;
  f.deltaTime = at.delta ?? 0;
  (f as unknown as { frameId: number }).frameId++;
  (f as unknown as { renderId: number }).renderId++;
  for (const n of c.builder.updateNodes) f.updateNode(n);
  return c.uniforms.map((u) => {
    const value = flat(u.read(), u.type);
    return { name: u.name, type: u.type, value, ...(u.array ? { array: u.array } : {}) };
  });
}

const warned = new Set<string>();

/** a node material as raea's ThreeShaderMaterial, or undefined with the reason named once (then drawn flat) */
export function nodeMaterialOf(m: THREE.Material, at: NodeAt): Part | undefined {
  try {
    const c = compile(m, at);
    const vals = values(c, m, at);
    const props: Props = {
      // <ThreeShaderMaterial wgsl>: the contract's two stages and its uniforms' types; this frame's values apart
      wgsl: { vertex: c.vertex, fragment: c.fragment, uniforms: vals.map(({ name, type, array }) => ({ name, type, ...(array ? { array } : {}) })), ...(c.attributes.length ? { attributes: c.attributes } : {}) },
      uniforms: Object.fromEntries(vals.map((u) => [u.name, u.value])),
      textures: Object.fromEntries(c.textures.map(({ name, texture }) => [name, { src: srcOf(texture)!, colorSpace: texture.colorSpace === THREE.SRGBColorSpace ? "srgb" : "linear" }])),
      // three's node materials write linear colour (the renderer's output pass tone-maps and encodes), as raea's do
      output: "linear",
      transparent: m.transparent,
      depthWrite: m.depthWrite,
      side: m.side === THREE.BackSide ? "back" : m.side === THREE.DoubleSide ? "double" : "front",
      blending: m.blending === THREE.AdditiveBlending ? "add" : m.blending === THREE.MultiplyBlending ? "multiply" : m.blending === THREE.NoBlending ? "none" : "normal",
    };
    if (m.alphaTest > 0) props.alphaTest = m.alphaTest;
    return { type: "ThreeShaderMaterial", props };
  } catch (e) {
    if (!(e instanceof Unsupported)) throw e;
    const k = `${m.type}:${e.message}`;
    if (!warned.has(k)) {
      warned.add(k);
      console.warn(`raea/r3f: ${m.type} "${m.name || m.uuid.slice(0, 8)}" can't run in raea's engine: ${e.message}; drawn flat in its colour`);
    }
    return undefined;
  }
}

/** a node material with no node of its own (a MeshStandardNodeMaterial as three's MeshStandardMaterial): the classic
 *  material's type, which raea draws natively, lit by its own lights */
export function classicOf(m: THREE.Material): string | undefined {
  const own = m as unknown as Record<string, unknown>;
  const custom = Object.keys(own).some((k) => k.endsWith("Node") && own[k] != null && (own[k] as AnyNode).isNode);
  if (custom) return undefined;
  const t = m.type.replace("NodeMaterial", "Material");
  return t !== m.type ? t : undefined;
}
