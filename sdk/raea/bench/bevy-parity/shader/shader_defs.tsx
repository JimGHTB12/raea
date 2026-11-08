// Bevy v0.20.0 examples/shader/shader_defs.rs in raea's TSX (docs/control.feature FC0a, FC2a): two 1 m cubes with the
// example's own Material, assets/shaders/shader_defs.wesl as written (its uniform a CustomMaterial struct at binding
// 0, given as { color }): at x -1 LinearRgba::BLUE, and at x 1 LinearRgba::GREEN with is_red, which the example's
// specialize turns into the IS_RED shader def (here `defs`, each set compiling its own pipeline as Bevy's key does);
// the camera at (-2, 2.5, 5), in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)

const SHADER = staticFile("shaders/shader_defs.wesl");

function Shot() {
  return (
    <>
      <Mesh3d transform={Transform.fromXyz(-1, 0.5, 0)}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <Material fragment={SHADER} uniforms={{ material: { color: [0, 0, 1, 1] } }} />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(1, 0.5, 0)}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <Material fragment={SHADER} uniforms={{ material: { color: [0, 1, 0, 1] } }} defs={{ IS_RED: true }} />
      </Mesh3d>
      <Camera3d transform={Transform.fromXyz(-2, 2.5, 5).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
