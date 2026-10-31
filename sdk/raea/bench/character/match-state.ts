// Matching by state: a library of 100STYLE's Neutral and a few styles (walk and stand each), one route walked with no
// state and in each state. For a style the library has, the lookup (route plus state) should pick that style's own
// frames; for one it hasn't, the walk is Neutral's frames under the style's rig effect. Each walk's style is compared
// with the style's real walk on the spread of the 100 styles' walks.
//
//   bun sdk/raea/bench/character/match-state.ts SKELETON.json LIBRARY
import { readFileSync } from "node:fs";
import { readClips, type Clip, type ClipFile } from "raea/clip";
import { captured, type Skeleton } from "raea/character";
import { motionLibrary, perform, STATE, type MotionState } from "raea/matching";
import { styleMean, type StyleVector } from "raea/style";
import type { RigEffect } from "raea/effects";

const [skelPath, L] = process.argv.slice(2);
const skel = JSON.parse(readFileSync(skelPath, "utf8")) as Skeleton;
const json = (p: string) => JSON.parse(readFileSync(`${L}/${p}`, "utf8"));
const IN = ["Neutral", "Old", "Depressed", "Proud", "Zombie", "Angry", "Crouched"];
const OUT = ["LimpLeft", "BentForward", "Elated", "Stiff"];
const clips = new Map<string, Clip>();
for (const s of IN) for (const [k, c] of readClips(json(`100style/${s}.json`) as ClipFile)) clips.set(k, c);
const lib = motionLibrary(skel, clips, { table: IN.map((s) => json(`100style/${s}.table.json`)) });
const effects = json("effects.json") as Record<string, RigEffect>;
const means = json("means.json") as Record<string, { v: StyleVector }>;
const walks = Object.entries(means).filter(([k]) => k.endsWith("_FW")).map(([, m]) => m.v);
const sd = Object.fromEntries(STATE.map((k) => { const xs = walks.map((v) => v[k]), m = xs.reduce((a, b) => a + b, 0) / xs.length; return [k, Math.sqrt(xs.reduce((a, b) => a + (b - m) ** 2, 0) / xs.length) || 1]; }));
const gap = (a: StyleVector, b: StyleVector) => Math.sqrt(STATE.reduce((s, k) => s + ((a[k] - b[k]) / sd[k]) ** 2, 0) / STATE.length);
const path: [number, number][] = process.env.STRAIGHT ? [[0, 0], [14, 0]] : [[0, 0], [4, 0], [4, 3], [0, 3], [0, 0.2]];
const walk = (state?: MotionState) => {
  const p = perform(lib, { at: [0, 0, 0], facing: Math.PI / 2, steps: [{ walk: path }], state }, { effects, stateWeight: Number(process.env.WS ?? 2) });
  const nj = captured(skel), n = p.frames, root = new Float32Array(n * 3), turns = new Float32Array(n * nj * 4);
  for (let f = 0; f < n; f++) { const q = p.pose(f); root.set(q.root, f * 3); q.turns.slice(0, nj).forEach((t, i) => turns.set(t, (f * nj + i) * 4)); }
  // which style each frame played, over the walk's middle
  const share: Record<string, number> = {};
  for (const x of p.played.slice(45, n - 45)) { const s = x.clip.split("_")[0]; share[s] = (share[s] ?? 0) + 1 / (n - 90); }
  return { v: styleMean(skel, { name: "w", fps: 30, frames: n, root, turns }, 45, n - 45), share };
};
const plain = walk("Neutral");
const top = (share: Record<string, number>) => Object.entries(share).sort((a, b) => b[1] - a[1]).slice(0, 3).map(([s, x]) => `${s} ${(100 * x).toFixed(0)}%`).join(", ");
console.log(`library: ${IN.join(", ")}`);
console.log(`Neutral: frames from ${top(plain.share)}`);
console.log("state".padEnd(14), "in library", "  own frames", "  gap Neutral -> state", "  frames from");
for (const s of [...IN.slice(1), ...OUT]) {
  const w = walk(s), real = means[`${s}_FW`].v;
  console.log(s.padEnd(14), (IN.includes(s) ? "yes" : "no").padStart(10), `${(100 * (w.share[s] ?? 0)).toFixed(0)}%`.padStart(12), `${gap(plain.v, real).toFixed(2)} -> ${gap(w.v, real).toFixed(2)}`.padStart(24), "  ", top(w.share));
}
// a mix of a style the library has and one it hasn't: the first's frames, the second's effect over them
const mix = walk({ Old: 0.6, LimpLeft: 0.6 });
console.log(`Old 0.6 + LimpLeft 0.6: frames from ${top(mix.share)}`);
// two styles the library has, half each: frames of both
console.log(`Old 0.5 + Depressed 0.5: frames from ${top(walk({ Old: 0.5, Depressed: 0.5 }).share)}`);
// a state changed mid-walk: the frames change with it
{
  const p = perform(lib, { at: [0, 0, 0], facing: Math.PI / 2, steps: [{ walk: [[0, 0], [5, 0]], state: "Old" }, { walk: [[5, 0], [10, 0]], state: "Angry" }] }, { effects });
  const stepOf = (f: number) => p.guide[Math.min(f, p.guide.length - 1)].step;
  for (const i of [0, 1]) { const fr = p.played.filter((_, f) => stepOf(f) === i).slice(30); const sh: Record<string, number> = {}; for (const x of fr) { const s = x.clip.split("_")[0]; sh[s] = (sh[s] ?? 0) + 1 / fr.length; } console.log(`walk ${i + 1} of 2 (${i ? "Angry" : "Old"}), past its first second: frames from ${top(sh)}`); }
}
