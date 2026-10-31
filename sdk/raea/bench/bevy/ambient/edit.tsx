// An AmbientLight on a camera is that camera's own (docs/sdk.feature S2a, Bevy 0.20): the scene's bright
// GlobalAmbientLight gives way to the camera's dim one, so the cube reads as it does in only.tsx, lit by the dim one
// alone. ambient/check.sh renders both and compares them.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { AmbientLight, GlobalAmbientLight } from "raea/light";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000000" }}>
      <GlobalAmbientLight brightness={4000} />
      <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])}>
        <AmbientLight brightness={300} />
      </Camera3d>
      <Mesh3d transform={Transform.fromXyz(0, 0, 0)}>
        <Cuboid half_size={[1, 1, 1]} />
        <StandardMaterial base_color="#8090a0" />
      </Mesh3d>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="ambient" component={Shot} width={320} height={240} fps={30} durationInFrames={1} />);
