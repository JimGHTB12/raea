// Bevy v0.20.0 examples/3d/two_passes.rs in raea's TSX (docs/control.feature FC0a, FC2): one scene drawn by two
// cameras into the same window. A 5 m plane of srgb(0.3, 0.5, 0.3), a 1 m cube of srgb(0.8, 0.7, 0.6) on it and a
// shadow-casting PointLight at (4, 8, 4); the first camera at (-2, 2.5, 5) and a second, order 1 with
// ClearColorConfig::None, at (10, 10, -5), drawn over the first without clearing it, both looking at the origin. In
// Bevy's default 1280 x 720 window.
import { Composition, registerRoot } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { PointLight } from "raea/light";

function Shot() {
  return (
    <>
      <Mesh3d>
        <Plane3d half_size={[2.5, 2.5]} />
        <StandardMaterial base_color="rgb(76.5, 127.5, 76.5)" />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(0, 0.5, 0)}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <StandardMaterial base_color="rgb(204, 178.5, 153)" />
      </Mesh3d>
      <PointLight shadow_maps_enabled transform={Transform.fromXyz(4, 8, 4)} />
      <Camera3d transform={Transform.fromXyz(-2, 2.5, 5).lookingAt([0, 0, 0])} />
      <Camera3d order={1} clear_color="None" transform={Transform.fromXyz(10, 10, -5).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
