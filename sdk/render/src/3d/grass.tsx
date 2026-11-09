// <Grass>: a field of stylized grass the engine makes itself (raea-sys-grass) from a few functions of the ground,
// sampled once into grids, instead of a placement per clump: clumps of tapered, leaning blades, root to tip in their
// patch's greens with a warm highlight on each tip, their normals the ground's. The field is cut into tiles, each as
// detailed as its distance from the camera asks, so a frame draws what it can show and a phone keeps up. Under a paint
// look (<Look mode="paint" wind>) the wind bends the blades, on a slope as on the flat.
import { type PlanBox } from "./all";

/** One step of detail: tiles nearer than `to` metres draw `keep` of their clumps (0 to 1), each with its first
 * `blades` blades in `segments` segments, `widen` times as wide. */
export interface GrassLod {
  /** tiles nearer than this many metres take this step */
  to: number;
  /** the share of their clumps they draw, 0 to 1 */
  keep: number;
  /** how many of each clump's blades they draw */
  blades: number;
  /** segments each blade is drawn in */
  segments: number;
  /** how many times as wide each blade is drawn, to fill in for the ones left out */
  widen: number;
}

export interface GrassProps {
  /** the entity's name, for picking, the inspector and world.* */
  name?: string;
  /** the ground the field covers: a box on the plan (x, z, metres) */
  area: PlanBox;
  /** the ground's height at (x, z): the clumps stand on it, tilted to its slope */
  height: (x: number, z: number) => number;
  /** clumps per square metre at (x, z) (0: bare) */
  density: number | ((x: number, z: number) => number);
  /** which of `tints` the point (x, z) belongs to (0 everywhere without it) */
  patch?: (x: number, z: number) => number;
  /** a multiple of the clumps' size at (x, z) (1 without it) */
  size?: (x: number, z: number) => number;
  /** each patch's root and tip colours: ["#2c4a1a", "#5f8a2c"] */
  tints?: [string, string][];
  /** the tips' highlight colour ("#f2e7a0") and how much of it each tip takes (0.22) */
  highlight?: string;
  /** how much of the highlight each tip takes (0.22) */
  highlightMix?: number;
  /** blades in a clump (14), their height in metres (0.3; each 0.55 to 1.3 times it), width range ([0.034, 0.05]) */
  blades?: number;
  /** their height in metres (0.3; each 0.55 to 1.3 times it) */
  bladeHeight?: number;
  /** a blade's width range, metres ([0.034, 0.05]) */
  width?: [number, number];
  /** how far from its clump's centre a blade stands (0.06 m), and how far it leans over its height ([0.2, 0.7]) */
  spread?: number;
  /** how far a blade leans over its height ([0.2, 0.7]) */
  lean?: [number, number];
  /** each clump's scale, a random pick between these ([0.85, 1.35]), times `size` */
  scale?: [number, number];
  /** which randomness places the clumps */
  seed?: number;
  /** grid points a metre the functions are sampled at (2) */
  resolution?: number;
  /** a tile's side in metres (4) */
  tile?: number;
  /** the steps of detail, nearest first (the engine's: every clump within 5 m, half of them in 8 blades to 14 m, a
   * fifth in 5 blades beyond) */
  lod?: GrassLod[];
  /** cast shadows (false) */
  shadow?: boolean;
}

const fn = (f: number | ((x: number, z: number) => number)) => (typeof f === "number" ? () => f : f);
const r4 = (v: number) => Math.round(v * 1e4) / 1e4;

/** The part's props as the page carries them, made once per field: React draws the part every frame, and a field's
 * grids are thousands of numbers. Kept by its `height` function and remade when anything else is new. */
const made = new WeakMap<object, { deps: unknown[]; json: string }>();

function spec(p: GrassProps): string {
  const deps = [p.density, p.patch, p.size, JSON.stringify({ ...p, height: 0, density: typeof p.density === "number" ? p.density : 0, patch: 0, size: 0 })];
  const had = made.get(p.height);
  if (had && had.deps.every((d, i) => d === deps[i])) return had.json;
  const { min, max } = p.area;
  if (!(max[0] > min[0] && max[1] > min[1])) throw new RangeError(`Grass: area's max is past its min on both axes, got ${JSON.stringify(p.area)}`);
  const res = Math.max(0.1, p.resolution ?? 2);
  const nx = Math.max(2, Math.ceil((max[0] - min[0]) * res) + 1), nz = Math.max(2, Math.ceil((max[1] - min[1]) * res) + 1);
  const height: number[] = [], density: number[] = [], patch: number[] = [], size: number[] = [];
  const d = fn(p.density), pa = p.patch, sz = p.size;
  for (let k = 0; k < nz; k++) {
    for (let i = 0; i < nx; i++) {
      const x = min[0] + ((max[0] - min[0]) * i) / (nx - 1), z = min[1] + ((max[1] - min[1]) * k) / (nz - 1);
      height.push(r4(p.height(x, z)));
      density.push(r4(Math.max(0, d(x, z))));
      if (pa) patch.push(Math.max(0, Math.round(pa(x, z))));
      if (sz) size.push(r4(sz(x, z)));
    }
  }
  if (![...height, ...density].every(Number.isFinite)) throw new RangeError("Grass: height and density are finite everywhere in the area");
  const s = {
    area: [min[0], min[1], max[0], max[1]], grid: [nx, nz], height, density,
    ...(pa && { patch }), ...(sz && { size }),
    tints: p.tints, highlight: p.highlight, highlightMix: p.highlightMix, blades: p.blades, bladeHeight: p.bladeHeight, width: p.width,
    spread: p.spread, lean: p.lean, scale: p.scale, seed: p.seed, tile: p.tile, lod: p.lod,
  };
  const json = JSON.stringify({ name: p.name, shadow: p.shadow ?? false, spec: s });
  made.set(p.height, { deps, json });
  return json;
}

/** A field of grass over `area`, made by the engine from the ground's functions (sampled once at `resolution`).
 */
export function Grass(props: GrassProps) {
  if (typeof props.height !== "function") throw new TypeError("Grass: height is a function of (x, z)");
  return <div data-3d="Grass" data-props={spec(props)} />;
}
