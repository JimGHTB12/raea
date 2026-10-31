// gap: G7
// Two cameras side by side, through Bevy's own Camera: the <PerspectiveCamera> given a viewport on the left half (its
// Camera patched, the engine's clear colour and target kept), a second camera on a bare <Entity> (Camera3d + Camera,
// order 1, no clear) on the right half, drawing into the same frame. Left looks at a red cube, right at a blue one 10 m away.
// probe: 16 32 r>200 b<60
// probe: 48 32 b>200 r<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const half = (x: number) => ({ physical_position: [x, 0], physical_size: [32, 64], depth: { start: 0, end: 1 } });

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(-5, 0, 4).lookingAt([-5, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }}>
          <BevyComponent type="bevy_camera::camera::Camera" value={{ viewport: half(0) }} />
        </Camera3d>
        <Entity name="right" position={[5, 0, 4]}>
          <BevyComponent type="bevy_camera::components::Camera3d" value={{}} />
          <BevyComponent type="bevy_camera::camera::Camera" value={{ viewport: half(32), order: 1, clear_color: "None" }} />
        </Entity>
        <Mesh3d transform={Transform.fromXyz(-5, 0, 0)} name="red"><Cuboid half_size={[0.75, 0.75, 0.75]} /><StandardMaterial base_color="#ff0000" unlit /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(5, 0, 0)} name="blue"><Cuboid half_size={[0.75, 0.75, 0.75]} /><StandardMaterial base_color="#0000ff" unlit /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
