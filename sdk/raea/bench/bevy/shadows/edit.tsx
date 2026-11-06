// Shadows fixture: a colonnade along a 120 m street under a low sun that turns over the shot, a spot on the near
// column, and a <Shadows> that fits the sun's cascades to the street (sharp up close, still there at the far end),
// a 4096 map and a little more bias than Bevy's. The camera dollies along the columns, so their shadows cross from one
// cascade to the next.
import { AbsoluteFill, Composition, interpolate, registerRoot, useCurrentFrame } from "raea";
import { Shadows } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d, Plane3d, ConicalFrustum } from "raea/mesh";
import { AmbientLight, DirectionalLight, SpotLight } from "raea/light";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const FPS = 30;
const FRAMES = 60;

function Shot() {
  const f = useCurrentFrame();
  const sun = interpolate(f, [0, FRAMES - 1], [-0.6, 0.6]);
  const z = interpolate(f, [0, FRAMES - 1], [6, 2]);
  const columns = Array.from({ length: 24 }, (_, i) => -i * 5);
  return (
    <AbsoluteFill style={{ background: "#9fb4c8" }}>
      <>
        <Camera3d transform={Transform.fromXyz(-3.5, 1.7, z).lookingAt([1, 1.4, -20])} projection={{ Perspective: { fov: 0.785398163397448 } }} />
        <Shadows mapSize={4096} distance={90} firstCascade={6} overlap={0.3} bias={0.03} normalBias={2} />
        <AmbientLight color="#c8d8ff" brightness={100} />
        <DirectionalLight color="#ffe6c4" transform={Transform.fromXyz(40 * Math.sin(sun), 22, 12 * Math.cos(sun)).lookingAt([0, 0, -20])} illuminance={8800} shadow_maps_enabled />
        <SpotLight color="#ffb070" transform={Transform.fromXyz(-2, 4, 2).lookingAt([1.5, 0.5, 0])} intensity={3600000} outer_angle={0.5} inner_angle={0.3} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, 0, -55).withRotation(Quat.fromEuler("XYZ", -Math.PI / 2, 0, 0))} name="street">
          <Plane3d normal={[0, 0, 1]} half_size={[15, 65]} />
          <StandardMaterial base_color="#b9a888" perceptual_roughness={0.9} />
        </Mesh3d>
        {columns.map((cz) => (
          <Mesh3d key={cz} transform={Transform.fromXyz(1.5, 2, cz)}>
            <ConicalFrustum radius_top={0.35} radius_bottom={0.4} height={4} />
            <StandardMaterial base_color="#e6dccb" perceptual_roughness={0.7} />
          </Mesh3d>
        ))}
        <Mesh3d transform={Transform.fromXyz(1.5, 4.2, -57.5)} name="entablature">
          <Cuboid half_size={[0.6, 0.2, 58.5]} />
          <StandardMaterial base_color="#d8ccb8" perceptual_roughness={0.75} />
        </Mesh3d>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="shadows" component={Shot} width={960} height={540} fps={FPS} durationInFrames={FRAMES} />);
