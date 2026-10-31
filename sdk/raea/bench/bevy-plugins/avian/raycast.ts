// avian3d from a TypeScript program (sdk/raea/docs/bevy-plugins.md): a headless App with the avian plugin, a floor
// and a ball spawned from the generated handles, 90 frames of physics, then a raycast through the call bridge
// (avian3d/cast_ray, a method in bevy_remote's table), the same through avian's own reflected RayCaster component,
// gravity read back as a resource, and the collision that started, observed. Prints one JSON line and fails on a
// wrong answer.
//   raea-bevy-sdk motion run sdk/raea/bench/bevy-plugins/avian/raycast.ts      (run.sh runs it)
import { BevyApp, at, with as w } from "raea/bevy";
import { print } from "raea/native";
import { avian } from "raea/plugins/avian";

function check(ok: boolean, what: string) {
  if (!ok) {
    // printed too: a thrown Error's stack in JavaScriptCore leaves its message out
    print(`FAIL avian raycast: ${what}`);
    throw new Error(`avian raycast: ${what}`);
  }
}

export default function main() {
  // a throw's message printed: JavaScriptCore's stack leaves it out
  try {
    run();
  } catch (e) {
    print(`FAIL: ${e instanceof Error ? e.message : String(e)}`);
    throw e;
  }
}

function run() {
  const app = BevyApp.open({ plugins: ["avian"], fps: 60 });
  check(app.methods.includes("avian3d/cast_ray"), `no avian3d/cast_ray method (has ${app.methods.join(", ")})`);
  app.setResource(avian.Gravity, [0, -9.81, 0]);
  const floor = app.spawn(
    w(avian.RigidBody, "Static"),
    w(avian.ColliderConstructor, { Cuboid: { x_length: 20, y_length: 1, z_length: 20 } }),
    at(0, -0.5, 0),
  );
  const ball = app.spawn(
    w(avian.RigidBody, "Dynamic"),
    w(avian.ColliderConstructor, { Sphere: { radius: 0.5 } }),
    at(0, 4, 0),
    w(avian.CollisionEventsEnabled, {}),
  );
  // a ray caster entity looking down from above the ball: avian fills its RayHits every step
  const caster = app.spawn(at(0, 10, 0), w(avian.RayCaster, { ...defaults(app), direction: [0, -1, 0] } as never));
  app.step(90);
  const rest = app.get(ball, avian.Position)!;
  check(Math.abs(rest[1] - 0.5) < 0.05, `the ball should rest on the floor at y 0.5, it's at ${rest[1]}`);
  const hit = app.call<{ entity: number; distance: number; normal: number[]; point: number[] } | null>("avian3d/cast_ray", { origin: [0, 10, 0], direction: [0, -1, 0] });
  check(hit !== null && hit.entity === ball, `the ray should hit the ball first, hit ${JSON.stringify(hit)}`);
  check(Math.abs(hit!.distance - 9) < 0.06, `the ball's top is 9 below the ray's origin, the hit is ${hit!.distance} away`);
  const past = app.call<{ entity: number } | null>("avian3d/cast_ray", { origin: [0, 10, 0], direction: [0, -1, 0], exclude: [ball] });
  check(past !== null && past.entity === floor, `with the ball excluded the ray should hit the floor, hit ${JSON.stringify(past)}`);
  const hits = app.get(caster, avian.RayHits);
  const collisions = app.call<{ a: number; b: number }[]>("avian3d/collisions");
  check(collisions.some((c) => [c.a, c.b].includes(ball) && [c.a, c.b].includes(floor)), `the ball's landing should be a collision, got ${JSON.stringify(collisions)}`);
  print(JSON.stringify({ proof: "avian", ball: rest, cast_ray: hit, excluded: past, ray_caster_hits: hits, collisions, gravity: app.resource(avian.Gravity) }));
  app.close();
}

// RayCaster has no default in avian (its fields are all required), so the program gives them all
function defaults(_app: BevyApp) {
  return { enabled: true, origin: [0, 0, 0], global_origin: [0, 0, 0], global_direction: [0, -1, 0], max_distance: 100, max_hits: 4, solid: true, ignore_self: true, query_filter: { mask: 0xffffffff, excluded_entities: [] } };
}
