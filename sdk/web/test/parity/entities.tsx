// raea-web parity: an ECS world drawn by the engine with no React element per entity (<Entities>), as conformance
// golden 06: a ship (rounded box) and its gun (a cylinder, its child), turned by a system each frame.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { Canvas3D, Entities } from "raea/3d";
import { Camera3d } from "raea/camera";
import { DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { createWorld } from "raea/ecs";

const game = createWorld();
const ship = game.add({ Mesh: { shape: "rounded_box(2, 0.6, 1, 0.1)", color: "#2563eb", roughness: 0.5, metalness: 0 }, Transform: { x: 0, y: 0, z: 0, ry: 0 } });
game.add({ Mesh: { shape: "cylinder(0.15, 1.2)", color: "#f59e0b", roughness: 0.4, metalness: 0 }, Transform: { x: 0, y: 0.6, z: 0 } }, { parent: ship });
game.system((w, f) => w.insert(ship, { Transform: { x: 0, y: 0, z: 0, ry: f * 0.05 } }));

function Ships() {
  const frame = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#0b1020" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 1.5, 6).lookingAt([0, 0, 0])} />
        <DirectionalLight illuminance={8000} transform={Transform.fromXyz(2, 3, 4).lookingAt([0, 0, 0])} />
        <Entities world={game.at(frame)} />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Ships} durationInFrames={30} fps={30} width={320} height={180} />);
