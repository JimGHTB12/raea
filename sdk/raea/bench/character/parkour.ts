// The parkour course's levels (examples/character/parkour-levels.ts), each run and checked strictly against its own
// geometry: body through obstacles, feet sinking or sliding, falls, floating, pops, things dropped or thrown wide, and
// the library's own limits on every joint. Exits 1 if any level fails.
//   bun bench/character/parkour.ts [LEVEL...] [--body kid|man|woman|all] [--json OUT.json]
import { writeFileSync } from "node:fs";
import { checkCourse, courseReport, layCourse } from "raea/course";
import { interact } from "raea/items";
import { perform } from "raea/matching";
import { BODIES, parkour, type Body } from "../../../../examples/character/parkour-levels";

const args = process.argv.slice(2), jsonAt = args.indexOf("--json"), bodyAt = args.indexOf("--body");
const out = jsonAt >= 0 ? args[jsonAt + 1] : undefined, bodyArg = bodyAt >= 0 ? args[bodyAt + 1] : "kid";
const pick = args.filter((a, i) => !a.startsWith("--") && !(jsonAt >= 0 && i === jsonAt + 1) && !(bodyAt >= 0 && i === bodyAt + 1));
const bodies: Body[] = bodyArg === "all" ? BODIES : [bodyArg as Body];
const results = [];
let failed = 0, total = 0;
for (const body of bodies) {
const { skel: kid, lib, levels: all } = parkour(body);
const levels = pick.length ? all.filter((l) => pick.some((p) => l.name.startsWith(p))) : all;
if (bodies.length > 1 || body !== "kid") console.log(`== ${body}`);
total += levels.length;
for (const spec of levels) {
  const t0 = performance.now();
  const course = layCourse(lib, kid, spec);
  const perf = perform(lib, course.plan.spec, { trace: true, quiet: true });
  const live = interact(kid, perf, course.plan);
  const check = checkCourse(kid, course, perf, live, lib);
  console.log(courseReport(check) + `\n  (${((performance.now() - t0) / 1000).toFixed(1)} s)\n`);
  if (!check.pass) failed++;
  results.push({ body, name: check.name, pass: check.pass, frames: check.frames, finished: check.finished, obstacles: check.obstacles, flags: check.flags });
}
}
console.log(`${total - failed}/${total} levels pass`);
if (out) writeFileSync(out, JSON.stringify(results, null, 1));
process.exit(failed ? 1 : 0);
