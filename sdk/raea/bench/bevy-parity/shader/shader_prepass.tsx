// Bevy v0.20.0 examples/shader/shader_prepass.rs in raea's TSX (docs/control.feature FC0a, FC2a): the camera at
// (-2, 3, 5) with Bevy's DepthPrepass, NormalPrepass and MotionVectorPrepass and Msaa::Off; a 5 m plane in srgb(0.3,
// 0.5, 0.3); three 1 m cubes at x -1, 0, 1: the example's own Material (assets/shaders/custom_material.wesl, white,
// Bevy's icon) opaque and turning about Z by (sin(t) * 0.5 + 0.5) * TAU, a StandardMaterial of the icon masked at 1.0,
// and the own Material blended (which the prepass leaves out); a shadow-casting PointLight at (4, 8, 4). In front of
// the camera, a 20 m quad with the example's PrepassOutputMaterial (assets/shaders/show_prepass.wesl as written,
// blended, no shadow) shows what the prepass wrote: Space (bench/bevy-parity/input/shader_prepass.json) steps it from
// transparent through depth, normals and motion vectors, as toggle_prepass_view does, and the label with it. In Bevy's
// default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame, useGameState, useInput } from "raea";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d, Plane3d } from "raea/mesh";
import { PointLight } from "raea/light";
import { StandardMaterial } from "raea/pbr";
import { BevyComponent, Canvas3D } from "raea/3d";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)

const FONT = `@font-face { font-family: "Fira Mono"; src: url("${staticFile("fonts/FiraMono-Medium.ttf")}"); }`;
const ICON = staticFile("branding/icon.png");
const LABELS = ["transparent", "depth", "normals", "motion vectors"];
// a Rectangle (XY, facing +Z) is a Plane3d (XZ, facing +Y) turned +PI/2 about X
const UPRIGHT = Quat.fromRotationX(Math.PI / 2);
const QUAD = Transform.fromXyz(-0.75, 1.25, 3).lookingAt([2, -2.5, -5]);

/** toggle_prepass_view: each Space press steps the view (its Local), and the material's settings with it */
function step(s: { view: number; space: boolean }, input: { keys: string[] }) {
  const space = input.keys.includes("Space");
  return { view: space && !s.space ? (s.view + 1) % 4 : s.view, space };
}

function Custom({ x, alpha_mode, turn }: { x: number; alpha_mode: "Opaque" | "Blend"; turn?: number }) {
  return (
    <Mesh3d transform={{ translation: [x, 0.5, 0], rotation: Quat.fromRotationZ(turn ?? 0) }}>
      <Cuboid half_size={[0.5, 0.5, 0.5]} />
      <Material fragment={staticFile("shaders/custom_material.wesl")} uniforms={{ material_color: [1, 1, 1, 1] }} textures={{ material_color_texture: ICON }} alpha_mode={alpha_mode} />
    </Mesh3d>
  );
}

function Shot() {
  // Bevy's picture of frame f is drawn after update f, which read frame f's keys: the fold (updates before f) and f's own
  const { view } = step(useGameState({ view: 0, space: false }, step), useInput());
  // rotate: Time::elapsed_secs, f updates of 1/60 s (none on the first), as an f32
  const t = Math.fround(useCurrentFrame() * 0.016666667);
  const turn = Math.fround((Math.sin(t) * 0.5 + 0.5) * Math.PI * 2);
  const settings = { show_depth: view === 1 ? 1 : 0, show_normals: view === 2 ? 1 : 0, show_motion_vectors: view === 3 ? 1 : 0, padding_1: 0, padding_2: 0 };
  return (
    <AbsoluteFill>
      <style>{FONT}</style>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(-2, 3, 5).lookingAt([0, 0, 0])}>
          <BevyComponent type="bevy_render::view::Msaa" value="Off" />
          <BevyComponent type="bevy_core_pipeline::prepass::DepthPrepass" value={{}} />
          <BevyComponent type="bevy_core_pipeline::prepass::NormalPrepass" value={{}} />
          <BevyComponent type="bevy_core_pipeline::prepass::MotionVectorPrepass" value={{}} />
        </Camera3d>
        <Mesh3d>
          <Plane3d half_size={[2.5, 2.5]} />
          <StandardMaterial base_color="rgb(76.5, 127.5, 76.5)" />
        </Mesh3d>
        <Mesh3d transform={{ translation: QUAD.translation, rotation: Quat.mul(QUAD.rotation, UPRIGHT) }}>
          <Plane3d half_size={[10, 10]} />
          <Material fragment={staticFile("shaders/show_prepass.wesl")} uniforms={{ settings }} alpha_mode="Blend" />
          <BevyComponent type="bevy_light::NotShadowCaster" value={{}} />
        </Mesh3d>
        <Custom x={-1} alpha_mode="Opaque" turn={turn} />
        <Mesh3d transform={Transform.fromXyz(0, 0.5, 0)}>
          <Cuboid half_size={[0.5, 0.5, 0.5]} />
          <StandardMaterial alpha_mode={{ Mask: 1 }} base_color_texture={ICON} />
        </Mesh3d>
        <Custom x={1} alpha_mode="Blend" />
        <PointLight shadow_maps_enabled transform={Transform.fromXyz(4, 8, 4)} />
      </Canvas3D>
      <div style={{ position: "absolute", top: 12, left: 12, fontFamily: "Fira Mono", fontSize: 20, lineHeight: 1.2, color: "#fff", whiteSpace: "pre" }}>{`Prepass Output: ${LABELS[view]}\n\n\nControls\n---------------\nSpace - Change output\n`}</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={71} fps={60} width={1280} height={720} />);
