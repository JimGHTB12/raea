// raea in a browser (docs/web-export.md M3): a registered composition played live in a page, its React in the
// browser's own engine and its page as real DOM, its 3D drawn by raea's own engine (raea-web: raea-render on wasm32 and
// WebGPU) into a canvas under the page. A Clock is stepped 1/fps at a time off requestAnimationFrame with the person's
// input recorded per frame (useInput reads it), the frame's data-3d parts are collected and handed to the engine, and
// only what changed crosses (collect.ts). The same edit.tsx renders natively with `raea render`.
import { flushSync } from "react-dom";
import { createRoot } from "react-dom/client";
import { Clock, getCompositions, type InputTimeline, type RegisteredComposition } from "raea";
import { Input } from "@raea/playable/host/input";
import { Collector, UNDER3D_CSS, clearUnder3d } from "./collect";

/** raea-web's engine, as wasm-bindgen exports it (engine/web: `import init, { Raea } from "./raea_web.js"`). */
export interface RaeaEngine {
  frame(scene: string, clear: string | undefined, f: number): boolean;
  resize(width: number, height: number): void;
  free(): void;
}
export type RaeaEngineClass = new (canvas: HTMLCanvasElement, width: number, height: number, fps: number) => RaeaEngine;

/** raea-web's module, as wasm-bindgen's `--target web` output exports it. */
export interface RaeaWebModule {
  default: () => Promise<unknown>;
  Raea: RaeaEngineClass;
}

const loaded = new WeakMap<() => Promise<RaeaWebModule>, Promise<RaeaWebModule>>();
/** raea-web's module imported and its wasm instantiated, once per loader however often it's asked (React's StrictMode
 *  runs effects twice, and two concurrent inits make two instances whose JS glue then shares one binding, which
 *  tears the engine's memory a frame later). Pass the same loader each time: `loadRaea(() => import("./raea_web.js"))`
 *  with the arrow hoisted to module scope. */
export function loadRaea(load: () => Promise<RaeaWebModule>): Promise<RaeaWebModule> {
  let p = loaded.get(load);
  if (!p) {
    p = load().then(async (m) => (await m.default(), m));
    loaded.set(load, p);
  }
  return p;
}

export interface PlayOptions {
  /** where it plays: filled by the canvas and the page over it */
  stage: HTMLElement;
  /** raea-web's Raea class, its wasm already initialised (loadRaea) */
  Raea: RaeaEngineClass;
  /** the composition's id (else the first registered) */
  composition?: string;
  /** the 3D drawn at the composition's size divided by this (4: a quarter, for a pixel look's chunky pixels); the
   *  canvas shows it scaled up, nearest-neighbour */
  pixel?: number;
  /** hold this one frame instead of playing: a poster's frame, or one to compare with `raea render --frame N` */
  hold?: number;
  /** called once the first frame with 3D is on the canvas */
  onFirstFrame?: () => void;
  /** called when a frame fails (the engine's error); playing stops */
  onError?: (e: unknown) => void;
}

export interface Playing {
  stop(): void;
  /** frames drawn and the time each took, for the budgets (docs/web-export.md) */
  stats(): { frames: number; msPerFrame: number };
}

/** Plays the registered composition in `stage`. */
export function play(o: PlayOptions): Playing {
  const comps = getCompositions();
  if (!comps.length) throw new Error("raea web: the edit registered no composition (registerRoot)");
  const comp: RegisteredComposition = (o.composition ? comps.find((c) => c.id === o.composition) : comps[0]) ?? (() => { throw new Error(`raea web: no composition "${o.composition}" (${comps.map((c) => c.id).join(", ")})`); })();
  const pixel = Math.max(1, o.pixel ?? 1);
  const w = Math.max(1, Math.round(comp.width / pixel));
  const h = Math.max(1, Math.round(comp.height / pixel));

  const style = document.createElement("style");
  style.textContent = `${UNDER3D_CSS}.raea-web-canvas,.raea-web-page{position:absolute;inset:0;width:100%;height:100%}.raea-web-canvas{image-rendering:pixelated}`;
  document.head.append(style);
  const canvas = document.createElement("canvas");
  canvas.className = "raea-web-canvas";
  const page = document.createElement("div");
  page.className = "raea-web-page";
  o.stage.append(canvas, page);

  // drawn smaller than the composition (pixel > 1), the canvas is the drawn size and the browser scales it up with hard
  // pixels (image-rendering: pixelated), covering the stage as the composition's frame would; otherwise it's the
  // stage's own pixels and the engine scales its frame onto them
  if (pixel > 1) canvas.style.objectFit = "cover";
  const fit = () => {
    const r = o.stage.getBoundingClientRect();
    const dpr = Math.min(2, devicePixelRatio || 1);
    canvas.width = pixel > 1 ? w : Math.max(1, Math.round(r.width * dpr));
    canvas.height = pixel > 1 ? h : Math.max(1, Math.round(r.height * dpr));
    engine?.resize(canvas.width, canvas.height);
  };
  let engine: RaeaEngine | undefined;
  fit();
  engine = new o.Raea(canvas, w, h, comp.fps);
  const resized = new ResizeObserver(fit);
  resized.observe(o.stage);

  const input = new Input(o.stage, () => comp);
  const root = createRoot(page);
  const collector = new Collector(pixel);
  const timeline: InputTimeline = {};
  const C = comp.component;
  const hold = o.hold === undefined ? undefined : Math.max(0, Math.floor(o.hold)) % comp.durationInFrames;
  let frame = hold ?? 0, last = performance.now(), acc = 0, raf = 0, stopped = false, first = false;
  let drawn = 0, spent = 0;

  const draw = () => {
    flushSync(() =>
      root.render(
        <Clock frame={frame} fps={comp.fps} width={comp.width} height={comp.height} durationInFrames={comp.durationInFrames} input={timeline} defaultProps={comp.defaultProps}>
          <C {...comp.defaultProps} />
        </Clock>,
      ),
    );
    const t0 = performance.now();
    const ok = engine!.frame(JSON.stringify(collector.parts(page)), clearUnder3d(page), frame);
    if (ok) {
      drawn++;
      spent += performance.now() - t0;
      if (!first) {
        first = true;
        o.onFirstFrame?.();
      }
    }
  };

  const tick = (now: number) => {
    if (stopped) return;
    raf = requestAnimationFrame(tick);
    const step = 1000 / comp.fps;
    acc += Math.min(250, now - last);
    last = now;
    let stepped = !first;
    while (acc >= step) {
      acc -= step;
      timeline[frame] = input.now();
      // a composition loops: a site's scene plays on past its last frame from the start again
      frame = hold ?? (frame + 1) % comp.durationInFrames;
      delete timeline[frame];
      stepped = true;
    }
    if (!stepped) return;
    try {
      draw();
    } catch (e) {
      // stopped, the engine kept: after a fault its memory may be torn, and wgpu's callbacks still hold it
      stopped = true;
      cancelAnimationFrame(raf);
      if (o.onError) o.onError(e);
      else throw e;
    }
  };
  raf = requestAnimationFrame(tick);

  function stop() {
    stopped = true;
    cancelAnimationFrame(raf);
    resized.disconnect();
    root.unmount();
    // after a panic the engine is still borrowed by the call that panicked: it's dropped with the page instead
    try {
      engine?.free();
    } catch {}
    engine = undefined;
    canvas.remove();
    page.remove();
    style.remove();
  }
  return { stop, stats: () => ({ frames: drawn, msPerFrame: drawn ? spent / drawn : 0 }) };
}
