// gap: G15
// A live game's <BevyComponent> changing mid-game (LiveScene diffs component rows; Engine::set_rows applies them
// through the registry): a bare entity's material, red on the first frames, green from frame 4 on. A render sets a
// component once, at load; live play now keeps it as the game's frames give it.
// play: live-world.json 0.4
// probe: 32 32 g>150 r<100
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  const f = useCurrentFrame();
  const c = f < 4 ? { red: 1, green: 0, blue: 0, alpha: 1 } : { red: 0, green: 1, blue: 0, alpha: 1 };
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <Entity name="card">
          <BevyComponent type="bevy_mesh::components::Mesh3d" value="shape:box(3, 3, 0.1)" />
          <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={{ base_color: { Srgba: c }, unlit: true }} />
        </Entity>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={300} fps={30} width={64} height={64} />);
