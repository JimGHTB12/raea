// DR4's proof, the lyric's cheat: the line "made 2D" by leaving it 3D type with a hair of depth, facing the camera so
// it looks flat, and nothing in front of it. (No footage: a page layer is drawn over the 3D, and would hide the line.)
// checks/lyric.ts must fail it.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Text3D } from "raea/3d";
import { Camera3d } from "raea/camera";
import { DirectionalLight } from "raea/light";
import { Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";

function Lyric() {
  return (
    <AbsoluteFill style={{ background: "#16202e" }}>
      <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} />
      <DirectionalLight illuminance={9000} transform={Transform.fromXyz(0, 2, 6).lookingAt([0, 0, 0])} />
      <Mesh3d name="line" transform={Transform.fromXyz(0, 1, 0)}>
        <Text3D size={0.25} depth={0.002}>OUT OF THE GREY</Text3D>
        <StandardMaterial base_color="#ffffff" />
      </Mesh3d>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="lyric-cheat" component={Lyric} width={540} height={960} fps={30} durationInFrames={60} />);
