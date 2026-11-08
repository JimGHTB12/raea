// gap: G17
// Handles through world calls, world save/load and diagnostics (raea-render's host methods in engine/bevy-world's Methods
// table, G17b and G18), from a render's TypeScript (raea/world):
//   middle     world.spawn_entity with a Mesh3d as "shape:box(...)" and an inline StandardMaterial: plain BRP can't read a
//              handle field; raea-render's Fields processor builds both, so a red box appears
//   top left   raea/world.save of the named entities: green once Bevy's DynamicWorld of them comes back as JSON
//              holding both <Entity> parts (the load of it is raea-render's world_save_and_load test: a render answers a call a
//              frame later, so a load chained on the save's answer lands after the frame is read)
//   top right  raea/diagnostics: green once Bevy's DiagnosticsStore lists fps and entity_count (their values come in
//              over the first frames of a live game)
// frame: 3
// probe: 32 40 r>150 g<80
// probe: 16 8 g>150 r<100
// probe: 48 8 g>150 r<100
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, Entity } from "raea/3d";
import { useWorldAnswer, world } from "raea/world";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

world.send("world.spawn_entity", {
  components: {
    "bevy_transform::components::transform::Transform": { translation: [0, -0.5, 0], rotation: [0, 0, 0, 1], scale: [1, 1, 1] },
    "bevy_mesh::components::Mesh3d": "shape:box(1.5, 1.5, 0.1)",
    "bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>": { base_color: { Srgba: { red: 1, green: 0, blue: 0, alpha: 1 } }, unlit: true },
  },
});
const saved = world.send("raea/world.save", {});
const diag = world.send("raea/diagnostics", {});

function Shot() {
  // DynamicWorldSerializer's JSON: { resources: {}, entities: { bits: { components } } }
  const s = useWorldAnswer<{ entities: Record<string, { components: Record<string, unknown> }> }>(saved)?.value;
  const names = Object.values(s?.entities ?? {}).map((e) => JSON.stringify(e.components["bevy_ecs::name::Name"] ?? ""));
  const okSave = names.some((n) => n.includes("marker")) && names.some((n) => n.includes("other"));
  const d = useWorldAnswer<Record<string, unknown>>(diag)?.value ?? {};
  const okDiag = "fps" in d && "entity_count" in d;
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <Entity name="marker" />
        <Entity name="other" position={[0, 9, 0]} />
      </Canvas3D>
      <div style={{ position: "absolute", left: 0, top: 0, width: 32, height: 16, background: okSave ? "#00ff00" : "#ff0000" }} />
      <div style={{ position: "absolute", left: 32, top: 0, width: 32, height: 16, background: okDiag ? "#00ff00" : "#ff0000" }} />
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={300} fps={30} width={64} height={64} />);
