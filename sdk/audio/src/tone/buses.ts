// raea's own: buses, ducking, generated sources.
import { tokensynthNote, type GeneratedAsset } from "@raea/asset/assets";
import { Channel, Frequency, toMidi } from "./all";

// ---- raea: buses, ducking, generated sources -----------------------------------------------------------------------

export const busMap = new Map<string, Channel>();

/** raea: a named bus, a Channel into the Destination that <Audio>/<Video bus="..."> and Tone nodes play into. */
export function bus(name: string): Channel {
  let b = busMap.get(name);
  if (!b) {
    b = new Channel();
    b.busName = name;
    b.toDestination();
    busMap.set(name, b);
  }
  return b;
}

/** raea: the standard buses (music, sfx, voice, ambience, ui). `buses.music.duck(buses.voice)` sidechains the music. */
export const buses = {
  get music() {
    return bus("music");
  },
  get sfx() {
    return bus("sfx");
  },
  get voice() {
    return bus("voice");
  },
  get ambience() {
    return bus("ambience");
  },
  get ui() {
    return bus("ui");
  },
};

/** raea: a TokenSynth instrument (MIDI to a stem by a timbre prompt, rendered locally once and cached as PCM). Use as a
 *  Sampler's urls: `new Tone.Sampler({ urls: tokensynth("a warm upright bass", ["C2", "C3", "C4"]) })`; each root note is
 *  rendered (`seconds` long) and the Sampler pitches between them. */
export function tokensynth(timbre: string, notes: Frequency[] = ["C2", "C3", "C4"], opts: { seconds?: number; velocity?: number; seed?: number; precision?: "fp32" | "fp16" | "bf16" | "int8" | "coreml" } = {}): Record<string, GeneratedAsset> {
  const out: Record<string, GeneratedAsset> = {};
  for (const n of notes) {
    const midi = Math.round(toMidi(n));
    out[String(midi)] = tokensynthNote(timbre, midi, opts);
  }
  return out;
}
