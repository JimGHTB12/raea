// Bevy v0.20.0 examples/3d/animated_material.rs in raea's TSX (docs/control.feature FC0a): nine 0.5 m cubes in a 3 x 3
// grid, each StandardMaterial an Hsla stepped by the golden angle from red, every hue turning 100 degrees a second, lit
// by Pisa's prefiltered EnvironmentMapLight at 2000 on the camera at (3, 1, 3), in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { EnvironmentMapLight } from "raea/light";

const GOLDEN_ANGLE = Math.fround(137.50777);

/** Bevy's Hsla (s 1, l 0.5) as sRGB 0-255, through its Hsva as bevy_color converts it */
function rgbOf(h: number): string {
  const c = 1, x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const [r, g, b] = h < 60 ? [c, x, 0] : h < 120 ? [x, c, 0] : h < 180 ? [0, c, x] : h < 240 ? [0, x, c] : h < 300 ? [x, 0, c] : [c, 0, x];
  return `rgb(${r * 255}, ${g * 255}, ${b * 255})`;
}

/** rotate_hue: (hue + degrees) mod 360, in f32, from the start and then by each update's delta_secs x 100 */
function hueAt(start: number, f: number): number {
  let h = start;
  for (let k = 1; k <= f; k++) h = Math.fround(Math.fround(h + Math.fround(Math.fround(1 / 60) * 100)) % 360);
  return h;
}

const START: number[] = [];
for (let i = 0, h = 0; i < 9; i++, h = Math.fround(Math.fround(h + GOLDEN_ANGLE) % 360)) START.push(h);

function Shot() {
  const f = useCurrentFrame();
  const cubes = [];
  for (let x = -1, i = 0; x < 2; x++) {
    for (let z = -1; z < 2; z++, i++) {
      cubes.push(
        <Mesh3d key={i} transform={Transform.fromXyz(x, 0, z)}>
          <Cuboid half_size={[0.25, 0.25, 0.25]} />
          <StandardMaterial base_color={rgbOf(hueAt(START[i], f))} />
        </Mesh3d>,
      );
    }
  }
  return (
    <>
      {cubes}
      <Camera3d transform={Transform.fromXyz(3, 1, 3).lookingAt([0, -0.5, 0])} />
      <EnvironmentMapLight diffuse_map={staticFile("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2")} specular_map={staticFile("environment_maps/pisa_specular_rgb9e5_zstd.ktx2")} intensity={2000} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
