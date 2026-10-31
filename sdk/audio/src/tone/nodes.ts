// Nodes: connect, chain, toDestination, and the destination.
import { isGeneratedAsset } from "@raea/asset/assets";
import { Param, S, ctx } from "./all";

// ---- nodes --------------------------------------------------------------------------------------------------------

export type Options = Record<string, any>;

function merge(base: Options, over: Options | undefined): Options {
  const out: Options = { ...base };
  for (const [k, v] of Object.entries(over ?? {})) {
    if (v && typeof v === "object" && !Array.isArray(v) && !isGeneratedAsset(v) && base[k] && typeof base[k] === "object") out[k] = merge(base[k], v);
    else out[k] = v;
  }
  return out;
}

/** Every Tone node: an id in the edit's graph, its options (Tone's names), and its connections. */
export class ToneAudioNode {
  readonly id: number;
  options: Options;
  name: string;
  /** a bus's name (Audio and Video elements play into it with bus="...") */
  busName?: string;
  constructor(kind: string, defaults: Options, opts?: Options) {
    this.name = kind;
    this.options = merge(defaults, opts);
    this.id = kind === "Destination" ? 0 : ctx.nextId++;
    ctx.nodes.set(this.id, this);
    if (ctx.graph !== null && kind !== "Destination") {
      // a node made after the first frame: the graph is rebuilt with it (make instruments at the top level)
      ctx.graph = null;
    }
  }
  get volume(): Param {
    return new Param(this, ["volume"], "decibels");
  }
  connect(dest: ToneAudioNode | Param | AudioParamLike, _out = 0, _in = 0): this {
    // Tone.Destination (the proxy) is the Destination node
    if ((dest as unknown) === Destination) dest = getDestination();
    if (dest instanceof ToneAudioNode) {
      ctx.edges.push([this.id, dest.id, 0]);
      if (ctx.graph !== null) ctx.graph = null;
    }
    return this;
  }
  disconnect(dest?: ToneAudioNode): this {
    ctx.edges = ctx.edges.filter(([a, b]) => !(a === this.id && (dest === undefined || b === dest.id)));
    if (ctx.graph !== null) ctx.graph = null;
    return this;
  }
  toDestination(): this {
    return this.connect(getDestination());
  }
  toMaster(): this {
    return this.toDestination();
  }
  chain(...nodes: ToneAudioNode[]): this {
    let prev: ToneAudioNode = this;
    for (const n of nodes) {
      prev.connect(n);
      prev = n;
    }
    return this;
  }
  fan(...nodes: ToneAudioNode[]): this {
    for (const n of nodes) this.connect(n);
    return this;
  }
  set(o: Options): this {
    if (ctx.graph === null) this.options = merge(this.options, o);
    else for (const [k, v] of Object.entries(o)) if (typeof v === "number") ctx.emit({ n: this.id, t: ctx.now, e: "param", p: k, val: v, ramp: 0 });
    return this;
  }
  get(): Options {
    return JSON.parse(JSON.stringify(this.options));
  }
  dispose(): this {
    return this;
  }
  sync(): this {
    return this;
  }
  unsync(): this {
    return this;
  }
  get numberOfInputs() {
    return 1;
  }
  get numberOfOutputs() {
    return 1;
  }
  toString() {
    return this.name;
  }
  /** before the graph is written: settings that come from properties set after construction */
  beforeFreeze() {}
  spec(): Options {
    return { id: this.id, kind: this.name, options: this.options, ...(this.busName ? { name: this.busName } : {}) };
  }
}

export type AudioParamLike = { value: number };

export class DestinationClass extends ToneAudioNode {
  constructor() {
    super("Destination", { volume: 0, mute: false });
  }
  get mute(): boolean {
    return !!this.options.mute;
  }
  set mute(m: boolean) {
    this.options.mute = m;
  }
  get maxChannelCount() {
    return 2;
  }
  override connect(): this {
    return this;
  }
}

export function getDestination(): DestinationClass {
  return (S.destination ??= new DestinationClass());
}

export const Destination = new Proxy({} as DestinationClass, { get: (_, k) => (getDestination() as any)[k], set: (_, k, v) => (((getDestination() as any)[k] = v), true) });

export const Master = Destination;
