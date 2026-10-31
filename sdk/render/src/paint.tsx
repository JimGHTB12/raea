// Simulated volumes (re-exported by raea/3d): <Volume>, a fluid simulated on a grid in the engine (raea-render's
// volume.rs), stepped 30 times a second from the start, so any frame alone equals that frame in play; under
// <Look mode="paint" | "cartoon"> it's cut into the look's flat tones with torn edges, otherwise drawn as a soft lit
// volume. Smoke, clouds and contrails are recipes over it (examples/volumes), as are a boat's wake and bunting
// (examples/objects).

import { useCurrentFrame, useVideoConfig } from "@raea/app/clock";
import { toHex, type Vec3 } from "./3d";

function vec(owner: string, k: string, v: unknown): Vec3 {
  if (!Array.isArray(v) || v.length !== 3 || !v.every((n) => typeof n === "number" && Number.isFinite(n))) throw new TypeError(`${owner}: ${k} must be three finite numbers [x, y, z], got ${JSON.stringify(v)}`);
  return v as Vec3;
}

function pos(owner: string, k: string, v: unknown, min = 0): number | undefined {
  if (v === undefined) return undefined;
  if (!(typeof v === "number" && Number.isFinite(v) && v >= min)) throw new RangeError(`${owner}: ${k} must be a number ${min} or more, got ${JSON.stringify(v)}`);
  return v;
}

/** three colours [lit, mid, dark] */
function tones(owner: string, v: unknown, d: [string, string, string]): [string, string, string] {
  if (v === undefined) return d;
  if (!(Array.isArray(v) && v.length === 3)) throw new TypeError(`${owner}: colors is [lit, mid, dark], got ${JSON.stringify(v)}`);
  return v.map((c) => toHex(owner, "colors", c)) as [string, string, string];
}


export interface VolumeCommon {
  /** parts are matched by name across frames */
  name?: string;
  /** seconds on the simulation's clock (the Sequence's time, frame / fps, by default) */
  time?: number;
  /** cells along the box's longest side (64 by default; 32 is a quarter the cost a step, 96 finer) */
  resolution?: number;
  /** the same seed simulates the same volume (0) */
  seed?: number;
  /** its tones under a paint look: [lit, mid, dark] */
  colors?: [string, string, string];
  /** the density its painted surface sits at (0.12): lower is fatter, higher thinner */
  threshold?: number;
  /** how much noise tears the painted surface's edge (0.08), and the tears' size in metres (0.6) */
  erode?: number;
  erodeSize?: number;
  /** how much light a unit of it stops (its shadow on itself; 1.2) */
  absorb?: number;
  /** the wind, m/s ([0, 0, 0]); how fast the volume takes it up, a second (1.2 smoke, 0.6 otherwise); the height
   * it's full from, metres, weaker below (0: full everywhere) */
  wind?: Vec3;
  windPull?: number;
  windHeight?: number;
  /** velocity the still air takes away a second (0.4) */
  drag?: number;
  /** curl kept curling (6); a drifting turbulent force, m/s² (0), and its size in metres (2) */
  vorticity?: number;
  turbulence?: number;
  turbulenceSize?: number;
  /** samples along each ray (96) */
  steps?: number;
  /** its shadow, cast in the scene's light as a mesh's is (on by default): soft moving shadows on the ground, meshes
   * and models under it, and, with a `volumetric` sun and a <FogVolume> under <VolumetricFog>, the sun's shafts
   * breaking through the gaps between clouds. The shadow is where the volume has stopped half the sun; a number scales
   * the density it sees: 1 as drawn, more reaches into its thinner parts (a fatter shadow; up to 4, a thin contrail's
   * made to show), less keeps to the dense cores. false or 0 casts none. Under a paint look the shadow is the painted
   * shape. Soften its edge with the sun's `softShadow` (2-4 for clouds) */
  shadow?: boolean | number;
}

/** a volume's `shadow` as the row's number: 1 by default, 0 for none */
export function volumeShadow(owner: string, v: unknown): number {
  if (v === undefined || v === true) return 1;
  if (v === false) return 0;
  if (!(typeof v === "number" && Number.isFinite(v) && v >= 0 && v <= 4)) throw new RangeError(`${owner}: shadow must be true, false or a number from 0 to 4, got ${JSON.stringify(v)}`);
  return v;
}

function volumeNode(owner: string, p: VolumeCommon, position: Vec3, size: Vec3, sim: Record<string, unknown>) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const time = p.time ?? frame / fps;
  if (!Number.isFinite(time)) throw new RangeError(`${owner}: time must be a finite number of seconds, got ${time}`);
  if (!size.every((s) => s > 0)) throw new RangeError(`${owner}: size must be above 0 on every side, got ${JSON.stringify(size)}`);
  if (p.resolution !== undefined && !(Number.isInteger(p.resolution) && p.resolution >= 8 && p.resolution <= 160)) throw new RangeError(`${owner}: resolution must be a whole number from 8 to 160, got ${p.resolution}`);
  if (p.steps !== undefined && !(Number.isInteger(p.steps) && p.steps >= 8 && p.steps <= 256)) throw new RangeError(`${owner}: steps must be a whole number from 8 to 256, got ${p.steps}`);
  for (const k of ["threshold", "erode", "erodeSize", "absorb", "vorticity", "turbulence", "turbulenceSize", "windPull", "windHeight", "drag"] as const) pos(owner, k, p[k]);
  if (p.wind !== undefined) vec(owner, "wind", p.wind);
  const [lit, mid, dark] = tones(owner, p.colors, sim.colors as [string, string, string]);
  const spec: Record<string, unknown> = {
    ...sim, colors: undefined, lit, mid, dark,
    res: p.resolution ?? sim.res, seed: p.seed === undefined ? undefined : Math.round(Math.abs(p.seed)) + 1, threshold: p.threshold ?? sim.threshold, erode: p.erode ?? sim.erode, erodeSize: p.erodeSize ?? sim.erodeSize,
    absorb: p.absorb ?? sim.absorb, wind: p.wind ?? sim.wind, vorticity: p.vorticity ?? sim.vorticity, turbulence: p.turbulence ?? sim.turbulence, turbulenceSize: p.turbulenceSize ?? sim.turbulenceSize, steps: p.steps ?? sim.steps,
    windPull: p.windPull ?? sim.windPull, windHeight: p.windHeight ?? sim.windHeight, drag: p.drag ?? sim.drag,
  };
  for (const k of Object.keys(spec)) if (spec[k] === undefined) delete spec[k];
  const props = { ...(p.name ? { name: p.name } : {}), position, scale: size, time, sim: spec, shadow: volumeShadow(owner, p.shadow) };
  return <div data-3d="Fire" data-props={JSON.stringify(props)} />;
}

/** where density and heat come in: a burner, or an engine moving `at` -> `to` (a contrail) */
export interface VolumeSource {
  /** where (metres from the box's foot), and where it has moved to by `until` if it moves */
  at: Vec3;
  to?: Vec3;
  /** when it puts out, seconds ([0, Infinity]) */
  from?: number;
  until?: number;
  /** its radius (0.5 m); the density and heat it puts out a second at its centre (4 and 4); the velocity it pushes out
   * with, m/s ([0, 0, 0]) */
  radius?: number;
  rate?: number;
  heat?: number;
  speed?: Vec3;
}

export interface VolumeProps extends VolumeCommon {
  /** the box's foot (the centre of its floor), metres ([0, 0, 0]) */
  position?: Vec3;
  /** the box it lives in, [width, height, depth] in metres: what leaves it is cut */
  size: Vec3;
  /** where density and heat come in over time, and the puffs of density it starts with ([x, y, z, radius] in metres
   * from the foot) */
  sources?: VolumeSource[];
  puffs?: [number, number, number, number][];
  /** heat's lift (4) and the density's weight (0.4), m/s² a unit; the heat lost a second (0.8) and the density lost a
   * second (0.02), fractions */
  buoyancy?: number;
  weight?: number;
  cooling?: number;
  dissipation?: number;
  /** fire where it's hot: three colours [red, orange, yellow] (paint's by default), or false for none; and the heat
   * (a fraction of the hottest source's) each colour starts at ([0.35, 0.6, 0.85]) */
  fire?: [string, string, string] | false;
  fireAt?: [number, number, number];
}

/** A simulated volume: density and heat in a box, put in by its sources and puffs, lifted by heat, weighed down,
 * carried by the wind and its own curls, drawn lit and shadowed by itself. Smoke, a cloud or a contrail is a choice of
 * these (examples/volumes).
 * Simulated and raymarched by engine/render volume.rs: @bevy bevy_pbr::mesh_material::MeshMaterial3d<raea_render::engine::volume::VolumeMaterial>
 */
export function Volume(props: VolumeProps) {
  const size = vec("Volume", "size", props.size);
  for (const k of ["buoyancy", "weight", "cooling", "dissipation"] as const) pos("Volume", k, props[k]);
  const sources = props.sources?.map((s) => {
    for (const k of ["radius", "rate", "heat", "from", "until"] as const) pos("Volume", `a source's ${k}`, s[k]);
    return {
      at: vec("Volume", "a source's at", s.at), to: s.to === undefined ? undefined : vec("Volume", "a source's to", s.to), t0: s.from ?? 0,
      t1: s.until === undefined || s.until === Infinity ? 1e9 : s.until, radius: s.radius, rate: s.rate, heat: s.heat,
      speed: s.speed === undefined ? undefined : vec("Volume", "a source's speed", s.speed),
    };
  });
  for (const p of props.puffs ?? []) if (!(Array.isArray(p) && p.length === 4 && p.every(Number.isFinite) && p[3] > 0)) throw new TypeError(`Volume: each puff is [x, y, z, radius], got ${JSON.stringify(p)}`);
  const fire = props.fire === false ? [] : props.fire?.map((c) => toHex("Volume", "fire", c));
  return volumeNode("Volume", props, props.position === undefined ? [0, 0, 0] : vec("Volume", "position", props.position), size, {
    colors: ["#4a4a4c", "#2c2c2e", "#141415"], sources, puffs: props.puffs, buoyancy: props.buoyancy, weight: props.weight, cooling: props.cooling,
    dissipation: props.dissipation, fire, fireAt: props.fireAt,
  });
}
