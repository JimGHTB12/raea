// Bevy v0.20.0 examples/shader/storage_buffer.rs in raea's TSX (docs/control.feature FC0a, FC2a): 13 x 7 cubes of
// 0.3 m at (i, j, 0) for i in -6..=6 and j in -3..=3, each with a MeshTag (its index mod 5), all with the example's
// own Material: assets/shaders/storage_buffer.wesl as written, its vertex and fragment shaders both, reading its
// colours from a read-only storage buffer of five vec4s by the cube's tag. The example's update system writes the
// buffer every update from the time (sin(t + i) / 2 + 0.5 and the same 2 and 4 later, t = 5 x elapsed seconds, in
// f32), as the port's buffer does each frame. The camera at (0, 0, 10), in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { BevyComponent } from "raea/3d";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)

const SHADER = staticFile("shaders/storage_buffer.wesl");
const f = Math.fround;
const CUBES = Array.from({ length: 13 * 7 }, (_, n) => ({ x: Math.floor(n / 7) - 6, y: (n % 7) - 3, tag: n % 5 }));

function Shot() {
  // update: elapsed_secs as of this update (no delta on the first), t = elapsed x 5, each colour channel in f32
  const t = f(f(useCurrentFrame() / 60) * 5);
  const colors = Array.from({ length: 5 }, (_, i) => [0, 2, 4].map((k) => f(f(Math.sin(f(f(t + i) + k)) / 2) + 0.5)).concat(1));
  return (
    <>
      {CUBES.map(({ x, y, tag }, n) => (
        <Mesh3d key={n} transform={Transform.fromXyz(x, y, 0)}>
          <Cuboid half_size={[0.15, 0.15, 0.15]} />
          <Material fragment={SHADER} vertex={SHADER} buffers={{ colors }} />
          <BevyComponent type="bevy_mesh::components::MeshTag" value={{ value: tag }} />
        </Mesh3d>
      ))}
      <Camera3d transform={Transform.fromXyz(0, 0, 10).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
