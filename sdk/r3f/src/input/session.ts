// React Three Fiber run headless, a frame at a time: its own root on a stand-in canvas and renderer (raea draws; three
// never does), with frameloop "never" so nothing ticks on its own. Frame f of the composition is R3F's clock at f/fps
// seconds: the scene is rendered with frame f's children, then advance() runs its useFrame callbacks once with the
// step since the last frame. Going backwards (a seek, a scrub) starts the root again from frame 0 and steps forward, so
// a frame's scene is the same whichever frame came before it: a render equals its play (raea's first rule).
import * as THREE from "three";
import { advance, createRoot, extend, flushSync, type ReconcilerRoot, type RenderProps, type RootState } from "@react-three/fiber";
import type { ReactNode } from "react";
import { renderInto } from "./targets";

extend(THREE as unknown as Parameters<typeof extend>[0]);

/** the renderer R3F configures and never draws with: what it sets on it (shadows, tone mapping) is read back */
export function standInRenderer(): THREE.WebGLRenderer {
  const noop = () => {};
  const size = { width: 1, height: 1 };
  // a WebGL context that answers what a composer asks as it is made (@react-three/postprocessing, three's own), and
  // takes every other call as it renders (its depth blit: bindFramebuffer, blitFramebuffer) doing nothing, its
  // constants (READ_FRAMEBUFFER, DEPTH_BUFFER_BIT) 0: raea draws the effect (post.ts)
  const gl = {
    getContextAttributes: () => ({ alpha: true, depth: true, stencil: false, antialias: false, premultipliedAlpha: true }),
    getExtension: () => null, getParameter: () => 0, getSupportedExtensions: () => [], isContextLost: () => false,
  } as Record<string | symbol, unknown>;
  const context = new Proxy(gl, { get: (t, k) => (k in t ? t[k] : typeof k !== "string" || k === "then" ? undefined : /^[A-Z0-9_]+$/.test(k) ? 0 : /^[a-z]/.test(k) ? noop : undefined) });
  // a pass an app draws into a render target itself (drei's <ContactShadows>) is done on the CPU (targets.ts)
  let target: THREE.RenderTarget | null = null;
  const clear: [number, number, number, number] = [0, 0, 0, 0];
  const own = {
    render: (scene: THREE.Object3D, camera: THREE.Camera) => {
      if (target) renderInto(target, scene, camera, clear);
    },
    setRenderTarget: (t: THREE.RenderTarget | null) => void (target = t),
    getRenderTarget: () => target,
    setClearColor: (c: THREE.ColorRepresentation, a?: number) => {
      const col = new THREE.Color(c);
      clear[0] = col.r, clear[1] = col.g, clear[2] = col.b;
      if (a !== undefined) clear[3] = a;
    },
    setClearAlpha: (a: number) => void (clear[3] = a),
    getClearAlpha: () => clear[3], setSize: (w: number, h: number) => void Object.assign(size, { width: w, height: h }), setPixelRatio: noop, getPixelRatio: () => 1,
    getSize: (v: THREE.Vector2) => v.set(size.width, size.height), getDrawingBufferSize: (v: THREE.Vector2) => v.set(size.width, size.height),
    getContext: () => context, getClearColor: (c: THREE.Color) => c.setRGB(clear[0], clear[1], clear[2]),
    dispose: noop, compile: noop, autoClear: true, autoClearColor: true, autoClearDepth: true, autoClearStencil: true,
    domElement: { style: {}, addEventListener: noop, removeEventListener: noop },
    xr: { enabled: false, isPresenting: false, addEventListener: noop, removeEventListener: noop, setAnimationLoop: noop },
    shadowMap: { enabled: false, type: THREE.PCFShadowMap },
    outputColorSpace: THREE.SRGBColorSpace, toneMapping: THREE.NoToneMapping, toneMappingExposure: 1,
    capabilities: { isWebGL2: true, maxTextureSize: 16384, maxSamples: 4, logarithmicDepthBuffer: false }, info: { autoReset: true, reset: noop }, properties: { get: () => ({}) },
  } as Record<string | symbol, unknown>;
  // any other of WebGLRenderer's methods (setRenderTarget, clear, readRenderTargetPixels ...) does nothing: raea draws
  // (verbs only: a catch-all would answer `then`, and the renderer would read as a promise)
  const verb = /^(set|get|clear|read|copy|init|reset|render|dispose|compile|update|bind)[A-Z]?/;
  return new Proxy(own, { get: (t, k) => (k in t ? t[k] : typeof k === "string" && verb.test(k) ? noop : undefined) }) as unknown as THREE.WebGLRenderer;
}

function standInCanvas(width: number, height: number): HTMLCanvasElement {
  const noop = () => {};
  return { style: {}, width, height, addEventListener: noop, removeEventListener: noop, getBoundingClientRect: () => ({ left: 0, top: 0, width, height, x: 0, y: 0, right: width, bottom: height }) } as unknown as HTMLCanvasElement;
}

export interface Session {
  root: ReconcilerRoot<HTMLCanvasElement>;
  state: RootState;
  /** the last frame stepped */
  frame: number;
  /** the settings it was configured with, as JSON: a change starts it again */
  config: string;
  /** its useFrame callbacks when it last stepped: more on a redraw means part of it mounted late (it suspended) */
  subs: number;
  /** its Math.random's state: seeded as it starts, so a root started again draws the same numbers */
  rand: number;
}

/** Math.random as `s`'s own generator (mulberry32) while `fn` runs: drei's <Float> phase, <Sparkles>, three's uuids
 *  come out the same each time the root is stepped from frame 0, so a render after a seek equals its play */
function seeded<T>(s: Session, fn: () => T): T {
  const was = Math.random;
  Math.random = () => {
    let t = (s.rand = (s.rand + 0x6d2b79f5) | 0);
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
  try {
    return fn();
  } finally {
    Math.random = was;
  }
}

const seedOf = (key: string) => [...key].reduce((h, c) => Math.imul(h ^ c.charCodeAt(0), 16777619), 2166136261) | 0;

const sessions = new Map<string, Session>();

export type Config = Omit<RenderProps<HTMLCanvasElement>, "gl" | "frameloop" | "size" | "events" | "dpr">;

function start(key: string, width: number, height: number, config: Config, json: string): Session {
  const root = createRoot(standInCanvas(width, height));
  let state: RootState | undefined;
  void root.configure({ ...config, gl: standInRenderer(), frameloop: "never", dpr: 1, size: { width, height, top: 0, left: 0 }, events: undefined, onCreated: (s) => { state = s; config.onCreated?.(s); } });
  const s: Session = { root, state: state as unknown as RootState, frame: -1, config: json, subs: 0, rand: seedOf(key) };
  sessions.set(key, s);
  return s;
}

/** The root `key` at local frame `frame` (fps a second), each step rendering `scene(i)` first. */
export function stepTo(key: string, frame: number, fps: number, width: number, height: number, config: Config, scene: (i: number) => ReactNode): Session {
  const json = JSON.stringify({ width, height, config }, (_k, v) => (typeof v === "function" ? String(v) : v instanceof THREE.Object3D ? v.uuid : v));
  let s = sessions.get(key);
  if (s && (frame < s.frame || s.config !== json)) {
    end(key);
    s = undefined;
  }
  s ??= start(key, width, height, config, json);
  const live = s;
  const render = (i: number) => {
    let store: ReturnType<typeof live.root.render> | undefined;
    seeded(live, () => flushSync(() => { store = live.root.render(scene(i)); }));
    if (store) live.state = store.getState();
  };
  // the same frame drawn again (a live replay): its children may have changed, its clock hasn't. When parts of it
  // mounted only now (a model, a font: what suspended while the frames were stepped, loaded since), their useFrame
  // never ran: the root starts again, and with the loads cached it mounts at once and steps from frame 0 as if they
  // had been there all along (as three's own page waits for its loads and then draws the frame)
  if (frame === s.frame) {
    render(frame);
    if (subsOf(live) > live.subs && !late.has(key)) {
      late.add(key);
      end(key);
      return stepTo(key, frame, fps, width, height, config, scene);
    }
  }
  for (let i = s.frame + 1; i <= frame; i++) {
    render(i);
    seeded(live, () => advance(i / fps, true, live.state));
    live.frame = i;
    live.subs = subsOf(live);
  }
  return live;
}

/** roots started again for parts that mounted late: once each, so one whose callbacks keep growing can't loop */
const late = new Set<string>();

const subsOf = (s: Session) => (s.state as unknown as { internal?: { subscribers?: unknown[] } }).internal?.subscribers?.length ?? 0;

/** the root `key` unmounted (the <R3F> left the tree, or starts again) */
export function end(key: string) {
  const s = sessions.get(key);
  if (!s) return;
  s.root.unmount();
  sessions.delete(key);
}
