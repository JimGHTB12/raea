// Bevy v0.20.0 examples/transforms/align.rs in raea's TSX (docs/control.feature FC0a): the ship
// (models/ship/craft_speederD.gltf's scene) at identity (nothing presses T, so it never turns), its axes as gizmo
// arrows 1.5 m long (red along -Z, srgb(0.65, 0, 0) along +X), the two random target directions the example's
// ChaCha8Rng draws (seed 19878367467712, bench/bevy-parity/lib/rand.ts's random_dir3) as white and gray arrows from
// the origin, a 100 m plane of srgb(0.3, 0.5, 0.3) at y = -2, a shadowed default PointLight at (4, 7, -4), the camera at
// (3, 2.5, 4) looking at the origin, and the instructions in Bevy's default font, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D, Entity, Model } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { PointLight } from "raea/light";
import { Gizmo } from "raea/gizmos";
import { Srgba } from "raea/color";
import { ChaCha8Rng } from "../../../../../bench/bevy-parity/lib/rand";

type V3 = [number, number, number];
const f = Math.fround;
const add = (a: V3, b: V3): V3 => [f(a[0] + b[0]), f(a[1] + b[1]), f(a[2] + b[2])];
const scale = (a: V3, k: number): V3 => [f(a[0] * k), f(a[1] * k), f(a[2] * k)];
const norm = (a: V3): V3 => scale(a, 1 / Math.hypot(...a));

/** glam's Quat::from_rotation_arc(X, to): the shortest turn taking +X onto `to` (both unit) */
function arcFromX(to: V3): [number, number, number, number] {
  const dot = to[0];
  if (dot < -1 + 2e-7) return [0, 1, 0, 0];
  const q: [number, number, number, number] = [0, -to[2], to[1], 1 + dot];
  const n = Math.hypot(...q);
  return q.map((x) => x / n) as [number, number, number, number];
}

/** rotate v by the unit quaternion q = [x, y, z, w] */
function rot(q: [number, number, number, number], v: V3): V3 {
  const [x, y, z, w] = q;
  const t: V3 = [2 * (y * v[2] - z * v[1]), 2 * (z * v[0] - x * v[2]), 2 * (x * v[1] - y * v[0])];
  return [v[0] + w * t[0] + (y * t[2] - z * t[1]), v[1] + w * t[1] + (z * t[0] - x * t[2]), v[2] + w * t[2] + (x * t[1] - y * t[0])];
}

/** bevy_gizmos' arrow(start, end): the line and four tip lines of length / 10 */
function arrow(start: V3, end: V3): [V3, V3][] {
  const tip = Math.hypot(end[0] - start[0], end[1] - start[1], end[2] - start[2]) / 10;
  const arc = arcFromX(norm(add(end, scale(start, -1))));
  const tips = ([[-1, 1, 0], [-1, 0, 1], [-1, -1, 0], [-1, 0, -1]] as V3[]).map((v) => [end, add(end, rot(arc, scale(norm(v), tip)))] as [V3, V3]);
  return [[start, end], ...tips];
}

const rng = ChaCha8Rng.seed_from_u64(19878367467712n);
const FIRST = rng.random_dir3();
const SECOND = rng.random_dir3();
const O: V3 = [0, 0, 0];

const FONT = `@font-face { font-family: "Fira Mono"; src: url("${staticFile("fonts/FiraMono-Medium.ttf")}"); }`;
const INSTRUCTIONS = [
  "The bright red axis is the primary alignment axis, and it will always be",
  "made to coincide with the primary target direction (white) exactly.",
  "The fainter red axis is the secondary alignment axis, and it is made to",
  "line up with the secondary target direction (gray) as closely as possible.",
  "Press 'R' to generate random target directions.",
  "Press 'T' to align the ship to those directions.",
  "Click and drag the mouse to rotate the camera.",
  "Press 'H' to hide/show these instructions.",
].join("\n");

function Shot() {
  return (
    <AbsoluteFill>
      <style>{FONT}</style>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Camera3d transform={Transform.fromXyz(3, 2.5, 4).lookingAt([0, 0, 0])} />
        <Mesh3d transform={Transform.fromXyz(0, -2, 0)}>
          <Plane3d half_size={[50, 50]} />
          <StandardMaterial base_color="rgb(76.5, 127.5, 76.5)" />
        </Mesh3d>
        <PointLight shadow_maps_enabled transform={Transform.fromXyz(4, 7, -4)} />
        <Entity transform={Transform.IDENTITY}>
          <Model src={staticFile("models/ship/craft_speederD.gltf")} />
        </Entity>
        <Gizmo lines={arrow(O, [0, 0, -1.5])} color="#ff0000" />
        <Gizmo lines={arrow(O, [1.5, 0, 0])} color={new Srgba(0.65, 0, 0, 1)} />
        <Gizmo lines={arrow(O, scale(FIRST, 1.5))} color="#ffffff" />
        <Gizmo lines={arrow(O, scale(SECOND, 1.5))} color={new Srgba(0.5, 0.5, 0.5, 1)} />
      </Canvas3D>
      <div style={{ position: "absolute", top: 12, left: 12, fontFamily: "Fira Mono", fontSize: 20, lineHeight: 1.2, color: "#fff", whiteSpace: "pre" }}>{INSTRUCTIONS}</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={31} fps={60} width={1280} height={720} />);
