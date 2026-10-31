// Ambient occlusion fixture: <AmbientOcclusion> (and with it SMAA in place of multisampling) on a corner of two walls,
// stacked boxes, a sphere rolling toward the wall and a torus lying on the floor, lit mostly by indirect light (an
// AmbientLight and an interior panorama) so the occlusion in the creases and under things shows. A weak sun casts
// the only shadows.
import { AbsoluteFill, Composition, interpolate, registerRoot, staticFile, useCurrentFrame } from "raea";
import { AmbientOcclusion, Environment, Sphere, Torus } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 60;

function Shot() {
  const f = useCurrentFrame();
  const x = interpolate(f, [0, FRAMES - 1], [1.6, 0.2]);
  const stone = "#cfc8bc";
  return (
    <AbsoluteFill style={{ background: "#000000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(3.2, 2.2, 4.2).lookingAt([0, 0.6, 0])} projection={{ Perspective: { fov: 0.785398163397448 } }} />
        <AmbientOcclusion quality="high" />
        <AmbientLight brightness={240} />
        <Environment src={staticFile("museum_of_ethnography_1k.hdr")} intensity={0.6} />
        <DirectionalLight transform={Transform.fromXyz(2, 6, 3).lookingAt([0, 0, 0])} illuminance={2000} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[4, 0.05, 4]} />
          <StandardMaterial base_color={stone} perceptual_roughness={0.9} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, 1.5, -1.05)}>
          <Cuboid half_size={[4, 1.5, 0.05]} />
          <StandardMaterial base_color={stone} perceptual_roughness={0.9} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(-1.55, 1.5, 0)}>
          <Cuboid half_size={[0.05, 1.5, 4]} />
          <StandardMaterial base_color={stone} perceptual_roughness={0.9} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(-0.9, 0.35, -0.55)}>
          <Cuboid half_size={[0.35, 0.35, 0.35]} />
          <StandardMaterial base_color={stone} perceptual_roughness={0.8} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(-0.9, 0.95, -0.55).withRotation(Quat.fromEuler("XYZ", 0, 0.5, 0))}>
          <Cuboid half_size={[0.25, 0.25, 0.25]} />
          <StandardMaterial base_color={stone} perceptual_roughness={0.8} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(x, 0.4, 0.2).withRotation(Quat.fromEuler("XYZ", 0, 0, -x / 0.4))}>
          <Sphere radius={0.4} />
          <StandardMaterial base_color="#b8b0a4" perceptual_roughness={0.7} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0.6, 0.12, 1.3).withRotation(Quat.fromEuler("XYZ", Math.PI / 2, 0, 0))}>
          <Torus major_radius={0.35} minor_radius={0.12} />
          <StandardMaterial base_color="#b8b0a4" perceptual_roughness={0.7} />
        </Mesh3d>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="ssao" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
