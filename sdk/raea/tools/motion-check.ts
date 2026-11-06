// Motion check: a body's performances on the fixed suite (tools/motion-suite.ts) measured against the motion goal's bars.
//
//   bun sdk/raea/tools/motion-check.ts SKELETON.json [--only NAME,NAME] [--out FILE]
//
// Every measure reads the delivered poses and the library's own clips, never the pipeline's internals, so a new
// pipeline is judged the same way as the old:
//   slide   a foot's contact point (the sole under the ankle, the heel, and under the ball, each turned with its joint;
//           down = within 1.5 cm of the floor) moving over the floor during one contact: the most over any contact, cm (bar < 1)
//   pops    frames where a down foot moves more than 2 cm in a frame (bar 0)
//   switch  at each switch (the played clip changes, or skips more than 2 frames) and 3 frames round it, the most any
//           joint turns in a frame beyond what the source turns it: the faster of the clip showing (at the rate it
//           plays) and the clip before the switch carried on at its rate (bar < 10 degrees)
//   inside  frames with a hand or forearm more than 1.5 cm into the trunk, % (bar under 1%)
//   kinks   frames where a joint turns more than 4 degrees further than the mean of the frames either side, % (bar < 2)
//   grip    a held thing's place in its hand's frame drifting from where it was taken, cm and degrees (bar < 0.5 cm)
import { readFileSync, writeFileSync } from "node:fs";
import { B, SIDES, joint, mirrorName, palmAt, pose, qinv, qmul, qrot, surface, type Pose, type Quat, type Skeleton } from "raea/character";
import { clipFrame } from "raea/clip";
import { perform, type MotionLibrary } from "raea/matching";
import { interact } from "raea/items";
import { locoClips, suiteCases, suiteLibrary } from "./motion-suite";
import { faults, reference } from "./motion-faults";
import { sceneFaults } from "./scene-faults";
import { buildDb, perform as performBody } from "raea/body";

const BARS = { slide: 1, pops: 0, switch: 10, inside: 1, kinks: 2, grip: 0.5 };
const DEG = 180 / Math.PI;
const ang = (q: Quat) => 2 * Math.acos(Math.min(1, Math.abs(q[3]))) * DEG;
const sub = (a: number[], b: number[]) => a.map((x, i) => x - b[i]), dot = (a: number[], b: number[]) => a.reduce((s, x, i) => s + x * b[i], 0);
const TRUNK = /^(hips|waist|torso|chest)$/, ARM = /^(palm|foreArm)$/;

type World = ReturnType<typeof pose>;

/** each joint's turn from the frame before, degrees */
const speeds = (W: World[]) => W.map((w, f) => (f ? w.rot.map((q, i) => ang(qmul(q, qinv(W[f - 1].rot[i])))) : w.rot.map(() => 0)));

function feet(skel: Skeleton, W: World[]) {
  const J = skel.joints, z = (n: string) => J[joint(skel, n)].head[1];
  let slide = 0, pops = 0, where = "";
  for (const s of SIDES) {
    for (const n of [B.foot(s), B.ball(s)]) {
      // (the sole under the joint: the heel under the ankle, the tread under the ball, turned as the joint turns)
      const i = joint(skel, n), drop = [0, -z(n), 0] as any, sole = (w: World) => sub(w.pos[i], [0, 0, 0]).map((x, k) => x + (qrot(w.rot[i], drop) as number[])[k]);
      let start: number[] | null = null, far = 0;
      for (let f = 0; f < W.length; f++) {
        const p = sole(W[f]), down = p[1] < 0.015;
        if (!down) { start = null; continue; }
        if (!start) { start = p; continue; }
        const d = Math.hypot(p[0] - start[0], p[2] - start[2]);
        if (d > far) { far = d; if (far > slide) where = `${n}@${f}`; }
        const q = sole(W[f - 1]);
        if (Math.hypot(p[0] - q[0], p[2] - q[2]) > 0.02) pops++;
      }
      slide = Math.max(slide, far);
    }
  }
  return { slide: slide * 100, pops, slideAt: where };
}

function switches(skel: Skeleton, lib: MotionLibrary, played: { clip: string; f: number }[], W: World[], sp: number[][]) {
  // what the source does through a switch is both clips at the rate they play: the clip played before it, carried on at
  // its rate (what the blend fades out of), and the one after; a joint's turn in a frame beyond the faster of the two
  // is what the switch added (measured against one-frame steps of the new clip alone, a body slowing out of a clip
  // played at 1.5x read as a jump)
  const byName = new Map(lib.clips.map((c) => [c.name + (c.mirrored ? "~" : ""), c]));
  const swap = skel.joints.map((j) => joint(skel, mirrorName(j.name)));
  const worlds = new Map<string, World[]>();
  const worldOf = (name: string) => {
    const c = byName.get(name);
    if (!c) return null;
    let w = worlds.get(name);
    if (!w) {
      const nj = c.clip.turns.length / (c.clip.frames * 4);
      w = Array.from({ length: c.clip.frames }, (_, k) => pose(skel, clipFrame(c.clip, k, nj)));
      worlds.set(name, w);
    }
    return w;
  };
  /** each joint's turn in the named clip from frame a to frame b (clamped), degrees */
  const step = (name: string, a: number, b: number) => {
    const w = worldOf(name), c = byName.get(name);
    if (!w || !c) return null;
    const A = w[Math.max(0, Math.min(w.length - 1, Math.round(a)))], Bf = w[Math.max(0, Math.min(w.length - 1, Math.round(b)))];
    const row = A.rot.map((q, i) => ang(qmul(Bf.rot[i], qinv(q))));
    return c.mirrored ? row.map((_, i) => row[swap[i]]) : row;
  };
  const at: number[] = [];
  for (let f = 1; f < played.length; f++) if (played[f].clip !== played[f - 1].clip || Math.abs(played[f].f - played[f - 1].f) > 2) at.push(f);
  let worst = 0, where = "", detail = "";
  for (const s of at) {
    const o = played[s - 1], r = s >= 2 && played[s - 2].clip === o.clip ? o.f - played[s - 2].f : 1;
    for (let f = Math.max(1, s - 3); f <= Math.min(W.length - 1, s + 3); f++) {
      const p = played[f], q = played[f - 1];
      // the clip showing now, at the rate it plays (one frame at its switch, which has no frame before in it)
      // (a clip played below 1x shows a frame twice: it is between frames, not still, so the step is the frame before's)
      const now = step(p.clip, q.clip === p.clip && f !== s && q.f !== p.f ? q.f : p.f - 1, p.f);
      // the clip before the switch carried on at its rate
      const was = f >= s ? step(o.clip, o.f + r * (f - s), o.f + r * (f - s + 1)) : null;
      if (!now) continue;
      sp[f].forEach((v, i) => { const d = v - Math.max(now[i], was ? was[i] : 0); if (d > worst) { worst = d; where = `${skel.joints[i].name}@${f}`; detail = `${v.toFixed(0)} vs ${now[i].toFixed(0)}/${was ? was[i].toFixed(0) : "-"}`; } });
    }
  }
  return { switch: worst, switchAt: where, switchDetail: detail, switches: at.length };
}

function inside(skel: Skeleton, W: World[]) {
  let bad = 0, n = 0;
  for (let f = 0; f < W.length; f += 2) {
    n++;
    const trunk = surface(skel, W[f], TRUNK), arms = surface(skel, W[f], ARM);
    for (const a of arms) {
      let best = Infinity, depth = 0;
      for (const x of trunk) { const d = sub(a.at, x.at), dd = dot(d, d); if (dd < best) { best = dd; depth = -dot(d, x.normal); } }
      if (depth > 0.015 && best < 0.05 * 0.05) { bad++; break; }
    }
  }
  return (100 * bad) / Math.max(1, n);
}

function kinks(sp: number[][]) {
  let k = 0;
  for (let f = 1; f + 1 < sp.length; f++) if (sp[f].some((v, i) => v - (sp[f - 1][i] + sp[f + 1][i]) / 2 > 4)) k++;
  return (100 * k) / Math.max(1, sp.length);
}

/** each held stretch: the thing's place and turn in the nearer palm's frame (the hand joint's turn), its most drift
 * from six frames after it is taken (the hand settled) to its release */
function grips(skel: Skeleton, W: World[], live: ReturnType<typeof interact>, ids: string[]) {
  let cm = 0, deg = 0;
  for (const id of ids) {
    let ref: { s: (typeof SIDES)[number]; p: number[]; q: Quat } | null = null, since = 0;
    for (let f = 0; f < W.length; f++) {
      if (!live.held(id, f)) { ref = null; since = 0; continue; }
      if (++since < 6) continue;
      const it = live.item(id, f);
      const local = (s: (typeof SIDES)[number]) => {
        const h = W[f].rot[joint(skel, B.hand(s))], inv = qinv(h);
        return { p: qrot(inv, sub(it.at, palmAt(skel, W[f], s)) as any) as number[], q: qmul(inv, it.rot) };
      };
      if (!ref) {
        const s = SIDES.reduce((a, b) => (Math.hypot(...sub(it.at, palmAt(skel, W[f], a))) <= Math.hypot(...sub(it.at, palmAt(skel, W[f], b))) ? a : b));
        ref = { s, ...local(s) };
        continue;
      }
      const now = local(ref.s);
      cm = Math.max(cm, Math.hypot(...sub(now.p, ref.p)) * 100);
      deg = Math.max(deg, ang(qmul(now.q, qinv(ref.q))));
    }
  }
  return { grip: cm, gripDeg: deg };
}

const args = process.argv.slice(2), flag = (k: string) => { const i = args.indexOf(k); return i >= 0 ? args[i + 1] : undefined; };
const skel = JSON.parse(readFileSync(args[0], "utf8")) as Skeleton;
const only = flag("--only")?.split(",");
const lib = suiteLibrary(skel);
const rows: Record<string, any>[] = [];
console.log("case             frames  slide cm  pops  switch°  (where)          inside%  kinks%  grip cm/°    pass");
// --body: the new layer (sdk/body's locomotion, docs/character.md C3) on the body library; errands wait for C5/C6
const body = args.includes("--body") ? buildDb(skel as any, process.env.RAEA_MOTION_LIB ?? `${process.env.HOME}/.cache/raea-mocap/lib/turbo`) : null;
const performNew = (spec: any) => {
  const p = performBody(body!, spec, JSON.parse(process.env.RAEA_BODY_OPTS ?? "{}"));
  return { ...p, guide: p.guide.map((g) => ({ ...g, moving: g.want > 0.05 || g.speed > 0.05 })) } as any;
};
for (const c of suiteCases(lib, skel)) {
  if (only && !only.includes(c.name)) continue;
  if (body && c.plan) continue;
  const perf = body ? performNew(c.spec) : c.plan ? perform(lib, c.plan.spec) : perform(lib, c.spec);
  const live = c.plan ? interact(skel, perf, c.plan) : null;
  const poses: Pose[] = Array.from({ length: perf.frames }, (_, f) => (live ? live.pose(f) : perf.pose(f)));
  const W = poses.map((p) => pose(skel, p)), sp = speeds(W);
  const r: Record<string, any> = { name: c.name, frames: perf.frames, ...feet(skel, W), ...switches(skel, lib, perf.played, W, sp), inside: inside(skel, W), kinks: kinks(sp) };
  if (live && c.plan) Object.assign(r, grips(skel, W, live, Object.keys((c.plan as any).items ?? { ball: 0, cup: 0 })));
  if (live && c.plan) { const P = c.plan as any; r.scene = sceneFaults(skel, perf, live, P.things ?? {}, P.items ?? {}, c.seat); }
  const fails = Object.entries(BARS).filter(([k, bar]) => r[k] !== undefined && (k === "pops" ? r[k] > bar : r[k] >= bar)).map(([k]) => k);
  r.pass = fails.length === 0; r.fails = fails;
  r.faults = faults(skel, lib, perf, W);
  rows.push(r);
  const g = r.grip !== undefined ? `${r.grip.toFixed(1)}/${r.gripDeg.toFixed(0)}` : "-";
  console.log(`${c.name.padEnd(16)} [slide ${r.slideAt}] ${String(r.frames).padStart(6)} ${r.slide.toFixed(1).padStart(9)} ${String(r.pops).padStart(5)} ${r.switch.toFixed(0).padStart(8)}  ${r.switchAt.padEnd(16)} ${r.inside.toFixed(1).padStart(7)} ${r.kinks.toFixed(1).padStart(7)} ${g.padStart(10)}    ${r.pass ? "yes" : "no: " + fails.join(", ")}`);
}
// --stages: the same measures at each of the pipeline's stages (perform's trace), to find which stage brings a fault in
if (args.includes("--stages")) for (const c of suiteCases(lib, skel)) {
  if (c.plan || (only && !only.includes(c.name))) continue;
  const perf = perform(lib, c.spec, { trace: true });
  for (const [stage, poses] of Object.entries(perf.trace!.stages)) {
    const W = poses.map((p) => pose(skel, p)), sp = speeds(W), ft = feet(skel, W), sw = switches(skel, lib, perf.played, W, sp);
    console.log(`  ${c.name.padEnd(16)} ${stage.padEnd(7)} slide ${ft.slide.toFixed(1).padStart(5)} [${ft.slideAt}]  pops ${String(ft.pops).padStart(3)}  switch ${sw.switch.toFixed(0).padStart(3)} [${sw.switchAt}]  kinks ${kinks(sp).toFixed(1)}`);
  }
}
// the taxonomy's faults (tools/motion-faults.ts, docs/locomotion.md), the library's own clips first as the reference
{
  const ref = reference(skel, lib, [...locoClips().keys()]);
  const cols = ["skate", "float", "sink", "reach", "drift", "driftMax", "crab", "spin", "overrun", "switchesPerS", "returns", "flips", "stall", "armPhase", "armBack", "headPitch", "headRoll", "trunk", "jitter", "froude", "cadence"];
  console.log("\nfaults           " + cols.map((c) => c.slice(0, 7).padStart(8)).join(""));
  console.log("library          " + cols.map((c) => (c in ref ? (ref as any)[c].toFixed(2) : "").padStart(8)).join(""));
  for (const r of rows) console.log(r.name.padEnd(17) + cols.map((c) => (r.faults[c] ?? 0).toFixed(c === "returns" || c === "flips" ? 0 : 2).padStart(8)).join(""));
  // the scene's faults (plan cases): tools/scene-faults.ts
  for (const r of rows) if (r.scene) { const x = r.scene; console.log(`scene ${r.name}: clip ${x.clip.toFixed(1)}% (deepest ${x.clipDeep.toFixed(1)} cm, ${x.clipAt})  inThing ${x.inThing.toFixed(1)}% (deepest ${x.inDeep.toFixed(1)} cm, ${x.inAt})  gripGap ${x.gripGap.toFixed(1)}%  sitLoop ${x.sitLoop.toFixed(2)}/s\n  hands in things by item: ${JSON.stringify(x.inBy)}  grip gaps: ${JSON.stringify(x.gapBy)} widest ${JSON.stringify(x.gapMax)}\n  clip frames by thing/stage: ${JSON.stringify(x.clipBy)}\n  deepest by thing: ${JSON.stringify(x.deepBy)}`); }
}
const passed = rows.filter((r) => r.pass).length;
console.log(`${passed}/${rows.length} cases pass`);
const out = flag("--out");
if (out) writeFileSync(out, JSON.stringify({ bars: BARS, rows }, null, 1));
