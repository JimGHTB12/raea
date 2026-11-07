// raea run bench.ts: every object through its category's checks; red-team objects must fail theirs.
import { print } from "raea/native";
import * as C from "./checks.ts";
import { objects, type Obj } from "./objects.ts";
import { claims, toWorld } from "./world.ts";

const SIZE: Record<Obj["kind"], { h?: C.Range; w?: C.Range }> = {
  chair: { h: [0.75, 1.1], w: [0.38, 0.75] },
  stool: { h: [0.4, 0.85], w: [0.25, 0.6] },
  mug: { h: [0.07, 0.13], w: [0.07, 0.15] },
  tree: { h: [5, 25], w: [3, 25] },
};

function run(o: Obj): C.Result[] {
  const S = C.sample(o.solid);
  const r = [C.connected(o.solid), C.grounded(S), C.scale(S, SIZE[o.kind])];
  if (o.kind === "chair" || o.kind === "stool") r.push(C.stands(S), C.seat(S, o.kind === "stool" ? [0.4, 0.8] : [0.38, 0.55], o.kind === "stool" ? 0.25 : 0.3));
  if (o.kind === "mug") r.push(C.stands(S), C.holdsWater(o.solid, S));
  if (o.kind === "tree") r.push(C.treeForm(o.solid, S));
  if (o.asm?.joins.length) r.push(C.joins(o.asm.parts, o.asm.joins));
  if (o.asm) r.push(...claims(o.asm));
  return r;
}

export default function main() {
  let wrong = 0;
  for (const o of objects()) {
    const t0 = Date.now();
    const rs = run(o);
    const ms = Date.now() - t0;
    print(`\n${o.name}${o.redTeam ? `   [red team: must fail "${o.redTeam}"]` : ""}  (${ms} ms)`);
    for (const r of rs) print(`  ${r.ok ? "pass" : "FAIL"}  ${r.check.padEnd(30)} ${r.got}`);
    if (o.redTeam) {
      const target = rs.find((r) => r.check === o.redTeam);
      if (!target || target.ok) { wrong++; print(`  !! red team not caught`); }
    } else if (o.arm === "A") { if (rs.some((r) => !r.ok)) print(`  (arm A sample: its failures are data, not a bench error)`); }
    else if (rs.some((r) => !r.ok)) { wrong++; print(`  !! a good object failed`); }
  }
  // the labels in the world: one ECS world holding every good B object, queried as a game would
  const objs = objects().filter((o) => o.asm && !o.redTeam);
  let world: any = undefined;
  for (const o of objs) world = toWorld(o.asm!, world).world;
  const names = (ids: number[]) => ids.map((id) => {
    const n = (world.component(id, "Name") as any)?.name, p = world.parent(id);
    return `${(world.component(p, "Name") as any)?.name}/${n}`;
  }).join(", ");
  print(`\nworld: ${world.find({ with: ["Name"] }).length} entities from ${objs.length} objects`);
  print(`  find Sittable:            ${names(world.find({ with: ["Sittable"] }))}`);
  print(`  find Graspable:           ${names(world.find({ with: ["Graspable"] }))}`);
  print(`  find Container:           ${names(world.find({ with: ["Container"] }))}`);
  print(`  find Part.material oak:   ${names(world.find({ with: ["Part"], where: [["Part.material", "==", "oak"]] }))}`);
  print(`  find Sittable, Place.y>0.5: ${names(world.find({ with: ["Sittable", "Place"], where: [["Place.y", ">", 0.5]] }))}`);
  print(`\n${wrong === 0 ? "bench ok" : `bench: ${wrong} wrong`}`);
}
