// gap: FC1c
// A reflected value keyed per frame in the plain build (docs/control.feature FC1c): a bare <Entity> whose
// MeshMaterial3d<StandardMaterial>, given as a <BevyComponent>, turns from red to green at frame 10. At frame 15 it is
// green (live: played live, one World across the change).
// play: live-world.json 0.5
// probe: 32 32 g>150 r<100
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";

const rgb = (r: number, g: number) => ({ Srgba: { red: r, green: g, blue: 0, alpha: 1 } });

function Cube() {
  const f = useCurrentFrame();
  return (
    <Entity name="cube">
      <BevyComponent type="bevy_mesh::components::Mesh3d" value="shape:box(1.2, 1.2, 1.2)" />
      <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={{ base_color: f < 10 ? rgb(1, 0) : rgb(0, 1), unlit: true }} />
    </Entity>
  );
}

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} />
        <Cube />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={30} fps={30} width={64} height={64} />);
