// gap: G1
// Bevy's own Projection component set on the camera through the TypeRegistry (<BevyComponent> inside the camera), in
// the default build: the same picture as the ortho case.
// probe: 52 32 r>200 g<60
// probe: 12 32 r>200 g<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 10).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }}>
          <BevyComponent type="bevy_camera::projection::Projection" value={{ Orthographic: { near: 0.1, far: 100, viewport_origin: [0.5, 0.5], scaling_mode: { FixedVertical: { viewport_height: 4 } }, scale: 1, area: { min: [-1, -1], max: [1, 1] } } }} />
        </Camera3d>
        <Mesh3d transform={Transform.fromXyz(-1, 0, 0)} name="near"><Cuboid half_size={[0.5, 0.5, 0.5]} /><StandardMaterial base_color="#ff0000" unlit /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(1, 0, -6)} name="far"><Cuboid half_size={[0.5, 0.5, 0.5]} /><StandardMaterial base_color="#ff0000" unlit /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
