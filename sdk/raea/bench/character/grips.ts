// The grips (examples/character/grip-scenes.ts): each thing taken, carried and put down, judged by the skin, how deep
// a hand's skin (palm, thumb and fingers) and the body's (trunk, head, arms, thighs) go into the thing while it is
// held, through the pick, the carry (the walk) and the put-down. A thing "clips" on a frame where a hand or the body is
// more than `--tol` cm into it (0.5 by default); a scene fails on more than `--frames` such frames (10). Each scene
// also reports its closure: the share of held frames on which the skin touching the thing (the hands', the arms' and
// the trunk's, within 3 mm) could hold it up against gravity with friction 0.5 (grasp.ts's holds), reported, not judged.
//   bun bench/character/grips.ts [SCENE...] [--body kid|man|woman|all] [--tol CM] [--frames N] [--lib parkour|db]
// --lib db performs over the body's whole motion database (~/.cache/raea-mocap/lib/BODY/motion.db, docs/clips.feature)
import { add, pose, qinv, qrot, surface, type Vec3 } from "raea/character";
import { contactsOf, holds } from "raea/grasp";
import { interact, thingSdf } from "raea/items";
import { perform } from "raea/matching";
import { SCENES, gripRun } from "../../../../examples/character/grip-scenes";
import { BODIES, parkour, type Body } from "../../../../examples/character/parkour-levels";
import { dbLibraryAt } from "@raea/character/clips-bun";

const args = process.argv.slice(2), opt = (k: string, d: string) => { const i = args.indexOf(k); return i >= 0 ? args[i + 1] : d; };
const bodyArg = opt("--body", "kid"), tol = Number(opt("--tol", "0.5")) / 100, most = Number(opt("--frames", "10"));
const libArg = opt("--lib", "parkour");
const pick = args.filter((a, i) => !a.startsWith("--") && !args[i - 1]?.startsWith("--"));
const bodies: Body[] = bodyArg === "all" ? BODIES : [bodyArg as Body];
const HAND = /^(palm|thumb|index|middle|ring|pinky)/;
let failed = 0, total = 0;
for (const body of bodies) {
  const { skel, lib: own } = parkour(body);
  const lib = libArg === "db" ? dbLibraryAt(skel, `${process.env.HOME}/.cache/raea-mocap/lib/${body}`) : own;
  if (bodies.length > 1 || body !== "kid") console.log(`== ${body}`);
  for (const sc of SCENES.filter((s) => !pick.length || pick.includes(s.name))) {
    total++;
    const t0 = performance.now();
    const r = gripRun(body, sc, lib), perf = perform(lib, r.plan.spec, { quiet: true }), live = interact(skel, perf, r.plan), sdf = thingSdf(r.plan.items.it);
    // the worst of each, by step (a pick's or a put-down's wait, the carry's walk)
    const worst: Record<string, { hand: [number, number, string]; body: [number, number, string] }> = {};
    let held = 0, clips = 0, held3 = 0;
    for (let f = 0; f < live.frames; f++) {
      if (!live.held("it", f)) continue;
      held++;
      const t = live.item("it", f), field = (q: Vec3) => sdf(qrot(qinv(t.rot), [q[0] - t.at[0], q[1] - t.at[1], q[2] - t.at[2]]));
      const step = perf.step(f)?.kind ?? "end", w = (worst[step] ??= { hand: [0, -1, ""], body: [0, -1, ""] });
      let deep = 0;
      for (const x of surface(skel, pose(skel, live.pose(f)))) {
        const d = -field(x.at), k = HAND.test(x.part) ? "hand" : "body";
        deep = Math.max(deep, d);
        if (d > w[k][0]) w[k] = [d, f, `${x.side ?? ""}${x.part}`];
      }
      if (deep > tol) clips++;
      const mid = add(t.at, qrot(t.rot, [0, r.plan.items.it.size[1] / 2, 0]));
      if (holds(contactsOf(skel, pose(skel, live.pose(f)), field, { parts: /./ }), mid)) held3++;
    }
    const pass = clips <= most;
    if (!pass) failed++;
    const cm = (v: [number, number, string]) => (v[1] < 0 ? "0" : `${(v[0] * 100).toFixed(1)} cm ${v[2]} @${v[1]}`);
    console.log(`${pass ? "PASS" : "FAIL"} ${sc.name} (${r.plan.beats[0].how.grip}): ${clips} of ${held} held frames over ${(tol * 100).toFixed(1)} cm; closure ${Math.round((100 * held3) / Math.max(1, held))}%  (${((performance.now() - t0) / 1000).toFixed(1)} s)`);
    for (const [step, w] of Object.entries(worst)) console.log(`  ${step.padEnd(5)} hand ${cm(w.hand)}; body ${cm(w.body)}`);
  }
}
console.log(`\n${total - failed}/${total} pass`);
process.exit(failed ? 1 : 0);
