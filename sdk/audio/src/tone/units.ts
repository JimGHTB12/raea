// Tone's units: time notation, frequencies and notes.
import { ctx, transport } from "./all";

// ---- units --------------------------------------------------------------------------------------------------------

export type Time = number | string;

export type Frequency = number | string;

export const NOTE = /^([a-gA-G])(#{1,2}|b{1,2}|x)?(-?\d+)$/;

const OFFSETS: Record<string, number> = { c: 0, d: 2, e: 4, f: 5, g: 7, a: 9, b: 11 };

export function mtof(m: number): number {
  return 440 * Math.pow(2, (m - 69) / 12);
}

export function ftom(f: number): number {
  return Math.round(69 + 12 * Math.log2(f / 440));
}

export function dbToGain(db: number): number {
  return Math.pow(10, db / 20);
}

export function gainToDb(g: number): number {
  return 20 * Math.log10(g);
}

export function intervalToFrequencyRatio(i: number): number {
  return Math.pow(2, i / 12);
}

export function toMidi(n: Frequency): number {
  if (typeof n === "number") return 69 + 12 * Math.log2(n / 440);
  const s = n.trim();
  const m = NOTE.exec(s);
  if (m) {
    const acc = m[2] === "x" ? 2 : m[2] ? (m[2][0] === "#" ? m[2].length : -m[2].length) : 0;
    return 12 * (Number(m[3]) + 1) + OFFSETS[m[1].toLowerCase()] + acc;
  }
  if (/hz$/i.test(s)) return 69 + 12 * Math.log2(parseFloat(s) / 440);
  if (/^\d+(\.\d+)?$/.test(s)) return 69 + 12 * Math.log2(parseFloat(s) / 440);
  throw new RangeError(`Tone: "${n}" isn't a note or a frequency`);
}

export function toHz(n: Frequency): number {
  if (typeof n === "number") return n;
  return mtof(toMidi(n));
}

const NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

class FrequencyClass {
  constructor(private v: Frequency, private units?: string) {}
  private hz() {
    return this.units === "midi" ? mtof(Number(this.v)) : toHz(this.v);
  }
  toFrequency() {
    return this.hz();
  }
  toMidi() {
    return Math.round(69 + 12 * Math.log2(this.hz() / 440));
  }
  toNote() {
    const m = this.toMidi();
    return `${NAMES[((m % 12) + 12) % 12]}${Math.floor(m / 12) - 1}`;
  }
  toSeconds() {
    return 1 / this.hz();
  }
  transpose(i: number) {
    return new FrequencyClass(this.hz() * intervalToFrequencyRatio(i));
  }
  harmonize(is: number[]) {
    return is.map((i) => this.transpose(i));
  }
  valueOf() {
    return this.hz();
  }
}

export function Frequency(v: Frequency, units?: string) {
  return new FrequencyClass(v, units);
}

export function Midi(v: number) {
  return new FrequencyClass(v, "midi");
}

/** Tone's time notation in seconds, at the Transport's tempo and meter. */
export function seconds(t: Time | undefined, relativeTo = ctx.now): number {
  if (t === undefined || t === null) return relativeTo;
  if (typeof t === "number") return t;
  let s = String(t).trim();
  if (s === "") return relativeTo;
  if (s.startsWith("+")) return relativeTo + seconds(s.slice(1), 0);
  if (s.startsWith("@")) {
    const q = seconds(s.slice(1), 0);
    return q > 0 ? Math.ceil(relativeTo / q - 1e-9) * q : relativeTo;
  }
  if (s.includes(" + ")) return s.split(" + ").reduce((a, x) => a + seconds(x, 0), 0);
  const tr = transport;
  const beat = 60 / tr.bpm.value;
  const [num, den] = tr.ts();
  const bar = num * beat * (4 / den);
  let m = /^(-?\d+(?:\.\d+)?):(\d+(?:\.\d+)?)(?::(\d+(?:\.\d+)?))?$/.exec(s);
  if (m) return Number(m[1]) * bar + Number(m[2]) * beat * (4 / den) + Number(m[3] ?? 0) * (beat / 4);
  m = /^(\d+(?:\.\d+)?)(n|t|m)(\.?)$/.exec(s);
  if (m) {
    const v = Number(m[1]);
    let x = m[2] === "m" ? v * bar : (4 / v) * beat;
    if (m[2] === "t") x *= 2 / 3;
    if (m[3] === ".") x *= 1.5;
    return x;
  }
  m = /^(\d+(?:\.\d+)?)i$/.exec(s);
  if (m) return (Number(m[1]) / tr.PPQ) * beat;
  m = /^(\d+(?:\.\d+)?)hz$/i.exec(s);
  if (m) return 1 / Number(m[1]);
  if (/^-?\d+(\.\d+)?s?$/.test(s)) return parseFloat(s);
  throw new RangeError(`Tone: "${t}" isn't a time (seconds, "4n", "8t", "1m", "+0.5", "1:2:3" or "192i")`);
}

class TimeClass {
  constructor(private t: Time) {}
  toSeconds() {
    return seconds(this.t, 0);
  }
  toMilliseconds() {
    return this.toSeconds() * 1000;
  }
  toTicks() {
    return Math.round((this.toSeconds() / (60 / transport.bpm.value)) * transport.PPQ);
  }
  toFrequency() {
    return 1 / this.toSeconds();
  }
  toBarsBeatsSixteenths() {
    return position(this.toSeconds());
  }
  /** raea: the time in frames at `fps` (cuts on beats: <Series.Sequence durationInFrames={Tone.Time("1m").toFrames(30)}>) */
  toFrames(fps = 30) {
    return Math.round(this.toSeconds() * fps);
  }
  valueOf() {
    return this.toSeconds();
  }
}

export function Time(t: Time) {
  return new TimeClass(t);
}

export const TransportTime = Time;

export const Ticks = (t: Time) => new TimeClass(t).toTicks();

export function position(secs: number): string {
  const beat = 60 / transport.bpm.value;
  const [num, den] = transport.ts();
  const qn = secs / beat;
  const per = num * (4 / den);
  const bars = Math.floor(qn / per + 1e-9);
  const beats = Math.floor(qn - bars * per + 1e-9);
  const six = Math.round((qn - bars * per - beats) * 4 * 1000) / 1000;
  return `${bars}:${beats}:${six}`;
}
