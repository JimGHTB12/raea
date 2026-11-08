// gap: G16
// In a render (not live play): the live-world case rendered, its calls queued at load and answered by the Engine before the frame is read back; TypeScript reaching the Bevy World its scene draws in (raea/world, the layer BevyApp and
// void's spaces share): queued at load, a read of the mesh "cube"'s Transform (by its SDK name) and a write moving
// the bare entity "marker" through world.mutate_components then its read back, answered from the Engine's next update.
// Left half green once the read says the cube is at x 1; right half green once the marker reads x 4.
// frame: 2
// probe: 16 8 g>150 r<100
// probe: 48 8 g>150 r<100
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, Entity } from "raea/3d";
import { useWorldAnswer, world } from "raea/world";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const T = "bevy_transform::components::transform::Transform";
const read = world.send("world.get_components", { entity: "cube", components: [T] });
world.send("world.mutate_components", { entity: "marker", component: T, path: "translation.x", value: 4 });
const back = world.send("world.get_components", { entity: "marker", components: [T] });
type Got = { components: Record<string, { translation: [number, number, number] }> };

function Shot() {
  const a = useWorldAnswer<Got>(read)?.value?.components[T]?.translation[0];
  const b = useWorldAnswer<Got>(back)?.value?.components[T]?.translation[0];
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <Mesh3d transform={Transform.fromXyz(1, 0, 0)} name="cube"><Cuboid half_size={[0.25, 0.25, 0.25]} /><StandardMaterial base_color="#0000ff" unlit /></Mesh3d>
        <Entity name="marker" />
      </Canvas3D>
      <div style={{ position: "absolute", left: 0, top: 0, width: 32, height: 16, background: a === 1 ? "#00ff00" : "#ff0000" }} />
      <div style={{ position: "absolute", left: 32, top: 0, width: 32, height: 16, background: b === 4 ? "#00ff00" : "#ff0000" }} />
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={300} fps={30} width={64} height={64} />);
