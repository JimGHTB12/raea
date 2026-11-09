// The components: <Animate>, <Stagger>, and moves in 3D.
import { Children, createElement, memo, useRef, type CSSProperties, type ReactNode } from "react";
import type { AnimTrack, Animation } from "../animate.gen";
import { GridFrom, Keys, Move, MoveLike, NATIVE, Plan, TrackSpec, gridDelay, move, propsFingerprint, reverse, sampleAnimation, toMove, useTime, useWire, wireTracks } from "./all";

// ---- the components ----

export interface AnimateOwnProps {
  /** the entrance: a move's name, a move, or several together */
  enter?: MoveLike;
  /** the exit, played at `exitAt` (an entrance reversed: `exit="rise"` sinks back out) */
  exit?: MoveLike;
  /** free tracks by property, in local frames: `{ backgroundColor: track({ 0: "#fff", 20: "#000" }) }` */
  animate?: Record<string, TrackSpec | TrackSpec[] | Keys> | Move;
  /** local frame the entrance starts (0) */
  at?: number;
  /** local frame the exit starts */
  exitAt?: number;
  /** a timeline's plan and this element's part in it: the part's move, at its start (plus `at`) */
  timeline?: Plan;
  /** the part in \`timeline\` whose move this element plays */
  part?: string;
  /** "always" (default): in the tree, and its layout, the whole time (invisible before an entrance that fades in);
   * "during": only from the entrance's start until the exit ends (it takes its space when it arrives) */
  mount?: "during" | "always";
  /** the element: "div" by default */
  as?: string;
  /** inline style */
  style?: CSSProperties;
  /** Tailwind or CSS classes */
  className?: string;
  /** what moves */
  children?: ReactNode;
}

export type AnimateProps = AnimateOwnProps & { [attr: string]: unknown };

const OWN = new Set(["enter", "exit", "animate", "at", "exitAt", "timeline", "part", "mount", "as", "style", "children"]);

/** An element whose animation is declared once and drawn by the engine each frame. Under the engine it reads no
 * per-frame value: its place in time comes from the TimeBase, the frame and fps from Rust, and a mount window
 * (mount="during") is Rust's to show and hide. A render whose props have the same content as the last gives the same
 * element object back, so the frame allocates nothing for it. */
function AnimateImpl(props: AnimateProps) {
  const { enter, exit, animate, at = 0, exitAt, timeline: plan, part, mount = "always", as = "div", style, children } = props as AnimateOwnProps;
  const time = useTime("<Animate>");
  const wire = useWire(
    () => {
      const out: AnimTrack[] = [];
      let start = at;
      let enterMove: Move | null = enter !== undefined ? toMove(enter) : null;
      if (plan) {
        const p = part !== undefined ? plan.parts[part] : undefined;
        if (!p) throw new Error(`<Animate>: the timeline has no part "${part}" (${Object.keys(plan.parts).join(", ")})`);
        start = p.at + at;
        enterMove = enterMove ? toMove([p.move, enterMove]) : p.move;
      }
      if (enterMove) wireTracks(out, enterMove, start);
      if (exit !== undefined) {
        if (exitAt === undefined) throw new Error("<Animate>: an exit needs exitAt (the local frame it starts)");
        wireTracks(out, reverse(exit), exitAt);
      }
      if (animate) wireTracks(out, (animate as Move).frames !== undefined && (animate as Move).tracks ? (animate as Move) : move(animate as Record<string, TrackSpec>), 0);
      return out;
    },
    enter, exit, animate, at, exitAt, plan, part,
    time.origin,
    time.freeze,
    time.fps,
    mount === "during",
  );
  const memo = useRef<{ fp: number; wire: Wire; el: ReturnType<typeof createElement> | null } | null>(null);
  if (NATIVE) {
    // the same content as last render: the same element (nothing allocated, nothing for React or the runtime to diff)
    const fp = propsFingerprint(props);
    const m = memo.current;
    if (m && m.fp === fp && m.wire === wire) return m.el;
    const el = element(as, props, style, children ?? textOf(wire), wire.json);
    memo.current = { fp, wire, el };
    return el;
  }
  // plain bun: sampled here, at this frame
  if (mount === "during") {
    const [from, until] = spanOf(wire);
    const f = time.freeze ?? time.frame;
    if (from !== Infinity && f < from) return null;
    if (exitAt !== undefined && f >= until + 1) return null;
  }
  const sampled = sampleAnimation(wire.anim, time.origin + (time.freeze ?? time.frame));
  return element(as, props, sampled ? { ...style, ...sampled.style } : style, sampled.text !== undefined ? sampled.text : children ?? textOf(wire), wire.json);
}

/** The element's own props (everything but the declaration's) in one object, with its declaration's text. */
function element(as: string, props: AnimateProps, style: unknown, kids: ReactNode, json: string) {
  const out: Record<string, unknown> = {};
  for (const k in props) if (!OWN.has(k)) out[k] = props[k];
  out.style = style;
  out["data-raea-anim"] = json;
  return createElement(as, out, kids);
}

/** <Animate>, memoised by its props' content (React skips a render whose props fingerprint matches the last). The
 * fingerprint is on it as `Animate.fingerprint` for a runtime's own bailout. Its declaration is the engine's
 * Animation (@bevy raea_core_animate::spec::Animation), typed from the registry (animate.gen.ts), sent as data-raea-anim and
 * read back as that type by sdk/react animate.rs. */
export const Animate = Object.assign(memo(AnimateImpl, (a, b) => propsFingerprint(a) === propsFingerprint(b)), { fingerprint: propsFingerprint });

export type Wire = { json: string; anim: Animation; window?: boolean; span?: [number, number]; text?: string | null };

/** The first and last key of a declaration, worked out once. */
function spanOf(w: Wire): [number, number] {
  if (!w.span) {
    let from = Infinity;
    let until = -Infinity;
    for (const tr of w.anim.tracks) for (const k of tr.keys) { from = Math.min(from, k.at); until = Math.max(until, k.at); }
    w.span = [from, until];
  }
  return w.span;
}

/** A counter's first text, worked out once. */
function textOf(w: Wire): string | undefined {
  if (w.text === undefined) w.text = initialText(w.anim) ?? null;
  return w.text ?? undefined;
}

/** A counter's first value, as its text (the engine rewrites it each frame). */
function initialText(a: Animation): string | undefined {
  const t = a.tracks.find((x) => x.prop === "text");
  return t ? sampleAnimation({ ...a, tracks: [t] }, a.origin + (t.keys[0]?.at ?? 0)).text : undefined;
}

export interface StaggerProps extends Omit<AnimateOwnProps, "timeline" | "part"> {
  /** frames between one child's entrance and the next */
  each: number;
  /** spread by grid distance instead of order */
  grid?: { cols: number; rows?: number; from?: GridFrom };
  /** the exit staggered too, `exitEach` frames apart (0: all together) */
  exitEach?: number;
}

/** Each child in its own <Animate>, entrances `each` frames apart (or by grid distance). The same content as last
 * render gives the same element back. */
function StaggerImpl(props: StaggerProps) {
  const memoEl = useRef<{ fp: number; el: ReturnType<typeof createElement> } | null>(null);
  const fp = propsFingerprint(props);
  if (memoEl.current && memoEl.current.fp === fp) return memoEl.current.el;
  const { each, grid, children, at = 0, exitAt, exitEach = 0, as, style, className, ...anim } = props;
  const items = Children.toArray(children);
  const rows = grid ? grid.rows ?? Math.ceil(items.length / grid.cols) : 0;
  const el = createElement(
    as ?? "div",
    { style, className },
    items.map((child, i) => {
      const d = grid ? gridDelay(grid.cols, rows, grid.from ?? "top-left", i) * each : i * each;
      return createElement(Animate, { key: i, ...anim, at: at + d, exitAt: exitAt === undefined ? undefined : exitAt + i * exitEach }, child);
    }),
  );
  memoEl.current = { fp, el };
  return el;
}

export const Stagger = Object.assign(memo(StaggerImpl, (a, b) => propsFingerprint(a) === propsFingerprint(b)), { fingerprint: propsFingerprint });

/** For a 3D node (<Mesh animate>, materials): the declaration's text, or under plain bun the props it sets now. */
export function useAnimation3d(spec: Record<string, TrackSpec | TrackSpec[] | Keys> | Move | undefined): { attr?: string; values?: Record<string, number | string> } {
  const time = useTime("animate");
  const wire = useWire(() => {
    const out: AnimTrack[] = [];
    if (spec) wireTracks(out, (spec as Move).tracks && (spec as Move).frames !== undefined ? (spec as Move) : move(spec as Record<string, TrackSpec>), 0);
    return out;
  }, spec, undefined, undefined, undefined, undefined, undefined, undefined, time.origin, time.freeze, time.fps, false);
  if (!spec) return {};
  if (NATIVE) return { attr: wire.json };
  return { attr: wire.json, values: sampleAnimation(wire.anim, time.origin + (time.freeze ?? time.frame)).values };
}

/** What a move sets at local frame `frame` when it starts at `start`: the same rules the engine samples <Animate>
 * with (x, y, opacity, scale, rotate, colours...), for parts that are animated one by one in TypeScript (a 3D text's
 * glyphs: <RichText enter>). An empty object before it starts is its first key's values (an entrance holds them). */
export function sampleMove(m: MoveLike, start: number, frame: number, fps = 30): Record<string, number | string> {
  const tracks: AnimTrack[] = [];
  wireTracks(tracks, toMove(m), start);
  return sampleAnimation({ origin: 0, freeze: null, fps, tracks, show_from: null, show_until: null }, frame).values;
}
