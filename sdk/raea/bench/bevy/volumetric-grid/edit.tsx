// Volumetric fixture for what only the froxel grid draws: <VolumetricFog mode="froxel"> with a ground fog (a wide
// <FogVolume> that thins going up, falloff), a <Fire> whose light the fog takes up around it, smoke (an absorbing
// volume) rolling in over the fire from frame 20 and overlapping the ground fog, and a volumetric spot through a
// window's blind laying its bars in the haze. check.sh holds render equal to play.
import { AbsoluteFill, Composition, interpolate, registerRoot, useCurrentFrame } from "raea";
import { Fire, FogVolume, VolumetricFog } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, SpotLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 60;
const SLATS = Array.from({ length: 9 }, (_, i) => 1.28 + i * 0.16);

function Shot() {
  const f = useCurrentFrame();
  const smoke = interpolate(f, [20, 50], [0, 0.6], { extrapolateLeft: "clamp", extrapolateRight: "clamp" });
  return (
    <AbsoluteFill style={{ background: "#000000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(4.2, 1.7, 5.2).lookingAt([-0.3, 1.0, -1])} projection={{ Perspective: { fov: 0.907571211037051 } }} />
        <VolumetricFog mode="froxel" />
        <AmbientLight brightness={20} />
        <SpotLight color="#dfe8ff" volumetric={true} transform={Transform.fromXyz(-6, 4.2, -1).lookingAt([0.5, 0, 0.5])} intensity={3600000} outer_angle={0.32} inner_angle={0.288} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[6, 0.05, 6]} />
          <StandardMaterial base_color="#5c564e" perceptual_roughness={0.9} />
        </Mesh3d>
        {/* a wall with a window (1.2 to 2.6 m up, 1.8 m wide), its blind's slats across it */}
        {([[0.6, 1.2, 0, 6], [3.3, 1.4, 0, 6], [1.9, 1.4, -1.95, 2.1], [1.9, 1.4, 1.95, 2.1]] as const).map(([y, h, z, w], i) => (
          <Mesh3d key={`w${i}`} transform={Transform.fromXyz(-3, y, z)} name={`wall${i}`}>
            <Cuboid half_size={[0.1, h / 2, w / 2]} />
            <StandardMaterial base_color="#4a4640" perceptual_roughness={0.9} />
          </Mesh3d>
        ))}
        {SLATS.map((y, i) => (
          <Mesh3d key={i} transform={Transform.fromXyz(-3, y, 0).withRotation(Quat.fromEuler("XYZ", 0, 0, 0.6))} name={`slat${i}`}>
            <Cuboid half_size={[0.01, 0.05, 0.9]} />
            <StandardMaterial base_color="#2a2824" />
          </Mesh3d>
        ))}
        <Fire name="fire" position={[1.2, 0, -0.6]} scale={1.1} intensity={10} seed={2} />
        <FogVolume name="ground" position={[0, 2, 0]} scale={[12, 4, 12]} density={0.12} falloff={0.8} color="#d8dde8" scattering={0.5} absorption={0.1} anisotropy={0.5} />
        <FogVolume name="smoke" position={[1.2, 2.2, -0.6]} scale={[1.6, 2.4, 1.6]} density={smoke} color="#3a3632" scattering={0.2} absorption={0.8} anisotropy={0.2} />
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="volumetric-grid" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
