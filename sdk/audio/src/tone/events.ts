// The Transport's schedulers: Loop, Part, Sequence and Pattern.
import { Time, ctx, seconds, start, transport } from "./all";

/** A seeded random (mulberry32): Pattern's random walks and probability are the same every run. */
function rng(seed: number) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

class Event<V = any> {
  probability = 1;
  humanize: boolean | Time = false;
  mute = false;
  playbackRate = 1;
  iterations = Infinity;
  protected ids: number[] = [];
  protected rand = rng(0x5eed + ctx.nextId);
  protected fire(cb: (time: number, v: V) => void, time: number, v: V) {
    if (this.mute) return;
    if (this.probability < 1 && this.rand() >= this.probability) return;
    cb(time, v);
  }
  stop(time?: Time) {
    const at = time === undefined ? 0 : seconds(time, 0);
    const ids = this.ids.splice(0);
    transport.schedule(() => ids.forEach((id) => transport.clear(id)), at);
    return this;
  }
  cancel() {
    this.ids.splice(0).forEach((id) => transport.clear(id));
    return this;
  }
  dispose() {
    return this.cancel();
  }
  get state() {
    return this.ids.length ? "started" : "stopped";
  }
}

/** Tone's Loop: the callback every `interval` from its start, in transport time. */
export class Loop extends Event<undefined> {
  interval: Time;
  constructor(private cb: ((time: number) => void) | { callback: (time: number) => void; interval?: Time }, interval: Time = "4n") {
    super();
    if (typeof cb === "object") {
      this.interval = cb.interval ?? interval;
      this.cb = cb.callback;
    } else this.interval = interval;
  }
  start(time: Time = 0) {
    let n = 0;
    const at = seconds(time, 0);
    const iv = seconds(this.interval, 0) / this.playbackRate;
    this.ids.push(transport.scheduleRepeat((t) => {
      if (n++ >= this.iterations) return;
      this.fire(this.cb as (time: number) => void, t, undefined);
    }, iv, at));
    return this;
  }
}

type PartEvent<V> = [Time, V] | (V & { time: Time });

/** Tone's Part: values at times, the callback at each, looped when asked. */
export class Part<V = any> extends Event<V> {
  loop: boolean | number = false;
  loopStart: Time = 0;
  loopEnd: Time = "1m";
  private evs: [number, V][] = [];
  constructor(private cb: (time: number, value: V) => void, events: PartEvent<V>[] = []) {
    super();
    for (const e of events) this.add(e as any);
  }
  add(time: Time | PartEvent<V>, value?: V) {
    if (Array.isArray(time)) this.evs.push([seconds(time[0], 0), time[1]]);
    else if (typeof time === "object" && time !== null && "time" in (time as any)) this.evs.push([seconds((time as any).time, 0), time as V]);
    else this.evs.push([seconds(time as Time, 0), value as V]);
    return this;
  }
  remove(time: Time) {
    const t = seconds(time, 0);
    this.evs = this.evs.filter(([x]) => Math.abs(x - t) > 1e-9);
    return this;
  }
  clear() {
    this.evs = [];
    return this;
  }
  at(time: Time, value?: V) {
    if (value !== undefined) this.add(time, value);
    return this.evs.find(([x]) => Math.abs(x - seconds(time, 0)) < 1e-9)?.[1];
  }
  get length() {
    return this.evs.length;
  }
  start(time: Time = 0, offset: Time = 0) {
    const at = seconds(time, 0);
    const off = seconds(offset, 0);
    if (this.loop) {
      const ls = seconds(this.loopStart, 0);
      const le = seconds(this.loopEnd, 0);
      const len = le - ls;
      const times = typeof this.loop === "number" ? this.loop : Infinity;
      for (const [t, v] of this.evs) {
        if (t < ls || t >= le) continue;
        let count = 0;
        this.ids.push(transport.scheduleRepeat((tt) => {
          if (count++ >= times) return;
          this.fire(this.cb, tt, v);
        }, len / this.playbackRate, at + (t - ls - off) / this.playbackRate));
      }
    } else {
      for (const [t, v] of this.evs) if (t >= off) this.ids.push(transport.schedule((tt) => this.fire(this.cb, tt, v), at + (t - off) / this.playbackRate));
    }
    return this;
  }
}

type Seq<V> = (V | null | Seq<V>)[];

/** Tone's Sequence: one value per subdivision (null rests; a nested array splits its step), looped by default. */
export class Sequence<V = any> extends Event<V> {
  loop: boolean | number = true;
  subdivision: Time;
  private evs: Seq<V>;
  loopStart = 0;
  loopEnd = 0;
  constructor(private cb: (time: number, value: V) => void, events: Seq<V> = [], subdivision: Time = "8n") {
    super();
    if (typeof cb === "object" && cb !== null) {
      const o = cb as any;
      this.cb = o.callback;
      events = o.events ?? [];
      subdivision = o.subdivision ?? "8n";
      if (o.loop !== undefined) this.loop = o.loop;
    }
    this.evs = events;
    this.subdivision = subdivision;
    this.loopEnd = events.length;
  }
  get events() {
    return this.evs;
  }
  set events(e: Seq<V>) {
    this.evs = e;
    this.loopEnd = e.length;
  }
  start(time: Time = 0, offset = 0) {
    const sub = seconds(this.subdivision, 0) / this.playbackRate;
    const at = seconds(time, 0);
    const flat: [number, V][] = [];
    const walk = (list: Seq<V>, start: number, span: number) => {
      const w = span / list.length;
      list.forEach((x, i) => {
        if (Array.isArray(x)) walk(x as Seq<V>, start + i * w, w);
        else if (x !== null && x !== undefined) flat.push([start + i * w, x as V]);
      });
    };
    walk(this.evs.slice(this.loopStart, this.loopEnd || this.evs.length), 0, (this.loopEnd || this.evs.length) - this.loopStart);
    const len = ((this.loopEnd || this.evs.length) - this.loopStart) * sub;
    const times = this.loop === true ? Infinity : typeof this.loop === "number" ? this.loop : 1;
    for (const [steps, v] of flat) {
      const t = steps * sub - offset * sub;
      if (t < 0 && times === 1) continue;
      let count = 0;
      if (times === 1) this.ids.push(transport.schedule((tt) => this.fire(this.cb, tt, v), at + t));
      else
        this.ids.push(transport.scheduleRepeat((tt) => {
          if (count++ >= times) return;
          this.fire(this.cb, tt, v);
        }, len, at + t));
    }
    return this;
  }
}

/** Tone's Pattern: the values in an order ("up", "down", "upDown", "downUp", "alternateUp", "alternateDown", "random",
 *  "randomOnce", "randomWalk"), one per interval. Random orders are seeded: the same every run. */
export class Pattern<V = any> extends Event<V> {
  interval: Time = "4n";
  private i = 0;
  private dir = 1;
  private order: number[] = [];
  constructor(private cb: (time: number, value: V) => void, public values: V[] = [], public pattern = "up") {
    super();
    if (typeof cb === "object" && cb !== null) {
      const o = cb as any;
      this.cb = o.callback;
      this.values = o.values ?? [];
      this.pattern = o.pattern ?? "up";
      if (o.interval) this.interval = o.interval;
    }
    this.order = this.values.map((_, k) => k);
    if (this.pattern === "randomOnce") for (let k = this.order.length - 1; k > 0; k--) { const j = Math.floor(this.rand() * (k + 1)); [this.order[k], this.order[j]] = [this.order[j], this.order[k]]; }
    if (this.pattern.startsWith("down")) this.i = this.values.length - 1;
  }
  private next(): V {
    const n = this.values.length;
    const p = this.pattern;
    let idx = this.i;
    if (p === "random") idx = Math.floor(this.rand() * n);
    else if (p === "randomOnce") idx = this.order[this.i % n];
    else if (p === "randomWalk") {
      idx = this.i;
      this.i = Math.max(0, Math.min(n - 1, this.i + (this.rand() < 0.5 ? -1 : 1)));
      return this.values[idx];
    }
    const v = this.values[idx];
    if (p === "up" || p === "randomOnce") this.i = (this.i + 1) % n;
    else if (p === "down") this.i = (this.i - 1 + n) % n;
    else if (p === "upDown" || p === "downUp") {
      if (n > 1) {
        if (this.i + this.dir >= n || this.i + this.dir < 0) this.dir = -this.dir;
        this.i += this.dir;
      }
    } else if (p === "alternateUp") this.i = (this.i + (this.dir > 0 ? 2 : -1) + n) % n, (this.dir = -this.dir);
    else if (p === "alternateDown") this.i = (this.i + (this.dir > 0 ? -2 : 1) + n) % n, (this.dir = -this.dir);
    return v;
  }
  start(time: Time = 0) {
    this.ids.push(transport.scheduleRepeat((t) => this.fire(this.cb, t, this.next()), seconds(this.interval, 0) / this.playbackRate, seconds(time, 0)));
    return this;
  }
}
