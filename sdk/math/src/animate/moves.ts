// The named moves.
import { Curve, Move, TrackSpec, made, move, track } from "./all";

// ---- the named moves, from the timings the approved pieces were measured to use (craft: animating.md) ----

type Dir = "left" | "right" | "up" | "down";

const OUT: Curve = "out-cubic";

/** y offset `distance` to 0 over `frames` (30 px over 6), fading in over `fade` (5). */
function riseImpl(o: { distance?: number; frames?: number; fade?: number; ease?: Curve } = {}): Move {
  const { distance = 30, frames = 6, fade = 5, ease = OUT } = o;
  return move({ y: track({ 0: distance, [frames]: 0 }, ease), opacity: track({ 0: 0, [fade]: 1 }, ease) });
}

/** scale `from` (0.6), `peak` (1.08) at `frames`/2, 1 at `frames` (8); in over 3 frames. */
function popImpl(o: { from?: number; peak?: number; frames?: number; ease?: Curve } = {}): Move {
  const { from = 0.6, peak = 1.08, frames = 8, ease = OUT } = o;
  return move({ scale: track({ 0: from, [Math.round(frames / 2)]: peak, [frames]: 1 }, ease), opacity: track({ 0: 0, 3: 1 }, ease) });
}

/** a hit: scale `from` (1.3), `dip` (0.96) at `frames`/2, 1 at `frames` (8); in over 2 frames. */
function slamImpl(o: { from?: number; dip?: number; frames?: number; ease?: Curve } = {}): Move {
  const { from = 1.3, dip = 0.96, frames = 8, ease = OUT } = o;
  return move({ scale: track({ 0: from, [Math.round(frames / 2)]: dip, [frames]: 1 }, ease), opacity: track({ 0: 0, 2: 1 }, ease) });
}

/** arrives from a side: `distance` px (30) over `frames` (7), fading in over 5. `from` is where it comes from. */
function slideImpl(o: { from?: Dir; distance?: number; frames?: number; ease?: Curve } = {}): Move {
  const { from = "left", distance = 30, frames = 7, ease = OUT } = o;
  const axis = from === "left" || from === "right" ? "x" : "y";
  const sign = from === "left" || from === "up" ? -1 : 1;
  return move({ [axis]: track({ 0: sign * distance, [frames]: 0 }, ease), opacity: track({ 0: 0, 5: 1 }, ease) });
}

/** a reveal wiping across from `from` (left) over `frames` (8), in-out-cubic, by clipping. */
function wipeImpl(o: { from?: Dir; frames?: number; ease?: Curve } = {}): Move {
  const { from = "left", frames = 8, ease = "in-out-cubic" } = o;
  const side = { left: "clipRight", right: "clipLeft", up: "clipBottom", down: "clipTop" }[from];
  return move({ [side]: track({ 0: 100, [frames]: 0 }, ease) });
}

/** a bar filling: width 0 to `to` (100%) over `frames` (24), out-cubic. */
function fillImpl(o: { to?: number; frames?: number; unit?: "px" | "%"; ease?: Curve } = {}): Move {
  const { to = 100, frames = 24, unit = "%", ease = OUT } = o;
  return move({ width: track({ 0: 0, [frames]: to }, ease, { unit }) });
}

/** a number counting from `from` (0) to `to` over `frames` (24), written as the element's text: `digits` decimals,
 * `separator` between its thousands ("," or true writes 12,345; none by default), `prefix` and `suffix` around it
 * ("$", " pts"). Hidden until it starts, then in over `fade` frames (2; 0 shows it at `from` the whole time). */
function tickImpl(o: { to: number; from?: number; frames?: number; digits?: number; ease?: Curve; prefix?: string; suffix?: string; separator?: string | boolean; fade?: number }): Move {
  const { to, from = 0, frames = 24, digits = 0, ease = OUT, prefix = "", suffix = "", fade = 2 } = o;
  const separator = o.separator === true ? "," : o.separator || "";
  const text = track({ 0: from, [frames]: to }, ease, { digits, prefix, suffix, separator });
  return fade > 0 ? move({ text, opacity: track({ 0: 0, [fade]: 1 }, ease) }) : move({ text });
}

/** a strike-through line drawn: width 0 to 100% over `frames` (8). */
function strikeImpl(o: { frames?: number; ease?: Curve } = {}): Move {
  const { frames = 8, ease = OUT } = o;
  return move({ width: track({ 0: 0, [frames]: 100 }, ease, { unit: "%" }) });
}

/** a slow push-in on footage: scale 1 to 1 + `amount` (0.04) across `frames` (90), linear. */
function pushInImpl(o: { amount?: number; frames?: number } = {}): Move {
  const { amount = 0.04, frames = 90 } = o;
  return move({ scale: track({ 0: 1, [frames]: 1 + amount }, "linear") });
}

/** opacity 0 to 1 over `frames` (6). */
function fadeImpl(o: { frames?: number; ease?: Curve } = {}): Move {
  const { frames = 6, ease = OUT } = o;
  return move({ opacity: track({ 0: 0, [frames]: 1 }, ease) });
}

// each move by its options' content: the same options give the same Move object
export const rise: typeof riseImpl = (o?: any) => made(20, o, undefined, undefined, () => riseImpl(o));

export const pop: typeof popImpl = (o?: any) => made(21, o, undefined, undefined, () => popImpl(o));

export const slam: typeof slamImpl = (o?: any) => made(22, o, undefined, undefined, () => slamImpl(o));

export const slide: typeof slideImpl = (o?: any) => made(23, o, undefined, undefined, () => slideImpl(o));

export const wipe: typeof wipeImpl = (o?: any) => made(24, o, undefined, undefined, () => wipeImpl(o));

export const fill: typeof fillImpl = (o?: any) => made(25, o, undefined, undefined, () => fillImpl(o));

export const tick: typeof tickImpl = (o?: any) => made(26, o, undefined, undefined, () => tickImpl(o));

export const strike: typeof strikeImpl = (o?: any) => made(27, o, undefined, undefined, () => strikeImpl(o));

export const pushIn: typeof pushInImpl = (o?: any) => made(28, o, undefined, undefined, () => pushInImpl(o));

export const fade: typeof fadeImpl = (o?: any) => made(29, o, undefined, undefined, () => fadeImpl(o));

export const moves = { rise, pop, slam, slide, wipe, fill, tick, strike, pushIn, fade };

export type MoveName = "rise" | "pop" | "slam" | "slide" | "wipe" | "fill" | "strike" | "pushIn" | "fade";

export type MoveLike = MoveName | Move | MoveLike[];

const NAMED = new Map<string, Move>();

/** A name, a move or several moves together, as one move. */
export function toMove(m: MoveLike): Move {
  if (typeof m === "string") {
    let v = NAMED.get(m);
    if (!v) {
      const f = (moves as Record<string, (o?: object) => Move>)[m];
      if (!f || (m as string) === "tick") throw new Error(`<Animate>: no move named "${m}" (${Object.keys(moves).filter((k) => k !== "tick").join(", ")})`);
      NAMED.set(m, (v = f()));
    }
    return v;
  }
  if (Array.isArray(m)) {
    const all = m.map(toMove);
    const tracks: Record<string, TrackSpec[]> = {};
    for (const mm of all) for (const [k, v] of Object.entries(mm.tracks)) (tracks[k] ??= []).push(...(Array.isArray(v) ? v : [v]));
    return { tracks, frames: Math.max(0, ...all.map((x) => x.frames)) };
  }
  return m;
}
