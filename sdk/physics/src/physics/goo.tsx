// Mesh sequences and goo.
import { useCurrentFrame, useVideoConfig } from "@raea/app/clock";
import { hasNative, native } from "@raea/app/native";
import { toHex, type Vec3 } from "raea/3d";
import { node } from "./all";

// ---- mesh sequences and goo -----------------------------------------------------------------------------------

export interface MeshSequenceProps {
  /** the entity's name, for picking and the inspector */
  name?: string;
  /** a .rseq (tools/goo writes them; raea_render::meshseq) */
  src: string;
  /** which of its frames (clamped) */
  frame: number;
  /** where it stands, metres (x, y, z) */
  position?: Vec3;
  /** its rotation, Euler angles in radians (x, y, z) */
  rotation?: Vec3;
  /** a uniform scale */
  scale?: number;
  /** multiplies the vertex colours */
  color?: string;
  /** 0..1, smooth to matte */
  roughness?: number;
  /** 0..1, a dielectric to a metal */
  metalness?: number;
  /** 0..1 strength of a clear varnish layer over the surface */
  clearcoat?: number;
  /** 0..1 roughness of that clear layer */
  clearcoatRoughness?: number;
  /** 0..1 specular strength at normal incidence (0.5: 4%, a dielectric's) */
  reflectance?: number;
  /** 0..1 light let through (diffuse transmission): a jelly's glow */
  subsurface?: number;
  /** how thick the mesh is, metres, for the light let through */
  thickness?: number;
  /** 0..1 light let straight through and bent (a clear liquid: water, tea, juice), with its index of refraction (1.33
   * water) and, for a tinted one, the colour it takes after `attenuationDistance` metres through it. Screen space: it
   * refracts what's on screen behind it, and one clear layer doesn't show another through it */
  transmission?: number;
  /** the index of refraction of what light passes through (1.33 water) */
  ior?: number;
  /** the colour light takes after `attenuationDistance` metres through it */
  attenuationColor?: string;
  /** metres through it until light has taken `attenuationColor` */
  attenuationDistance?: number;
  /** 0..1, see-through to solid */
  opacity?: number;
  /** a picture over its texture coordinates */
  map?: string;
  /** draw the back faces too (an open shell, a petal) */
  doubleSided?: boolean;
}

/** A baked simulation, one mesh a frame. */
export function MeshSequence(p: MeshSequenceProps) {
  if (typeof p.src !== "string" || !p.src.endsWith(".rseq")) throw new TypeError(`MeshSequence: src must be a .rseq file, got ${JSON.stringify(p.src)}`);
  if (!Number.isFinite(p.frame)) throw new RangeError(`MeshSequence: frame must be a number, got ${p.frame}`);
  const hexOf = (k: "color" | "attenuationColor") => (p[k] !== undefined ? toHex("MeshSequence", k, p[k] as string) : undefined);
  return node("MeshSequence", { ...p, frame: Math.max(0, Math.round(p.frame)), color: hexOf("color"), attenuationColor: hexOf("attenuationColor") });
}

export interface GooObject {
  /** the object's name; the bake's stats list it, with the frame its original model should go */
  name: string;
  /** a glTF's nodes (world space as they stand in it), moved by `transform` (column-major 4x4) if given; or, with no
   * src, `cup` and `level`: a liquid filling one of the goo's cups to `level` metres above its foot */
  src?: string;
  /** the glTF nodes it's made of (all of them when left out) */
  nodes?: string[];
  /** with no src: the index of the cup in `cups` it fills */
  cup?: number;
  /** with no src: metres above the cup's foot it's filled to */
  level?: number;
  /** a column-major 4x4 that moves the glTF's nodes */
  transform?: number[];
  /** sph: what it is ("water" by default: coffee, tea, milk, juice, oil, syrup, honey set its colour, density and
   * viscosity), and where it comes from if not a cup or a glTF (filled): a box of it, or an emitter pouring a round
   * stream from `at` along `dir` (down) at `speed` m/s, `radius` m (6 mm) across, from `start` to `end` seconds */
  liquid?: "water" | "coffee" | "tea" | "milk" | "juice" | "oil" | "syrup" | "honey";
  /** sph: a box of liquid, world min and max corners */
  block?: { min: Vec3; max: Vec3 };
  /** sph: a round stream poured from `at` along `dir`, `radius` m across, at `speed` m/s, from `start` to `end` seconds */
  emitter?: { at: Vec3; dir?: Vec3; radius?: number; speed?: number; start?: number; end?: number };
  /** seconds from the goo's start when it begins to soften, and how long until it runs (mpm) */
  start: number;
  /** seconds from `start` until it runs (2) */
  melt?: number;
  /** of the solid (3e4) and the fluid it becomes (4) */
  stiffness?: number;
  /** how thick the fluid it becomes is (4) */
  viscosity?: number;
  /** its density, kg/m³ (1000) */
  density?: number;
  /** the share of its pressure it keeps under tension (0.6): goo holds together in strands, a liquid (0.02) barely */
  cohesion?: number;
  /** the share of its shear stiffness a fluid keeps (0.12) and the strain it must pass to flow (0.005): 0 and 0 for a
   * liquid */
  residual?: number;
  /** the strain a fluid must pass to flow (0.005; 0 for a liquid) */
  yield?: number;
  /** "solid" fills its volume; "shell" a skin `thickness` thick (plates, petals) */
  fill?: "solid" | "shell";
  /** with `fill: "shell"`: the skin's thickness, metres */
  thickness?: number;
  /** its goo's colour (else its texture's) */
  color?: string;
}

export interface GooCup {
  radius: number;
  height: number;
  base?: number;
  wall?: number;
  friction?: number;
  poses: number[][];
}

export interface GooProps {
  /** the entity's name, for picking and the inspector */
  name?: string;
  /** how long, seconds, from `from` (a frame of the enclosing Sequence) */
  seconds: number;
  /** the frame of the enclosing Sequence the bake starts at (0) */
  from?: number;
  /** what goes in: solids that melt and liquids that pour */
  objects: GooObject[];
  /** static boxes the goo lies on and runs off (world) */
  colliders?: { box: { min: Vec3; max: Vec3 }; friction?: number }[];
  /** hollow cylinders on a pose track (a mug lifted and tipped) that carry the goo and pour it (tools/goo's cups):
   * radius inside, height foot to rim, base (the inside floor's height), wall, friction, and poses [t, ...the cup's
   * frame to world, 16 numbers column-major], t in seconds from the goo's start, its axis +y */
  cups?: GooCup[];
  /** the surface drawn round the particles: blur (grid nodes, 1), iso level (of the rest density, 0.33) and flat (the
   * blur across over the blur up, 1); a wider, flatter blur and a lower level close a thin spill into one sheet.
   * maxTris (120000): past it the grid coarsens, which drops a film one particle thick; a wide one needs more */
  surface?: { blur?: number; level?: number; flat?: number; maxTris?: number };
  /** "mpm" (the default: goo, melting things) or "sph": thin liquids that pour, splash and spread (coffee dumped on a
   * counter, water from a tap), simulated by SPlisHSPlasH (tools/goo/sph.py); each object a liquid, from a cup, a
   * block, a filled glTF or an emitter */
  solver?: "mpm" | "sph";
  /** the sph solver's settings: particle radius (m, 0.001), viscosity (0.002), surface tension (0: off; at millimetre
   * particles the solver alone holds a spill 4-5 mm deep, as coffee is, and even 30 beads it into a ball) with how much
   * the liquid wets what it lies on (0.5) and its damping (0.05), the largest time step (s, 0.001) */
  sph?: { radius?: number; viscosity?: number; tension?: number; tensionBoundary?: number; tensionViscosity?: number; step?: number;
    /** threads (1: a bake the same byte for byte every time; more is faster and differs run to run), the speed (m/s,
     * 25) past which the bake stops as blown up, and the boundary handling ("volume" maps or sampled "particles") */
    threads?: number; maxSpeed?: number; boundary?: "volume" | "particles" };
  /** sph: glTF meshes the liquid runs over and around (all their meshes, or the named nodes, moved by `transform`),
   * standing, or moved along `poses` like a cup's ([t, 16 numbers column-major], from where the glTF has them) */
  obstacles?: { src: string; nodes?: string[]; transform?: number[]; poses?: number[][] }[];
  /** the bake's own frame rate (the composition's by default): one bake shared by compositions at different rates */
  fps?: number;
  /** the box it lives in: goo leaving it is gone */
  bounds?: { min: Vec3; max: Vec3 };
  /** grid cell, metres (0.005): finer is slower and smoother */
  dx?: number;
  /** the random seed the particles are placed with: the same seed bakes the same goo (0) */
  seed?: number;
  /** gravity, m/s² (0, -9.81, 0) */
  gravity?: Vec3;
  /** the surface: glossy, a little light through it */
  roughness?: number;
  /** 0..1 clear varnish over the surface */
  clearcoat?: number;
  /** 0..1 light let through the surface */
  subsurface?: number;
  /** the goo's colour where an object gives none */
  color?: string;
}

export interface GooBake {
  /** the baked .rseq this goo plays */
  file: string;
  /** what the bake measured, with each object's name and the frame its original model should go */
  stats: { objects?: { name: string; hideFrame: number }[]; [k: string]: unknown };
}

const goos = new Map<string, GooBake>();

/** The spec tools/goo bakes for these props at the composition's `fps`: what `physics goo SPEC.json` takes, and the bake's
 *  key, so a script can bake (or check) a goo without rendering. */
export function gooSpec(p: GooProps, fps: number): Record<string, unknown> {
  return {
    fps: p.fps ?? fps, frames: Math.max(1, Math.round(p.seconds * (p.fps ?? fps))), seed: p.seed ?? 0, dx: p.dx ?? 0.005, gravity: p.gravity ?? [0, -9.81, 0],
    bounds: p.bounds ?? { min: [-0.75, 0.3, -0.95], max: [0.75, 1.25, 0.95] }, colliders: p.colliders ?? [],
    ...(p.cups ? { cups: p.cups } : {}),
    ...(p.surface ? { surface: p.surface } : {}),
    ...(p.solver ? { solver: p.solver } : {}),
    ...(p.sph ? { sph: p.sph } : {}),
    ...(p.obstacles ? { obstacles: p.obstacles } : {}),
    // mpm's defaults; an sph liquid takes its preset's
    objects: p.solver === "sph" ? p.objects : p.objects.map((o) => ({ fill: "solid", thickness: 0.006, stiffness: 3e4, viscosity: 4, density: 1000, color: null, transform: [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1], ...o })),
  };
}

/** The goo's bake for these props (made once and kept), and the frame (of the enclosing Sequence) at which each
 *  object's own part should go, the goo having taken it over. */
export function useGoo(p: GooProps): GooBake & { gone: (name: string) => boolean; frame: number } {
  const { fps } = useVideoConfig();
  const f = useCurrentFrame();
  const spec = gooSpec(p, fps);
  const key = JSON.stringify(spec);
  let g = goos.get(key);
  if (!g) {
    if (!hasNative()) throw new Error("Goo: baking needs the raea engine (raea render)");
    g = native("physics.goo", { spec } as never) as GooBake;
    goos.set(key, g);
  }
  // the bake's frame now: the composition's frames since `from`, at the bake's rate
  const k = p.fps ? Math.round(((f - (p.from ?? 0)) / fps) * p.fps) : f - (p.from ?? 0);
  const hide = new Map((g.stats.objects ?? []).map((o) => [o.name, o.hideFrame]));
  return { ...g, frame: k, gone: (name: string) => k >= (hide.get(name) ?? Math.round((p.objects.find((o) => o.name === name)?.start ?? Infinity) * (p.fps ?? fps)) + 6) };
}

/** Objects melting into goo: the bake drawn by the frame (nothing before `from`). */
export function Goo(p: GooProps) {
  const g = useGoo(p);
  if (g.frame < 0) return null;
  return (
    <MeshSequence name={p.name ?? "goo"} src={g.file} frame={g.frame} color={p.color ?? "#ffffff"} roughness={p.roughness ?? 0.12} clearcoat={p.clearcoat ?? 1} clearcoatRoughness={0.05}
      subsurface={p.subsurface ?? 0.25} thickness={0.03} />
  );
}
