// RR0's film, its set and its six shots (docs/rerender.feature): film.tsx registers them as compositions, and the
// iPhone bench (ios/bench.sh) bundles one composition at a time from these. See film.tsx for what the film is.
import { AbsoluteFill, useCurrentFrame } from "raea";
import {
  AmbientOcclusion, AntiAliasing, Atmosphere, Bloom, Canvas3D, DepthOfField, Fog, FogVolume, Grade, Grass, Instances,
  ToneMapping, Shadows, VolumetricFog, grow, manifold, scatter,
} from "raea/3d";
import { Camera3d } from "raea/camera";
import { AmbientLight, DirectionalLight, PointLight, SpotLight } from "raea/light";
import { Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

export const FPS = 30;
export const SHOT = 150;
type V3 = [number, number, number];

// ---------------------------------------------------------------- the set, built once

const OAK = { height: 7, crown: "dome" as const, points: 1500, leaves: 4, leaf: 0.14, seed: 3 };
const PINE = { height: 10, crown: "cone" as const, points: 1100, leaves: 5, leaf: 0.1, width: 0.5, seed: 4 };
const OAK_WOOD = grow.tree(OAK, "wood");
const OAK_LEAVES = grow.tree(OAK, "leaves");
const PINE_WOOD = grow.tree(PINE, "wood");
const PINE_LEAVES = grow.tree(PINE, "leaves");
const ROCK = grow.boulder({ size: 1.4, rough: 0.5, seed: 5 });
// the forest and meadow, west of the street; the room's walls and the street's blocks are kept clear
const WOODS = { min: [-110, -70] as [number, number], max: [-8, 30] as [number, number] };
const GLADE = [{ min: [-40, -12] as [number, number], max: [-22, 6] as [number, number] }];
const OAKS = scatter({ area: WOODS, count: 110, spacing: 5, seed: 11, scale: [0.7, 1.3], avoid: GLADE });
const PINES = scatter({ area: WOODS, count: 90, spacing: 5, seed: 13, scale: [0.8, 1.4], avoid: GLADE, clearOf: OAKS });
// the meadow under the trees: grass the engine grows, about 40k clumps
const MEADOW = { min: [-75, -40] as [number, number], max: [-5, 45] as [number, number] };
const flat = () => 0;
const ROCKS = scatter({ area: { min: [-90, -60], max: [-10, 30] }, count: 40, spacing: 3, seed: 12, scale: [0.4, 1.6] });
// the room: a 10 x 4 x 8 box with its inside carved out and a window in the east wall
const ROOM = manifold.box(10.4, 4.4, 8.4).minus(manifold.box(10, 4, 8)).minus(manifold.box(1, 1.6, 3).move(5, 0.4, 0));
const TABLE = manifold.box(2.4, 0.08, 1.2).move(0, 0.76, 0)
  .union(manifold.box(0.08, 0.76, 0.08).move(-1.1, 0.38, -0.5)).union(manifold.box(0.08, 0.76, 0.08).move(1.1, 0.38, -0.5))
  .union(manifold.box(0.08, 0.76, 0.08).move(-1.1, 0.38, 0.5)).union(manifold.box(0.08, 0.76, 0.08).move(1.1, 0.38, 0.5));
const CUP = manifold.cylinder(0.05, 0.11).minus(manifold.cylinder(0.042, 0.11).move(0, 0, 0.01)).rotate(-90, 0, 0);
// the street: two rows of blocks with windows, lamps between them
const BLOCK = [0, 1, 2, 3].reduce((s, i) => s.minus(manifold.box(1.2, 1.6, 0.4).move(-3 + i * 2, 2, 3)), manifold.box(8, 9, 6).move(0, 4.5, 0));
const BLOCKS: { at: V3; h: number }[] = Array.from({ length: 36 }, (_, i) => ({ at: [40 + (i % 12) * 9, 0, [-8, 8, -22][Math.floor(i / 12)]], h: 0.8 + ((i * 37) % 5) / 6 }));
// the room's shelves: 480 books along the back wall
const BOOK = manifold.box(0.04, 0.24, 0.17);
const BOOKS = Array.from({ length: 480 }, (_, i) => {
  const row = Math.floor(i / 120), col = i % 120;
  return { position: [-4.6 + col * 0.075, 0.6 + row * 0.42, 63.8] as V3, rotation: [0, 0, ((i * 13) % 7 - 3) * 0.02] as V3, scale: [1, 0.8 + ((i * 7) % 5) * 0.08, 1] as V3 };
});
const SHELF = manifold.box(9.2, 0.03, 0.3);
const GROUND = manifold.box(400, 1, 400).move(0, -0.5, 0);
const BULB = manifold.sphere(0.18);
const LAMP_GLOW = "#ffd890";
const LAMPS: V3[] = [0, 1, 2, 3, 4].map((i) => [42 + i * 10, 4, i % 2 ? 4 : -4]);

function Set() {
  return (
    <>
      <Mesh3d name="ground" shape={GROUND}><StandardMaterial base_color="#5a5e48" perceptual_roughness={0.95} /></Mesh3d>
      <Instances at={OAKS} shape={OAK_WOOD}><StandardMaterial base_color="#4a3826" perceptual_roughness={0.9} /></Instances>
      <Instances at={OAKS} shape={OAK_LEAVES}><StandardMaterial base_color="#3f6b2c" perceptual_roughness={0.8} /></Instances>
      <Instances at={PINES} shape={PINE_WOOD}><StandardMaterial base_color="#3d2c1e" perceptual_roughness={0.9} /></Instances>
      <Instances at={PINES} shape={PINE_LEAVES}><StandardMaterial base_color="#28472a" perceptual_roughness={0.85} /></Instances>
      <Grass name="meadow" area={MEADOW} height={flat} density={7} />
      <Instances at={ROCKS} shape={ROCK}><StandardMaterial base_color="#77736a" perceptual_roughness={0.85} /></Instances>
      <Mesh3d name="room" shape={ROOM} transform={Transform.fromXyz(0, 2, 60)}><StandardMaterial base_color="#d8cdb8" perceptual_roughness={0.9} /></Mesh3d>
      <Mesh3d name="table" shape={TABLE} transform={Transform.fromXyz(0, 0, 60)}><StandardMaterial base_color="#6b4a2f" perceptual_roughness={0.6} /></Mesh3d>
      {[-0.6, 0, 0.5].map((x, i) => (
        <Mesh3d key={i} name={`cup-${i}`} shape={CUP} transform={Transform.fromXyz(x, 0.825, 60 + (i - 1) * 0.2)}><StandardMaterial base_color={["#e9e4da", "#2f4f6f", "#b34a2f"][i]} perceptual_roughness={0.3} /></Mesh3d>
      ))}
      {BLOCKS.map((b, i) => (
        <Mesh3d key={i} name={`block-${i}`} shape={BLOCK} transform={Transform.fromXyz(...b.at).withScale([1, b.h, 1])}><StandardMaterial base_color={i % 3 ? "#8b8580" : "#6d6660"} perceptual_roughness={0.8} /></Mesh3d>
      ))}
      <Instances at={BOOKS} shape={BOOK}><StandardMaterial base_color="#7a3b2e" perceptual_roughness={0.7} /></Instances>
      {[0, 1, 2, 3].map((r) => <Mesh3d key={r} name={`shelf-${r}`} shape={SHELF} transform={Transform.fromXyz(0, 0.465 + r * 0.42, 63.8)}><StandardMaterial base_color="#5b412b" perceptual_roughness={0.6} /></Mesh3d>)}
      {LAMPS.map((p, i) => (
        <Mesh3d key={i} name={`lamp-${i}`} shape={BULB} transform={Transform.fromXyz(...p)}><StandardMaterial base_color="#fff1c8" emissive={LAMP_GLOW} /></Mesh3d>
      ))}
    </>
  );
}

// ---------------------------------------------------------------- the six shots

interface Look {
  sky: boolean;
  /** the sky's colour behind the set where no Atmosphere draws one */
  bg: string;
  sun: { from: V3; lux: number; color: string };
  ambient: number;
  tone: "agx" | "aces" | "tony" | "neutral";
  exposure: number;
  fog?: { color: string; near: number; far: number };
  volume?: { at: V3; size: V3; density: number };
  bloom?: number;
  ao?: boolean;
  dof?: number;
  grade: { temperature: number; tint?: number; saturation: number; contrast: number; vignette: number };
  lamps: number;
}
interface Shot {
  name: string;
  title: string;
  look: Look;
  /** the camera at t seconds into the shot */
  cam(t: number): { at: V3; look: V3; fov: number; focus?: number };
}

// shot 4's grade and light: the line change.sh rewrites, as a note would
export const NOIR = { saturation: 0, contrast: 1.35, temperature: -0.02, sun: 25000, ambient: 400 };

const lerp = (a: V3, b: V3, u: number): V3 => [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u, a[2] + (b[2] - a[2]) * u];

export const SHOTS: Shot[] = [
  {
    name: "dawn", title: "Dawn",
    look: { sky: true, bg: "#000000", sun: { from: [-60, 8, -40], lux: 30000, color: "#ffd2a0" }, ambient: 200, tone: "agx", exposure: 0, fog: { color: "#e8c9a8", near: 30, far: 160 }, grade: { temperature: 0.05, saturation: 1.05, contrast: 1, vignette: 0.2 }, lamps: 0 },
    cam: (t) => ({ at: lerp([-20, 2, 25], [-28, 2.2, 12], t / 5), look: [-45, 3, -10], fov: 40 }),
  },
  {
    name: "night", title: "Night",
    look: { sky: false, bg: "#06080e", sun: { from: [30, 40, 20], lux: 40, color: "#9fb4ff" }, ambient: 10, tone: "aces", exposure: 1.5, volume: { at: [62, 4, 0], size: [60, 10, 30], density: 0.12 }, bloom: 0.6, grade: { temperature: -0.08, tint: 0.06, saturation: 0.9, contrast: 1.1, vignette: 0.4 }, lamps: 400000 },
    cam: (t) => ({ at: [36, 2, lerp([-3, 0, 0], [3, 0, 0], t / 5)[0]], look: [70, 3, 0], fov: 45 }),
  },
  {
    name: "interior", title: "Interior",
    look: { sky: false, bg: "#1a1d22", sun: { from: [20, 10, 60], lux: 15000, color: "#fff0d8" }, ambient: 120, tone: "agx", exposure: 1, ao: true, dof: 1.8, grade: { temperature: 0.03, saturation: 1, contrast: 1.05, vignette: 0.3 }, lamps: 0 },
    cam: (t) => { const at = lerp([-3.5, 1.6, 57], [-1.6, 1.25, 58.8], t / 5); return { at, look: [0, 0.85, 60], fov: 35, focus: Math.hypot(at[0], at[1] - 0.85, at[2] - 60) }; },
  },
  {
    name: "noir", title: "Noir",
    look: { sky: false, bg: "#5a5a5a", sun: { from: [80, 30, -40], lux: NOIR.sun, color: "#ffffff" }, ambient: NOIR.ambient, tone: "tony", exposure: 1.5, grade: { temperature: NOIR.temperature, saturation: NOIR.saturation, contrast: NOIR.contrast, vignette: 0.55 }, lamps: 0 },
    cam: (t) => ({ at: lerp([38, 1.7, 0], [60, 1.7, 0], t / 5), look: lerp([48, 4, -6], [70, 4, -6], t / 5), fov: 50 }),
  },
  {
    name: "overcast", title: "Overcast",
    look: { sky: false, bg: "#c9ced4", sun: { from: [0, 60, 10], lux: 8000, color: "#e6ecf2" }, ambient: 600, tone: "neutral", exposure: 0.5, fog: { color: "#c9ced4", near: 10, far: 90 }, grade: { temperature: -0.02, saturation: 0.8, contrast: 0.95, vignette: 0.1 }, lamps: 0 },
    cam: (t) => ({ at: lerp([-30, 1.5, 30], [-30, 9, 30], t / 5), look: [-50, 4, -5], fov: 55 }),
  },
  {
    name: "sunset", title: "Sunset",
    look: { sky: true, bg: "#000000", sun: { from: [-100, 3, 20], lux: 40000, color: "#ff9a5a" }, ambient: 80, tone: "agx", exposure: 0.5, bloom: 0.35, dof: 2.8, grade: { temperature: 0.1, saturation: 1.15, contrast: 1.05, vignette: 0.5 }, lamps: 0 },
    cam: (t) => { const a = 0.6 + (t / 5) * 0.5; return { at: [-40 + Math.cos(a) * 30, 4, -5 + Math.sin(a) * 30], look: [-40, 3, -5], fov: 28, focus: 30 }; },
  },
];

/** One shot's frame: the set, the shot's look, its camera at t, and its title over the picture. */
export function Frame({ k, t }: { k: number; t: number }) {
  const s = SHOTS[k];
  const L = s.look;
  const c = s.cam(t);
  return (
    <AbsoluteFill style={{ background: L.bg }}>
      <Canvas3D>
        <Camera3d key={k === 5 ? "B" : "A"} transform={Transform.fromXyz(...c.at).lookingAt(c.look, [0, 1, 0])} projection={{ Perspective: { fov: (c.fov * Math.PI) / 180, near: 0.1, far: 400 } }}>
          {L.dof ? <DepthOfField focal_distance={c.focus ?? 10} aperture_f_stops={L.dof} /> : null}
        </Camera3d>
        {L.sky ? <Atmosphere light /> : null}
        <DirectionalLight name="sun" transform={Transform.fromXyz(...L.sun.from).lookingAt([0, 0, 0], [0, 1, 0])} illuminance={L.sun.lux} color={L.sun.color} shadow_maps_enabled soft_shadow_size={1.5} />
        {L.ao ? <SpotLight name="pendant" transform={Transform.fromXyz(0, 3.6, 60).lookingAt([0, 0, 60], [1, 0, 0])} intensity={60000} color="#ffd9a8" range={12} outer_angle={0.9} inner_angle={0.4} radius={0.08} shadow_maps_enabled soft_shadows_enabled /> : null}
        <AmbientLight brightness={L.ambient} />
        {LAMPS.map((p, i) => <PointLight key={i} name={`lamp-light-${i}`} transform={Transform.fromXyz(...p)} intensity={L.lamps} color={LAMP_GLOW} range={18} volumetric={L.lamps > 0} shadow_maps_enabled={L.lamps > 0} visibility={L.lamps > 0 ? "Visible" : "Hidden"} />)}
        <ToneMapping mode={L.tone} exposure={L.exposure} />
        {L.fog ? <Fog color={L.fog.color} near={L.fog.near} far={L.fog.far} /> : null}
        {L.volume ? <><VolumetricFog steps={96} /><FogVolume name="haze" position={L.volume.at} scale={L.volume.size} density={L.volume.density} /></> : null}
        {L.bloom ? <Bloom intensity={L.bloom} threshold={0.9} /> : null}
        {L.ao ? <AmbientOcclusion quality="ultra" /> : null}
        <AntiAliasing mode={L.ao ? "smaa" : "msaa"} />
        <Shadows mapSize={8192} distance={L.ao ? 20 : 120} cascades={4} />
        <Grade temperature={L.grade.temperature} tint={L.grade.tint ?? 0} saturation={L.grade.saturation} midtones={{ contrast: L.grade.contrast }} vignette={L.grade.vignette} />
        <Set />
      </Canvas3D>
      <div style={{ position: "absolute", left: 120, bottom: 150, fontFamily: "Helvetica, Arial, sans-serif", color: "#fff", textShadow: "0 2px 12px rgba(0,0,0,0.7)" }}>
        <div style={{ fontSize: 22, letterSpacing: "0.3em", opacity: 0.8 }}>SHOT {k + 1}</div>
        <div style={{ fontSize: 64, fontWeight: 700, opacity: Math.min(1, t * 2) }}>{s.title}</div>
      </div>
    </AbsoluteFill>
  );
}

/** The whole film: shot k for the frame's 5 s, each from its own start. */
export function Film() {
  const f = useCurrentFrame();
  return <Frame k={Math.min(5, Math.floor(f / SHOT))} t={(f % SHOT) / FPS} />;
}
/** Shot k alone, from its start. */
export const one = (k: number) => function OneShot() { return <Frame k={k} t={useCurrentFrame() / FPS} />; };
