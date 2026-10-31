// Model fixture: a glTF (media/spinner.glb, made by make_glb.py) with its "Spin" clip posed at the frame's time, a
// second copy playing "Bob" from a Sequence, on a floor under a key light with shadows.
import { AbsoluteFill, Composition, registerRoot, Sequence, staticFile } from "raea";
import { Model } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#1b2230" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 2.2, 5.5).lookingAt([0, 0.7, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <AmbientLight brightness={160} />
        <DirectionalLight transform={Transform.fromXyz(3, 6, 4).lookingAt([0, 0, 0])} illuminance={9600} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[4, 0.05, 4]} />
          <StandardMaterial base_color="#d9d4c7" perceptual_roughness={0.9} />
        </Mesh3d>
        <Model src={staticFile("spinner.glb")} animation="Spin" position={[-1, 0, 0]} />
        <Sequence from={20} layout="none">
          <Model src={staticFile("spinner.glb")} animation="Bob" position={[1.2, 0, 0]} scale={0.8} />
        </Sequence>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="model" component={Shot} width={960} height={540} fps={FPS} durationInFrames={60} />);
