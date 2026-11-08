// Bevy v0.20.0 examples/shader/automatic_instancing.rs in raea's TSX (docs/control.feature FC0a, FC2a): 65 536 cubes
// of side 0.01, one per pixel of branding/icon.png (256 x 256), each tagged with its index (MeshTag) and drawn by the
// example's own Material, assets/shaders/automatic_instancing.wesl as written: its vertex shader reads the tag's
// texel of the icon (textureLoad) as the colour its fragment returns. Every cube shares one mesh and one material,
// so Bevy draws them as instances. The update moves each to z = sin(elapsed_secs + index * 0.01); the camera at
// (0, 0, 5) looking at the origin.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { BevyComponent } from "raea/3d";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)

const SHADER = staticFile("shaders/automatic_instancing.wesl");
const ICON = staticFile("branding/icon.png");
const W = 256;

// Time::elapsed_secs at frame f: f steps of 1/60 s, summed in f32
function elapsedAt(f: number): number {
  let t = 0;
  for (let k = 1; k <= f; k++) t = Math.fround(t + Math.fround(16_666_667e-9));
  return t;
}

const places = Array.from({ length: W * W }, (_, i) => [(i % W - W / 2) / 50, -((Math.floor(i / W) - W / 2) / 50)]);

function Shot() {
  const t = elapsedAt(useCurrentFrame());
  return (
    <>
      {places.map(([x, y], i) => (
        <Mesh3d key={i} transform={Transform.fromXyz(x, y, Math.fround(Math.sin(Math.fround(t + Math.fround(i * 0.01)))))}>
          <Cuboid half_size={[0.005, 0.005, 0.005]} />
          <Material fragment={SHADER} vertex={SHADER} textures={{ texture: ICON }} />
          <BevyComponent type="bevy_mesh::components::MeshTag" value={{ value: i }} />
        </Mesh3d>
      ))}
      <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
