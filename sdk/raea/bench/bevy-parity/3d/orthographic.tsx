// Bevy v0.20.0 examples/3d/orthographic.rs in raea's TSX (docs/control.feature FC0a): an orthographic camera 6 m tall
// (ScalingMode::FixedVertical) at (5, 5, 5) looking at the origin, a 5 m green plane, four 1 m cubes of
// srgb(0.8, 0.7, 0.6) at (+-1.5, 0.5, +-1.5) and a default PointLight at (3, 8, 5), in Bevy's default 1280 x 720 window.
import { Composition, registerRoot } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { PointLight } from "raea/light";

const CUBES: [number, number][] = [[1.5, 1.5], [1.5, -1.5], [-1.5, 1.5], [-1.5, -1.5]];

function Shot() {
  return (
    <>
      <Camera3d projection={{ Orthographic: { scaling_mode: { FixedVertical: { viewport_height: 6 } } } }} transform={Transform.fromXyz(5, 5, 5).lookingAt([0, 0, 0])} />
      <Mesh3d transform={Transform.IDENTITY}>
        <Plane3d half_size={[2.5, 2.5]} />
        <StandardMaterial base_color="rgb(76.5, 127.5, 76.5)" />
      </Mesh3d>
      {CUBES.map(([x, z], i) => (
        <Mesh3d key={i} transform={Transform.fromXyz(x, 0.5, z)}>
          <Cuboid half_size={[0.5, 0.5, 0.5]} />
          <StandardMaterial base_color="rgb(204, 178.5, 153)" />
        </Mesh3d>
      ))}
      <PointLight transform={Transform.fromXyz(3, 8, 5)} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
