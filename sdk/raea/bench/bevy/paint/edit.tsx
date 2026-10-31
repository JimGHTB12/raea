// Paint fixture: <Look mode="paint"> over a few boxes and a sphere on a ground, a low sun throwing hard shadows, a flat
// two-tone sky, a palette-free three-band painting whose grain and edge wander are keyed (they ramp over the shot) and
// whose wander boils every 4 frames, so a baked play must key them as a render does; then a box changes colour, which
// the paint copy must follow. The camera orbits.
import { AbsoluteFill, Composition, interpolate, registerRoot, useCurrentFrame } from "raea";
import { Look, Shadows, Sphere } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 60;

function Shot() {
  const f = useCurrentFrame();
  const a = interpolate(f, [0, FRAMES - 1], [0.4, 1.0]);
  return (
    <AbsoluteFill style={{ background: "#3d8fd6" }}>
      <>
        <Camera3d transform={Transform.fromXyz(Math.sin(a) * 9, 5, Math.cos(a) * 9).lookingAt([0, 0.8, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <Look mode="paint" bands={3} shadow="#9aa0b8" mid="#d6b4ae" sky={["#3d8fd6", "#9ccfee"]} boil={4}
          grain={interpolate(f, [0, FRAMES - 1], [0.1, 0.5])} rag={interpolate(f, [0, FRAMES - 1], [1, 3])} />
        <DirectionalLight transform={Transform.fromXyz(-4, 3, 3).lookingAt([0, 0, 0])} illuminance={9600} shadow_maps_enabled />
        <Shadows distance={30} firstCascade={25} cascades={1} />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)} name="ground"><Cuboid half_size={[7, 0.05, 7]} /><StandardMaterial base_color="#6fb04a" /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, 1.5, 0)} name="tower"><Cuboid half_size={[0.5, 1.5, 0.5]} /><StandardMaterial base_color="#f4f0e8" /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(1.8, 0.4, 1)} name="crate">
          <Cuboid half_size={[0.4, 0.4, 0.4]} />
          <StandardMaterial base_color={f < 30 ? "#d8463a" : "#2f6fc4"} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(-1.6, 0.6, 1.2)} name="ball"><Sphere radius={0.6} /><StandardMaterial base_color="#f2b632" /></Mesh3d>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="paint" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
