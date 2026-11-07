// record: bevy-3d-rendering-3d-shapes
// Bevy's "3D shapes" example in raea's SDK: a row of primitives turning on a floor under a key light.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { Cone, Cylinder, Sphere, Torus } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";

const COLOURS = ["#e76f51", "#f4a261", "#e9c46a", "#2a9d8f", "#457b9d"];

function Shapes() {
  const f = useCurrentFrame();
  const spin = f * 0.04;
  const shapes = [<Cuboid half_size={[0.45, 0.45, 0.45]} />, <Sphere radius={0.55} />, <Cylinder radius={0.45} half_height={0.5} />, <Cone radius={0.55} height={1.1} />, <Torus major_radius={0.42} minor_radius={0.16} />];
  return (
    <AbsoluteFill style={{ background: "#14181f" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 3, 8).lookingAt([0, 0.5, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <AmbientLight brightness={140} />
        <DirectionalLight transform={Transform.fromXyz(4, 8, 6).lookingAt([0, 0, 0])} illuminance={10400} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[7, 0.05, 3]} />
          <StandardMaterial base_color="#cfc8ba" perceptual_roughness={0.95} />
        </Mesh3d>
        {shapes.map((g, i) => (
          <Mesh3d key={i} transform={Transform.fromXyz((i - 2) * 2, 0.75, 0).withRotation(Quat.fromEuler("XYZ", spin * 0.5, spin + i, 0))} name={`shape-${i}`}>
            {g}
            <StandardMaterial base_color={COLOURS[i]} perceptual_roughness={0.45} />
          </Mesh3d>
        ))}
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="shapes" component={Shapes} width={1280} height={720} fps={30} durationInFrames={90} />);
