// gap: G8
// A masked clip node (<Model graph> mask): masks.gltf's clip "go" moves both chips, "top" (y +1) and
// "bottom" (y -1), to x +1.5; masked off "top", only the bottom chip moves and the top stays at its rest pose.
// probe: 32 18 r>150
// probe: 53 18 r<60
// probe: 53 46 r>150
// probe: 32 46 r<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, Model } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 4).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 1.0471975511966 } }} />
        <Model src="masks.gltf" graph={{ clip: "go", mask: ["top"] }} />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
