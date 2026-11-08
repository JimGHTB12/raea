// Bevy v0.20.0 examples/shader/shader_material_2d_bindless.rs in raea's TSX (docs/control.feature FC0a, FC2a): a
// Camera2d and four default Rectangles scaled to 128 x 32 px at (+-128, +-128), each with the example's own Material2d,
// assets/shaders/custom_material_2d_bindless.wesl as written: its color uniform (red, yellow, green, blue) times
// Bevy's dark or light logo, AlphaMode2d::Mask(0.5). raea's Material isn't bindless, so the shader's @if(BINDLESS) set
// compiles out and its @else set binds, as Bevy's does on a GPU without bindless. On Bevy's default clear colour, in
// Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Transform } from "raea/transform";
import { Mesh2d, Rectangle } from "raea/sprite";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)

const DARK = staticFile("branding/bevy_logo_dark.png");
const LIGHT = staticFile("branding/bevy_logo_light.png");
const PARTS: [number, number, number[], string][] = [
  [-1, -1, [1, 0, 0, 1], DARK],
  [1, -1, [1, 1, 0, 1], LIGHT],
  [-1, 1, [0, 1, 0, 1], DARK],
  [1, 1, [0, 0, 1, 1], LIGHT],
];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        {PARTS.map(([x, y, color, logo], i) => (
          <Mesh2d key={i} transform={Transform.fromXyz(x * 128, y * 128, 0).withScale([128, 32, 1])}>
            <Rectangle />
            <Material fragment={staticFile("shaders/custom_material_2d_bindless.wesl")} uniforms={{ material_color: { base_color: color } }} textures={{ material_color_texture: logo }} alpha_mode={{ Mask: 0.5 }} />
          </Mesh2d>
        ))}
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
