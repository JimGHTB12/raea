// Generated assets as values (spec: "Generated assets as values"). lyria() and sao() only describe what to make;
// the renderer generates each one once, under the run's paid-generation cap, and caches it by id.

export type AssetKind = "lyria" | "sao" | "fish" | "tokensynth";

export interface GeneratedAsset {
  kind: AssetKind;
  prompt: string;
  seconds: number;
  /** a stable hash of kind, prompt and seconds */
  id: string;
  /** "asset:<kind>/<id>" */
  src: string;
  /** the generator's own settings (a TokenSynth note's midi, velocity, seed and device; a Fish voice) */
  options?: Record<string, unknown>;
}

// Two independent 32-bit FNV-1a passes (different offset bases) give a 64-bit id as 16 hex digits.
function fnv1a(text: string, basis: number): string {
  let h = basis >>> 0;
  for (let i = 0; i < text.length; i++) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h.toString(16).padStart(8, "0");
}

function describe(kind: AssetKind, prompt: string, seconds: number): GeneratedAsset {
  if (typeof prompt !== "string" || prompt.trim() === "") throw new Error(`${kind}: give a prompt describing the sound`);
  if (typeof seconds !== "number" || !(seconds > 0) || !Number.isFinite(seconds)) {
    throw new RangeError(`${kind}("${prompt}"): seconds must be a positive number, got ${seconds}`);
  }
  const key = `${kind}\u0000${prompt}\u0000${seconds}`;
  const id = fnv1a(key, 0x811c9dc5) + fnv1a(key, 0x050c5d1f);
  return { kind, prompt, seconds, id, src: `asset:${kind}/${id}` };
}

/** Music an older edit generated (30 seconds by default). Retired: a take already made plays, a new one is refused. */
export function lyria(prompt: string, options: { seconds?: number } = {}): GeneratedAsset {
  return describe("lyria", prompt, options.seconds ?? 30);
}

/** A sound effect to generate (2 seconds by default). */
export function sao(prompt: string, options: { seconds?: number } = {}): GeneratedAsset {
  return describe("sao", prompt, options.seconds ?? 2);
}

/** Non-diegetic speech to generate (Fish: a narrator, a UI voice): the words are the prompt. `speed` is Fish's
 * prosody speed (1 is its own pace; a documentary narrator reads nearer 0.85). */
export function fish(text: string, options: { seconds?: number; voice?: string; speed?: number } = {}): GeneratedAsset {
  const a = describe("fish", text, options.seconds ?? 4);
  const o: Record<string, unknown> = {};
  if (options.voice) o.reference_id = options.voice;
  if (options.speed !== undefined) {
    if (!(options.speed >= 0.5 && options.speed <= 2)) throw new RangeError(`fish("${text.slice(0, 24)}..."): speed is 0.5 to 2, got ${options.speed}`);
    o.prosody = { speed: options.speed };
  }
  return Object.keys(o).length ? { ...a, options: o } : a;
}

/** One note of an instrument described in words, rendered by TokenSynth (local, CPU): a Sampler's root note. */
export function tokensynthNote(timbre: string, midi: number, options: { seconds?: number; velocity?: number; seed?: number; precision?: "fp32" | "fp16" | "bf16" | "int8" | "coreml" } = {}): GeneratedAsset {
  const seconds = options.seconds ?? 2;
  const a = describe("tokensynth", timbre, seconds);
  // fp16: 2.6x faster than fp32 with the same next token 99.9% of the time (tools/tokensynth/run.py); it's in the
  // params, so the cache key says which precision made the note
  const o = { midi, velocity: options.velocity ?? 100, seed: options.seed ?? 1234, device: "cpu", precision: options.precision ?? "fp16" };
  const key = `${a.id}:${midi}:${o.velocity}:${o.seed}:${o.precision}`;
  return { ...a, id: key, src: `asset:tokensynth/${key}`, options: o };
}

export function isGeneratedAsset(v: unknown): v is GeneratedAsset {
  return (
    typeof v === "object" &&
    v !== null &&
    ["lyria", "sao", "fish", "tokensynth"].includes((v as GeneratedAsset).kind) &&
    typeof (v as GeneratedAsset).src === "string"
  );
}
