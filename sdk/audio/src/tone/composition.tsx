// Tone in a composition.
import { type ReactNode } from "react";
import { useCurrentFrame, useVideoConfig } from "@raea/app/clock";
import { Context, Ev, Instrument, S, busMap, ctx, pendingSends, receivers, src, transport } from "./all";

// ---- in a composition ----------------------------------------------------------------------------------------------

/** How long a shown event stays shown (frames): a play that skips a frame still meets it (then it starts late). */
const SHOW = 3;

/**
 * The edit's Tone, for the frame: its graph (the same every frame) and the events whose time falls in the last few
 * frames. raea's registry renders it inside every composition once this module is imported; render it yourself only
 * outside one.
 */
export function ToneOut(): ReactNode {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  if (ctx.nodes.size === 0 && ctx.events.length === 0 && ctx.pending.length === 0) return null;
  const t0 = (frame - SHOW + 1) / fps;
  const t1 = (frame + 1) / fps;
  // the Transport's callbacks run up to a little past this frame, in time order (the same whatever frames ask)
  ctx.compileTo(t1 + 0.5);
  const graph = ctx.graphJson();
  const evs = ctx.window(t0, t1);
  return (
    <>
      <audio data-tone-graph={graph} />
      {evs.length > 0 && <audio data-tone-evs={JSON.stringify(evs)} />}
    </>
  );
}

(globalThis as any).__raeaToneOut = ToneOut;

/**
 * raea: a game's one-shot. When `when` is true on a frame (a state machine's transition, a game event), `play(time)`
 * runs with that frame's time and its triggers sound on that exact frame, keyed by the frame, so a replay plays the
 * same. Seeded variation comes from the instrument's `variation` option; polyphony and voice stealing from its
 * maxPolyphony.
 *   <Sfx when={hit} play={(t) => sfx.triggerAttackRelease("C4", "16n", t)} />
 * Its events are keyed by `name` (else a hash of its play code) and the frame: give two <Sfx> with the same code names.
 */
export function Sfx({ when, play, name }: { when: boolean | number | null | undefined; play: (time: number, frame: number) => void; name?: string }): ReactNode {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  // its key: the name, else its play code (React's useId and live play's runtime number elements differently, and the
  // key seeds the variation). Two <Sfx> with the same code on one frame need names.
  const id = name ?? codeKey(play);
  if (!when) return null;
  const evs: Ev[] = [];
  const prev = [ctx.sink, ctx.sinkKey, ctx.seq, ctx.now] as const;
  ctx.sink = evs;
  ctx.sinkKey = `sfx${id}:${frame}`;
  ctx.seq = 0;
  ctx.now = frame / fps;
  try {
    play(frame / fps, frame);
  } finally {
    [ctx.sink, ctx.sinkKey, ctx.seq, ctx.now] = prev;
  }
  return evs.length ? <audio data-tone-evs={JSON.stringify(evs)} /> : null;
}

function codeKey(f: Function): string {
  let h = 0x811c9dc5;
  const src = f.toString();
  for (let i = 0; i < src.length; i++) h = Math.imul(h ^ src.charCodeAt(i), 0x01000193) >>> 0;
  return h.toString(36);
}

/** raea: an instrument's seeded variation (each event's own: pitch jitter in semitones, volume jitter in dB). */
export function vary<T extends Instrument>(i: T, v: { pitch?: number; volume?: number }): T {
  i.options.variation = { pitch: v.pitch ?? 0, volume: v.volume ?? 0 };
  return i;
}

/** Testing: the edit's Tone so far (graph JSON and events up to `until` seconds). */
export function snapshot(until = 10): { graph: any; events: Ev[] } {
  ctx.compileTo(until);
  return { graph: JSON.parse(ctx.graphJson()), events: ctx.window(-Infinity, until) };
}

/** Testing: a fresh context (one edit per process otherwise). */
export function reset() {
  const fresh = new Context();
  Object.assign(ctx, fresh);
  ctx.nodes = new Map();
  ctx.edges = [];
  ctx.events = [];
  ctx.pending = [];
  S.destination = null;
  busMap.clear();
  receivers.clear();
  pendingSends.length = 0;
  S.mods = [];
  transport.tempo.knots = [];
  transport.tempo.base = 120;
  Object.assign(transport, { timeSignature: 4, swing: 0, loop: false, state: "stopped", startTime: 0, offset: 0 });
  (transport as any).waiting = [];
  (transport as any).ids = 0;
  (transport as any).cancelled = new Set();
  transport.bpm.value = 120;
}
