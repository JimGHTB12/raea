// Fixture: declared 3D animation (<Mesh animate>, a material's animate), sampled by the engine from the frame clock
// (raea-animate) with no useCurrentFrame: a box drops in on a spring curve and turns, a sphere rises from a Sequence,
// its colour and glow blend in Oklab. Render and the baked play with its 3D must agree (check.sh).
import { AbsoluteFill, Composition, Sequence, curve, registerRoot, track } from "raea";
import { Sphere } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { LinearRgba, Srgba } from "raea/color";

const DROP = { y: track({ 0: 3, 24: 0.5 }, curve.spring({ damping: 12, stiffness: 140 })), rotateY: track({ 0: 0, 59: Math.PI }, "in-out-sine") };
const RISE = { y: track({ 0: -1.5, 14: 0.6 }, "out-back"), scale: track({ 0: 0.2, 14: 1 }, "out-cubic") };
const GLOW = { base_color: track({ 0: "#3060ff", 30: "#ff4070" }, "in-out-cubic"), emissive: track({ 0: LinearRgba.from(Srgba.hex("#ff4070")).mul(0), 30: LinearRgba.from(Srgba.hex("#ff4070")).mul(2.5) }, "out-cubic") };

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#141a26" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 2.2, 7).lookingAt([0, 0.6, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <AmbientLight brightness={200} />
        <DirectionalLight transform={Transform.fromXyz(3, 6, 4).lookingAt([0, 0, 0])} illuminance={10400} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[5, 0.05, 3]} />
          <StandardMaterial base_color="#d9d4c7" perceptual_roughness={0.9} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(-1.2, 3, 0)} name="box" animate={DROP}>
          <Cuboid half_size={[0.5, 0.5, 0.5]} />
          <StandardMaterial base_color="#ffb020" perceptual_roughness={0.4} />
        </Mesh3d>
        <Sequence from={20} layout="none">
          <Mesh3d transform={Transform.fromXyz(1.4, -1.5, 0)} name="ball" animate={RISE}>
            <Sphere radius={0.6} />
            <StandardMaterial base_color="#3060ff" perceptual_roughness={0.3} emissive="#ff4070" animate={GLOW} />
          </Mesh3d>
        </Sequence>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="animate3d" component={Shot} width={960} height={540} fps={30} durationInFrames={60} />);
