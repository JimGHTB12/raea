// Bevy v0.20.0 examples/3d/shadow_caster_receiver.rs in raea's TSX (docs/control.feature FC0a): a red sphere that casts
// and a blue one that doesn't (radius 0.25, 2 m up, x -1 and 1), a lime 20 m plane at (0, 1, -10) that neither casts nor
// receives, a white rough 20 m floor, a PointLight at (5, 5, 0) at intensity 0 (its range 500, shadows on: the L key's
// other light) and an OVERCAST_DAY DirectionalLight turned by Quat::from_euler(ZYX, 0, PI/2, -PI/4) with the example's
// cascades (first far bound 7 m, maximum 25 m), the camera at (-5, 5, 5) looking at (-1, 1, 0), in Bevy's default
// 1280 x 720 window. The cascades and the shadow map's size go in a <Shadows> at Bevy's own values (raea's sun keeps a
// film's cascades otherwise).
import { Composition, registerRoot } from "raea";
import { Shadows } from "raea/3d";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Mesh3d, Plane3d, Sphere } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { DirectionalLight, PointLight, light_consts } from "raea/light";

function Shot() {
  return (
    <>
      <Shadows mapSize={2048} distance={25} firstCascade={7} cascades={4} overlap={0.2} />
      <Mesh3d transform={Transform.fromXyz(-1, 2, 0)}>
        <Sphere radius={0.25} />
        <StandardMaterial base_color="#ff0000" />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(1, 2, 0)} castShadow={false}>
        <Sphere radius={0.25} />
        <StandardMaterial base_color="#0000ff" />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(0, 1, -10)} castShadow={false} receiveShadow={false}>
        <Plane3d half_size={[10, 10]} />
        <StandardMaterial base_color="#00ff00" />
      </Mesh3d>
      <Mesh3d>
        <Plane3d half_size={[10, 10]} />
        <StandardMaterial base_color="#ffffff" perceptual_roughness={1} />
      </Mesh3d>
      <PointLight intensity={0} range={500} shadow_maps_enabled transform={Transform.fromXyz(5, 5, 0)} />
      <DirectionalLight illuminance={light_consts.lux.OVERCAST_DAY} shadow_maps_enabled transform={Transform.fromRotation(Quat.fromEuler("ZYX", 0, Math.PI / 2, -Math.PI / 4))} />
      <Camera3d transform={Transform.fromXyz(-5, 5, 5).lookingAt([-1, 1, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
