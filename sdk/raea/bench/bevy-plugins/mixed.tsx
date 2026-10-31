// S2d's done line (docs/sdk.feature): one query mixes a component declared in TypeScript and one generated from a Rust
// plugin, here on the engine's World (Bevy's, through ecs.rs; conformance/data.test.ts runs it under bun). The query's
// results set the bars' widths, which run.sh reads back: Position.x after += LinearVelocity[0] (1, so 100 px), and
// LinearVelocity[2] after a write query (9, so 90 px); the bars are green only when the World is the engine's.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { component, entity, f32, has, read, typed, write } from "@raea/app/ecs";
import { avian } from "raea/plugins/avian";

const Position = component("Position", { x: f32, y: f32 });

function measure() {
  const w = typed({ native: true });
  const a = w.spawn(Position.of({ x: 0, y: 0 }), [avian.LinearVelocity, [1, 2, 3]]);
  w.spawn(Position.of({ x: 5, y: 5 }));
  for (const [, p, v] of w.query(entity, write(Position), read(avian.LinearVelocity))) p.x += v[0];
  for (const [v] of w.query(write(avian.LinearVelocity), has(Position))) v[2] = 9;
  return { x: w.get(a, Position)?.x ?? 0, z: w.get(a, avian.LinearVelocity)?.[2] ?? 0, native: w.motion.native };
}

function Shot() {
  const m = measure();
  const color = m.native ? "#00ff00" : "#ff0000";
  return (
    <AbsoluteFill style={{ background: "#000000" }}>
      <div style={{ position: "absolute", left: 0, top: 20, width: m.x * 100, height: 20, background: color }} />
      <div style={{ position: "absolute", left: 0, top: 60, width: m.z * 10, height: 20, background: color }} />
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="mixed" component={Shot} width={200} height={100} fps={30} durationInFrames={1} />);
