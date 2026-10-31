// gap: G16
// Component hooks and observers inside the App (BevyApp.hook, raea/observe in engine/bevy-world, G16b): an Add hook on
// Name gives each named entity a Transform at x 9 in the same flush as its spawn, so the very next read sees it, with
// no frame stepped; a Remove hook records the entity; after stop() a new name gets nothing. Prints one JSON line; a
// wrong answer prints FAIL and throws.
//   sdk/raea/bench/bevy-plugins/motion.sh run sdk/raea/bench/bevy-ts/programs/hooks.ts
import { BevyApp, Transform } from "raea/bevy";
import { print } from "raea/native";

const NAME = "bevy_ecs::name::Name";

function check(ok: boolean, what: string) {
  if (!ok) {
    print(`FAIL hooks: ${what}`);
    throw new Error(`hooks: ${what}`);
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
  const added = app.hook("Add", NAME, [{ method: "world.insert_components", params: { entity: "$entity", components: { [Transform.path]: { translation: [9, 0, 0], rotation: [0, 0, 0, 1], scale: [1, 1, 1] } } } }]);
  const removed = app.hook("Remove", NAME);
  const e = app.call<{ entity: number }>("world.spawn_entity", { components: { [NAME]: "kid" } }).entity;
  const x = app.get(e, Transform)?.translation?.[0];
  check(x === 9, `the Add hook's Transform: x ${x}, wants 9 (before any frame)`);
  const seen = added.seen();
  check(seen.length === 1 && seen[0].entity === e, `the Add hook saw ${JSON.stringify(seen)}`);
  app.call("world.remove_components", { entity: e, components: [NAME] });
  check(removed.seen().map((s) => s.entity).join() === String(e), "the Remove hook didn't see the removal");
  added.stop();
  const e2 = app.call<{ entity: number }>("world.spawn_entity", { components: { [NAME]: "late" } }).entity;
  check(app.get(e2, Transform) === undefined, "a stopped hook still ran");
  print(JSON.stringify({ ok: true, x, added: seen.length }));
  app.close();
}
