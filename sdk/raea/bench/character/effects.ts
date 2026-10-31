// Rig effects rebuilt against the real capture: for each 100STYLE style, its effect learned from the first half of
// its forward walk against Neutral's, laid over Neutral's walk, and compared with the second half (held out) of the
// style's own walk. Scores, each on the library's own scale (z-scores of the style measurements; degrees for the
// joints): how far plain Neutral is from the style, and how far Neutral with the effect is. An effect that works
// closes most of the gap.
//
//   bun sdk/raea/bench/character/effects.ts SKELETON.json LIBRARY [Style ...] [--json OUT]
import { readFileSync, writeFileSync } from "node:fs";
import { clipPose, readClips, type Clip, type ClipFile } from "raea/clip";
import { captured, pose, qinv, qmul, type Skeleton } from "raea/character";
import { applyEffect, clipMeans, learnEffect, qlog } from "raea/effects";
import { STATE } from "raea/matching";
import { styleOf, type StyleVector } from "raea/style";

const args = process.argv.slice(2);
const ji = args.indexOf("--json"), jsonOut = ji >= 0 ? args.splice(ji, 2)[1] : undefined;
const [skelPath, lib, ...only] = args;
const skel = JSON.parse(readFileSync(skelPath, "utf8")) as Skeleton;
const load = (s: string) => readClips(JSON.parse(readFileSync(`${lib}/100style/${s}.json`, "utf8")) as ClipFile).get(`${s}_FW`)!;
const ix = JSON.parse(readFileSync(`${lib}/index.json`, "utf8")) as { clips: { label: string; move?: string }[] };
const styles = only.length ? only : [...new Set(ix.clips.filter((c) => c.move === "FW").map((c) => c.label))].filter((s) => s !== "Neutral");
const neutral = load("Neutral");

/** a clip's frames as another clip of `n` frames (played at `rate`, with or without an effect laid over) */
function render(c: Clip, n: number, rate: number, f: (p: ReturnType<typeof clipPose>) => ReturnType<typeof clipPose>): Clip {
  const nj = captured(skel), root = new Float32Array(n * 3), turns = new Float32Array(n * nj * 4);
  for (let k = 0; k < n; k++) {
    const p = f(clipPose(c, (k * rate) / c.fps, { loop: true }));
    root.set(p.root, k * 3);
    p.turns.slice(0, nj).forEach((q, i) => turns.set(q, (k * nj + i) * 4));
  }
  return { name: "x", fps: c.fps, frames: n, root, turns };
}
/** the style measurements over a clip, a second every half second, averaged */
function meanStyle(c: Clip, f0 = 0, f1 = c.frames): StyleVector {
  const out = {} as StyleVector;
  const rows: StyleVector[] = [];
  for (let f = f0; f + c.fps <= f1; f += c.fps / 2) rows.push(styleOf(skel, c, f, f + c.fps));
  for (const k of Object.keys(rows[0]) as (keyof StyleVector)[]) out[k] = rows.reduce((a, r) => a + r[k], 0) / rows.length;
  return out;
}
/** the joints' mean turns apart, degrees, averaged over the joints */
function postureGap(a: Clip, b: Clip, b0: number, b1: number): number {
  const ma = clipMeans(skel, a), mb = clipMeans(skel, b, b0, b1);
  let s = 0;
  for (let i = 0; i < ma.turns.length; i++) { const v = qlog(qmul(qinv(ma.turns[i]), mb.turns[i])); s += Math.hypot(...v) * 57.2958; }
  return s / ma.turns.length;
}

// the measurements' spread over the styles' walks, for z-scores
const rowsAll: StyleVector[] = [];
const results: { style: string; before: number; after: number; postureBefore: number; postureAfter: number; worst: string }[] = [];
const cache = new Map<string, { real: StyleVector; plain: StyleVector; fx: StyleVector; pb: number; pa: number }>();
for (const s of styles) {
  const c = load(s), half = Math.floor(c.frames / 2);
  const e = learnEffect(skel, neutral, c, { s: [0, half] });
  const base = clipMeans(skel, neutral);
  const n = Math.min(neutral.frames, 60 * c.fps);
  const plain = render(neutral, n, 1, (p) => p);
  const fx = render(neutral, n, e.rate, (p) => applyEffect(skel, p, e, { base }));
  const real = meanStyle(c, half, c.frames), sp = meanStyle(plain), sf = meanStyle(fx);
  rowsAll.push(real, sp, sf);
  cache.set(s, { real, plain: sp, fx: sf, pb: postureGap(plain, c, half, c.frames), pa: postureGap(fx, c, half, c.frames) });
}
const sd = {} as Record<string, number>;
for (const k of STATE) { const xs = rowsAll.map((r) => r[k]), m = xs.reduce((a, b) => a + b, 0) / xs.length; sd[k] = Math.sqrt(xs.reduce((a, b) => a + (b - m) ** 2, 0) / xs.length) || 1; }
const dist = (a: StyleVector, b: StyleVector) => Math.sqrt(STATE.reduce((s, k) => s + ((a[k] - b[k]) / sd[k]) ** 2, 0) / STATE.length);
for (const [s, r] of cache) {
  const worst = STATE.map((k) => [k, Math.abs(r.fx[k] - r.real[k]) / sd[k]] as const).sort((a, b) => b[1] - a[1])[0];
  results.push({ style: s, before: dist(r.plain, r.real), after: dist(r.fx, r.real), postureBefore: r.pb, postureAfter: r.pa, worst: `${worst[0]} ${worst[1].toFixed(2)}` });
}
results.sort((a, b) => b.before - a.before);
console.log("style".padEnd(20), "gap before", " after", "  closed", " posture deg before/after", "  worst after");
for (const r of results) console.log(r.style.padEnd(20), r.before.toFixed(2).padStart(10), r.after.toFixed(2).padStart(6), `${(100 * (1 - r.after / r.before)).toFixed(0)}%`.padStart(8), `${r.postureBefore.toFixed(1)} / ${r.postureAfter.toFixed(1)}`.padStart(26), "  ", r.worst);
const mb = results.reduce((a, r) => a + r.before, 0) / results.length, ma = results.reduce((a, r) => a + r.after, 0) / results.length;
console.log(`mean gap ${mb.toFixed(2)} -> ${ma.toFixed(2)} (${(100 * (1 - ma / mb)).toFixed(0)}% closed) over ${results.length} styles`);
if (jsonOut) writeFileSync(jsonOut, JSON.stringify({ results, sd }, null, 1));
