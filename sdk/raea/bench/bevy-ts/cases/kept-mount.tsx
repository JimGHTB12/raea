// gap: FC1b
// The 3D kept across frames (docs/control.feature FC1b: a subtree no commit touched is the last frame's): a red cube,
// the only child of a plain wrapper inside the Canvas3D, there for frames 10 to 19 alone, played live (one World, so
// a stale subtree would show): at 0.5 s (frame 15) it has mounted and shows.
// play: live-world.json 0.5
// probe: 32 32 r>150 g<80
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { Canvas3D } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Blink() {
  const f = useCurrentFrame();
  return <div>{f >= 10 && f < 20 ? <Mesh3d transform={Transform.fromXyz(0, 0, 0)}><Cuboid half_size={[0.6, 0.6, 0.6]} /><StandardMaterial base_color="#ff0000" unlit /></Mesh3d> : null}</div>;
}

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} />
        <Blink />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={30} fps={30} width={64} height={64} />);
