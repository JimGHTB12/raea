// Lights fixture: a point light orbiting with shadows, a soft spot from above, a dim sun; the camera dollies in
// (position and lookAt animated); a Group turns its children together; a pillar grows (non-uniform scale); a sphere
// appears at frame 30 from a Sequence.
import { AbsoluteFill, Composition, interpolate, registerRoot, Sequence, useCurrentFrame } from "raea";
import { Cylinder, Sphere, Torus, Entity } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight, PointLight, SpotLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 60;

function Shot() {
  const f = useCurrentFrame();
  const a = (f / FRAMES) * Math.PI * 2;
  const dolly = interpolate(f, [0, FRAMES - 1], [9, 6.5]);
  const grow = interpolate(f, [0, FRAMES - 1], [0.4, 1.6]);
  return (
    <AbsoluteFill style={{ background: "#0b0d12" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 3, dolly).lookingAt([0, 0.6, 0])} projection={{ Perspective: { fov: 0.785398163397448 } }} />
        <AmbientLight brightness={60} />
        <DirectionalLight transform={Transform.fromXyz(-4, 6, 2).lookingAt([0, 0, 0])} illuminance={2000} />
        <PointLight color="#ffd29a" transform={Transform.fromXyz(Math.cos(a) * 2.5, 1.8, Math.sin(a) * 2.5)} intensity={1080000} range={12} shadow_maps_enabled />
        <SpotLight color="#9ad0ff" transform={Transform.fromXyz(0, 6, 0).lookingAt([0, 0, 0], [0, 0, -1])} intensity={2250000} outer_angle={0.5} inner_angle={0.3} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[6, 0.05, 6]} />
          <StandardMaterial base_color="#9aa0a6" perceptual_roughness={0.8} />
        </Mesh3d>
        <Entity transform={Transform.fromXyz(0, 0.8, 0).withRotation(Quat.fromEuler("XYZ", 0, a / 2, 0))}>
          <Mesh3d transform={Transform.fromXyz(-1.4, 0, 0)}>
            <Cuboid half_size={[0.4, 0.4, 0.4]} />
            <StandardMaterial base_color="#e4572e" perceptual_roughness={0.4} />
          </Mesh3d>
          <Mesh3d transform={Transform.fromXyz(1.4, 0, 0).withRotation(Quat.fromEuler("XYZ", Math.PI / 2, 0, 0))}>
            <Torus major_radius={0.45} minor_radius={0.16} />
            <StandardMaterial base_color="#29335c" perceptual_roughness={0.3} metallic={0.5} />
          </Mesh3d>
        </Entity>
        <Mesh3d transform={Transform.fromXyz(0, grow / 2, -1.5).withScale([0.5, grow, 0.5])}>
          <Cylinder radius={0.5} half_height={0.5} />
          <StandardMaterial base_color="#f3a712" perceptual_roughness={0.5} />
        </Mesh3d>
        <Sequence from={30} layout="none">
          <Mesh3d transform={Transform.fromXyz(0, 0.5, 1.6)} name="late-ball">
            <Sphere radius={0.5} />
            <StandardMaterial base_color="#669bbc" perceptual_roughness={0.2} />
          </Mesh3d>
        </Sequence>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="lights" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
