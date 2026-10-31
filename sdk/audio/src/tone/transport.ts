// The Transport: tempo map, BPM and its clock.
import { Param, Time, ToneAudioNode, ctx, position, seconds, start } from "./all";

// ---- the Transport and its schedulers -------------------------------------------------------------------------------

/** The Transport's tempo over the timeline: knots of (time s, bpm), linear between them (Tone's bpm.rampTo and
 *  linearRampToValueAtTime; exponential ramps are taken as linear), held before the first and after the last. */
class TempoMap {
  knots: [number, number][] = [];
  constructor(public base: number) {}
  at(t: number): number {
    const k = this.knots;
    if (!k.length || t <= k[0][0]) return k.length ? k[0][1] : this.base;
    for (let i = 1; i < k.length; i++) if (t < k[i][0]) {
      const [t0, b0] = k[i - 1];
      const [t1, b1] = k[i];
      return t1 > t0 ? b0 + (b1 - b0) * ((t - t0) / (t1 - t0)) : b1;
    }
    return k[k.length - 1][1];
  }
  add(t: number, v: number) {
    this.knots.push([t, v]);
    this.knots.sort((a, b) => a[0] - b[0]);
  }
  /** the segments (from, to, bpm at from, bpm at to) from t0 on, the last one open-ended */
  private segs(t0: number): [number, number, number, number][] {
    const ts = [t0, ...this.knots.map((k) => k[0]).filter((t) => t > t0), Infinity];
    const out: [number, number, number, number][] = [];
    for (let i = 0; i + 1 < ts.length; i++) {
      const a = ts[i];
      const b = ts[i + 1];
      out.push([a, b, this.at(a + 1e-12), b === Infinity ? this.at(a + 1e-12) : this.at(b - 1e-12)]);
    }
    return out;
  }
  /** beats from t0 to t */
  beats(t0: number, t: number): number {
    let n = 0;
    for (const [a, b, ba, bb] of this.segs(t0)) {
      if (t <= a) break;
      const e = Math.min(t, b);
      const L = b === Infinity ? 1 : b - a;
      const d = e - a;
      const slope = b === Infinity ? 0 : (bb - ba) / L;
      n += (ba * d + 0.5 * slope * d * d) / 60;
      if (t <= b) break;
    }
    return n;
  }
  /** the time `beats` after t0 */
  timeFor(t0: number, beats: number): number {
    let left = beats;
    for (const [a, b, ba, bb] of this.segs(t0)) {
      const L = b === Infinity ? Infinity : b - a;
      const slope = b === Infinity ? 0 : (bb - ba) / L;
      const segBeats = b === Infinity ? Infinity : (ba * L + 0.5 * slope * L * L) / 60;
      if (left <= segBeats + 1e-12) {
        // solve ba*d + slope*d^2/2 = 60*left for d
        const c = 60 * left;
        const d = Math.abs(slope) < 1e-12 ? c / ba : (-ba + Math.sqrt(Math.max(0, ba * ba + 2 * slope * c))) / slope;
        return a + d;
      }
      left -= segBeats;
    }
    return t0 + (beats * 60) / this.base;
  }
}

/** The Transport's bpm: a Param whose ramps go to the tempo map, so the grid follows them. */
class BpmParam extends Param {
  constructor(owner: ToneAudioNode, private map: TempoMap) {
    super(owner, ["bpm"], "bpm");
  }
  get value(): number {
    return this.map.at(ctx.now);
  }
  set value(v: number) {
    if (ctx.graph === null && ctx.now <= 0 && this.map.knots.length === 0) {
      this.map.base = v;
      this.owner.options.bpm = v;
    } else {
      this.map.add(ctx.now, this.map.at(ctx.now));
      this.map.add(ctx.now + 1e-9, v);
    }
  }
  protected ramp(v: number, over: number, at: number) {
    if (over <= 0 && at <= 0 && this.map.knots.length === 0) {
      this.map.base = v;
      return this;
    }
    const from = this.map.at(at);
    this.map.add(at, from);
    this.map.add(at + Math.max(over, 1e-9), v);
    return this;
  }
}

class TransportClass {
  readonly PPQ = 192;
  readonly bpm: Param;
  readonly tempo = new TempoMap(120);
  private bpmHolder = new (class extends ToneAudioNode {})("TransportBpm", { bpm: 120 });
  timeSignature: number | [number, number] = 4;
  swing = 0;
  swingSubdivision: Time = "8n";
  loop = false;
  loopStart: Time = 0;
  loopEnd: Time = "4m";
  state: "started" | "stopped" | "paused" = "stopped";
  startTime = 0;
  offset = 0;
  /** schedule() calls waiting for start(), in transport time */
  private waiting: { at: number; seq: number; fn: (time: number) => void; repeat?: number; until?: number }[] = [];
  private ids = 0;
  private cancelled = new Set<number>();
  constructor() {
    ctx.nodes.delete(this.bpmHolder.id);
    this.bpm = new BpmParam(this.bpmHolder, this.tempo);
  }
  ts(): [number, number] {
    return Array.isArray(this.timeSignature) ? this.timeSignature : [this.timeSignature, 4];
  }
  get seconds() {
    return Math.max(0, ctx.now - this.startTime + this.offset);
  }
  set seconds(s: number) {
    this.offset = s;
  }
  get position(): string {
    return position(this.seconds);
  }
  set position(p: string | number) {
    this.offset = seconds(p, 0);
  }
  get ticks() {
    return Math.round((this.seconds / (60 / this.bpm.value)) * this.PPQ);
  }
  get progress() {
    return 0;
  }
  private swung(t: number): number {
    if (!this.swing) return t;
    const sub = seconds(this.swingSubdivision, 0);
    const k = t / sub;
    const odd = Math.abs(k - Math.round(k)) < 1e-6 && Math.round(k) % 2 === 1;
    return odd ? t + this.swing * sub * 0.5 : t;
  }
  private abs(at: number) {
    const x = this.swung(at) - this.offset;
    if (this.tempo.knots.length === 0) return this.startTime + x;
    // transport seconds are counted at the starting tempo: beats, then the time those beats take on the tempo map
    const b0 = this.tempo.at(this.startTime);
    return this.tempo.timeFor(this.startTime, (x * b0) / 60);
  }
  private enqueue(w: { at: number; seq: number; fn: (time: number) => void; repeat?: number; until?: number }) {
    if (this.loop) return this.enqueueLooped(w);
    const id = w.seq;
    const fire = (at: number) => {
      if (this.cancelled.has(id)) return;
      ctx.schedule({
        time: this.abs(at),
        run: (time) => {
          if (this.cancelled.has(id) || this.state !== "started") return;
          w.fn(time);
          if (w.repeat && w.repeat > 0) {
            const next = at + w.repeat;
            if (w.until === undefined || next < w.until) fire(next);
          }
        },
      });
    };
    if (w.at >= this.offset || w.repeat) {
      let at = w.at;
      if (w.repeat && at < this.offset) at += Math.ceil((this.offset - at) / w.repeat - 1e-9) * w.repeat;
      fire(at);
    }
  }
  /** Tone's loop: the transport goes back to loopStart at loopEnd. An event inside the window fires on every pass; one
   *  before it, once; one past its end, never. A repeating event fires at its places inside the window. */
  private enqueueLooped(w: { at: number; seq: number; fn: (time: number) => void; repeat?: number; until?: number }) {
    const id = w.seq;
    const ls = seconds(this.loopStart, 0);
    const le = seconds(this.loopEnd, 0);
    const len = le - ls;
    if (!(len > 0)) return;
    const places: number[] = [];
    if (w.repeat && w.repeat > 0) {
      for (let x = w.at; x < le && (w.until === undefined || x < w.until) && places.length < 100000; x += w.repeat) places.push(x);
    } else places.push(w.at);
    for (const x of places) {
      if (x >= le) continue;
      const fire = (pass: number) => {
        if (this.cancelled.has(id)) return;
        ctx.schedule({
          time: this.abs(x + pass * len),
          run: (time) => {
            if (this.cancelled.has(id) || this.state !== "started") return;
            w.fn(time);
            if (x >= ls && this.loop) fire(pass + 1);
          },
        });
      };
      fire(0);
    }
  }
  schedule(fn: (time: number) => void, time: Time): number {
    const w = { at: seconds(time, 0), seq: ++this.ids, fn };
    if (this.state === "started") this.enqueue(w);
    else this.waiting.push(w);
    return w.seq;
  }
  scheduleOnce(fn: (time: number) => void, time: Time): number {
    return this.schedule(fn, time);
  }
  scheduleRepeat(fn: (time: number) => void, interval: Time, startTime: Time = 0, duration?: Time): number {
    const at = seconds(startTime, 0);
    const w = { at, seq: ++this.ids, fn, repeat: seconds(interval, 0), until: duration !== undefined ? at + seconds(duration, 0) : undefined };
    if (this.state === "started") this.enqueue(w);
    else this.waiting.push(w);
    return w.seq;
  }
  clear(id: number) {
    this.cancelled.add(id);
    return this;
  }
  cancel(_after: Time = 0) {
    for (const w of this.waiting) this.cancelled.add(w.seq);
    for (let i = 1; i <= this.ids; i++) this.cancelled.add(i);
    return this;
  }
  start(time?: Time, offset?: Time) {
    this.startTime = seconds(time);
    if (offset !== undefined) this.offset = seconds(offset, 0);
    this.state = "started";
    for (const w of this.waiting.splice(0)) this.enqueue(w);
    return this;
  }
  stop(time?: Time) {
    const at = seconds(time);
    ctx.schedule({ time: at, run: () => (this.state = "stopped") });
    return this;
  }
  pause(time?: Time) {
    return this.stop(time);
  }
  toggle(time?: Time) {
    return this.state === "started" ? this.stop(time) : this.start(time);
  }
  setLoopPoints(start: Time, end: Time) {
    this.loopStart = start;
    this.loopEnd = end;
    return this;
  }
  /** raea: frames at `fps` for a transport time (cuts on bars and beats) */
  toFrames(t: Time, fps = 30) {
    return Math.round(seconds(t, 0) * fps);
  }
  nextSubdivision(sub: Time) {
    const s = seconds(sub, 0);
    return this.startTime + Math.ceil(this.seconds / s - 1e-9) * s;
  }
  getTicksAtTime(t: Time) {
    return Math.round(((seconds(t) - this.startTime) / (60 / this.bpm.value)) * this.PPQ);
  }
  getSecondsAtTime(t: Time) {
    return seconds(t) - this.startTime;
  }
  dispose() {
    return this;
  }
}

export const transport = new TransportClass();

export function getTransport() {
  return transport;
}

export const Transport = transport;

export const Draw = { schedule(_cb: () => void, _time: Time) { return this; }, cancel() { return this; }, anticipation: 0.008, expiration: 0.25 };
