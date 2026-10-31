// Volumetric fixture for see-through surfaces in the froxel grid (<VolumetricFog mode="froxel">): a pane across a
// spot light's shaft and the haze, which fogs itself at its own depth (froxel_glass.rs), and a fire behind it (held at one time, so frames compare). Frames
// 0-19 an opaque pane; 20-39 the pane blended at opacity 1, which must draw as the opaque one; 40-49 blended at
// opacity 0, which must draw as no pane (50-59); 60-89 at 0.4 with a tint.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { Fire, FogVolume, VolumetricFog } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, SpotLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Srgba } from "raea/color";

const FPS = 30;
const FRAMES = 90;

function Shot() {
  const f = useCurrentFrame();
  const opacity = f < 40 ? 1 : f < 60 ? 0 : 0.4;
  return (
    <AbsoluteFill style={{ background: "#000000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(3.6, 1.6, 5.4).lookingAt([-0.4, 1.2, -1])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <VolumetricFog mode="froxel" ambient={0.05} ambientColor="#8090a8" />
        <AmbientLight brightness={32} />
        <SpotLight color="#ffe6c4" volumetric={true} transform={Transform.fromXyz(-3.5, 4.5, 1.5).lookingAt([0.8, 0, -0.6])} intensity={2700000} outer_angle={0.3} inner_angle={0.255} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)}>
          <Cuboid half_size={[5, 0.05, 5]} />
          <StandardMaterial base_color="#6a6258" perceptual_roughness={0.9} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, 2.5, -2.6)}>
          <Cuboid half_size={[4, 2.5, 0.1]} />
          <StandardMaterial base_color="#5a5248" perceptual_roughness={0.9} />
        </Mesh3d>
        <Fire name="fire" position={[-1.2, 0, -1.4]} scale={0.8} intensity={8} seed={3} time={1.5} />
        {/* the pane, across the shaft: opaque, or see-through */}
        {f < 20 && (
          <Mesh3d transform={Transform.fromXyz(0.2, 1.3, 0.6).withRotation(Quat.fromEuler("XYZ", 0, 0.5, 0))} name="opaque">
            <Cuboid half_size={[1.2, 1.1, 0.015]} />
            <StandardMaterial base_color="#9fc4d8" perceptual_roughness={0.2} />
          </Mesh3d>
        )}
        {f >= 20 && (f < 50 || f >= 60) && (
          <Mesh3d transform={Transform.fromXyz(0.2, 1.3, 0.6).withRotation(Quat.fromEuler("XYZ", 0, 0.5, 0))} name="pane">
            <Cuboid half_size={[1.2, 1.1, 0.015]} />
            <StandardMaterial base_color={Srgba.hex("#9fc4d8").with_alpha(opacity)} perceptual_roughness={0.2} alpha_mode={opacity < 1 ? "Blend" : "Opaque"} />
          </Mesh3d>
        )}
        <FogVolume name="air" position={[0, 2, 0.5]} scale={[8, 4, 6]} density={0.15} color="#e8e0d4" scattering={0.45} absorption={0.15} anisotropy={0.5} />
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="volumetric-glass" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
