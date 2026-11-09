// Bevy v0.20.0 examples/shader_advanced/texture_binding_array.rs in raea's TSX (docs/control.feature FC2d): a
// BindlessMaterial of 16 pictures bound as one binding_array<texture_2d<f32>> and drawn by
// assets/shaders/texture_binding_array.wesl as written, each face of a default Cuboid (1 m) split 4 x 4 into the 16
// tiles, the textures/rpg/tiles pictures of TILE_ID in order, sampled with nearest filtering (ImagePlugin's
// default_nearest, whose fallback sampler the example binds as nearest_sampler); the camera at (2, 2, 2) looking at
// the origin, in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile } from "raea";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Material } from "@raea/render/bevy3d";

const TILE_ID = [19, 23, 4, 33, 12, 69, 30, 48, 10, 65, 40, 47, 57, 41, 44, 46];
const NEAREST = { filter: "nearest", mipmap: "nearest", wrap: "clamp" } as const;
const TILES = TILE_ID.map((id) => ({ src: staticFile(`textures/rpg/tiles/generic-rpg-tile${String(id).padStart(2, "0")}.png`), sampler: NEAREST }));

function Shot() {
  return (
    <>
      <Mesh3d>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <Material fragment={staticFile("shaders/texture_binding_array.wesl")} textures={{ textures: TILES }} />
      </Mesh3d>
      <Camera3d transform={Transform.fromXyz(2, 2, 2).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
