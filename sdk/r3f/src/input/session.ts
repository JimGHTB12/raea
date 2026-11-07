// React Three Fiber run headless, a frame at a time: its own root on a stand-in canvas and renderer (raea draws; three
// never does), with frameloop "never" so nothing ticks on its own. Frame f of the composition is R3F's clock at f/fps
// seconds: the scene is rendered with frame f's children, then advance() runs its useFrame callbacks once with the
// step since the last frame. Going backwards (a seek, a scrub) starts the root again from frame 0 and steps forward, so
// a frame's scene is the same whichever frame came before it: a render equals its play (raea's first rule).
import * as THREE from "three";
import { advance, createRoot, extend, flushSync, type ReconcilerRoot, type RenderProps, type RootState } from "@react-three/fiber";
import type { ReactNode } from "react";

extend(THREE as unknown as Parameters<typeof extend>[0]);

/** the renderer R3F configures and never draws with: what it sets on it (shadows, tone mapping) is read back */
export function standInRenderer(): THREE.WebGLRenderer {
  const noop = () => {};
  return {
    render: noop, setSize: noop, setPixelRatio: noop, getPixelRatio: () => 1, dispose: noop, setClearColor: noop, compile: noop,
    domElement: { style: {}, addEventListener: noop, removeEventListener: noop },
    xr: { enabled: false, isPresenting: false, addEventListener: noop, removeEventListener: noop, setAnimationLoop: noop },
    shadowMap: { enabled: false, type: THREE.PCFShadowMap },
    outputColorSpace: THREE.SRGBColorSpace, toneMapping: THREE.NoToneMapping, toneMappingExposure: 1,
    capabilities: { isWebGL2: true, maxTextureSize: 16384 }, info: { autoReset: true, reset: noop }, properties: { get: () => ({}) },
  } as unknown as THREE.WebGLRenderer;
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
}

const sessions = new Map<string, Session>();

export type Config = Omit<RenderProps<HTMLCanvasElement>, "gl" | "frameloop" | "size" | "events" | "dpr">;

function start(key: string, width: number, height: number, config: Config, json: string): Session {
  const root = createRoot(standInCanvas(width, height));
  let state: RootState | undefined;
  void root.configure({ ...config, gl: standInRenderer(), frameloop: "never", dpr: 1, size: { width, height, top: 0, left: 0 }, events: undefined, onCreated: (s) => { state = s; config.onCreated?.(s); } });
  const s: Session = { root, state: state as unknown as RootState, frame: -1, config: json };
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
    flushSync(() => { store = live.root.render(scene(i)); });
    if (store) live.state = store.getState();
  };
  // the same frame drawn again (a live replay): its children may have changed, its clock hasn't
  if (frame === s.frame) render(frame);
  for (let i = s.frame + 1; i <= frame; i++) {
    render(i);
    advance(i / fps, true, live.state);
    live.frame = i;
  }
  return live;
}

/** the root `key` unmounted (the <R3F> left the tree, or starts again) */
export function end(key: string) {
  const s = sessions.get(key);
  if (!s) return;
  s.root.unmount();
  sessions.delete(key);
}
