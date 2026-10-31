// Fixture: a copy of examples/games/animated-character.tsx (keep them equal).
// A rigged glTF character playing its clips: the Bevy "animated mesh" example in raea's SDK. It expects
// media/RobotExpressive.glb (CC0, Quaternius; three.js ships it as examples/models/gltf/RobotExpressive).
// One robot walks across the floor (clip "Walking", its position from the frame), a second waves from a Sequence.
import { AbsoluteFill, Composition, interpolate, registerRoot, Sequence, staticFile, useCurrentFrame } from "raea";
import { Model } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 90;
// the model is about 4.5 units tall at scale 1, its origin at the feet, facing +z
const SCALE = 0.4;

function Shot() {
  const f = useCurrentFrame();
  const x = interpolate(f, [0, FRAMES - 1], [-2.4, 0.6]);
  return (
    <AbsoluteFill style={{ background: "#1b2230" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 2.4, 7).lookingAt([0, 0.9, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <AmbientLight brightness={180} />
        <DirectionalLight transform={Transform.fromXyz(3, 6, 4).lookingAt([0, 0, 0])} illuminance={10400} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[6, 0.05, 4]} />
          <StandardMaterial base_color="#d9d4c7" perceptual_roughness={0.9} />
        </Mesh3d>
        <Model name="walker" src={staticFile("RobotExpressive.glb")} animation="Walking" position={[x, 0, 0]} rotation={[0, Math.PI / 2, 0]} scale={SCALE} />
        <Sequence from={15} layout="none">
          <Model name="waver" src={staticFile("RobotExpressive.glb")} animation="Wave" position={[2, 0, -1]} rotation={[0, -0.5, 0]} scale={SCALE} />
        </Sequence>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="character" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
