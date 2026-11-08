// Bevy v0.20.0 examples/gltf/gltf_skinned_mesh.rs in raea's TSX (docs/control.feature FC0a): SimpleSkin's glTF scene
// under GlobalAmbientLight 750, seen from (-2, 2.5, 5) looking at (0, 1, 0), its second joint (the unnamed node 2,
// GltfNode2 to Bevy's loader) turned each update to Quat::from_rotation_z(PI / 2 * sin(t)) at Bevy's elapsed time.
// In Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Model } from "raea/3d";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { GlobalAmbientLight } from "raea/light";

// Time::elapsed_secs at frame f: f steps of 1/60 s, summed in f32
function elapsedAt(f: number): number {
  let t = 0;
  for (let k = 1; k <= f; k++) t = Math.fround(t + Math.fround(16_666_667e-9));
  return t;
}

function Shot() {
  const t = elapsedAt(useCurrentFrame());
  const q = Quat.fromRotationZ(Math.fround((Math.PI / 2) * Math.fround(Math.sin(t))));
  return (
    <>
      <GlobalAmbientLight brightness={750} />
      <Camera3d transform={Transform.fromXyz(-2, 2.5, 5).lookingAt([0, 1, 0])} />
      <Model src={staticFile("models/SimpleSkin/SimpleSkin.gltf")} nodes={{ GltfNode2: { rotation: q } }} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={91} fps={60} width={1280} height={720} />);
