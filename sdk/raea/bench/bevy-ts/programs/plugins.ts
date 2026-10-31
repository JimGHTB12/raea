// gap: S6c (docs/sdk.feature)
// Plugins in TypeScript as bevy_app has them, on the engine: a headless App opened with a Rust plugin by name and a
// TypeScript plugin together. The TS plugin's Startup system spawns an entity once, before the first frame; its Update
// system moves every Transform 1 along x; a Last system reads the GlobalTransform Bevy's PostUpdate propagated. After
// 3 frames x is 3 and Startup ran once. A Rust plugin named after the App opened is refused, and the group's disabled
// member stays out. Prints one JSON line; a wrong answer prints FAIL and throws.
//   sdk/raea/bench/bevy-plugins/motion.sh run sdk/raea/bench/bevy-ts/programs/plugins.ts
import { BevyApp, PluginGroup, Startup, Transform, Update, Last, at, definePlugin } from "raea/bevy";
import { print } from "raea/native";

const GT = "bevy_transform::components::global_transform::GlobalTransform";

function check(ok: boolean, what: string) {
  if (!ok) {
    print(`FAIL plugins: ${what}`);
    throw new Error(`plugins: ${what}`);
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
  let startups = 0;
  let entity = -1;
  const seen: number[] = [];
  const mover = definePlugin({
    name: "mover",
    build(app) {
      app.addSystems(Startup, (a) => {
        startups++;
        entity = a.spawn(at(0, 0, 0));
      });
      app.addSystems(Update, (a) => {
        for (const row of a.query({ data: [Transform] })) {
          const t = row.components[Transform.path] as { translation: [number, number, number] };
          a.call("world.mutate_components", { entity: row.entity, component: Transform.path, path: "translation.x", value: t.translation[0] + 1 });
        }
      });
      app.addSystems(Last, (a) => {
        const g = a.call<{ components: Record<string, number[]> }>("world.get_components", { entity, components: [GT] }).components[GT];
        seen.push(g[g.length - 3]);
      });
    },
  });
  const never = definePlugin({ name: "never", build: () => check(false, "a disabled plugin was built") });
  const app = BevyApp.open({ plugins: [new PluginGroup("Game", ["tweening", mover, never]).disable("never")], fps: 60 });
  check(app.isPluginAdded("tweening") && app.isPluginAdded("mover") && !app.isPluginAdded("never"), "the plugins the App says it has");
  check(startups === 0, `Startup ran ${startups} times before the first frame was asked for`);
  app.step(3);
  check(startups === 1, `Startup ran ${startups} times`);
  check(JSON.stringify(seen) === "[1,2,3]", `GlobalTransform x by frame: ${JSON.stringify(seen)}, wants [1,2,3]`);
  let threw = "";
  try {
    app.addPlugins("avian");
  } catch (err) {
    threw = err instanceof Error ? err.message : String(err);
  }
  check(threw.includes("name it when the App opens"), `a Rust plugin after the App opened: ${threw || "no error"}`);
  app.close();
  print(JSON.stringify({ ok: true, startups, seen }));
}
