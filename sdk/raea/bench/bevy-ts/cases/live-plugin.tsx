// gap: G16
// A live game calling a compiled-in plugin's method on the World its scene draws in: from frame 10 (once avian has
// made the floor's collider) the game queues avian3d/cast_ray straight down from 5 m through raea/world, and
// paints the frame green when the hit is the floor's top, 4.85 m down. Plugin build only.
// build: plugins
// play: live-world.json 0.8
// probe: 32 8 g>150 r<100
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { BevyComponent, Canvas3D } from "raea/3d";
import { world } from "raea/world";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

let ticket = 0;

function Shot() {
  const f = useCurrentFrame();
  if (f >= 10 && ticket === 0) ticket = world.send("avian3d/cast_ray", { origin: [0, 5, 0], direction: [0, -1, 0] });
  const d = ticket ? world.answer<{ distance: number } | null>(ticket)?.value?.distance : undefined;
  const hit = d !== undefined && Math.abs(d - 4.85) < 0.05;
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 2, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <Mesh3d name="floor">
          <Cuboid half_size={[2, 0.15, 2]} />
          <StandardMaterial base_color="#333333" unlit />
          <BevyComponent type="avian3d::dynamics::rigid_body::RigidBody" value="Static" />
          <BevyComponent type="avian3d::collision::collider::constructor::ColliderConstructor" value={{ Cuboid: { x_length: 4, y_length: 0.3, z_length: 4 } }} />
        </Mesh3d>
      </Canvas3D>
      <div style={{ position: "absolute", left: 0, top: 0, width: 64, height: 16, background: hit ? "#00ff00" : "#ff0000" }} />
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={300} fps={30} width={64} height={64} />);
