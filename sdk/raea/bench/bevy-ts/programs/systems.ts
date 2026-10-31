// gap: G16
// TypeScript systems in Bevy's frame (BevyApp.system, docs/bevy-ts.feature G16): a headless App with a Transform; an
// Update system written in TypeScript moves every Transform 1 along x (a query, then a reflect-path mutate), and a Last
// system reads the GlobalTransform Bevy's own PostUpdate propagated from it in the same frame, so the TS change is in
// the World before the next schedule runs. After 5 frames x is 5 in both; removed, the system stops moving it.
// Prints one JSON line; a wrong answer prints FAIL and throws.
//   sdk/raea/bench/bevy-plugins/motion.sh run sdk/raea/bench/bevy-ts/programs/systems.ts
import { BevyApp, Transform, at } from "raea/bevy";
import { print } from "raea/native";

const GT = "bevy_transform::components::global_transform::GlobalTransform";

function check(ok: boolean, what: string) {
  if (!ok) {
    print(`FAIL systems: ${what}`);
    throw new Error(`systems: ${what}`);
  }
}

export default function main() {
  try {
    run();
  } catch (e) {
    print(`FAIL: ${e instanceof Error ? e.message : String(e)}`);
    throw e;
  }
}

function run() {
  const app = BevyApp.open({ plugins: ["tweening"], fps: 60 });
  const schedules = app.frameSchedules();
  check(schedules.includes("Update") && schedules.indexOf("Update") < schedules.indexOf("PostUpdate"), `the frame's schedules: ${schedules.join(", ")}`);
  const e = app.spawn(at(0, 0, 0));
  const seen: number[] = [];
  const stop = app.system("Update", (a) => {
    for (const row of a.query({ data: [Transform] })) {
      const t = row.components[Transform.path] as { translation: [number, number, number] };
      a.call("world.mutate_components", { entity: row.entity, component: Transform.path, path: "translation.x", value: t.translation[0] + 1 });
    }
  });
  app.system("Last", (a, frame) => {
    const g = a.call<{ components: Record<string, number[]> }>("world.get_components", { entity: e, components: [GT] }).components[GT];
    // GlobalTransform is an Affine3A: its last three numbers are the translation
    seen.push(g[g.length - 3]);
    const want = Math.min(frame, 5);
    check(g[g.length - 3] === want, `frame ${frame}: GlobalTransform x ${g[g.length - 3]}, wants ${want} (the Update system's moves, propagated in PostUpdate)`);
  });
  const frame = app.step(5);
  check(frame === 5, `stepped to ${frame}`);
  stop();
  app.step(2);
  const x = app.get(e, Transform)?.translation?.[0];
  check(x === 5, `after the system was removed x is ${x}, wants 5`);
  let threw = "";
  try {
    app.system("Nowhere", () => {});
  } catch (err) {
    threw = err instanceof Error ? err.message : String(err);
  }
  check(threw.includes("no schedule"), `an unknown schedule: ${threw || "no error"}`);
  app.close();

  // FixedUpdate placement: a fixed system moves y 1 a step; an Update system in the same frame already sees it
  const fx = BevyApp.open({ plugins: ["tweening"], fps: 60 });
  const b = fx.spawn(at(0, 0, 0));
  const ys: number[] = [];
  fx.system("FixedUpdate", (a) => {
    const t = a.get(b, Transform)!;
    a.call("world.mutate_components", { entity: b, component: Transform.path, path: "translation.y", value: (t.translation?.[1] ?? 0) + 1 });
  });
  fx.system("Update", (a, frame) => {
    const y = a.get(b, Transform)?.translation?.[1];
    ys.push(y ?? -1);
    check(y === frame, `frame ${frame}: Update sees y ${y}, wants ${frame} (one fixed step a frame, before Update)`);
  });
  fx.step(4);
  fx.close();
  print(JSON.stringify({ ok: true, seen, x, fixed: ys }));
}
