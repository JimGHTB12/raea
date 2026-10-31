// Fire fixture: a <Fire> on a stack of logs, burning up over the shot (its intensity keyed), the camera circling it,
// a second, smaller blue-white fire on a block that appears at frame 30 from a Sequence, under Bloom. Every pixel of a
// flame is a function of its time, so frame 59 rendered alone equals frame 59 reached in play.
import { AbsoluteFill, Composition, interpolate, registerRoot, Sequence, useCurrentFrame } from "raea";
import { Bloom, Cylinder, Fire, ToneMapping } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight } from "raea/light";
import { Transform, Quat } from "raea/transform";
import { Camera3d } from "raea/camera";
import { LinearRgba, Srgba } from "raea/color";

const FPS = 30;
const FRAMES = 60;

function Shot() {
  const f = useCurrentFrame();
  const a = interpolate(f, [0, FRAMES - 1], [-0.5, 0.4]);
  const burn = interpolate(f, [0, FRAMES - 1], [4, 9]);
  return (
    <AbsoluteFill style={{ background: "#000000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(Math.sin(a) * 5, 1.6, Math.cos(a) * 5).lookingAt([0, 1.1, 0])} projection={{ Perspective: { fov: 0.785398163397448 } }} />
        <ToneMapping mode="agx" />
        <Bloom intensity={0.4} threshold={0.9} />
        <AmbientLight color="#6f86b8" brightness={20} />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[6, 0.05, 6]} />
          <StandardMaterial base_color="#5a5248" perceptual_roughness={0.9} />
        </Mesh3d>
        {[0, 1, 2, 3].map((i) => (
          <Mesh3d key={i} transform={Transform.fromTranslation(i < 2 ? [0, 0.11, i * 0.5 - 0.25] : [(i - 2) * 0.5 - 0.25, 0.33, 0]).withRotation(Quat.fromEuler("XYZ", ...((i < 2 ? [0, 0, Math.PI / 2] : [Math.PI / 2, 0, 0]) as [number, number, number])))}>
            <Cylinder radius={0.11} half_height={0.7} />
            <StandardMaterial base_color="#3a2a1e" perceptual_roughness={1} emissive={LinearRgba.from(Srgba.hex("#ff4a10")).mul(0.4)} />
          </Mesh3d>
        ))}
        <Fire name="pyre" position={[0, 0.2, 0]} scale={2.2} intensity={burn} light={1.2} />
        <Mesh3d transform={Transform.fromXyz(2, 0.3, -1)}>
          <Cuboid half_size={[0.3, 0.3, 0.3]} />
          <StandardMaterial base_color="#8e8476" perceptual_roughness={0.9} />
        </Mesh3d>
        <Sequence from={30} layout="none">
          <Fire name="lamp" position={[2, 0.6, -1]} scale={[0.35, 0.8, 0.35]} intensity={8} color="#8fb4ff" seed={3} light={0.4} />
        </Sequence>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="fire" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
