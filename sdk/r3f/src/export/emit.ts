// A recording (record.tsx) written as React Three Fiber source (docs/r3f.feature R3F9): one TSX module that needs only
// react, three and @react-three/fiber. `Scene` is the 3D (for any <Canvas>); the default export is a <Canvas> with
// raea's camera, tone mapping and shadows. What moves is a table of frames applied in useFrame at the recording's fps;
// what holds still is plain JSX.
import * as THREE from "three";
import { DRAWN_EFFECTS } from "../three/effects";
import { EXPOSURE } from "../units";
import { r5, SRGB_MAPS, type Rec, type Recording } from "./record";

const PRIMS = new Set(["BoxGeometry", "SphereGeometry", "PlaneGeometry", "CylinderGeometry", "ConeGeometry", "TorusGeometry", "TorusKnotGeometry", "CapsuleGeometry", "CircleGeometry", "RingGeometry", "IcosahedronGeometry", "OctahedronGeometry", "TetrahedronGeometry", "DodecahedronGeometry"]);
const TAGS: Record<string, string> = {
  Group: "group", Object3D: "group", Scene: "group", Mesh: "mesh", InstancedMesh: "instancedMesh", Points: "points", Line: "line",
  LineSegments: "lineSegments", LineLoop: "lineLoop", Sprite: "sprite", AmbientLight: "ambientLight", DirectionalLight: "directionalLight",
  PointLight: "pointLight", SpotLight: "spotLight", RectAreaLight: "rectAreaLight", HemisphereLight: "hemisphereLight",
};
const camel = (t: string) => t[0].toLowerCase() + t.slice(1);
const same = <T,>(xs: T[]) => xs.every((x) => JSON.stringify(x) === JSON.stringify(xs[0]));
const j = (v: unknown) => JSON.stringify(v);

/** a geometry three makes from numbers (BoxGeometry ...): its JSX, else undefined */
function primitive(g: THREE.BufferGeometry): string | undefined {
  const params = (g as unknown as { parameters?: Record<string, unknown> }).parameters;
  if (!PRIMS.has(g.type) || !params || !Object.values(params).every((v) => typeof v === "number" || typeof v === "boolean")) return undefined;
  return `<${camel(g.type)} args={${j(Object.values(params).map((v) => (typeof v === "number" ? r5(v) : v)))}} />`;
}

function geometryData(g: THREE.BufferGeometry): string {
  const a: Record<string, unknown> = {};
  for (const k of ["position", "normal", "uv", "color"]) {
    const at = g.getAttribute(k) as THREE.BufferAttribute | undefined;
    if (at) a[k] = [at.itemSize, Array.from(at.array as ArrayLike<number>, r5)];
  }
  if (g.index) a.index = Array.from(g.index.array as ArrayLike<number>);
  if (g.groups.length) a.groups = g.groups.map((x) => [x.start, x.count, x.materialIndex ?? 0]);
  return `geo(${j(a)})`;
}

/** a material's settings as JSX attributes: textures through tex() */
function attrs(props: Record<string, unknown>): string {
  return Object.entries(props)
    .filter(([k]) => k !== "uniforms")
    .map(([k, v]) => {
      if (v && typeof v === "object" && "tex" in (v as object)) return `${k}={tex(${j(v)}, ${SRGB_MAPS.has(k)})}`;
      return typeof v === "string" ? `${k}=${j(v)}` : `${k}={${j(v)}}`;
    })
    .join(" ");
}

const toneName = (t: THREE.ToneMapping) => Object.entries(THREE).find(([k, v]) => k.endsWith("ToneMapping") && v === t)?.[0] ?? "AgXToneMapping";

export function emit(rec: Recording, name: string, title: string): string {
  const { recs } = rec;
  const N = rec.frames;
  const targets = new Set(recs.flatMap((r) => ((r.obj as THREE.DirectionalLight).target ? [(r.obj as THREE.DirectionalLight).target] : [])));
  const kids = new Map<number, Rec[]>();
  for (const r of recs) if (!targets.has(r.obj)) kids.set(r.parent, [...(kids.get(r.parent) ?? []), r]);
  const G: string[] = [], gIndex = new Map<number, number>();
  const gref = (i: number) => {
    if (!gIndex.has(i)) gIndex.set(i, G.push(geometryData(rec.geometries[i])) - 1);
    return gIndex.get(i)!;
  };
  const tracks: Record<number, Record<string, unknown>> = {};
  const models: Record<number, unknown[]> = {};
  const uses = { rect: false, sky: false, model: false };
  const track = (r: Rec, k: string, xs: unknown[]) => {
    if (xs.length === N && !same(xs)) (tracks[r.id] ??= {})[k] = xs;
  };

  const node = (r: Rec, pad: string): string => {
    const o = r.obj, t = o.type;
    const m0 = r.mat[0] ? (JSON.parse(r.mat[0]) as { type: string; props: Record<string, unknown> }[]) : [];
    track(r, "p", r.p.map((x) => x ?? r.p.find(Boolean)));
    track(r, "q", r.q.map((x) => x ?? r.q.find(Boolean)));
    track(r, "s", r.s.map((x) => x ?? r.s.find(Boolean)));
    track(r, "v", Array.from({ length: N }, (_x, i) => !!r.v[i]));
    if (r.mat.length) track(r, "m", r.mat.map((x) => JSON.parse(x ?? r.mat.find(Boolean)!)));
    if (r.light.length) track(r, "l", r.light.map((x) => JSON.parse(x ?? r.light.find(Boolean)!)));
    if (r.inst.length) track(r, "i", r.inst.map((x) => x ?? r.inst.find(Boolean)));
    if (r.geo.length && !same(r.geo.filter((x) => x !== undefined))) track(r, "g", r.geo.map((x) => gref(x ?? r.geo.find((y) => y !== undefined)!)));
    const p0 = r.p.find(Boolean)!, q0 = r.q.find(Boolean)!, s0 = r.s.find(Boolean)!;
    const place = [
      `position={${j(p0)}}`,
      q0[3] !== 1 ? `quaternion={${j(q0)}}` : "",
      s0.some((x) => x !== 1) ? `scale={${j(s0)}}` : "",
      r.v[0] === false ? "visible={false}" : "",
      o.name ? `name=${j(o.name)}` : "",
      `ref={(o: THREE.Object3D | null) => { if (o) O[${r.id}] = o; }}`,
    ].filter(Boolean).join(" ");
    const inner = (kids.get(r.id) ?? []).map((k) => node(k, pad + "  "));
    if (r.model.length) {
      uses.model = true;
      models[r.id] = r.model.map((x) => JSON.parse(x ?? r.model.find(Boolean)!));
      const src = (models[r.id][0] as { src: string }).src;
      return `${pad}<Suspense fallback={null}><Model ${place} src=${j(src)} states={MODELS[${r.id}]} /></Suspense>`;
    }
    if (t === "Sky" || (o as unknown as { isSky?: boolean }).isSky) {
      uses.sky = true;
      const u = m0[0]?.props.uniforms ?? {};
      (tracks[r.id] ??= {}).u = r.mat.length && !same(r.mat) ? r.mat.map((x) => JSON.parse(x ?? r.mat[0])[0].props.uniforms) : [u];
      return `${pad}<primitive object={SKIES[${r.id}] ??= new Sky()} ${place} />`;
    }
    const light = r.light[0] ? (JSON.parse(r.light[0]) as Record<string, unknown>) : undefined;
    const tag = TAGS[t] ?? "group";
    const a: string[] = [place];
    if ((o as THREE.Mesh).isMesh || (o as THREE.Points).isPoints) a.push(o.castShadow ? "castShadow" : "", o.receiveShadow ? "receiveShadow" : "");
    if ((o as THREE.InstancedMesh).isInstancedMesh) {
      a.push(`args={[undefined, undefined, ${(o as THREE.InstancedMesh).count}]}`, `onUpdate={(m: THREE.InstancedMesh) => instances(m, ${j(r.inst[0])})}`);
    }
    if (light) {
      if (t === "RectAreaLight") uses.rect = true;
      const { target, shadow, ...rest } = light as { target?: number[]; shadow?: { mapSize: number[]; bias: number; normalBias: number; camera: Record<string, number> } };
      a.push(attrs(rest));
      if (target) a.push(`target-position={${j(target)}}`, `onUpdate={(l: THREE.DirectionalLight) => l.target.updateMatrixWorld()}`);
      if (shadow) {
        a.push(`shadow-mapSize={${j(shadow.mapSize)}} shadow-bias={${shadow.bias}} shadow-normalBias={${shadow.normalBias}}`);
        for (const [k, v] of Object.entries(shadow.camera)) a.push(`shadow-camera-${k}={${v}}`);
      }
    }
    const parts: string[] = [];
    const geo = (o as THREE.Mesh).geometry;
    if (geo && r.geo.length) {
      const prim = !tracks[r.id]?.g ? primitive(geo) : undefined;
      parts.push(prim ?? `<primitive object={G[${gref(r.geo.find((x) => x !== undefined)!)}]} attach="geometry" />`);
    }
    m0.forEach((m, i) => {
      if ((THREE as unknown as Record<string, unknown>)[m.type] === undefined || m.type === "ShaderMaterial" || m.type === "RawShaderMaterial") return;
      parts.push(`<${camel(m.type)} ${m0.length > 1 ? `attach={"material-${i}"} ` : ""}${attrs(m.props)} />`);
    });
    const body = [...parts.map((x) => pad + "  " + x), ...inner];
    return body.length ? `${pad}<${tag} ${a.filter(Boolean).join(" ")}>\n${body.join("\n")}\n${pad}</${tag}>` : `${pad}<${tag} ${a.filter(Boolean).join(" ")} />`;
  };

  const tree = (kids.get(-1) ?? []).map((r) => node(r, "      ")).join("\n");
  const c = rec.camera, s = rec.settings;
  const camTrack = { p: same(c.p) ? undefined : c.p, q: same(c.q) ? undefined : c.q, fov: c.fov.length && !same(c.fov) ? c.fov : undefined, height: c.height.length && !same(c.height) ? c.height : undefined };
  const exposure = r5(s.exposure * (s.toneMapping === THREE.ACESFilmicToneMapping ? 0.6 : 1));
  const env = rec.environment as { src: string; intensity: number; rotation: number; background: boolean; backgroundIntensity: number } | undefined;
  // raea's frame effects and Tony are drawn by raea-effects.ts, the composer <Raea> uses (three and its examples only)
  const kinds = [...new Set(rec.effects.flat().map((e) => e.kind))];
  const drawn = kinds.filter((k) => DRAWN_EFFECTS.includes(k));
  const undrawn = kinds.filter((k) => !DRAWN_EFFECTS.includes(k));
  const fx = s.tony || drawn.length > 0;
  const notes = [
    fx ? `raea's ${[s.tony ? "tone mapping (TonyMcMapface, from Bevy's LUT)" : "", drawn.join(", ")].filter(Boolean).join(" and ")} drawn by ./raea-effects.ts, the composer raea's <Raea> uses.` : "",
    undrawn.length ? `${undrawn.join(", ")}: raea's engine draws these and three.js doesn't (sdk/r3f/src/three/kinds.ts).` : "",
  ].filter(Boolean);
  const fxConsts = fx
    ? `/** the frame effects (one list, or a list a frame), the camera's focus (one, or one a frame), whether the tone mapping is Tony */
const EFFECTS: Effect[] | Effect[][] = ${j(same(rec.effects) ? rec.effects[0] : rec.effects)};
const FOCUS: number | number[] = ${j(same(rec.focus) ? rec.focus[0] : rec.focus)};
const TONY = ${s.tony};
`
    : "";
  const fxScene = fx
    ? `  // after the frame's updates (priority 1, so R3F leaves the drawing to this): through raea's composer
  const composer = useMemo(() => new Effects(gl), [gl]);
  useEffect(() => () => composer.dispose(), [composer]);
  useFrame((state, dt) => {
    const f = current;
    const list = (Array.isArray(EFFECTS[0]) ? (EFFECTS as Effect[][])[f] : EFFECTS) as Effect[];
    const focus = Array.isArray(FOCUS) ? FOCUS[f] : FOCUS;
    if (composer.set(list, state.scene, state.camera, size.width, size.height, focus, TONY)) composer.render(state.scene, dt);
    else state.gl.render(state.scene, state.camera);
  }, 1);
`
    : "";
  const loader = env ? (/\.exr$/i.test(env.src) ? ["EXRLoader", "three/examples/jsm/loaders/EXRLoader.js"] : /\.hdr$/i.test(env.src) ? ["HDRLoader", "three/examples/jsm/loaders/HDRLoader.js"] : ["TextureLoader", ""]) : undefined;

  const TONE = s.tony ? "AgXToneMapping" : toneName(s.toneMapping);
  return `// ${title}: written by raea's source export (raea/r3f, docs/r3f.feature R3F9), ${N} frame${N > 1 ? "s" : ""} at ${rec.fps} fps, ${rec.width}x${rec.height}.
// Needs only react, three and @react-three/fiber. <Scene /> goes in any <Canvas>; the default export is a <Canvas> set
// up as raea's camera, tone mapping and shadows. Frames play at FPS from mount, looping, unless \`frame\` pins one.
${notes.map((n) => `// ${n}`).join("\n")}
import { Canvas, useFrame, useLoader, useThree } from "@react-three/fiber";
import { Suspense, ${fx ? "useEffect, " : ""}useLayoutEffect, useMemo } from "react";
import * as THREE from "three";
${fx ? `import { Effects, type Effect } from "./raea-effects";\n` : ""}${uses.model ? `import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";\nimport { clone } from "three/examples/jsm/utils/SkeletonUtils.js";\n` : ""}${uses.rect ? `import { RectAreaLightUniformsLib } from "three/examples/jsm/lights/RectAreaLightUniformsLib.js";\n` : ""}${uses.sky ? `import { Sky } from "three/examples/jsm/objects/Sky.js";\n` : ""}${loader?.[1] ? `import { ${loader[0]} } from "${loader[1]}";\n` : ""}
const FPS = ${rec.fps};
const FRAMES = ${N};
${fxConsts}${uses.rect ? "RectAreaLightUniformsLib.init();\n" : ""}
function geo(a: Record<string, unknown>): THREE.BufferGeometry {
  const g = new THREE.BufferGeometry();
  for (const k of ["position", "normal", "uv", "color"]) {
    const v = a[k] as [number, number[]] | undefined;
    if (v) g.setAttribute(k, new THREE.Float32BufferAttribute(v[1], v[0]));
  }
  if (a.index) g.setIndex(a.index as number[]);
  for (const [start, count, m] of (a.groups as number[][] | undefined) ?? []) g.addGroup(start, count, m);
  return g;
}

const textures = new Map<string, THREE.Texture>();
function tex(t: { tex: string; repeat?: number[]; offset?: number[]; rotation?: number; wrap?: number[]; nearest?: boolean; flipY?: boolean }, srgb: boolean): THREE.Texture {
  const key = JSON.stringify([t, srgb]);
  let x = textures.get(key);
  if (x) return x;
  if (/\\.(mp4|webm|mov)$/i.test(t.tex)) {
    const v = Object.assign(document.createElement("video"), { src: t.tex, muted: true, loop: true, playsInline: true, crossOrigin: "anonymous" });
    void v.play();
    x = new THREE.VideoTexture(v);
  } else x = new THREE.TextureLoader().load(t.tex);
  if (srgb) x.colorSpace = THREE.SRGBColorSpace;
  if (t.repeat) x.repeat.fromArray(t.repeat);
  if (t.offset) x.offset.fromArray(t.offset);
  if (t.rotation) x.rotation = t.rotation;
  if (t.wrap) [x.wrapS, x.wrapT] = t.wrap as [THREE.Wrapping, THREE.Wrapping];
  if (t.nearest) x.magFilter = x.minFilter = THREE.NearestFilter;
  if (t.flipY === false) x.flipY = false;
  textures.set(key, x);
  return x;
}

function instances(m: THREE.InstancedMesh, matrices: number[]) {
  (m.instanceMatrix.array as Float32Array).set(matrices);
  m.instanceMatrix.needsUpdate = true;
}

/** a material or light set from its frame's settings (colours as hex, textures through tex) */
function set(o: Record<string, unknown>, props: Record<string, unknown>) {
  for (const [k, v] of Object.entries(props)) {
    if (k === "target" && Array.isArray(v)) {
      (o.target as THREE.Object3D).position.fromArray(v);
      (o.target as THREE.Object3D).updateMatrixWorld();
    } else if (k === "shadow" || k === "uniforms") continue;
    else if (o[k] instanceof THREE.Color) (o[k] as THREE.Color).set(v as string);
    else if (v && typeof v === "object" && "tex" in v) o[k] = tex(v as never, k === "map" || k === "emissiveMap");
    else o[k] = v;
  }
  if ((o as unknown as THREE.Material).isMaterial) (o as unknown as THREE.Material).needsUpdate = true;
}

const G: THREE.BufferGeometry[] = [
${G.map((g) => "  " + g + ",").join("\n")}
];
/** what changes from frame to frame, by object: p position, q quaternion, s scale, v visible, m materials, l light, i instances, g geometry, u sky uniforms */
const TRACKS: Record<number, { p?: number[][]; q?: number[][]; s?: number[][]; v?: boolean[]; m?: { props: Record<string, unknown> }[][]; l?: Record<string, unknown>[]; i?: number[][]; g?: number[]; u?: Record<string, unknown>[] }> = ${j(tracks)};
const CAMERA: { p?: number[][]; q?: number[][]; fov?: number[]; height?: number[]; p0: number[]; q0: number[]; fov0?: number; height0?: number } = ${j({ ...camTrack, p0: c.p[0], q0: c.q[0], fov0: c.fov[0], height0: c.height[0] })};
${uses.model ? `const MODELS: Record<number, ModelState[]> = ${j(models)};\n` : ""}const O: THREE.Object3D[] = [];
${uses.sky ? "const SKIES: Record<number, Sky> = {};\n" : ""}
function pose(f: number) {
  for (const [id, t] of Object.entries(TRACKS)) {
    const o = O[Number(id)];
    if (!o) continue;
    if (t.p) o.position.fromArray(t.p[f]);
    if (t.q) o.quaternion.fromArray(t.q[f]);
    if (t.s) o.scale.fromArray(t.s[f]);
    if (t.v) o.visible = t.v[f];
    if (t.g) (o as THREE.Mesh).geometry = G[t.g[f]];
    if (t.m) {
      const mats = (o as THREE.Mesh).material;
      t.m[f].forEach((m, i) => set((Array.isArray(mats) ? mats[i] : mats) as never, m.props));
    }
    if (t.l) set(o as never, t.l[f]);
    if (t.i) instances(o as THREE.InstancedMesh, t.i[f]);
    if (t.u) for (const [k, v] of Object.entries(t.u[Math.min(f, t.u.length - 1)])) {
      const u = ((o as THREE.Mesh).material as THREE.ShaderMaterial).uniforms[k];
      if (!u) continue;
      if (u.value?.fromArray) u.value.fromArray(v as number[]);
      else if (u.value instanceof THREE.Color) u.value.set(v as string);
      else u.value = v;
    }
  }
}
${uses.model ? MODEL : ""}${env ? envComponent(loader![0], env) : ""}
export function Scene({ frame }: { frame?: number }) {
  const { camera, size, gl } = useThree();
  // raea's tone mapping, exposure and shadows on whichever canvas this is in
  useLayoutEffect(() => {
    gl.toneMapping = THREE.${TONE};
    gl.toneMappingExposure = ${exposure};
    gl.shadowMap.enabled = ${s.shadows};
    gl.shadowMap.type = THREE.PCFSoftShadowMap;
  }, [gl]);
  useFrame((state) => {
    const f = frame ?? Math.floor(state.clock.elapsedTime * FPS) % FRAMES;
    current = f;
    pose(f);
    const p = CAMERA.p?.[f] ?? CAMERA.p0, q = CAMERA.q?.[f] ?? CAMERA.q0;
    camera.position.fromArray(p);
    camera.quaternion.fromArray(q);
    if ((camera as THREE.PerspectiveCamera).isPerspectiveCamera) (camera as THREE.PerspectiveCamera).fov = CAMERA.fov?.[f] ?? CAMERA.fov0 ?? 50;
    else (camera as THREE.OrthographicCamera).zoom = size.height / (CAMERA.height?.[f] ?? CAMERA.height0 ?? 10);
    (camera as THREE.PerspectiveCamera).near = ${c.near};
    (camera as THREE.PerspectiveCamera).far = ${c.far};
    camera.updateProjectionMatrix();
  }, -1);
${fxScene}  return (
    <>
${rec.background ? `      <color attach="background" args={[${j(rec.background)}]} />\n` : ""}${rec.fog ? (rec.fog.density !== undefined ? `      <fogExp2 attach="fog" args={[${j(rec.fog.color)}, ${rec.fog.density}]} />\n` : `      <fog attach="fog" args={[${j(rec.fog.color)}, ${rec.fog.near}, ${rec.fog.far}]} />\n`) : ""}${env ? "      <Suspense fallback={null}><Environment /></Suspense>\n" : ""}${tree}
    </>
  );
}
let current = 0;

export default function ${name}() {
  return (
    <Canvas${s.shadows ? " shadows" : ""}${c.kind === "orthographic" ? " orthographic" : ""} gl={{ toneMapping: THREE.${s.tony ? "AgXToneMapping" : toneName(s.toneMapping)}, toneMappingExposure: ${exposure} }} camera={{ position: ${j(c.p[0])}, near: ${c.near}, far: ${c.far}${c.kind === "perspective" ? `, fov: ${c.fov[0]}` : ""} }} style={{ aspectRatio: "${rec.width} / ${rec.height}" }}>
      <Scene />
    </Canvas>
  );
}
`;
}

function envComponent(loader: string, env: { src: string; intensity: number; rotation: number; background: boolean; backgroundIntensity: number }): string {
  const k = 1000 * EXPOSURE;
  return `
/** raea's <Environment>: its panorama lighting the scene${env.background ? " and drawn behind it" : ""} (raea's 1 is 1000 nits at Bevy's exposure) */
function Environment() {
  const { scene } = useThree();
  const t = useLoader(${loader}, ${j(env.src)}) as THREE.Texture;
  useLayoutEffect(() => {
    t.mapping = THREE.EquirectangularReflectionMapping;
    scene.environment = t;
    scene.environmentIntensity = ${r5(env.intensity * k)};
    scene.environmentRotation.set(0, ${env.rotation}, 0);
${env.background ? `    scene.background = t;\n    scene.backgroundIntensity = ${r5(env.backgroundIntensity * k)};\n    scene.backgroundRotation.set(0, ${env.rotation}, 0);\n` : ""}  }, [t, scene]);
  return null;
}
`;
}

const MODEL = `
interface ModelState { src: string; time: number; loop: boolean; animation?: string | number; animations?: { clip: string | number; time?: number; weight?: number }[]; hide?: string[]; morphs?: Record<string, number> | number[]; refinish?: Record<string, unknown>[] }

/** raea's <Model>: its glTF, its clip (or blend) at the frame's time, hidden nodes, morphs and re-covered materials */
function Model({ src, states, ...rest }: { src: string; states: ModelState[] } & Record<string, unknown>) {
  const gltf = useLoader(GLTFLoader, src);
  const model = useMemo(() => clone(gltf.scene), [gltf]);
  const mixer = useMemo(() => new THREE.AnimationMixer(model), [model]);
  useMemo(() => model.traverse((o) => { if ((o as THREE.Mesh).isMesh) o.castShadow = o.receiveShadow = true; }), [model]);
  useFrame(() => {
    const p = states[Math.min(current, states.length - 1)];
    const clip = (c: string | number) => (typeof c === "number" ? gltf.animations[c] : gltf.animations.find((a) => a.name === c));
    const hide = new Set(p.hide ?? []);
    model.traverse((o) => { if (o !== model && o.name) o.visible = !hide.has(o.name); });
    mixer.stopAllAction();
    const play = (c: THREE.AnimationClip | undefined, at: number, w: number) => {
      if (!c) return;
      const a = mixer.clipAction(c);
      a.play();
      a.setEffectiveWeight(w);
      a.time = p.loop ? ((at % c.duration) + c.duration) % c.duration : Math.min(at, c.duration);
    };
    if (p.animations) for (const b of p.animations) play(clip(b.clip), b.time ?? p.time, b.weight ?? 1);
    else if (p.animation !== undefined) play(clip(p.animation), p.time, 1);
    mixer.update(0);
    model.traverse((o) => {
      const m = o as THREE.Mesh;
      if (p.morphs && m.morphTargetInfluences) {
        if (Array.isArray(p.morphs)) p.morphs.forEach((w, i) => { m.morphTargetInfluences![i] = w; });
        else for (const [k, w] of Object.entries(p.morphs)) { const i = m.morphTargetDictionary?.[k]; if (i !== undefined) m.morphTargetInfluences[i] = w; }
      }
      if (m.isMesh && p.refinish?.length) for (const mat of Array.isArray(m.material) ? m.material : [m.material]) {
        const spec = p.refinish.find((r) => r.material === mat.name) ?? p.refinish.find((r) => r.material === undefined);
        if (spec) { const { material: _n, ...props } = spec; set(mat as never, props); }
      }
    });
  });
  return <primitive object={model} {...rest} />;
}
`;
