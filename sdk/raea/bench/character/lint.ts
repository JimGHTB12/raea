// The motion lint over simple walks: a straight walk at the library's pace and faster, a stop and start, and a walk
// with turns. Prints each finding with the stage that made it and what happened about then.
//   bun bench/character/lint.ts LIB.json TABLE.json SKELETON.json [--all]
import { readFileSync } from "node:fs";
import { readClips, type ClipFile } from "raea/clip";
import { motionLibrary, perform, type PerformanceSpec } from "raea/matching";
import { lintReport, motionLint } from "raea/lint";
import { footDrift } from "raea/feet";

const [libPath, tablePath, skelPath] = process.argv.slice(2).filter((a) => !a.startsWith("--"));
const all = process.argv.includes("--all");
const json = (p: string) => JSON.parse(readFileSync(p, "utf8"));
const skel = json(skelPath);
const lib = motionLibrary(skel, readClips(json(libPath) as ClipFile), { table: json(tablePath) });
const walks: Record<string, PerformanceSpec> = {
  "straight, library pace": { at: [0, 0, 0], facing: 0, steps: [{ walk: [[0, 0], [0, 6]], speed: 0.55 }, { wait: 1 }] },
  "straight, 1 m/s": { at: [0, 0, 0], facing: 0, steps: [{ walk: [[0, 0], [0, 8]], speed: 1 }, { wait: 1 }] },
  "stop and start": { at: [0, 0, 0], facing: 0, steps: [{ walk: [[0, 0], [0, 3]] }, { wait: 1.2 }, { walk: [[0, 3], [0, 6]] }, { wait: 1 }] },
  "turns": { at: [0, 0, 0], facing: 0, steps: [{ walk: [[0, 0], [0, 3], [2.5, 4], [2.5, 1]] }, { wait: 0.6, face: Math.PI }, { walk: [[2.5, 1], [0, 0.5]] }, { wait: 1 }] },
};
for (const [name, spec] of Object.entries(walks)) {
  const perf = perform(lib, spec, { trace: true });
  const d = footDrift(skel, perf.pose, perf.frames);
  console.log(`\n== ${name}: skate ${d.skateCms.toFixed(1)} cm/s, slide ${d.slideCm.toFixed(1)} cm a step, knee while standing max ${d.kneeMax.toFixed(1)}°/f`);
  console.log(lintReport(motionLint(lib, perf), { max: all ? 1000 : 25 }));
}
