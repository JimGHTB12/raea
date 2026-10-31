// Sampling here, for plain bun (the engine does this in Rust).
import { Easing } from "../easing";
import { spring as springValue, measureSpring } from "../spring";
import type { AnimTrack, Animation } from "../animate.gen";

// ---- sampling here, for plain bun (the engine does this in Rust: core/animate/src/spec.rs, the same rules) ----

const BACK1 = 1.70158, BACK2 = BACK1 * 1.525, BACK3 = BACK1 + 1;

const EL4 = (2 * Math.PI) / 3, EL5 = (2 * Math.PI) / 4.5;

function outBounce(t: number): number {
  const n = 7.5625, d = 2.75;
  if (t < 1 / d) return n * t * t;
  if (t < 2 / d) return n * (t -= 1.5 / d) * t + 0.75;
  if (t < 2.5 / d) return n * (t -= 2.25 / d) * t + 0.9375;
  return n * (t -= 2.625 / d) * t + 0.984375;
}

const CURVES: Record<string, (t: number) => number> = {
  linear: (t) => t,
  "in-quad": (t) => t * t, "out-quad": (t) => 1 - (1 - t) * (1 - t), "in-out-quad": (t) => (t < 0.5 ? 2 * t * t : 1 - ((-2 * t + 2) ** 2) / 2),
  "in-cubic": (t) => t * t * t, "out-cubic": (t) => { const u = 1 - t; return 1 - u * u * u; }, "in-out-cubic": (t) => { if (t < 0.5) return 4 * t * t * t; const u = -2 * t + 2; return 1 - (u * u * u) / 2; },
  "in-quart": (t) => t ** 4, "out-quart": (t) => 1 - (1 - t) ** 4, "in-out-quart": (t) => (t < 0.5 ? 8 * t ** 4 : 1 - (-2 * t + 2) ** 4 / 2),
  "in-quint": (t) => t ** 5, "out-quint": (t) => 1 - (1 - t) ** 5, "in-out-quint": (t) => (t < 0.5 ? 16 * t ** 5 : 1 - (-2 * t + 2) ** 5 / 2),
  "in-sine": (t) => 1 - Math.cos((t * Math.PI) / 2), "out-sine": (t) => Math.sin((t * Math.PI) / 2), "in-out-sine": (t) => -(Math.cos(t * Math.PI) - 1) / 2,
  "in-expo": (t) => (t === 0 ? 0 : 2 ** (10 * t - 10)), "out-expo": (t) => (t === 1 ? 1 : 1 - 2 ** (-10 * t)),
  "in-out-expo": (t) => (t === 0 || t === 1 ? t : t < 0.5 ? 2 ** (20 * t - 10) / 2 : (2 - 2 ** (-20 * t + 10)) / 2),
  "in-circ": (t) => 1 - Math.sqrt(1 - t * t), "out-circ": (t) => Math.sqrt(1 - (t - 1) ** 2),
  "in-out-circ": (t) => (t < 0.5 ? (1 - Math.sqrt(1 - (2 * t) ** 2)) / 2 : (Math.sqrt(1 - (-2 * t + 2) ** 2) + 1) / 2),
  "in-back": (t) => BACK3 * t * t * t - BACK1 * t * t, "out-back": (t) => { const u = t - 1; return 1 + BACK3 * u * u * u + BACK1 * u * u; },
  "in-out-back": (t) => (t < 0.5 ? ((2 * t) ** 2 * ((BACK2 + 1) * 2 * t - BACK2)) / 2 : ((2 * t - 2) ** 2 * ((BACK2 + 1) * (2 * t - 2) + BACK2) + 2) / 2),
  "in-elastic": (t) => (t === 0 || t === 1 ? t : -(2 ** (10 * t - 10)) * Math.sin((10 * t - 10.75) * EL4)),
  "out-elastic": (t) => (t === 0 || t === 1 ? t : 2 ** (-10 * t) * Math.sin((10 * t - 0.75) * EL4) + 1),
  "in-out-elastic": (t) => (t === 0 || t === 1 ? t : t < 0.5 ? -(2 ** (20 * t - 10) * Math.sin((20 * t - 11.125) * EL5)) / 2 : (2 ** (-20 * t + 10) * Math.sin((20 * t - 11.125) * EL5)) / 2 + 1),
  "in-bounce": (t) => 1 - outBounce(1 - t), "out-bounce": outBounce,
  "in-out-bounce": (t) => (t < 0.5 ? (1 - outBounce(1 - 2 * t)) / 2 : (1 + outBounce(2 * t - 1)) / 2),
  ease: Easing.bezier(0.42, 0, 1, 1),
};

const parsed = new Map<string, (t: number) => number>();

/** A curve name as a function (the engine's easing::Easing::parse, the same names). */
export function curveFn(name: string, fps = 30): (t: number) => number {
  const key = `${name}@${fps}`;
  let f = parsed.get(key);
  if (f) return f;
  const s = name.trim();
  const m = /^([a-z-]+)\((.*)\)$/.exec(s);
  const nums = () => m![2].split(",").map((x) => Number(x.trim()));
  if (s === "") f = CURVES.linear;
  else if (CURVES[s]) f = CURVES[s];
  else if (m && (m[1] === "bezier" || m[1] === "cubic-bezier")) { const [a, b, c, d] = nums(); f = Easing.bezier(a, b, c, d); }
  else if (m && m[1] === "steps") { const n = Math.max(1, nums()[0]); f = (t) => (t === 0 ? 0 : Math.min(1, Math.ceil(t * n) / n)); }
  else if (m && m[1] === "spring") {
    const [mass, damping, stiffness] = nums();
    const config = { mass, damping, stiffness };
    const n = measureSpring({ fps, config });
    f = (t) => springValue({ frame: t * n, fps, config });
  } else if (m && m[1] === "reverse") { const g = curveFn(m[2], fps); f = (t) => 1 - g(1 - t); }
  else throw new Error(`no easing named "${s}"`);
  const clamped = (t: number) => f!(Math.min(1, Math.max(0, t)));
  parsed.set(key, clamped);
  return clamped;
}

function fold(t: number, a: number, b: number, repeat: string): number {
  const span = b - a;
  const [kind, count] = repeat.split(":");
  const n = count === undefined ? null : Math.max(1, Number(count));
  if (span <= 0 || t < b || (t === b && kind !== "loop")) return t;
  const k = (t - a) / span;
  const mod = (x: number, y: number) => ((x % y) + y) % y;
  if (kind === "loop") return n !== null && k >= n ? b : a + mod(t - a, span);
  if (kind === "pingpong") {
    if (n !== null && k >= n) return n % 2 === 1 ? b : a;
    const c = mod(t - a, 2 * span);
    return c <= span ? a + c : a + 2 * span - c;
  }
  return t;
}

/** One track at local frame `t`: [key index, eased progress to the next]. */
function locate(tr: AnimTrack, t: number, fps: number): [number, number] {
  const k = tr.keys;
  if (k.length <= 1) return [0, 0];
  t = fold(t, k[0].at, k[k.length - 1].at, tr.repeat);
  if (t <= k[0].at) return [0, 0];
  if (t >= k[k.length - 1].at) return [k.length - 1, 0];
  let i = 0;
  while (i + 1 < k.length && k[i + 1].at <= t) i++;
  const p = Math.min(1, Math.max(0, (t - k[i].at) / Math.max(1e-12, k[i + 1].at - k[i].at)));
  return [i, curveFn(k[i].ease, fps)(p)];
}

const COLOR_PROPS = new Set(["color", "backgroundColor", "borderColor", "fill", "stroke", "outlineColor", "emissive", "textDecorationColor"]);

const ADD = new Set(["x", "y", "z", "rotate", "rotateZ", "rotateX", "rotateY"]);

const MUL = new Set(["opacity", "scale", "scaleX", "scaleY"]);

const r6 = (v: number) => { const x = Math.abs(v) < 1e-9 ? 0 : Math.round(v * 1e6) / 1e6; return Object.is(x, -0) ? 0 : x; };

/** A counter's text, as raea-animate's counter_text writes it: `digits` decimals (no "-0.00"), its thousands split by
 * `separator`, between `prefix` and `suffix`. */
function counterText(val: number, tr: AnimTrack): string {
  let s = val.toFixed(Math.min(6, tr.digits));
  if (/^-?[0.]+$/.test(s)) s = s.replace("-", "");
  const sign = s.startsWith("-") ? "-" : "";
  const [int, frac] = s.slice(sign.length).split(".");
  const sep = tr.separator ?? "";
  const grouped = sep ? int.replace(/\B(?=(\d{3})+(?!\d))/g, sep) : int;
  return `${tr.prefix ?? ""}${sign}${grouped}${frac !== undefined ? "." + frac : ""}${tr.suffix ?? ""}`;
}

/** What an Animation sets at composition frame `frame`: page style keys, a counter's text, or 3D props
 * (raea-animate's Sampled, here for plain bun). */
export function sampleAnimation(a: Animation, frame: number): { style: Record<string, string | number>; text?: string; values: Record<string, number | string> } {
  const t = a.freeze ?? frame - a.origin;
  const v: Record<string, number | string> = {};
  const units: Record<string, string> = {};
  const best: Record<string, AnimTrack> = {};
  const start = (tr: AnimTrack) => Math.min(...tr.keys.map((k) => k.at));
  for (const tr of a.tracks) {
    if (tr.keys.length === 0 || ADD.has(tr.prop) || MUL.has(tr.prop)) continue;
    const cur = best[tr.prop];
    const s = start(tr);
    if (!cur) best[tr.prop] = tr;
    else {
      const cs = start(cur);
      if (s <= t ? cs > t || s >= cs : cs > t && s < cs) best[tr.prop] = tr;
    }
  }
  let text: string | undefined;
  for (const tr of a.tracks) {
    if (tr.keys.length === 0) continue;
    const [i, p] = locate(tr, t, a.fps);
    const k = tr.keys;
    const isColor = COLOR_PROPS.has(tr.prop) || k.some((x) => x.color !== "");
    if (!ADD.has(tr.prop) && !MUL.has(tr.prop) && best[tr.prop] !== tr) continue;
    if (isColor) {
      v[tr.prop] = p === 0 || i + 1 >= k.length ? k[i].color : `color-mix(in oklab, ${k[i].color} ${r6((1 - p) * 100)}%, ${k[i + 1].color})`;
      continue;
    }
    const val = p === 0 || i + 1 >= k.length ? k[i].value : k[i].value + (k[i + 1].value - k[i].value) * p;
    const prop = tr.prop === "rotateZ" ? "rotate" : tr.prop;
    if (ADD.has(prop)) v[prop] = ((v[prop] as number) ?? 0) + val;
    else if (prop === "scale") { v.scaleX = ((v.scaleX as number) ?? 1) * val; v.scaleY = ((v.scaleY as number) ?? 1) * val; }
    else if (MUL.has(prop)) v[prop] = ((v[prop] as number) ?? 1) * val;
    else if (prop === "text") text = counterText(val, tr);
    else v[prop] = val;
    if (prop === "x" || prop === "y" || prop === "width" || prop === "height") units[prop] = tr.unit || "px";
  }
  const style: Record<string, string | number> = {};
  const n = (x: unknown) => String(r6(x as number));
  if (v.opacity !== undefined) style.opacity = r6(Math.min(1, Math.max(0, v.opacity as number)));
  if (v.x !== undefined || v.y !== undefined || v.z !== undefined) style.translate = `${n(v.x ?? 0)}${units.x ?? "px"} ${n(v.y ?? 0)}${units.y ?? "px"}${v.z !== undefined ? ` ${n(v.z)}px` : ""}`;
  if (v.scaleX !== undefined || v.scaleY !== undefined) style.scale = `${n(v.scaleX ?? 1)} ${n(v.scaleY ?? 1)}`;
  if (v.rotate !== undefined) style.rotate = `${n(v.rotate)}deg`;
  for (const k of ["width", "height"]) if (v[k] !== undefined) style[k] = `${n(Math.max(0, v[k] as number))}${units[k]}`;
  const clips = ["clipTop", "clipRight", "clipBottom", "clipLeft"];
  if (clips.some((c) => v[c] !== undefined)) style.clipPath = `inset(${clips.map((c) => `${n(Math.min(100, Math.max(0, (v[c] as number) ?? 0)))}%`).join(" ")})`;
  if (v.blur !== undefined) style.filter = `blur(${n(Math.max(0, v.blur as number))}px)`;
  const own = new Set(["opacity", "x", "y", "z", "scaleX", "scaleY", "rotate", "rotateX", "rotateY", "width", "height", "blur", ...clips]);
  for (const [k, x] of Object.entries(v)) if (!own.has(k)) style[k] = typeof x === "number" ? r6(x) : x;
  return { style, text, values: v };
}
