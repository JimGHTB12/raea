// Bevy v0.20.0 examples/shader_advanced/custom_phase_item.rs in raea's TSX (docs/control.feature FC2d): a triangle
// queued straight into the camera's Opaque3d phase, not a mesh: its own pipeline of
// assets/shaders/custom_phase_item.wesl as written (`vertex` passes clip-space positions through, `fragment` their
// colour), its own vertex buffer (each vertex a vec3 position, a u32 pad, a vec3 colour and a pad: 32 bytes, the
// colour at 16) and index buffer [0, 1, 2], depth compared Always and not written. Its entity has an Aabb of half
// extents 0.5 at the origin so the camera sees it; the camera at (0, 0, 1) looking at the origin, in Bevy's default
// 1280 x 720 window.
import { Composition, registerRoot, staticFile } from "raea";
import { PhaseItem } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";

// position, pad, colour, pad
const VERTICES = [
  [-0.866, -0.5, 0.5, 0, 1, 0, 0, 0],
  [0.866, -0.5, 0.5, 0, 0, 1, 0, 0],
  [0, 1, 0.5, 0, 0, 0, 1, 0],
].flat();

function Shot() {
  return (
    <>
      <PhaseItem
        shader={staticFile("shaders/custom_phase_item.wesl")}
        stride={32}
        attributes={[{ format: "Float32x3", offset: 0, location: 0 }, { format: "Float32x3", offset: 16, location: 1 }]}
        words={VERTICES}
        indices={[0, 1, 2]}
        depth={{ write: false, compare: "Always" }}
        aabb={{ center: [0, 0, 0], half_extents: [0.5, 0.5, 0.5] }}
      />
      <Camera3d transform={Transform.fromXyz(0, 0, 1).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
