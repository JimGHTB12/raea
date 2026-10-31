// Effects and mixing.
import { isGeneratedAsset, type GeneratedAsset } from "@raea/asset/assets";
import { AudioParamLike, Options, Param, S, Time, ToneAudioNode, ctx, dbToGain, seconds, src } from "./all";

// ---- effects and mixing -------------------------------------------------------------------------------------------

/** Every effect and mixing node: its options, Params by Tone's names, wet, and start() for the LFO ones. */
export class Effect extends ToneAudioNode {
  get wet(): Param {
    return new Param(this, ["wet"]);
  }
  start(time?: Time): this {
    this.options.started = seconds(time);
    return this;
  }
  stop(_time?: Time): this {
    return this;
  }
}

function fx(kind: string, defaults: Options, positional: string[] = [], params: string[] = []) {
  const C = class extends Effect {
    constructor(...args: any[]) {
      const o: Options = args.length === 1 && args[0] && typeof args[0] === "object" ? args[0] : Object.fromEntries(positional.map((k, i) => [k, args[i]]).filter(([, v]) => v !== undefined));
      for (const k of ["delayTime", "frequency", "preDelay"]) if (typeof o[k] === "string") o[k] = k === "frequency" ? 1 / seconds(o[k], 0) : seconds(o[k], 0);
      super(kind, defaults, o);
    }
  };
  for (const p of params) Object.defineProperty(C.prototype, p, { get(this: Effect) { return new Param(this, [p]); } });
  return C;
}

export const Reverb = fx("Reverb", { decay: 1.5, preDelay: 0.01, wet: 1 }, ["decay"], ["decay", "preDelay"]);

export const Freeverb = fx("Freeverb", { roomSize: 0.7, dampening: 3000, wet: 1 }, ["roomSize", "dampening"], ["roomSize", "dampening"]);

export const JCReverb = fx("JCReverb", { roomSize: 0.5, wet: 1 }, ["roomSize"], ["roomSize"]);

export const FeedbackDelay = fx("FeedbackDelay", { delayTime: 0.25, feedback: 0.125, maxDelay: 1, wet: 1 }, ["delayTime", "feedback"], ["delayTime", "feedback"]);

export const PingPongDelay = fx("PingPongDelay", { delayTime: 0.25, feedback: 0.125, maxDelay: 1, wet: 1 }, ["delayTime", "feedback"], ["delayTime", "feedback"]);

export const Filter = fx("Filter", { frequency: 350, type: "lowpass", rolloff: -12, Q: 1, gain: 0 }, ["frequency", "type", "rolloff"], ["frequency", "Q", "gain"]);

export const AutoFilter = fx("AutoFilter", { frequency: 1, depth: 1, baseFrequency: 200, octaves: 2.6, filter: { type: "lowpass", rolloff: -12, Q: 1 }, wet: 1 }, ["frequency", "baseFrequency", "octaves"], ["frequency", "depth"]);

export const Compressor = fx("Compressor", { threshold: -24, ratio: 12, attack: 0.003, release: 0.25, knee: 30 }, ["threshold", "ratio"], ["threshold", "ratio", "attack", "release", "knee"]);

export const Limiter = fx("Limiter", { threshold: -12 }, ["threshold"], ["threshold"]);

export const EQ3 = fx("EQ3", { low: 0, mid: 0, high: 0, lowFrequency: 400, highFrequency: 2500 }, ["low", "mid", "high"], ["low", "mid", "high", "lowFrequency", "highFrequency"]);

export const Distortion = fx("Distortion", { distortion: 0.4, oversample: "none", wet: 1 }, ["distortion"]);

export const Chorus = fx("Chorus", { frequency: 1.5, delayTime: 3.5, depth: 0.7, spread: 180, type: "sine", wet: 0.5 }, ["frequency", "delayTime", "depth"], ["frequency", "depth", "spread"]);

export const BitCrusher = fx("BitCrusher", { bits: 4, wet: 1 }, ["bits"], ["bits"]);

export const Tremolo = fx("Tremolo", { frequency: 10, depth: 0.5, spread: 180, type: "sine", wet: 1 }, ["frequency", "depth"], ["frequency", "depth", "spread"]);

export const Vibrato = fx("Vibrato", { frequency: 5, depth: 0.1, type: "sine", wet: 1 }, ["frequency", "depth"], ["frequency", "depth"]);

export const Phaser = fx("Phaser", { frequency: 0.5, octaves: 3, baseFrequency: 350, Q: 10, wet: 1 }, ["frequency", "octaves", "baseFrequency"], ["frequency", "octaves", "baseFrequency", "Q"]);

export const AutoPanner = fx("AutoPanner", { frequency: 1, depth: 1, type: "sine", wet: 1 }, ["frequency"], ["frequency", "depth"]);

// Distortion's amount is its `distortion` property
Object.defineProperty(Distortion.prototype, "distortion", {
  get(this: Effect) {
    return this.options.distortion;
  },
  set(this: Effect, v: number) {
    this.options.distortion = v;
  },
});

export class Gain extends ToneAudioNode {
  constructor(gain: number | Options = 1, _units?: string) {
    super("Gain", { gain: 1 }, typeof gain === "object" ? gain : { gain: _units === "decibels" ? dbToGain(gain) : gain });
  }
  get gain(): Param {
    return new Param(this, ["gain"], "gain");
  }
}

export class Volume extends ToneAudioNode {
  constructor(volume: number | Options = 0) {
    super("Volume", { volume: 0, mute: false }, typeof volume === "object" ? volume : { volume });
  }
  get mute(): boolean {
    return !!this.options.mute;
  }
  set mute(m: boolean) {
    if (ctx.graph === null) this.options.mute = m;
    else ctx.emit({ n: this.id, t: ctx.now, e: "param", p: "mute", val: m ? 1 : 0, ramp: 0 });
  }
}

export class Panner extends ToneAudioNode {
  constructor(pan: number | Options = 0) {
    super("Panner", { pan: 0 }, typeof pan === "object" ? pan : { pan });
  }
  get pan(): Param {
    return new Param(this, ["pan"], "audioRange");
  }
}

export const receivers = new Map<string, Channel>();

export const pendingSends: [Channel, string, Gain][] = [];

/** Tone's Channel: volume, pan, mute and solo, with sends to named buses. raea: duck() under another channel. */
export class Channel extends ToneAudioNode {
  constructor(volume: number | Options = 0, pan = 0) {
    super("Channel", { volume: 0, pan: 0, mute: false, solo: false }, typeof volume === "object" ? volume : { volume, pan });
  }
  get pan(): Param {
    return new Param(this, ["pan"], "audioRange");
  }
  get mute(): boolean {
    return !!this.options.mute;
  }
  set mute(m: boolean) {
    if (ctx.graph === null) this.options.mute = m;
    else ctx.emit({ n: this.id, t: ctx.now, e: "param", p: "mute", val: m ? 1 : 0, ramp: 0 });
  }
  get muted(): boolean {
    return this.mute;
  }
  get solo(): boolean {
    return !!this.options.solo;
  }
  set solo(s: boolean) {
    this.options.solo = s;
    if (ctx.graph !== null) ctx.graph = null;
  }
  /** a send of this channel's signal, `volume` dB, to the channel that receive()s `name` */
  send(name: string, volume = 0): Gain {
    const g = new Gain(dbToGain(volume));
    this.connect(g);
    const r = receivers.get(name);
    if (r) g.connect(r);
    else pendingSends.push([this, name, g]);
    return g;
  }
  receive(name: string): this {
    receivers.set(name, this);
    for (const [, n, g] of pendingSends.filter((p) => p[1] === name)) g.connect(this);
    return this;
  }
  /** raea: this channel ducked under `key` (music and ambience under voice): `amount` dB when the key is over `threshold`. */
  duck(key: ToneAudioNode, opts: { amount?: number; threshold?: number; attack?: number; release?: number } = {}): this {
    this.options.duck = { amount: -12, threshold: -40, attack: 0.05, release: 0.3, ...opts };
    ctx.edges.push([key.id, this.id, 1]);
    if (ctx.graph !== null) ctx.graph = null;
    return this;
  }
}

export class PanVol extends Channel {}

export class Solo extends ToneAudioNode {
  constructor(solo: boolean | Options = false) {
    super("Solo", { solo: false }, typeof solo === "object" ? solo : { solo });
  }
  get solo(): boolean {
    return !!this.options.solo;
  }
  set solo(s: boolean) {
    this.options.solo = s;
  }
}

/** Analysis nodes have nothing to measure here: they pass sound through and read as silence. */
export class Meter extends ToneAudioNode {
  constructor(_o?: unknown) {
    super("Gain", { gain: 1 });
  }
  getValue() {
    return -Infinity;
  }
}

export class FFT extends Meter {
  getValue() {
    return new Float32Array(1024).fill(-Infinity);
  }
}

export class Waveform extends Meter {
  getValue() {
    return new Float32Array(1024);
  }
}

export const Analyser = FFT;

/** Tone's LFO: a low-frequency oscillator between `min` and `max`. Connected to a Param, its output is the parameter's
 *  value (as in Tone, the parameter's own value is overridden), sample by sample on the core, from start() on. */
export class LFO extends ToneAudioNode {
  constructor(frequency: Time | Options = "4n", min = 0, max = 1) {
    const o = typeof frequency === "object" ? frequency : { frequency, min, max };
    super("LFO", { frequency: 1, min: 0, max: 1, type: "sine", phase: 0, amplitude: 1, started: null }, o);
    if (typeof this.options.frequency === "string") this.options.frequency = 1 / seconds(this.options.frequency, 0);
  }
  get frequency(): Param {
    return new Param(this, ["frequency"], "frequency");
  }
  get min() {
    return this.options.min;
  }
  set min(v: number) {
    this.options.min = v;
  }
  get max() {
    return this.options.max;
  }
  set max(v: number) {
    this.options.max = v;
  }
  get type() {
    return this.options.type;
  }
  set type(t: string) {
    this.options.type = t;
  }
  override connect(dest: ToneAudioNode | Param | AudioParamLike): this {
    if (dest instanceof Param) {
      S.mods.push([this.id, dest.owner.id, dest.key[dest.key.length - 1]]);
      if (ctx.graph !== null) ctx.graph = null;
      return this;
    }
    return super.connect(dest as ToneAudioNode);
  }
  start(time?: Time): this {
    this.options.started = seconds(time);
    return this;
  }
  stop(_time?: Time): this {
    return this;
  }
  sync(): this {
    return this;
  }
}

/** Tone's Convolver: the sound convolved with an impulse response (a media file or a generated sound). */
export class Convolver extends Effect {
  constructor(url?: string | GeneratedAsset | Options) {
    const o: Options = typeof url === "string" || isGeneratedAsset(url) ? { url } : (url ?? {});
    super("Convolver", { normalize: true, wet: 1 }, { ...o, url: o.url !== undefined ? src(o.url) : undefined });
  }
  load(url: string | GeneratedAsset) {
    this.options.url = src(url);
    return Promise.resolve(this);
  }
}

/** Tone's PitchShift: two delay taps sweeping a window, cross-faded (Tone's own algorithm), `pitch` in semitones. */
export const PitchShift = fx("PitchShift", { pitch: 0, windowSize: 0.1, delayTime: 0, feedback: 0, wet: 1 }, ["pitch"], ["delayTime", "feedback"]);

Object.defineProperty(PitchShift.prototype, "pitch", {
  get(this: Effect) {
    return this.options.pitch;
  },
  set(this: Effect, v: number) {
    if (ctx.graph === null) this.options.pitch = v;
    else ctx.emit({ n: this.id, t: ctx.now, e: "param", p: "pitch", val: v, ramp: 0 });
  },
});

/** Tone.Offline renders a callback's sound into a buffer in the browser's audio engine; there is no audio engine in this
 *  JavaScript (the core renders a composition's sound), so it can't run here. */
export function Offline(): never {
  throw new Error("Tone.Offline: there's no audio engine in raea's JavaScript; render the composition (its sound is the core's bounce) instead");
}
