// Bevy v0.20.0 examples/3d/fog_volumes.rs in raea's TSX (docs/control.feature FC0a): a unit fog volume shaped by
// Bevy's bunny density texture (volumes/bunny.ktx2), lit through by a volumetric sun of 32 000 lux, seen by a camera
// marching 64 steps (Bevy's VolumetricFog) with no ambient light. The camera turns 0.01 rad about Y every update, the
// first too, looking at (0, 0.5, 0).
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Camera3d } from "raea/camera";
import { DirectionalLight, GlobalAmbientLight } from "raea/light";
import { FogVolume, VolumetricFog } from "raea/pbr";
import { Transform } from "raea/transform";

/** rotate_camera over the updates up to frame f: the translation turned by Ry(0.01) each time, in f32 */
function eyeAt(f: number): [number, number, number] {
  const [c, s] = [Math.fround(Math.cos(0.01)), Math.fround(Math.sin(0.01))];
  let [x, y, z] = [-0.75, 1, 2];
  for (let k = 0; k <= f; k++) [x, z] = [Math.fround(Math.fround(c * x) + Math.fround(s * z)), Math.fround(Math.fround(c * z) - Math.fround(s * x))];
  return [x, y, z];
}

function Shot() {
  const [x, y, z] = eyeAt(useCurrentFrame());
  return (
    <>
      <GlobalAmbientLight brightness={0} />
      <FogVolume position={[0, 0.5, 0]} density_texture={staticFile("volumes/bunny.ktx2")} density={1} scattering={1} />
      <DirectionalLight shadow_maps_enabled illuminance={32000} volumetric transform={Transform.fromXyz(1, 1, -0.3).lookingAt([0, 0.5, 0])} />
      <Camera3d transform={Transform.fromXyz(x, y, z).lookingAt([0, 0.5, 0])} />
      <VolumetricFog mode="march" steps={64} ambient={0} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
