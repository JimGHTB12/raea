// Bevy v0.20.0 examples/shader/fallback_image.rs in raea's TSX (docs/control.feature FC0a, FC2a): a 1 m cube at the
// origin with the example's own Material, assets/shaders/fallback_image_test.wesl as written: a texture of each
// dimension (1d, 2d, 2d_array, cube, cube_array, 3d) and a sampler for each, none given, so each binds Bevy's
// fallback image of its dimension; its fragment writes nothing. The camera at (5, 5, 5) looking at (1.5, 0, 0), in
// Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)

function Shot() {
  return (
    <>
      <Mesh3d transform={Transform.IDENTITY}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <Material fragment={staticFile("shaders/fallback_image_test.wesl")} />
      </Mesh3d>
      <Camera3d transform={Transform.fromXyz(5, 5, 5).lookingAt([1.5, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
