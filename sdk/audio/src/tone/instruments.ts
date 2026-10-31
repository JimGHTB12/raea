// Instruments: synths, samplers and players.
import { isGeneratedAsset, type GeneratedAsset } from "@raea/asset/assets";
import { Frequency, NOTE, Options, Param, Time, ToneAudioNode, ctx, mtof, seconds, toHz, toMidi } from "./all";

// ---- instruments --------------------------------------------------------------------------------------------------

const ENV = { attack: 0.005, decay: 0.1, sustain: 0.3, release: 1 };

/** Tone's monophonic and polyphonic instruments: triggerAttack, triggerRelease, triggerAttackRelease, releaseAll. */
export class Instrument extends ToneAudioNode {
  get oscillator(): Options {
    return (this.options.oscillator ??= {});
  }
  get envelope(): Options {
    return (this.options.envelope ??= {});
  }
  get detune(): Param {
    return new Param(this, ["detune"], "cents");
  }
  get portamento(): number {
    return this.options.portamento ?? 0;
  }
  set portamento(p: number) {
    this.options.portamento = p;
  }
  get frequency(): Param {
    return new Param(this, ["frequency"], "frequency");
  }
  protected notes(n: Frequency | Frequency[]): number[] {
    return (Array.isArray(n) ? n : [n]).map(toHz);
  }
  triggerAttack(note: Frequency | Frequency[], time?: Time, velocity = 1): this {
    ctx.emit({ n: this.id, t: seconds(time), e: "a", f: this.notes(note), v: velocity });
    return this;
  }
  triggerRelease(note?: Frequency | Frequency[] | Time, time?: Time): this {
    // Tone's monophonic instruments take triggerRelease(time); PolySynth and Sampler take (notes, time)
    if (this.poly() && note !== undefined && (typeof note !== "number" || time !== undefined)) {
      ctx.emit({ n: this.id, t: seconds(time), e: "r", f: this.notes(note as Frequency | Frequency[]) });
    } else ctx.emit({ n: this.id, t: seconds((this.poly() ? time : (note as Time)) ?? time), e: "ra" });
    return this;
  }
  triggerAttackRelease(note: Frequency | Frequency[], duration: Time | Time[], time?: Time, velocity = 1): this {
    const t = seconds(time);
    const notes = this.notes(note);
    if (Array.isArray(duration)) notes.forEach((f, i) => ctx.emit({ n: this.id, t, e: "ar", f: [f], d: seconds(duration[i % duration.length], 0), v: velocity }));
    else ctx.emit({ n: this.id, t, e: "ar", f: notes, d: seconds(duration, 0), v: velocity });
    return this;
  }
  releaseAll(time?: Time): this {
    ctx.emit({ n: this.id, t: seconds(time), e: "ra" });
    return this;
  }
  poly(): boolean {
    return !!this.options.poly;
  }
}

function inst(kind: string, defaults: Options) {
  return class extends Instrument {
    constructor(opts?: Options) {
      super(kind, defaults, opts);
    }
  };
}

export const Synth = inst("Synth", { oscillator: { type: "triangle" }, envelope: { ...ENV }, volume: 0, detune: 0, portamento: 0 });

export const MonoSynth = inst("MonoSynth", {
  oscillator: { type: "square" },
  envelope: { attack: 0.005, decay: 0.1, sustain: 0.9, release: 1 },
  filter: { Q: 1, type: "lowpass", rolloff: -12 },
  filterEnvelope: { attack: 0.6, decay: 0.2, sustain: 0.5, release: 2, baseFrequency: 200, octaves: 3, exponent: 2 },
  volume: 0,
  detune: 0,
  portamento: 0,
});

export const FMSynth = inst("FMSynth", { harmonicity: 3, modulationIndex: 10, oscillator: { type: "sine" }, envelope: { attack: 0.01, decay: 0.01, sustain: 1, release: 0.5 }, modulation: { type: "square" }, modulationEnvelope: { attack: 0.5, decay: 0, sustain: 1, release: 0.5 }, volume: 0, detune: 0 });

export const AMSynth = inst("AMSynth", { harmonicity: 3, oscillator: { type: "sine" }, envelope: { attack: 0.01, decay: 0.01, sustain: 1, release: 0.5 }, modulation: { type: "square" }, modulationEnvelope: { attack: 0.5, decay: 0, sustain: 1, release: 0.5 }, volume: 0, detune: 0 });

export const DuoSynth = inst("DuoSynth", { harmonicity: 1.5, modulationIndex: 0.5, oscillator: { type: "sine" }, envelope: { attack: 0.01, decay: 0, sustain: 1, release: 0.5 }, volume: 0 });

export const MembraneSynth = inst("MembraneSynth", { pitchDecay: 0.05, octaves: 10, oscillator: { type: "sine" }, envelope: { attack: 0.001, decay: 0.4, sustain: 0.01, release: 1.4 }, volume: 0 });

export const NoiseSynth = inst("NoiseSynth", { noise: { type: "white" }, envelope: { attack: 0.005, decay: 0.1, sustain: 0 , release: 1 }, volume: 0 });

export const MetalSynth = inst("MetalSynth", { frequency: 200, harmonicity: 5.1, modulationIndex: 32, resonance: 4000, octaves: 1.5, envelope: { attack: 0.001, decay: 1.4, sustain: 0, release: 0.2 }, volume: 0 });

export const PluckSynth = inst("PluckSynth", { attackNoise: 1, dampening: 4000, resonance: 0.7, release: 1, volume: 0 });

// NoiseSynth and MetalSynth take (duration, time) and (note?, duration, time) as Tone's do
const noiseAR = NoiseSynth.prototype.triggerAttackRelease;

NoiseSynth.prototype.triggerAttack = function (this: Instrument, time?: Time, velocity = 1) {
  ctx.emit({ n: this.id, t: seconds(time as Time), e: "a", f: [440], v: velocity });
  return this;
} as any;

NoiseSynth.prototype.triggerAttackRelease = function (this: Instrument, duration: Time, time?: Time, velocity = 1) {
  return noiseAR.call(this, 440, duration, time, velocity);
} as any;

MetalSynth.prototype.triggerAttackRelease = function (this: Instrument, a: any, b?: any, c?: any, d?: any) {
  // (duration, time, velocity) or (note, duration, time, velocity)
  // Tone 15: (note, duration, time, velocity); older code: (duration, time, velocity). A note name, or a frequency, first
  const withNote = (typeof a === "string" && NOTE.test(a.trim())) || (typeof a === "number" && a > 20);
  return withNote ? Instrument.prototype.triggerAttackRelease.call(this, a, b, c, d) : Instrument.prototype.triggerAttackRelease.call(this, 440, a, b, c ?? 1);
} as any;

/** Tone's PolySynth: many voices of one instrument (`new PolySynth(Synth, options)`), at most maxPolyphony. */
export class PolySynth extends Instrument {
  constructor(voice?: any, opts?: Options) {
    let V = voice;
    let o = opts;
    if (voice && typeof voice === "object" && !(typeof voice === "function")) {
      o = voice;
      V = voice.voice;
    }
    if (typeof voice === "number") {
      // the legacy (polyphony, voice, options) form
      V = opts as any;
      o = arguments[2];
    }
    const proto = V ? new V() : new Synth();
    ctx.nodes.delete(proto.id);
    super(proto.name, { ...proto.options, maxPolyphony: 32, poly: true }, o && "options" in o ? o.options : o);
    this.options.poly = true;
    if (o && typeof o.maxPolyphony === "number") this.options.maxPolyphony = o.maxPolyphony;
  }
  get maxPolyphony(): number {
    return this.options.maxPolyphony;
  }
  set maxPolyphony(n: number) {
    this.options.maxPolyphony = n;
  }
}

type UrlMap = Record<string | number, string | GeneratedAsset>;

export function src(u: string | GeneratedAsset, baseUrl = ""): unknown {
  if (isGeneratedAsset(u)) return { generate: u.kind, prompt: u.prompt, seconds: u.seconds, ...(u.options ?? {}) };
  return /^(https?:|\/|[a-z]:)/i.test(u as string) ? u : baseUrl + u;
}

/** Tone's Sampler: samples at notes, played at the rate that makes any other note (the nearest sample). */
export class Sampler extends Instrument {
  constructor(urls?: UrlMap | Options, onload?: (() => void) | Options, baseUrl?: string) {
    const o: Options = urls && "urls" in (urls as Options) ? (urls as Options) : { urls, ...(typeof onload === "object" ? onload : {}), baseUrl: baseUrl ?? (typeof onload === "object" ? onload.baseUrl : undefined) };
    const map: Options = {};
    for (const [k, v] of Object.entries((o.urls ?? {}) as UrlMap)) map[String(Math.round(toMidi(/^\d+$/.test(k) ? Number(k) > 127 ? Number(k) : mtof(Number(k)) : k)))] = src(v, o.baseUrl ?? "");
    super("Sampler", { attack: 0, release: 0.1, curve: "exponential", volume: 0, poly: true, maxPolyphony: 32 }, { ...o, urls: map, baseUrl: undefined });
    (typeof onload === "function" ? onload : o.onload)?.();
  }
  get loaded() {
    return true;
  }
  add(note: Frequency, url: string | GeneratedAsset, cb?: () => void) {
    this.options.urls[String(Math.round(toMidi(note)))] = src(url);
    cb?.();
    return this;
  }
  has(note: Frequency) {
    return String(Math.round(toMidi(note))) in this.options.urls;
  }
  get attack() {
    return this.options.attack;
  }
  set attack(v: number) {
    this.options.attack = v;
  }
  get release() {
    return this.options.release;
  }
  set release(v: number) {
    this.options.release = v;
  }
}

/** Tone's Player: one file, started and stopped on the timeline. */
export class Player extends Instrument {
  constructor(url?: string | GeneratedAsset | Options, onload?: () => void) {
    const o: Options = typeof url === "string" || isGeneratedAsset(url) ? { url } : (url ?? {});
    super("Player", { playbackRate: 1, loop: false, loopStart: 0, loopEnd: 0, fadeIn: 0, fadeOut: 0, volume: 0, poly: true, maxPolyphony: 32, reverse: false }, { ...o, url: o.url !== undefined ? src(o.url) : undefined });
    (onload ?? o.onload)?.();
    if (o.autostart) this.start(0);
  }
  get loaded() {
    return true;
  }
  get loop() {
    return !!this.options.loop;
  }
  set loop(v: boolean) {
    this.options.loop = v;
  }
  get playbackRate() {
    return this.options.playbackRate;
  }
  set playbackRate(v: number) {
    this.options.playbackRate = v;
  }
  get loopStart() {
    return this.options.loopStart;
  }
  set loopStart(v: number) {
    this.options.loopStart = seconds(v as Time, 0);
  }
  get loopEnd() {
    return this.options.loopEnd;
  }
  set loopEnd(v: number) {
    this.options.loopEnd = seconds(v as Time, 0);
  }
  get fadeIn() {
    return this.options.fadeIn;
  }
  set fadeIn(v: number) {
    this.options.fadeIn = v;
  }
  get fadeOut() {
    return this.options.fadeOut;
  }
  set fadeOut(v: number) {
    this.options.fadeOut = v;
  }
  load(url: string | GeneratedAsset) {
    this.options.url = src(url);
    return Promise.resolve(this);
  }
  start(time?: Time, offset?: Time, duration?: Time): this {
    ctx.emit({ n: this.id, t: seconds(time), e: "start", o: offset !== undefined ? seconds(offset, 0) : 0, ...(duration !== undefined ? { d: seconds(duration, 0) } : {}), v: 1 });
    return this;
  }
  stop(time?: Time): this {
    ctx.emit({ n: this.id, t: seconds(time), e: "stop" });
    return this;
  }
  restart(time?: Time, offset?: Time, duration?: Time): this {
    this.stop(time);
    return this.start(time, offset, duration);
  }
  seek(offset: Time, time?: Time): this {
    return this.restart(time, offset);
  }
  get state() {
    return "stopped";
  }
}

export const GrainPlayer = Player;

/** Tone's Players: named Players into one output. */
export class Players extends ToneAudioNode {
  private map = new Map<string, Player>();
  constructor(urls: Record<string, string | GeneratedAsset> = {}, onload?: (() => void) | Options) {
    super("Gain", { gain: 1, volume: 0 }, typeof onload === "object" ? onload : {});
    for (const [k, v] of Object.entries(urls)) this.add(k, v);
    (typeof onload === "function" ? onload : undefined)?.();
  }
  add(name: string, url: string | GeneratedAsset, cb?: () => void) {
    const p = new Player(url);
    p.connect(this);
    this.map.set(name, p);
    cb?.();
    return this;
  }
  player(name: string): Player {
    const p = this.map.get(name);
    if (!p) throw new Error(`Tone.Players: no player "${name}"`);
    return p;
  }
  has(name: string) {
    return this.map.has(name);
  }
  stopAll(time?: Time) {
    for (const p of this.map.values()) p.stop(time);
    return this;
  }
  get loaded() {
    return true;
  }
}

/** Tone's sources as instruments that start and stop: Oscillator and Noise (and their variants). */
export class Oscillator extends Instrument {
  constructor(frequency: Frequency | Options = 440, type = "sine") {
    const o = typeof frequency === "object" ? frequency : { frequency, type };
    super("Synth", { oscillator: { type: o.type ?? "sine" }, envelope: { attack: 0.003, decay: 0, sustain: 1, release: 0.003 }, volume: o.volume ?? 0, poly: false }, {});
    this.options.frequency = toHz(o.frequency ?? 440);
  }
  get type() {
    return this.options.oscillator.type;
  }
  set type(t: string) {
    this.options.oscillator.type = t;
  }
  start(time?: Time): this {
    ctx.emit({ n: this.id, t: seconds(time), e: "a", f: [this.options.frequency], v: 1 });
    return this;
  }
  stop(time?: Time): this {
    ctx.emit({ n: this.id, t: seconds(time), e: "ra" });
    return this;
  }
}

export const OmniOscillator = Oscillator;

export const FatOscillator = Oscillator;

export const PulseOscillator = Oscillator;

export class Noise extends Instrument {
  constructor(type: string | Options = "white") {
    const o = typeof type === "object" ? type : { type };
    super("NoiseSynth", { noise: { type: o.type ?? "white" }, envelope: { attack: 0.003, decay: 0, sustain: 1, release: 0.003 }, volume: o.volume ?? 0 }, {});
  }
  get type() {
    return this.options.noise.type;
  }
  set type(t: string) {
    this.options.noise.type = t;
  }
  start(time?: Time): this {
    ctx.emit({ n: this.id, t: seconds(time), e: "a", f: [440], v: 1 });
    return this;
  }
  stop(time?: Time): this {
    ctx.emit({ n: this.id, t: seconds(time), e: "ra" });
    return this;
  }
}
