// The effect layer under the matcher, for states the library has no frames of: a library of 100STYLE's Neutral only
// (walk and stand), one route walked with no state and with each style's state (its effect, learned from the style's
// walk's first half against Neutral's walk), and each walk's style compared with the second half of the style's real
// walk, which the effect never saw, on the spread of the 100 styles' walks. The effect works where the walk with it is
// nearer the real style than the walk without.
//
// The default 14 styles are the ones the gain limit was chosen on; name the other 85 to check it on styles it never saw.
//
//   bun sdk/raea/bench/character/states.ts SKELETON.json LIBRARY [Style ...]
import { readFileSync } from "node:fs";
import { readClips, type Clip, type ClipFile } from "raea/clip";
import { captured, type Skeleton } from "raea/character";
import { motionLibrary, perform, route, STATE } from "raea/matching";
import { styleMean, type StyleVector } from "raea/style";
import { learnEffect, type RigEffect } from "raea/effects";

const [skelPath, L, ...only] = process.argv.slice(2);
const skel = JSON.parse(readFileSync(skelPath, "utf8")) as Skeleton;
const file = (s: string) => readClips(JSON.parse(readFileSync(`${L}/100style/${s}.json`, "utf8")) as ClipFile);
const neutral = file("Neutral");
const means = JSON.parse(readFileSync(`${L}/means.json`, "utf8")) as Record<string, { v: StyleVector }>;
const lib = motionLibrary(skel, neutral, { table: JSON.parse(readFileSync(`${L}/100style/Neutral.table.json`, "utf8")) });
const styles = only.length ? only : ["Old", "Depressed", "Proud", "Elated", "Angry", "LimpLeft", "LimpRight", "DragLeftLeg", "Stiff", "Zombie", "Drunk", "Crouched", "BentForward", "Heavyset"];
// a loop round a 4 x 3 m room, like the capture's own walks round its volume
const path: [number, number][] = [[0, 0], [4, 0], [4, 3], [0, 3], [0, 0.2]];
const effects: Record<string, RigEffect> = {};
const walk = (state?: string) => {
  const p = perform(lib, { at: [0, 0, 0], facing: Math.PI / 2, steps: [{ walk: path }], state }, { effects });
  const nj = captured(skel), n = p.frames, root = new Float32Array(n * 3), turns = new Float32Array(n * nj * 4);
  for (let f = 0; f < n; f++) { const q = p.pose(f); root.set(q.root, f * 3); q.turns.slice(0, nj).forEach((t, i) => turns.set(t, (f * nj + i) * 4)); }
  // the walk's middle, past setting off and before stopping
  const c: Clip = { name: "w", fps: 30, frames: n, root, turns };
  return styleMean(skel, c, 45, n - 45);
};
// each style's effect from its walk's first half; its real style the second half's
const real = (s: string) => { const c = file(s).get(`${s}_FW`)!; return styleMean(skel, c, c.frames >> 1, c.frames); };
for (const s of styles) { const c = file(s).get(`${s}_FW`)!; effects[s] = learnEffect(skel, neutral.get("Neutral_FW")!, c, { s: [0, c.frames >> 1] }); }
// the spread of the 100 styles' walks
const walks = Object.entries(means).filter(([k]) => k.endsWith("_FW")).map(([, m]) => m.v);
const sd = Object.fromEntries(STATE.map((k) => { const xs = walks.map((v) => v[k]), m = xs.reduce((a, b) => a + b, 0) / xs.length; return [k, Math.sqrt(xs.reduce((a, b) => a + (b - m) ** 2, 0) / xs.length) || 1]; }));
const gap = (a: StyleVector, b: StyleVector, keys = STATE) => Math.sqrt(keys.reduce((s, k) => s + ((a[k] - b[k]) / sd[k]) ** 2, 0) / keys.length);
// the walk without an effect at the same pace: as Neutral (the library's own frames, no effect)
const plain = walk("Neutral");
console.log("style".padEnd(14), "no state", "  effect", "  closed", "  posture gap (trunk, head, elbows, knees) no state / effect");
const POSE = ["trunkPitch", "headPitch", "elbowFlex", "kneeBend", "handHeight", "shoulderRaise"] as typeof STATE;
let a = 0, b = 0, better = 0;
const byKey = Object.fromEntries(STATE.map((k) => [k, [0, 0]]));
for (const s of styles) {
  const r = real(s), fx = walk(s), g0 = gap(plain, r), g1 = gap(fx, r);
  for (const k of STATE) { byKey[k][0] += Math.abs(plain[k] - r[k]) / sd[k] / styles.length; byKey[k][1] += Math.abs(fx[k] - r[k]) / sd[k] / styles.length; }
  a += g0; b += g1; if (g1 < g0) better++;
  console.log(s.padEnd(14), g0.toFixed(2).padStart(8), g1.toFixed(2).padStart(8), `${(100 * (1 - g1 / g0)).toFixed(0)}%`.padStart(8), `   ${gap(plain, r, POSE).toFixed(2)} / ${gap(fx, r, POSE).toFixed(2)}`);
}
console.log(`mean ${(a / styles.length).toFixed(2)} -> ${(b / styles.length).toFixed(2)}; nearer the real style with the effect in ${better} of ${styles.length}`);
console.log("each measurement's mean gap, no state -> effect:", STATE.map((k) => `${k} ${byKey[k][0].toFixed(2)}->${byKey[k][1].toFixed(2)}`).join(", "));
