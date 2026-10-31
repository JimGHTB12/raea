// Volumetric fixture: a <VolumetricFog> on the camera, a spot light shafting through a slot in a wall and a low sun,
// both volumetric, inside a <FogVolume> that drifts, thickens and turns red over the shot; a second volume appears at
// frame 30 from a Sequence.
// Bevy's march (mode="march"; the froxel grid is the default since it scored at or above it), kept for its props.
import { AbsoluteFill, Composition, interpolate, registerRoot, Sequence, useCurrentFrame } from "raea";
import { FogVolume, Sphere, VolumetricFog, manifold } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight, SpotLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 60;

// a wall with a slot in it
const wall = manifold.box(8, 5, 0.3).minus(manifold.box(0.6, 2, 1).move(-1, 0.8, 0));

function Shot() {
  const f = useCurrentFrame();
  const drift = interpolate(f, [0, FRAMES - 1], [-0.5, 0.5]);
  const thick = interpolate(f, [0, FRAMES - 1], [0.08, 0.2]);
  const red = f < FRAMES / 2 ? "#fff0e0" : "#c0281c";
  return (
    <AbsoluteFill style={{ background: "#000000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(4.5, 2, 5).lookingAt([-0.5, 1.5, -1])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <VolumetricFog mode="march" steps={64} ambient={0.05} ambientColor="#8090a8" />
        <AmbientLight brightness={40} />
        <DirectionalLight color="#ffb070" volumetric={true} transform={Transform.fromXyz(3, 2, -4).lookingAt([0, 0, 0])} illuminance={1200} shadow_maps_enabled />
        <SpotLight color="#ffe6c4" volumetric={true} transform={Transform.fromXyz(-2.5, 6, -6).lookingAt([0, 0.5, 0.5])} intensity={2250000} outer_angle={0.3} inner_angle={0.24} />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[5, 0.05, 5]} />
          <StandardMaterial base_color="#6a6258" perceptual_roughness={0.9} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, 2.5, -2.5)} name="wall" shape={wall}>
          <StandardMaterial base_color="#5a5248" perceptual_roughness={0.9} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, 0.6, 0.5)}>
          <Sphere radius={0.6} />
          <StandardMaterial base_color="#d8d0c4" perceptual_roughness={0.6} />
        </Mesh3d>
        <FogVolume name="air" position={[drift, 2, 0.5]} scale={[8, 4, 6]} density={thick} color={red} scattering={0.45} absorption={0.15} anisotropy={0.6} />
        <Sequence from={30} layout="none">
          <FogVolume name="smoke" position={[2, 0.6, 1.5]} scale={[1.5, 1.2, 1.5]} density={0.5} color="#404040" absorption={0.6} scattering={0.2} />
        </Sequence>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="volumetric" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
