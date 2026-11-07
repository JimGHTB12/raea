// Timelines (who moves when, decided once), and the wire: an Animation for the engine.
import { useContext, useRef } from "react";
import { ClockContext, LocalTimeContext, TimeBaseContext, useClockTick } from "@raea/app/clock";
import type { AnimKey, AnimTrack } from "../animate.gen";
import { Move, MoveLike, move, toMove } from "./all";

// ---- timelines: who moves when, decided once ----

/** A timeline step: [part, move], a group from sequence/parallel/stagger, or a pause in frames. */
export type Step = [part: string, move: MoveLike] | { part: string; move: MoveLike; offset?: number } | Group | number;

export interface Group {
  kind: "sequence" | "parallel";
  steps: Step[];
  /** frames between steps of a sequence (negative overlaps) */
  gap: number;
}

/** Each step when the one before it ends (`gap` frames apart: sequenceGap). */
export function sequence(...steps: Step[]): Group {
  return { kind: "sequence", steps, gap: 0 };
}

export function sequenceGap(gap: number, ...steps: Step[]): Group {
  return { kind: "sequence", steps, gap };
}

/** Every step at once. */
export function parallel(...steps: Step[]): Group {
  return { kind: "parallel", steps, gap: 0 };
}

/** The parts `each` frames apart, each with `m`. With `grid`, delays spread by grid distance from an origin. */
export function stagger(parts: string[], each: number, m: MoveLike, grid?: { cols: number; rows?: number; from?: GridFrom }): Group {
  if (!grid) return { kind: "parallel", steps: parts.map((p, i) => ({ part: p, move: m, offset: i * each })), gap: 0 };
  const rows = grid.rows ?? Math.ceil(parts.length / grid.cols);
  return { kind: "parallel", steps: parts.map((p, i) => ({ part: p, move: m, offset: gridDelay(grid.cols, rows, grid.from ?? "top-left", i) * each })), gap: 0 };
}

export type GridFrom = "top-left" | "top-right" | "bottom-left" | "bottom-right" | "center" | "top" | "bottom" | "left" | "right";

/** A grid item's distance from an origin, in steps (raea-animate's StaggerPattern::Grid, the same rule). */
export function gridDelay(cols: number, rows: number, from: GridFrom, index: number): number {
  cols = Math.max(1, cols);
  rows = Math.max(1, rows);
  const i = Math.min(index, cols * rows - 1);
  const x = i % cols;
  const y = Math.floor(i / cols);
  const d = (a: number, b: number) => Math.abs(a - b);
  switch (from) {
    case "top-left": return x + y;
    case "top-right": return cols - 1 - x + y;
    case "bottom-left": return x + (rows - 1 - y);
    case "bottom-right": return cols - 1 - x + (rows - 1 - y);
    case "center": return Math.min(d(x, Math.floor((cols - 1) / 2)), d(x, Math.floor(cols / 2))) + Math.min(d(y, Math.floor((rows - 1) / 2)), d(y, Math.floor(rows / 2)));
    case "top": return y;
    case "bottom": return rows - 1 - y;
    case "left": return x;
    case "right": return cols - 1 - x;
  }
}

/** A timeline's plan: each part's start frame and move, and the whole length. */
export interface Plan {
  parts: Record<string, { at: number; move: Move }>;
  frames: number;
}

/** Steps placed in time (a sequence of them). Each part's start is decided here, once. */
export function timeline(...steps: Step[]): Plan {
  const parts: Plan["parts"] = {};
  const place = (s: Step, at: number): number => {
    if (typeof s === "number") return at + s;
    if (Array.isArray(s)) s = { part: s[0], move: s[1] };
    if ("kind" in s) {
      if (s.kind === "parallel") return Math.max(at, ...s.steps.map((x) => place(x, at)));
      let t = at;
      s.steps.forEach((x, i) => (t = place(x, i === 0 ? t : t + s.gap)));
      return t;
    }
    const m = toMove(s.move);
    const start = at + (s.offset ?? 0);
    if (parts[s.part]) throw new Error(`timeline: the part "${s.part}" is placed twice`);
    parts[s.part] = { at: start, move: m };
    return start + m.frames;
  };
  const frames = place(sequence(...steps), 0);
  return { parts, frames };
}

// ---- the wire: an Animation for the engine ----

export function wireTracks(out: AnimTrack[], m: Move, at: number) {
  for (const [prop, v] of Object.entries(m.tracks)) {
    for (const t of Array.isArray(v) ? v : [v]) {
      const eases = Array.isArray(t.ease) ? t.ease : null;
      out.push({
        prop,
        keys: t.keys.map(([f, val], i): AnimKey => ({
          at: f + at,
          value: typeof val === "number" ? val : 0,
          color: typeof val === "string" ? val : "",
          ease: i === t.keys.length - 1 ? "" : String(eases ? eases[i] : t.ease),
        })),
        unit: t.unit ?? "",
        repeat: t.repeat ?? "",
        digits: t.digits ?? 0,
        prefix: t.prefix ?? "",
        suffix: t.suffix ?? "",
        separator: t.separator ?? "",
      });
    }
  }
}

/** Under the engine raea-runtime-react samples declarations; under plain bun they are sampled here. */
function engineSamples(): boolean {
  return typeof (globalThis as { __bevyHost?: unknown }).__bevyHost !== "undefined";
}

/** Where an element is in time. Under the engine only the TimeBase (the Sequence's offset and any Freeze, an object
 * that changes when the element's place does, not each frame): Rust knows the frame, and the fps is the clock's.
 * Under plain bun, the frame and fps too, since the tracks are sampled here. */
type Time = { frame: number; origin: number; freeze: number | null; fps: number };

export const NATIVE = engineSamples();

export function useTime(caller: string): Time {
  const base = useContext(TimeBaseContext);
  const t = useRef<Time | null>(null);
  const cur = t.current ?? (t.current = { frame: 0, origin: 0, freeze: null, fps: 0 });
  cur.origin = base.origin;
  cur.freeze = base.freeze;
  if (!NATIVE) {
    // (a hook read only off the engine: NATIVE is fixed for the process, so the hook order never changes)
    const clock = useContext(ClockContext);
    if (!clock) throw new Error(`${caller} was called outside a Clock: render it inside a composition`);
    const local = useContext(LocalTimeContext);
    useClockTick(local ? null : clock);
    cur.frame = local ? local.frame : clock.frame;
    cur.fps = clock.config.fps;
  }
  return cur;
}
