// Bevy v0.20.0 examples/gltf/load_gltf.rs in raea's TSX (docs/control.feature FC0a): the FlightHelmet glTF's first scene,
// seen from (0.7, 0.7, 1) looking at (0, 0.3, 0) and lit by Pisa's prefiltered EnvironmentMapLight at 250 on the camera
// and a shadowed DirectionalLight with one cascade to 1.6 m in a 4096 shadow map, the light turned each update to
// Quat::from_euler(ZYX, 0, t * PI / 5, -PI / 4) at Bevy's elapsed time. In Bevy's default 1280 x 720 window.
import { Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Model, Shadows } from "raea/3d";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { DirectionalLight, EnvironmentMapLight } from "raea/light";

// Time::elapsed_secs at frame f: f steps of 1/60 s, summed in f32
function elapsedAt(f: number): number {
  let t = 0;
  for (let k = 1; k <= f; k++) t = Math.fround(t + Math.fround(16_666_667e-9));
  return t;
}

function Shot() {
  const t = elapsedAt(useCurrentFrame());
  const turn = Quat.fromEuler("ZYX", 0, Math.fround(Math.fround(t * Math.PI) / 5), -Math.PI / 4);
  return (
    <>
      <Shadows mapSize={4096} distance={1.6} cascades={1} overlap={0.2} />
      <Camera3d transform={Transform.fromXyz(0.7, 0.7, 1).lookingAt([0, 0.3, 0])} />
      <EnvironmentMapLight diffuse_map={staticFile("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2")} specular_map={staticFile("environment_maps/pisa_specular_rgb9e5_zstd.ktx2")} intensity={250} />
      <DirectionalLight shadow_maps_enabled transform={Transform.fromRotation(turn)} />
      <Model src={staticFile("models/FlightHelmet/FlightHelmet.gltf")} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
