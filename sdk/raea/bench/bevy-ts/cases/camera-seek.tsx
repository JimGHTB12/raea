// gap: PL11
// A camera row that moves every frame, rendered by seeking straight to frame 30: the camera pans along x at 0.1 m a
// frame, looking down -z, so at frame 30 it is at x = 3 and the red cube at (3, 0, -5) fills the middle (at frame 0
// it is off to the right). The scene's camera sync writes a camera only when its row moves, and an entity's first
// frame always writes: this is the seek that must still be exact (runtime/scene3d sync_camera).
// frame: 30
// probe: 32 32 r>150 g<80 b<80
// probe: 4 32 r<30 g<30 b<30
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { ToneMapping } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";

function Shot() {
  const x = useCurrentFrame() * 0.1;
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(x, 0, 0).lookingAt([x, 0, -1])} projection={{ Perspective: { fov: 0.785398 } }} />
        <ToneMapping mode="none" />
        <Mesh3d name="red" transform={Transform.fromXyz(3, 0, -5)}>
          <Cuboid half_size={[0.5, 0.5, 0.5]} />
          <StandardMaterial base_color="#ff0000" unlit />
        </Mesh3d>
      </>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={60} fps={30} width={64} height={64} />);
