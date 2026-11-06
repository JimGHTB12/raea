// The motion database's evaluation (docs/clips.feature, "The evaluation"): search methods compared over a body's
// full database on real queries recorded from perform runs (the grip scenes, a walk-wait-carry-stop), for recall of
// the exact best, the rank and cost gap of what each picks, time a search, memory, perform's time and the motion's
// quality with each as perform's search, and semantic retrieval (a step wanting carry, look-around, suitcase, box).
//   bun bench/character/clips.ts [--body kid] [--db FILE] [--methods A,B,C,D,E,F] [--max-queries N] [--json OUT]
//     [--no-perform] [--scenes ball,bigbox,...]
// A: the JS brute force (nearest()). B: the kernel's exact quantised cost, best 64-frame blocks, reranked (in TS). C: HDC Hamming over the motion's hypervectors, the tags as a bit filter, the best
// frames reranked. D: HDC with the tags bound into the hypervectors (no tag filter). E: a coarse scan of every 8th
// frame. F: IVF over k-means clusters.
import { readFileSync, writeFileSync } from "node:fs";
import { pose, qinv, qrot, surface, joint } from "raea/character";
import type { Vec3 } from "../../src/3d";
import { readDb } from "@raea/character/clips-bun";
import { interact, thingSdf } from "raea/items";
import { perform, type PerformanceSpec, type SearchBackend, type SearchQuery, type Step } from "raea/matching";
import { SCENES, gripRun } from "../../../../examples/character/grip-scenes";
import { parkour, type Body } from "../../../../examples/character/parkour-levels";
import { BLOCK, MODE, SCORE_MAX, coarseSearch, exactCost, fullHdc, hdcEncode, HW, FS, ivfSearch, kernelQuery, kernelScore, kernelSearch, queryHdc, recording, rerank, topKFast, dbLibrary, type DbLibrary } from "@raea/character/clips";

const args = process.argv.slice(2), opt = (k: string, d?: string) => { const i = args.indexOf(k); return i >= 0 ? args[i + 1] : d; };
const body = opt("--body", "kid") as Body, LIB = `${process.env.HOME}/.cache/raea-mocap/lib/${body}`, dbPath = opt("--db", `${LIB}/motion.db`)!;
const methods = opt("--methods", "A,B,C,D,E,F")!.split(","), maxQ = +(opt("--max-queries", "400")!), jsonOut = opt("--json");
const sceneNames = opt("--scenes")?.split(","), doPerform = !args.includes("--no-perform");
const rss = () => +(process.memoryUsage().rss / 1e9).toFixed(2);
const out: Record<string, unknown> = { body, db: dbPath };
const log = (...a: unknown[]) => console.log(...a);

// ---------------------------------------------------------------- the library
const { skel } = parkour(body);
let t0 = performance.now();
const mdb = readDb(dbPath);
const tRead = performance.now() - t0;
t0 = performance.now();
const lib = dbLibrary(skel, mdb, { read: (file) => JSON.parse(readFileSync(`${LIB}/${file}`, "utf8")) });
const tLib = performance.now() - t0;
out.load = { readMs: Math.round(tRead), libraryMs: Math.round(tLib), frames: lib.size, clips: mdb.header.clips.length, fileMB: +((Object.values(mdb.header.sections).reduce((a, s) => Math.max(a, s.offset + s.bytes), 0)) / 1e6).toFixed(1), rssGB: rss() };
log(`database: ${mdb.header.clips.length} clips, ${lib.size} frames (mirrored), read ${tRead.toFixed(0)} ms, library ${tLib.toFixed(0)} ms, rss ${rss()} GB`);

// the right captures for a tag
const RIGHT: Record<string, RegExp> = { carry: /^c(70_|79_25$|111_36$|113_26$|115_|69_69$)/, suitcase: /^c70_/, box: /^c(79_25|115_\d+)$/, "look-around": /^c40_10$/ };
const lookClips = new Set(mdb.header.clips.filter((c) => /look/i.test(c.label)).map((c) => c.name));
const rightFrame = (tag: string, k: number) => { const n = lib.clips[lib.frameClip[k]].name; return RIGHT[tag].test(n) || (tag === "look-around" && lookClips.has(n)); };

// ---------------------------------------------------------------- the scenes
const carryWalk = (len: number): PerformanceSpec => ({
  at: [0, 0, 0], facing: Math.PI / 2,
  steps: [
    { walk: [[0, 0], [3, 0]] }, { wait: 1.5 },
    // a heavy carry: slow, wanting carry frames
    { walk: [[3, 0], [3, len]], speed: 0.55, tags: { want: ["carry"] } }, { wait: 1.5 },
  ] as Step[],
});
interface Scene { name: string; spec: PerformanceSpec; grip?: ReturnType<typeof gripRun> }
const scenes: Scene[] = [];
for (const sc of SCENES.filter((s) => !sceneNames || sceneNames.includes(s.name))) {
  const r = gripRun(body, sc, lib);
  // a carry of something heavy wants carry frames (items.ts's walks don't say yet)
  const heavy = (r.plan.beats.find((b) => b.kind === "pick")?.how.load ?? 0) > 0.1;
  const spec = heavy ? { ...r.plan.spec, steps: r.plan.spec.steps.map((st) => ("walk" in st && st.speed !== undefined ? { ...st, tags: { want: ["carry"] } } : st)) } : r.plan.spec;
  scenes.push({ name: sc.name + (heavy ? " (carry)" : ""), spec, grip: { ...r, plan: { ...r.plan, spec } } });
}
if (!sceneNames || sceneNames.includes("wwcs")) scenes.push({ name: "walk-wait-carry-stop", spec: carryWalk(3) });

// ---------------------------------------------------------------- the motion's quality
const ranges = new WeakMap<object, Map<string, { start: number }>>();
const rangeOf = (L: typeof lib | ReturnType<typeof parkour>["lib"]) => { let m = ranges.get(L); if (!m) ranges.set(L, (m = new Map(L.clips.map((r) => [r.name + (r.mirrored ? "~" : ""), r])))); return m; };
const TAG = (t: string) => lib.db!.tagIndex.get(t)!;
const hasTag = (k: number, t: string) => { const i = TAG(t); return ((lib.db!.tags[k * 2 + (i >> 5)] >>> (i & 31)) & 1) === 1; };
const HAND = /^(palm|thumb|index|middle|ring|pinky)/;
function quality(sc: Scene, perf: ReturnType<typeof perform>, L: { contact: Uint8Array; db?: unknown; clips: { name: string; start: number; mirrored: boolean }[] } = lib) {
  const nr = rangeOf(L as any), frames = perf.played.map((p) => nr.get(p.clip)!.start + p.f), tagged = !!L.db;
  const rest = skel.joints[0].head[1];
  let midMin = Infinity;
  const feet = ["foot_l", "foot_r"].map((n) => joint(skel, n));
  let maxWalkY = -Infinity, walkN = 0, fast = 0, crouchedN = 0, carryN = 0, carryWalkN = 0, standY: number[] = [], minWalkY = Infinity, skateSum = 0, skateMax = 0;
  let prev: Vec3[] | null = null, prevK = -1;
  for (let f = 0; f < perf.frames; f++) {
    const st = perf.step(f), k = frames[f], P = perf.pose(f), y = P.root[1];
    const w = pose(skel, P), fp = feet.map((i) => w.pos[i]);
    if (st?.kind === "walk") {
      walkN++; if (tagged && hasTag(k, "fast")) fast++; if (tagged && hasTag(k, "crouched")) crouchedN++; minWalkY = Math.min(minWalkY, y); maxWalkY = Math.max(maxWalkY, y);
      if (st.since > 0.5 && st.left > 0.5) midMin = Math.min(midMin, y);
      const wantsCarry = "walk" in sc.spec.steps[st.index] && (sc.spec.steps[st.index] as any).tags?.want?.includes("carry");
      if (wantsCarry && tagged) { carryWalkN++; if (rightFrame("carry", k)) carryN++; }
    } else if (st?.kind === "wait" || !st) standY.push(y);
    if (prev && prevK >= 0) for (const s of [0, 1]) if (L.contact[k * 2 + s] && L.contact[prevK * 2 + s]) { const d = Math.hypot(fp[s][0] - prev[s][0], fp[s][2] - prev[s][2]); skateSum += d; skateMax = Math.max(skateMax, d); }
    prev = fp; prevK = k;
  }
  standY.sort((a, b) => a - b);
  const q: Record<string, number | string> = { walkFrames: walkN, fastShare: tagged ? +(fast / (walkN || 1)).toFixed(3) : "-", crouchedShare: tagged ? +(crouchedN / (walkN || 1)).toFixed(3) : "-",
    minWalkHips: +(minWalkY / rest).toFixed(3), dipCm: +(walkN ? (rest - minWalkY) * 100 : 0).toFixed(1), midDipCm: +(midMin < Infinity ? (rest - midMin) * 100 : 0).toFixed(1), maxWalkHips: +(maxWalkY / rest).toFixed(3), skateCm: +(skateSum * 100).toFixed(1), skateMaxCm: +(skateMax * 100).toFixed(2) };
  if (carryWalkN) q.carryShare = +(carryN / carryWalkN).toFixed(3);
  if (sc.grip) {
    const live = interact(skel, perf, sc.grip.plan), sdf = thingSdf(sc.grip.plan.items.it);
    let clips = 0, held = 0, deepest = 0;
    for (let f = 0; f < live.frames; f++) {
      if (!live.held("it", f)) continue;
      held++;
      const t = live.item("it", f), field = (v: Vec3) => sdf(qrot(qinv(t.rot), [v[0] - t.at[0], v[1] - t.at[1], v[2] - t.at[2]]));
      let deep = 0;
      for (const x of surface(skel, pose(skel, live.pose(f)))) deep = Math.max(deep, -field(x.at));
      if (deep > 0.005) clips++;
      deepest = Math.max(deepest, deep);
    }
    q.gripClipFrames = clips; q.gripPass = clips <= 10 ? "PASS" : "FAIL"; q.heldFrames = held;
  }
  return q;
}

// ---------------------------------------------------------------- A: perform, recording the queries
const backends: Record<string, () => SearchBackend> = {
  B: () => kernelSearch(16),
  C: () => hdcMotion(4096),
  D: () => hdcBound(4096),
  E: () => coarseSearch(8, 32),
  F: () => ivfSearch(lib, 256, 8),
};
const queries: { scene: string; q: SearchQuery }[] = [];
const perfRows: Record<string, unknown>[] = [];
for (const sc of scenes) {
  const qs: SearchQuery[] = [];
  lib.search = recording(qs);
  const t = performance.now();
  const perf = perform(lib, sc.spec, { quiet: true });
  const ms = performance.now() - t;
  const q = quality(sc, perf);
  perfRows.push({ method: "A", scene: sc.name, ms: Math.round(ms), searches: qs.length, ...q });
  log(`A ${sc.name}: ${perf.frames} frames, ${qs.length} searches, ${(ms / 1000).toFixed(1)} s; ${JSON.stringify(q)}`);
  for (const x of qs) queries.push({ scene: sc.name, q: x });
}
lib.search = undefined;
out.queries = queries.length;
// P: the scenes on today's library (parkour.json's 57 clips), for the motion's quality against
{
  const P = parkour(body).lib;
  for (const sc of scenes) {
    const g = sc.grip && gripRun(body, SCENES.find((s) => sc.name.startsWith(s.name))!);
    const s2: Scene = g ? { ...sc, spec: g.plan.spec, grip: g } : sc;
    const t = performance.now(), perf = perform(P, s2.spec, { quiet: true }), ms = performance.now() - t;
    const q = quality(s2, perf, P);
    perfRows.push({ method: "P parkour.json", scene: sc.name, ms: Math.round(ms), ...q });
    log(`P ${sc.name}: ${(ms / 1000).toFixed(1)} s; ${JSON.stringify(q)}`);
  }
}

// the queries for the Rust checks and timings (clips check|bench DB FILE): count, npen, nb, expected, then each
// query's words and pens, then the TS twin's block minima for the first few
const dumpAt = opt("--dump");
if (dumpAt) {
  const pick = queries.filter((_, i) => i % Math.max(1, Math.floor(queries.length / 200)) === 0).slice(0, 200).map(({ q }) => q);
  // (a few as Hamming queries too, motion-only and tag-bound, so the checks cover those terms)
  const ks = pick.map((q, i) => (i % 10 === 1 ? kernelQuery(lib, q, MODE.hamming | MODE.motion, { words: queryHdc(q, false), weight: 1 / 16384 }) : i % 10 === 2 ? kernelQuery(lib, q, MODE.exact | MODE.hamming, { words: queryHdc(q, true), weight: 0.5 }) : kernelQuery(lib, q)));
  const nb = Math.ceil(lib.size / BLOCK), ex = Math.min(3, ks.length), npen = ks[0].pen.length;
  const w: number[] = [ks.length, npen, nb, ex];
  for (const k of ks) w.push(...k.words, ...k.pen);
  const all = new Int32Array(w.length + ex * nb);
  all.set(w);
  for (let i = 0; i < ex; i++) { const mins = new Int32Array(nb).fill(SCORE_MAX); for (let k = 0; k < lib.size; k++) { const s = kernelScore(lib, ks[i], k); if (s < mins[(k / BLOCK) | 0]) mins[(k / BLOCK) | 0] = s; } all.set(mins, w.length + i * nb); }
  writeFileSync(dumpAt, new Uint8Array(all.buffer));
  log(`dumped ${ks.length} queries to ${dumpAt}`);
  if (args.includes("--dump-only")) process.exit(0);
}

// sampled down to at most maxQ, evenly
const sample = queries.length <= maxQ ? queries : Array.from({ length: maxQ }, (_, i) => queries[Math.floor((i * queries.length) / maxQ)]);
log(`\n${queries.length} queries recorded; evaluating ${sample.length}`);

// ---------------------------------------------------------------- recall, rank, gap, time
function hdcMotion(K: number): SearchBackend {
  return { name: `C hdc-motion K=${K}`, candidates: (l, q) => topKFast(hamming(q, false), K) };
}
function hdcBound(K: number): SearchBackend {
  return { name: `D hdc-bound K=${K}`, candidates: (l, q) => topKFast(hamming(q, true), K) };
}
let Hm: Uint32Array | undefined;
/** each frame's Hamming score for a query: C over the motion-only vectors with the tags as a filter, D over the
 * tag-bound vectors (the file's) with the wanted tags bound into the query and not filtered */
function hamming(q: SearchQuery, bound: boolean): Int32Array {
  let H: Uint32Array;
  if (bound) H = fullHdc(lib);
  else { if (!Hm) { Hm = new Uint32Array(lib.size * HW); for (let k = 0; k < lib.size; k++) hdcEncode(lib.full.F, k * FS, 0, 0, Hm, k * HW, false); } H = Hm; }
  const qs = bound && q.tags ? { ...q, tags: { ...q.tags, want: [0, 0] as [number, number] } } : q;
  const mode = MODE.hamming | (bound ? 0 : MODE.motion), hd = { words: queryHdc(q, bound), weight: 1 / (128 * 128) };
  const Q = kernelQuery(lib, qs, mode, hd);
  const sc = new Int32Array(lib.size);
  for (let k = 0; k < lib.size; k++) sc[k] = kernelScore(lib, Q, k, H);
  return sc;
}
/** the blocks' least kernel scores (B) */
function blocks(q: SearchQuery): Int32Array {
  const Q = kernelQuery(lib, q), nb = Math.ceil(lib.size / BLOCK), mins = new Int32Array(nb).fill(SCORE_MAX);
  for (let k = 0; k < lib.size; k++) { const s = kernelScore(lib, Q, k); if (s < mins[(k / BLOCK) | 0]) mins[(k / BLOCK) | 0] = s; }
  return mins;
}
const framesOf = (bl: ArrayLike<number>, K: number) => { const o: number[] = []; for (let i = 0; i < Math.min(K, bl.length); i++) for (let k = bl[i] * BLOCK; k < Math.min(lib.size, bl[i] * BLOCK + BLOCK); k++) o.push(k); return o; };
const rankOf = (sc: Int32Array, k: number) => { if (sc[k] === SCORE_MAX) return Infinity; let r = 0; for (let i = 0; i < sc.length; i++) if (sc[i] < sc[k] || (sc[i] === sc[k] && i < k)) r++; return r; };

interface Row { method: string; K: string; recall: number; gapMean: number; gapMax: number; rankMedian: number | string; ms: number }
const table: Row[] = [];
const ref = sample.map(({ q }) => { const t = performance.now(); const r = rerank(lib, q); return { ...r, ms: performance.now() - t }; });
const refMs = ref.reduce((a, r) => a + r.ms, 0) / ref.length;
table.push({ method: "A js brute force", K: "all", recall: 1, gapMean: 0, gapMax: 0, rankMedian: 0, ms: +refMs.toFixed(1) });
log(`A: ${refMs.toFixed(1)} ms a search (warm)`);

function evaluate(method: string, order: (q: SearchQuery) => { cands: (K: number) => ArrayLike<number>; rank: (k: number) => number }, Ks: number[], unit: string) {
  const rec = Ks.map(() => 0), gaps: number[][] = Ks.map(() => []), ranks: number[] = [], times: number[] = [];
  sample.forEach(({ q }, i) => {
    const a = ref[i];
    if (a.k < 0) return;
    const t = performance.now();
    const o = order(q);
    const c0 = o.cands(Ks[Ks.length >> 1]);
    rerank(lib, q, c0);
    times.push(performance.now() - t);
    ranks.push(o.rank(a.k));
    Ks.forEach((K, j) => {
      const c = o.cands(K), r = rerank(lib, q, c);
      if (r.k === a.k || (r.k >= 0 && Math.abs(r.d - a.d) <= 1e-9 * Math.max(1, a.d))) rec[j]++;
      gaps[j].push(r.k < 0 ? Infinity : r.d / Math.max(1e-9, a.d) - 1);
    });
  });
  const m = ref.filter((r) => r.k >= 0).length;
  ranks.sort((x, y) => x - y);
  Ks.forEach((K, j) => {
    const g = gaps[j].filter(Number.isFinite);
    table.push({ method, K: `${K} ${unit}`, recall: +(rec[j] / m).toFixed(4), gapMean: +(g.reduce((a, x) => a + x, 0) / (g.length || 1)).toFixed(4), gapMax: +Math.max(0, ...g).toFixed(3), rankMedian: ranks[ranks.length >> 1] === Infinity ? "excluded" : ranks[ranks.length >> 1], ms: +(times.reduce((a, x) => a + x, 0) / times.length).toFixed(1) });
  });
  log(table.slice(-Ks.length).map((r) => `${r.method} K=${r.K}: recall ${r.recall}, gap mean ${r.gapMean} max ${r.gapMax}, rank median ${r.rankMedian}, ${r.ms} ms`).join("\n"));
}

if (methods.includes("B")) evaluate("B kernel-ts exact", (q) => { const mins = blocks(q), bl = topKFast(mins, 256); return { cands: (K) => framesOf(bl, K), rank: (k) => rankOf(mins, (k / BLOCK) | 0) }; }, [1, 4, 16, 64], "blocks");
if (methods.includes("C")) evaluate("C hdc motion + tag filter", (q) => { const sc = hamming(q, false), top = topKFast(sc, 16384); return { cands: (K) => top.subarray(0, K), rank: (k) => rankOf(sc, k) }; }, [64, 1024, 4096, 16384], "frames");
if (methods.includes("D")) evaluate("D hdc tags bound", (q) => { const sc = hamming(q, true), top = topKFast(sc, 16384); return { cands: (K) => top.subarray(0, K), rank: (k) => rankOf(sc, k) }; }, [64, 1024, 4096, 16384], "frames");
if (methods.includes("E")) for (const [step, K] of [[8, 8], [8, 32], [16, 64]]) evaluate(`E coarse /${step}`, (q) => { const be = coarseSearch(step, K); const c = be.candidates(lib, q); const s = new Set(Array.from(c)); return { cands: () => c, rank: (k) => (s.has(k) ? 0 : Infinity) }; }, [K], "coarse");
if (methods.includes("F")) {
  const t = performance.now(), base = ivfSearch(lib, 256, 2), idx = [2, 8, 32].map((p) => ({ ...base, candidates: (base as any).probeWith(p) }));
  log(`F: k-means built in ${((performance.now() - t) / 1000).toFixed(1)} s`);
  idx.forEach((be, i) => evaluate(`F ivf 256 probe ${[2, 8, 32][i]}`, (q) => { const c = be.candidates(lib, q); const s = new Set(Array.from(c)); return { cands: () => c, rank: (k) => (s.has(k) ? 0 : Infinity) }; }, [[2, 8, 32][i]], "probes"));
}
out.table = table;
out.rssAfterSearchGB = rss();

// ---------------------------------------------------------------- semantic retrieval
// walk queries asking for a tag: the share of each method's returned frames from the right captures, and of its pick
const walkQs = sample.filter(({ q }) => q.ws !== undefined && !q.phase).slice(0, 40).map(({ q }) => q);
const sem: Record<string, unknown>[] = [];
for (const tag of ["carry", "suitcase", "box", "look-around"]) {
  const bit = TAG(tag), want: [number, number] = [0, 0];
  want[bit >> 5] = (1 << (bit & 31)) >>> 0;
  const variants: [string, (q: SearchQuery) => { cands: ArrayLike<number>; q: SearchQuery }][] = [
    ["A untagged", (q) => ({ cands: [], q: { ...q, tags: undefined } })],
    ["A tag filter", (q) => ({ cands: [], q: { ...q, tags: { want, forbid: q.tags?.forbid ?? [0, 0] } } })],
    ["B untagged K=16", (q) => { const qq = { ...q, tags: undefined }; return { cands: framesOf(topKFast(blocks(qq), 16), 16), q: qq }; }],
    ["B tag filter K=16", (q) => { const qq = { ...q, tags: { want, forbid: q.tags?.forbid ?? [0, 0] } as any }; return { cands: framesOf(topKFast(blocks(qq), 16), 16), q: qq }; }],
    ["C hdc + tag filter K=4096", (q) => { const qq = { ...q, tags: { want, forbid: q.tags?.forbid ?? [0, 0] } as any }; return { cands: topKFast(hamming(qq, false), 4096), q: qq }; }],
    ["D hdc tag bound K=4096", (q) => { const qq = { ...q, tags: { want, forbid: q.tags?.forbid ?? [0, 0] } as any }; return { cands: topKFast(hamming(qq, true), 4096), q: { ...qq, tags: undefined } }; }],
    ["D hdc bound + filter K=4096", (q) => { const qq = { ...q, tags: { want, forbid: q.tags?.forbid ?? [0, 0] } as any }; return { cands: topKFast(hamming(qq, true), 4096), q: qq }; }],
  ];
  for (const [name, f] of variants) {
    if (!methods.includes(name[0])) continue;
    let ret = 0, retRight = 0, pickRight = 0, picks = 0;
    for (const q0 of walkQs) {
      const { cands, q } = f(q0);
      const all = name.startsWith("A");
      if (!all) for (let i = 0; i < cands.length; i++) { const k = cands[i]; if (exactCost(lib, { ...q, tags: undefined }, k) < Infinity) { ret++; if (rightFrame(tag, k)) retRight++; } }
      const r = all ? rerank(lib, q) : rerank(lib, q, cands);
      if (r.k >= 0) { picks++; if (rightFrame(tag, r.k)) pickRight++; }
    }
    const row = { tag, method: name, returnedRight: ret ? +(retRight / ret).toFixed(3) : "-", pickRight: +(pickRight / (picks || 1)).toFixed(3), queries: walkQs.length };
    sem.push(row);
    log(`semantic ${tag} ${name}: returned ${row.returnedRight}, picks ${row.pickRight}`);
  }
}
out.semantic = sem;

// ---------------------------------------------------------------- perform with each method
if (doPerform) for (const m of methods.filter((x: string) => x !== "A" && backends[x])) {
  const be = backends[m]();
  lib.search = be;
  for (const sc of scenes) {
    const t = performance.now();
    const perf = perform(lib, sc.spec, { quiet: true });
    const ms = performance.now() - t;
    const q = quality(sc, perf);
    perfRows.push({ method: m, scene: sc.name, ms: Math.round(ms), ...q });
    log(`${m} ${sc.name}: ${(ms / 1000).toFixed(1)} s; ${JSON.stringify(q)}`);
  }
}
lib.search = undefined;
out.perform = perfRows;
out.rssGB = rss();

// ---------------------------------------------------------------- the tables
const md = (rows: Record<string, unknown>[]) => { if (!rows.length) return ""; const ks = [...new Set(rows.flatMap((r) => Object.keys(r)))]; return [`| ${ks.join(" | ")} |`, `|${ks.map(() => "---").join("|")}|`, ...rows.map((r) => `| ${ks.map((k) => r[k] ?? "").join(" | ")} |`)].join("\n"); };
log(`\n## search\n${md(table as any)}\n\n## semantic\n${md(sem)}\n\n## perform\n${md(perfRows)}`);
if (jsonOut) writeFileSync(jsonOut, JSON.stringify(out, null, 1));
