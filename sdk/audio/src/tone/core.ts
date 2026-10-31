// The edit's Tone: its graph and its events.
import { DestinationClass, ToneAudioNode } from "./all";

// ---- the edit's Tone: its graph and its events -----------------------------------------------------------------------

export type Ev = { k: string; n: number; t: number; e: string; f?: number[]; d?: number; v?: number; o?: number; p?: string; val?: number; ramp?: number };

type Item = { time: number; seq: number; run: (time: number) => void };

export class Context {
  nodes = new Map<number, ToneAudioNode>();
  edges: [number, number, number][] = [];
  events: Ev[] = [];
  sorted = true;
  seq = 0;
  nextId = 1;
  /** the time a callback runs at (Tone.now() inside it) */
  now = 0;
  /** where triggers go while an <Sfx> plays (its frame's events), else the edit's list */
  sink: Ev[] | null = null;
  sinkKey = "";
  pending: Item[] = [];
  compiled = 0;
  graph: string | null = null;
  readonly sampleRate = 48000;
  readonly lookAhead = 0;
  readonly latencyHint = "interactive";
  readonly state = "running";
  get currentTime() {
    return this.now;
  }
  resume() {
    return Promise.resolve();
  }
  emit(ev: Omit<Ev, "k"> & { k?: string }) {
    const k = ev.k ?? (this.sink ? `${this.sinkKey}:${this.seq++}` : `${ev.n}:${this.seq++}`);
    const e = { ...ev, k } as Ev;
    if (this.sink) this.sink.push(e);
    else {
      if (this.events.length && this.events[this.events.length - 1].t > e.t) this.sorted = false;
      this.events.push(e);
    }
  }
  schedule(item: Omit<Item, "seq">) {
    const it = { ...item, seq: this.seq++ };
    // sorted insert by (time, seq)
    let lo = 0;
    let hi = this.pending.length;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      const p = this.pending[mid];
      if (p.time < it.time || (p.time === it.time && p.seq < it.seq)) lo = mid + 1;
      else hi = mid;
    }
    this.pending.splice(lo, 0, it);
  }
  /** runs the scheduled callbacks up to `until` seconds, in time order (the same order whatever frames ask) */
  compileTo(until: number) {
    while (this.pending.length && this.pending[0].time < until) {
      const it = this.pending.shift()!;
      this.now = it.time;
      it.run(it.time);
    }
    this.compiled = Math.max(this.compiled, until);
    this.now = 0;
  }
  window(t0: number, t1: number): Ev[] {
    if (!this.sorted) {
      this.events.sort((a, b) => a.t - b.t || (a.k < b.k ? -1 : 1));
      this.sorted = true;
    }
    // the first event at or after t0
    let lo = 0;
    let hi = this.events.length;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (this.events[mid].t < t0) lo = mid + 1;
      else hi = mid;
    }
    const out: Ev[] = [];
    for (let i = lo; i < this.events.length && this.events[i].t < t1; i++) out.push(this.events[i]);
    return out;
  }
  graphJson(): string {
    if (this.graph === null) {
      for (const n of this.nodes.values()) n.beforeFreeze();
      const nodes = [...this.nodes.values()].map((n) => n.spec());
      this.graph = JSON.stringify({ nodes, edges: this.edges, mods: S.mods });
    }
    return this.graph;
  }
}

export const ctx = new Context();

/** LFOs connected to parameters: [lfo id, node id, parameter name] */
/** the module's state its files assign (each field was a top-level let) */ export const S: { mods: [number, number, string][]; destination: DestinationClass | null } = { mods: [], destination: null };

export function getContext() {
  return ctx;
}

export const context = ctx;

export function setContext(_c: unknown) {}

export function start() {
  return Promise.resolve();
}

export function loaded() {
  return Promise.resolve();
}

export function now() {
  return ctx.now;
}

export function immediate() {
  return ctx.now;
}

export const version = "15.1.22-raea";
