// gap: G3b
// A live gamepad through Bevy's own input (bevy_gilrs in raea-render's default build): play reads every Gamepad entity the
// 3D's Bevy App has (gilrs makes them from the real pads) and marks their buttons and axes on the live wire
// ("Pad0:South"), which useInput().gamepads reads. No pad is plugged into a test machine, so this case makes one the
// way gilrs does: Bevy's RawGamepadEvent messages (a connection, which gilrs also writes as a GamepadConnectionEvent,
// the one that makes the Gamepad component; then South pressed and the left stick pushed),
// written through world.*. Everything after gilrs's read is the real path: bevy_input's gamepad systems, the Gamepad
// component, play's marking, the wire, the game's fold. Green once the game has seen South held and the stick right.
// play: live-world.json 0.8
// probe: 32 32 g>150 r<100
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame, useGameState } from "raea";
import { Canvas3D } from "raea/3d";
import { world } from "raea/world";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const RAW = "bevy_input::gamepad::RawGamepadEvent";
const spawned = world.send("world.spawn_entity", { components: { "bevy_ecs::name::Name": "virtual pad" } });
let sent = 0;

function Shot() {
  const f = useCurrentFrame();
  const pad = world.answer<{ entity: number }>(spawned)?.value?.entity;
  if (pad !== undefined && sent === 0 && f >= 2) {
    const connection = { gamepad: pad, connection: { Connected: { name: "virtual pad", vendor_id: null, product_id: null } } };
    world.send("world.write_message", { message: RAW, value: { Connection: connection } });
    world.send("world.write_message", { message: "bevy_input::gamepad::GamepadConnectionEvent", value: connection });
    sent = 1;
  }
  if (pad !== undefined && sent === 1 && f >= 5) {
    world.send("world.write_message", { message: RAW, value: { Button: { gamepad: pad, button: "South", value: 1 } } });
    world.send("world.write_message", { message: RAW, value: { Axis: { gamepad: pad, axis: "LeftStickX", value: 0.8 } } });
    sent = 2;
  }
  const s = useGameState({ south: false, right: false }, (s, input) => {
    const p = input.gamepads?.[0];
    return { south: s.south || (p?.buttons ?? []).includes("South"), right: s.right || (p?.axes.LeftStickX ?? 0) > 0.5 };
  });
  const ok = s.south && s.right;
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.872664625997165 } }} />
        <Mesh3d name="lamp"><Cuboid half_size={[1.5, 1.5, 0.05]} /><StandardMaterial base_color={ok ? "#00ff00" : "#ff0000"} unlit /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={300} fps={30} width={64} height={64} />);
