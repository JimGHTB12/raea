// gap: P19
// Text in the 3D world, each glyph an entity read back over world.* (docs/ts-native.feature P19): "HI" as a <RichText
// name="hi">, its second glyph lifted 0.5 m and turned by its `glyph` function. In live play, from frame 3 on, the
// game asks Bevy for both glyphs' Transform and GlobalTransform by their names ("hi:0", "hi:1").
//   top left   green once hi:1's own Transform reads y 0.5 (its motion about its centre, from TypeScript) and hi:0's 0
//   top right  green once hi:1's GlobalTransform is right of hi:0's (the block's layout) and 0.5 m higher
//   bottom     the glyphs themselves, white and lit, over a black floor
// play: live-world.json 0.6
// probe: 16 8 g>150 r<100
// probe: 48 8 g>150 r<100
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { Canvas3D, RichText } from "raea/3d";
import { world } from "raea/world";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

const T = "bevy_transform::components::transform::Transform";
const G = "bevy_transform::components::global_transform::GlobalTransform";
type Got = { components: Record<string, unknown> };
let asks: number[] | null = null;

/** A Transform's or GlobalTransform's translation, however BRP writes it (a Transform's fields, an Affine3A's 12 numbers). */
function at(v: unknown): number[] | undefined {
  if (Array.isArray(v) && v.length === 12) return v.slice(9) as number[];
  const o = v as { translation?: number[] } | undefined;
  return o?.translation;
}

function Shot() {
  const f = useCurrentFrame();
  if (f >= 3 && !asks) asks = ["hi:0", "hi:1"].map((entity) => world.send("world.get_components", { entity, components: [T, G] }));
  const got = asks?.map((t) => world.answer<Got>(t)?.value?.components) ?? [];
  const local = got.map((c) => at(c?.[T]));
  const global = got.map((c) => at(c?.[G]));
  const okLocal = local[0]?.[1] === 0 && Math.abs((local[1]?.[1] ?? 0) - 0.5) < 1e-4;
  const okGlobal = !!global[0] && !!global[1] && global[1][0] > global[0][0] + 0.1 && Math.abs(global[1][1] - global[0][1] - 0.5) < 1e-3;
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <AmbientLight brightness={120} />
        <DirectionalLight transform={Transform.fromXyz(0, 2, 5).lookingAt([0, 0, 0])} illuminance={8000} shadow_maps_enabled />
        <RichText name="hi" position={[0, -0.6, 0]} size={1} glyph={(g) => (g.index === 1 ? { position: [0, 0.5, 0], rotation: [0, 0, 0.2] } : {})}>
          HI
        </RichText>
      </Canvas3D>
      <div style={{ position: "absolute", left: 0, top: 0, width: 32, height: 16, background: okLocal ? "#00ff00" : "#ff0000" }} />
      <div style={{ position: "absolute", left: 32, top: 0, width: 32, height: 16, background: okGlobal ? "#00ff00" : "#ff0000" }} />
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={300} fps={30} width={64} height={64} />);
