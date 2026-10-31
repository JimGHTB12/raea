// Simulated volumes in the scene's light: three <Cloud>s over a ground plane and a few things on it, a low sun behind
// them (volumetric, soft shadows) and a <FogVolume> of haze under <VolumetricFog mode="froxel">. The clouds are drawn
// into the sun's shadow maps (raea-render's volume_shadow.wesl), so their shadows sweep the ground and the things as they
// drift, and the sun's shafts break through the gaps between them in the haze. A long bar with shadow="only" hangs
// over the right of the field, unseen: only its shadow and the gap it cuts in the shafts show. Every value is the
// frame's, so render equals play (check.sh).
import { AbsoluteFill, Composition, interpolate, registerRoot, useCurrentFrame } from "raea";
import { Cylinder, FogVolume, Shadows, Sphere, ToneMapping, VolumetricFog } from "raea/3d";
import { Cloud } from "../../../../../examples/volumes/cloud";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 60;

/** toward the sun: 38 degrees up, off to the right and a little behind the clouds, so the shafts slant across the view */
const UP = (38 * Math.PI) / 180;
const SUN: [number, number, number] = [0.8 * Math.cos(UP), Math.sin(UP), -0.6 * Math.cos(UP)];

function Shot() {
  const f = useCurrentFrame();
  const t = f / FPS;
  // the clouds ride a wind across the sun
  const drift = interpolate(f, [0, FRAMES - 1], [-3, 3]);
  return (
    <AbsoluteFill style={{ background: "#8fb4dc" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 3, 30).lookingAt([0, 9, -12])} projection={{ Perspective: { fov: 1.08210413623648 } }} />
        <ToneMapping mode="agx" />
        <VolumetricFog mode="froxel" />
        <AmbientLight color="#a8c6ff" brightness={140} />
        <DirectionalLight name="sun" color="#fff0dc" volumetric={true} transform={Transform.fromXyz(SUN[0] * 200, SUN[1] * 200, SUN[2] * 200).lookingAt([0, 0, 0])} illuminance={32000} shadow_maps_enabled soft_shadow_size={4} />
        <Shadows distance={120} firstCascade={30} />
        <FogVolume name="haze" position={[0, 12, -10]} scale={[140, 26, 120]} density={0.012} color="#eef3fa" scattering={0.5} absorption={0.02} anisotropy={0.55} />
        {/* the clouds, gaps between them for the shafts */}
        <Cloud name="west" position={[-18 + drift, 17, -12]} size={[22, 10, 12]} seed={1} count={10} time={t + 4} resolution={48} />
        <Cloud name="mid" position={[6 + drift, 19, -16]} size={[20, 9, 11]} seed={4} count={9} time={t + 5} resolution={48} />
        <Cloud name="east" position={[30 + drift, 16, -10]} size={[18, 8, 10]} seed={9} count={8} time={t + 3} resolution={48} shadow={0.7} />
        {/* an unseen bar: its shadow alone */}
        <Mesh3d transform={Transform.fromXyz(12, 9, -2)} name="bar" shadow="only">
          <Cuboid half_size={[0.6, 0.6, 7]} />
          <StandardMaterial base_color="#ff00ff" />
        </Mesh3d>
        {/* the ground and the things on it */}
        <Mesh3d transform={Transform.fromXyz(0, -0.05, 0)} name="ground">
          <Cuboid half_size={[80, 0.05, 80]} />
          <StandardMaterial base_color="#7c8a5c" perceptual_roughness={0.95} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(-6, 4, 2)} name="column">
          <Cylinder args={[1, 1.2, 8]} />
          <StandardMaterial base_color="#d8d2c6" perceptual_roughness={0.7} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(4, 1.5, 6)} name="ball">
          <Sphere radius={1.5} />
          <StandardMaterial base_color="#c84a3a" perceptual_roughness={0.5} />
        </Mesh3d>
        <Mesh3d transform={Transform.fromXyz(9, 1.2, -4).withRotation(Quat.fromEuler("XYZ", 0, 0.5, 0))} name="crate">
          <Cuboid half_size={[1.2, 1.2, 1.2]} />
          <StandardMaterial base_color="#3a6ab0" perceptual_roughness={0.6} />
        </Mesh3d>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="volume-light" component={Shot} width={1920} height={1080} fps={FPS} durationInFrames={FRAMES} />);
