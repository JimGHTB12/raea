// ambient/check.sh's reference: the cube lit by the dim ambient light alone, as a camera's own over the
// scene's bright one should leave it (edit.tsx).
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { GlobalAmbientLight } from "raea/light";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000000" }}>
      <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} />
      <GlobalAmbientLight brightness={300} />
      <Mesh3d transform={Transform.fromXyz(0, 0, 0)}>
        <Cuboid half_size={[1, 1, 1]} />
        <StandardMaterial base_color="#8090a0" />
      </Mesh3d>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="ambient-only" component={Shot} width={320} height={240} fps={30} durationInFrames={1} />);
