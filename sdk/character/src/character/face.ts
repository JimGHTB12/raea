// A psx body's face: MakeHuman's 34 expression units (MPFB2's CC0 targets/expression/units, the FACS-like pieces
// MakeHuman mixes its own expressions from), baked into body.glb as morph targets by projects/psx (faces.py), each
// mixed for the person's ancestry. <Character face> sets any of them by name, or a preset (smile, frown ...) that is
// a mix of them; presets and units add, and each unit's weight is held to 0..1.

/** The 34 units, in the body's morph targets' order. */
export const FACE_UNITS = [
  "eye-left-closure", "eye-left-opened-up", "eye-left-slit", "eye-right-closure", "eye-right-opened-up", "eye-right-slit",
  "eyebrows-left-down", "eyebrows-left-extern-up", "eyebrows-left-inner-up", "eyebrows-left-up",
  "eyebrows-right-down", "eyebrows-right-extern-up", "eyebrows-right-inner-up", "eyebrows-right-up",
  "mouth-compression", "mouth-corner-puller", "mouth-depression", "mouth-depression-retraction", "mouth-elevation",
  "mouth-eversion", "mouth-open", "mouth-parling", "mouth-part-later", "mouth-protusion", "mouth-pursing",
  "mouth-retraction", "mouth-upward-retraction", "neck-platysma", "nose-compression", "nose-depression",
  "nose-left-dilatation", "nose-left-elevation", "nose-right-dilatation", "nose-right-elevation",
] as const;
export type FaceUnit = (typeof FACE_UNITS)[number];

const both = (unit: string, w: number) => ({ [unit.replace("SIDE", "left")]: w, [unit.replace("SIDE", "right")]: w }) as Partial<Record<FaceUnit, number>>;

/** Expressions as mixes of the units. Lips: open (a jaw drop), pucker (oo, w), wide (ee); feelings: smile, frown,
 * surprise, sad; and blink. */
export const FACE_PRESETS = {
  blink: both("eye-SIDE-closure", 1),
  // the full corner pull read as a grin at 1; this is a warm smile
  smile: { "mouth-corner-puller": 0.55, "mouth-upward-retraction": 0.15, ...both("eye-SIDE-slit", 0.25) },
  frown: { ...both("eyebrows-SIDE-down", 1), "mouth-depression": 0.7, ...both("nose-SIDE-elevation", 0.35) },
  surprise: { ...both("eyebrows-SIDE-up", 1), ...both("eye-SIDE-opened-up", 0.7) },
  open: { "mouth-open": 1 },
  pucker: { "mouth-pursing": 1, "mouth-protusion": 0.5 },
  wide: { "mouth-retraction": 1 },
  sad: { ...both("eyebrows-SIDE-inner-up", 1), "mouth-depression": 0.5 },
} satisfies Record<string, Partial<Record<FaceUnit, number>>>;
export type FacePreset = keyof typeof FACE_PRESETS;
export type Face = Partial<Record<FaceUnit | FacePreset, number>>;

/** every unit's weight for a face (all 34 named, so a model's morph names never change from frame to frame) */
export function faceWeights(face: Face): Record<FaceUnit, number> {
  const w = Object.fromEntries(FACE_UNITS.map((u) => [u, 0])) as Record<FaceUnit, number>;
  for (const [k, v] of Object.entries(face)) {
    if (v === undefined) continue;
    if (!Number.isFinite(v)) throw new RangeError(`Character: face.${k} is a weight, got ${v}`);
    const mix = (FACE_PRESETS as Record<string, Partial<Record<FaceUnit, number>>>)[k];
    if (mix) for (const [u, m] of Object.entries(mix)) w[u as FaceUnit] += m! * v;
    else if (k in w) w[k as FaceUnit] += v;
    else throw new RangeError(`Character: face.${k} is not a face unit or preset (${Object.keys(FACE_PRESETS).join(", ")}, or one of FACE_UNITS)`);
  }
  for (const u of FACE_UNITS) w[u] = Math.min(1, Math.max(0, w[u]));
  return w;
}
