// Bevy v0.20.0 examples/3d/spotlight.rs in raea's TSX (docs/control.feature FC0a): a 100 m white floor, 40 blue 0.5 m
// cubes placed by the example's ChaCha8Rng (seed 19878367467713, lib/rand.ts), a 4 x 4 grid of shadowed SpotLights at
// 40 000 lm 2 m up, each swaying and widening every update (light_sway: from_euler(XYZ, -PI/2 + sin(t * 2.01) * 0.5,
// sin(t * 3) * 0.5, 0), outer (sin(t * 1.2) + 1) * (PI/4 - 0.1), inner 0.8 of it) with its two emissive child spheres
// (red at the light, maroon 0.1 m along its -Z, which casts no shadow), GlobalAmbientLight at 20, the camera at
// (-4, 5, 10) looking at the origin, and the instructions in Bevy's default font, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { LinearRgba } from "raea/color";
import { Cuboid, Mesh3d, Plane3d, Sphere } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { GlobalAmbientLight, SpotLight } from "raea/light";
import { ChaCha8Rng } from "../../../../../bench/bevy-parity/lib/rand";

type V3 = [number, number, number];
const f = Math.fround;
const rng = ChaCha8Rng.seed_from_u64(19878367467713n);
const CUBES: V3[] = Array.from({ length: 40 }, () => [rng.random_range(-5, 5), rng.random_range(0, 3), rng.random_range(-5, 5)]);
const LIGHTS: V3[] = [];
for (let x = 0; x < 4; x++) for (let z = 0; z < 4; z++) LIGHTS.push([1 + (x - 2), 2, z - 2]);

const FONT = `@font-face { font-family: "Fira Mono"; src: url("${staticFile("fonts/FiraMono-Medium.ttf")}"); }`;
const INSTRUCTIONS = "Controls\n--------\nHorizontal Movement: WASD\nVertical Movement: Space and Shift\nRotate Camera: Left and Right Arrows";

function Shot() {
  // elapsed: f updates of 16_666_667 ns (none on the first), as f32; light_sway runs on every update, the first too
  const t = f(useCurrentFrame() * 16_666_667e-9);
  const rotation = Quat.fromEuler("XYZ", f(-Math.PI / 2 + f(f(Math.sin(f(f(t * 0.67) * 3))) * 0.5)), f(f(Math.sin(f(t * 3))) * 0.5), 0);
  const outer = f(f(f(Math.sin(f(t * 1.2))) + 1) * f(Math.PI / 4 - 0.1));
  const ahead = Quat.mulVec3(rotation, [0, 0, -0.1]);
  return (
    <AbsoluteFill>
      <style>{FONT}</style>
      <GlobalAmbientLight brightness={20} />
      <Mesh3d>
        <Plane3d half_size={[50, 50]} />
        <StandardMaterial base_color="#ffffff" />
      </Mesh3d>
      {CUBES.map((p, i) => (
        <Mesh3d key={`c${i}`} transform={Transform.fromTranslation(p)}>
          <Cuboid half_size={[0.25, 0.25, 0.25]} />
          <StandardMaterial base_color="rgb(124, 144, 255)" />
        </Mesh3d>
      ))}
      {LIGHTS.map((p, i) => (
        <SpotLight key={`l${i}`} intensity={40_000} shadow_maps_enabled outer_angle={outer} inner_angle={f(outer * 0.8)} transform={{ translation: p, rotation }} />
      ))}
      {LIGHTS.map((p, i) => (
        <Mesh3d key={`r${i}`} transform={{ translation: p, rotation }}>
          <Sphere radius={0.05} uv={[32, 18]} />
          <StandardMaterial base_color="#ff0000" emissive={new LinearRgba(1, 0, 0, 0)} />
        </Mesh3d>
      ))}
      {LIGHTS.map((p, i) => (
        <Mesh3d key={`m${i}`} transform={{ translation: [p[0] + ahead[0], p[1] + ahead[1], p[2] + ahead[2]], rotation }} castShadow={false}>
          <Sphere radius={0.1} uv={[32, 18]} />
          <StandardMaterial base_color="#800000" emissive={new LinearRgba(0.369, 0, 0, 0)} />
        </Mesh3d>
      ))}
      <Camera3d transform={Transform.fromXyz(-4, 5, 10).lookingAt([0, 0, 0])} />
      <div style={{ position: "absolute", top: 12, left: 12, fontFamily: "Fira Mono", fontSize: 20, lineHeight: 1.2, color: "#fff", whiteSpace: "pre" }}>{INSTRUCTIONS}</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={121} fps={60} width={1280} height={720} />);
