// gap: G15
// A live game's parts of the kinds outside the five diffed groups, changing mid-game: a bare <Entity> (its Mesh3d and
// material by <BevyComponent>) moved from the left to the right at frame 4, and a <RichText> block that only appears
// from frame 6 (spawned mid-play with its glyph rows, under it). Before G15 a bare entity was placed once at open and a
// kind first seen mid-game was never spawned.
// play: live-world.json 0.6
// probe: 48 20 g>150 r<100
// probe: 16 20 r<40 g<40 b<40
// probe: 32 48 r>120 g>120 b>120
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { BevyComponent, Canvas3D, Entity, RichText } from "raea/3d";
import { AmbientLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <AmbientLight brightness={1200} />
        <Entity name="card" position={[f < 4 ? -1 : 1, 0.6, 0]}>
          <BevyComponent type="bevy_mesh::components::Mesh3d" value="shape:box(1, 1, 0.1)" />
          <BevyComponent type="bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>" value={{ base_color: { Srgba: { red: 0, green: 1, blue: 0, alpha: 1 } }, unlit: true }} />
        </Entity>
        {f >= 6 && <RichText name="late" position={[0, -0.9, 0]} size={1.4} weight={900} unlit>I</RichText>}
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={300} fps={30} width={64} height={64} />);
