// How well clips retarget onto a body: each source BVH (100STYLE, CMU, ACCAD, on disk) retargeted and measured frame by
// frame against itself and against what a person can do (docs/skeleton.feature; Shane, 11-01: "the whole human rig
// breaks with these ... its all the bad retargeting"). Nothing here depends on a source's rest pose: the source is
// measured by where its joints are and how its bones turn frame to frame.
//
//   bun sdk/raea/tools/retarget-check.ts SKELETON.json [--method old|ik] [--sources 100style,cmu,accad] [--max N]
//     [--match REGEX] [--every K] [--out FILE.json] [--worst N]
//
// Per clip, the share of frames (every K-th) with each fault:
//   twist   a bone turned about its own length past a person's range (forearm 100°, wrist 60°, upper arm 100°, thigh
//           60°, shin 30°, foot 40°, neck 60°, head 60°), from the body's rest
//   bend    a knee or an elbow bent the wrong way, more than 10°
//   inside  a hand or forearm more than 1.5 cm into the trunk (hips, waist, torso, chest)
//   trunk   the trunk (pelvis to neck) more than 15° off the source's, or the neck (neck to head) more than 25° off,
//           each in its own body's heading
//   limbs   an upper arm, forearm, thigh or shin more than 15° off the source's bone
//   pop     a joint turning more than 15° further in one frame than its source joint does
import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { clipFrame, readBvh, retarget, type Bvh, type Clip } from "raea/clip";
import { B, SIDES, joint, pose, qinv, qmul, qrot, surface, type Quat, type Side, type Skeleton } from "raea/character";
import type { Vec3 } from "raea/3d";

const HOME = process.env.HOME!, M = (n: string) => JSON.parse(readFileSync(new URL(`./maps/${n}.json`, import.meta.url), "utf8")) as Record<string, string>;
const flag = (n: string) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : undefined; };
const skel = JSON.parse(readFileSync(process.argv[2], "utf8")) as Skeleton;
const method = flag("--method") ?? "old", every = +(flag("--every") ?? 3), max = +(flag("--max") ?? Infinity);
const match = new RegExp(flag("--match") ?? ".", "i"), sources = (flag("--sources") ?? "100style,cmu,accad").split(",");
const DEG = 180 / Math.PI;

// ---- the sources on disk: [name, bvh path, map, rest frame]
type Src = { name: string; path: string; map: Record<string, string>; rest?: number[] };
function list(): Src[] {
  const out: Src[] = [];
  if (sources.includes("100style")) {
    const d = `${HOME}/.cache/raea-mocap/100style-full/100STYLE`, map = M("100style");
    for (const st of readdirSync(d).sort()) for (const mv of ["FW", "FR", "ID", "TR1", "SW"]) {
      const p = `${d}/${st}/${st}_${mv}.bvh`;
      if (existsSync(p)) out.push({ name: `${st}_${mv}`, path: p, map });
    }
  }
  if (sources.includes("cmu")) {
    const d = `${HOME}/.cache/raea-mocap/cmu/bvh`, map = M("cmu");
    for (const f of readdirSync(d).filter((x) => x.endsWith(".bvh")).sort()) out.push({ name: `c${f.slice(0, -4)}`, path: `${d}/${f}`, map });
  }
  if (sources.includes("accad")) {
    const map = M("accad");
    for (const actor of ["Male1", "Male2", "Female1"]) {
      const d = `${HOME}/.cache/raea-mocap/accad/${actor}_bvh`, files = readdirSync(d).filter((x) => x.endsWith(".bvh")).sort();
      const stand = readBvh(readFileSync(`${d}/${files.find((x) => /_A0?1_Stand\.bvh$/.test(x))!}`, "utf8"));
      const shape = (b: Bvh) => b.joints.map((j) => j.name).join(" ");
      for (const f of files) {
        const b = readBvh(readFileSync(`${d}/${f}`, "utf8"));
        if (shape(b) === shape(stand)) out.push({ name: `a${f.slice(0, -4)}`, path: `${d}/${f}`, map, rest: stand.frames[0] });
      }
    }
  }
  return out.filter((s) => match.test(s.name)).slice(0, max);
}

// ---- small vector and turn helpers
const sub = (a: Vec3, b: Vec3): Vec3 => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const dot = (a: Vec3, b: Vec3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const nrm = (a: Vec3): Vec3 => { const l = Math.hypot(...a) || 1; return [a[0] / l, a[1] / l, a[2] / l]; };
const ang = (a: Vec3, b: Vec3) => Math.acos(Math.max(-1, Math.min(1, dot(nrm(a), nrm(b))))) * DEG;
const qang = (q: Quat) => 2 * Math.acos(Math.min(1, Math.abs(q[3]))) * DEG;
/** a turn's twist about `axis` (degrees, signed) */
const twist = (q: Quat, axis: Vec3) => { const a = nrm(axis), p = dot([q[0], q[1], q[2]], a); return 2 * Math.atan2(p, q[3]) * DEG; };
const wrap = (d: number) => ((d + 540) % 360) - 180;
/** v turned about up so `across` lies along +x: a vector in its body's own heading */
const unheaded = (v: Vec3, across: Vec3) => { const h = Math.atan2(-across[2], across[0]), c = Math.cos(h), s = Math.sin(h); return [c * v[0] - s * v[2], v[1], s * v[0] + c * v[2]] as Vec3; };

// ---- the source's forward kinematics (its own frames; nothing about its rest is used)
function sourceWorld(b: Bvh, vals: number[]) {
  let c = 0;
  const rot: Quat[] = [], pos: Vec3[] = [];
  b.joints.forEach((j, i) => {
    let q: Quat = [0, 0, 0, 1], t: Vec3 = [...j.offset] as Vec3;
    for (const ch of j.channels) {
      const v = vals[c++];
      if (ch.endsWith("position")) { t = [...t] as Vec3; t["XYZ".indexOf(ch[0])] = v; continue; }
      const h = (v * Math.PI) / 360, ax: Quat = ch[0] === "X" ? [Math.sin(h), 0, 0, Math.cos(h)] : ch[0] === "Y" ? [0, Math.sin(h), 0, Math.cos(h)] : [0, 0, Math.sin(h), Math.cos(h)];
      q = qmul(q, ax);
    }
    if (j.parent < 0) { rot[i] = q; pos[i] = t; return; }
    rot[i] = qmul(rot[j.parent], q);
    const o = qrot(rot[j.parent], j.offset);
    pos[i] = [pos[j.parent][0] + o[0], pos[j.parent][1] + o[1], pos[j.parent][2] + o[2]];
  });
  return { rot, pos };
}

// ---- the body's measures
const J = skel.joints, ji = (n: string) => joint(skel, n), restAxis = (i: number) => sub(J[i].tail, J[i].head);
const TWIST: [string, number][] = [
  ...SIDES.flatMap((s) => [[B.lowerarm(s), 100], [B.hand(s), 60], [B.upperarm(s), 100], [B.thigh(s), 60], [B.calf(s), 30], [B.foot(s), 40]] as [string, number][]),
  [B.neck, 60], [B.head, 60],
];
const LIMBS = SIDES.flatMap((s) => [[B.upperarm(s), B.lowerarm(s)], [B.lowerarm(s), B.hand(s)], [B.thigh(s), B.calf(s)], [B.calf(s), B.foot(s)]]);
const TRUNK = /^(hips|waist|torso|chest)$/, ARM = /^(palm|foreArm)$/;

type Faults = { twist: number; bend: number; inside: number; trunk: number; limbs: number; pop: number };
const KINDS: (keyof Faults)[] = ["twist", "bend", "inside", "trunk", "limbs", "pop"];

function check(src: Src, clip: Clip, b: Bvh) {
  const nj = clip.turns.length / (clip.frames * 4), S = (n: string) => b.joints.findIndex((x) => x.name === src.map[n]);
  const n = { ...Object.fromEntries(KINDS.map((k) => [k, 0])) } as Faults, worst: Record<string, [number, string]> = {};
  const note = (k: keyof Faults, v: number, what: string) => { if (v > (worst[k]?.[0] ?? 0)) worst[k] = [v, what]; };
  let frames = 0, prevT: Quat[] | undefined, prevS: Quat[] | undefined;
  for (let f = 0; f < clip.frames; f += every) {
    const t = f / clip.fps, bf = Math.min(b.frames.length - 1, Math.round(t / b.frameTime));
    const p = clipFrame(clip, f, nj), w = pose(skel, p), sw = sourceWorld(b, b.frames[bf]);
    const hit: Partial<Record<keyof Faults, boolean>> = {};
    // twist past a person's range, from rest (each joint's own turn against its parent's)
    for (const [name, lim] of TWIST) {
      const i = ji(name), d = Math.abs(wrap(twist(p.turns[i], restAxis(i))));
      if (d > lim) { hit.twist = true; note("twist", d - lim, `${name} ${d.toFixed(0)}°@${f}`); }
    }
    // a knee bent forward or an elbow bent back (the lower bone against the upper's rest frame)
    for (const s of SIDES) for (const [up, lo, sign] of [[B.thigh(s), B.calf(s), -1], [B.upperarm(s), B.lowerarm(s), 1]] as [string, string, number][]) {
      const u = ji(up), l = ji(lo), dl = qrot(qinv(w.rot[u]), qrot(w.rot[l], restAxis(l))), du = restAxis(u);
      const fwd = dot(nrm(dl), [0, 0, 1]) * sign, along = dot(nrm(dl), nrm(du));
      const back = Math.atan2(-fwd, along) * DEG;
      if (back > 10) { hit.bend = true; note("bend", back, `${lo} ${back.toFixed(0)}°@${f}`); }
    }
    // a hand or forearm in the trunk
    if (f % (every * 2) === 0) {
      const trunk = surface(skel, w, TRUNK), arms = surface(skel, w, ARM);
      for (const a of arms) {
        let best = Infinity, depth = 0;
        for (const x of trunk) { const d = sub(a.at, x.at), dd = dot(d, d); if (dd < best) { best = dd; depth = -dot(d, x.normal); } }
        // (no deeper than its distance to the nearest skin: a far nearest point means the arm is beside the trunk, not in it)
        if (depth > 0.015 && best < 0.05 * 0.05) { hit.inside = true; note("inside", depth * 100, `${a.side}${a.part} ${(depth * 100).toFixed(1)}cm@${f}`); break; }
      }
    }
    // the trunk, neck and limbs against the source's, each in its own heading
    const tAcross = sub(w.pos[ji(B.thigh("L"))], w.pos[ji(B.thigh("R"))]), sAcross = sub(sw.pos[S(B.thigh("L"))], sw.pos[S(B.thigh("R"))]);
    const tv = (a: string, c: string) => unheaded(sub(w.pos[ji(c)], w.pos[ji(a)]), tAcross);
    const sv = (a: string, c: string) => unheaded(sub(sw.pos[S(c)], sw.pos[S(a)]), sAcross);
    const tr = ang(tv(B.pelvis, B.neck), sv(B.pelvis, B.neck)), nk = ang(tv(B.neck, B.head), sv(B.neck, B.head));
    if (tr > 15 || nk > 25) { hit.trunk = true; note("trunk", Math.max(tr - 15, nk - 25), `trunk ${tr.toFixed(0)}° neck ${nk.toFixed(0)}°@${f}`); }
    for (const [a, c] of LIMBS) {
      if (S(a) < 0 || S(c) < 0) continue;
      const d = ang(tv(a, c), sv(a, c));
      if (d > 15) { hit.limbs = true; note("limbs", d, `${a} ${d.toFixed(0)}°@${f}`); }
    }
    // a joint turning faster than its source joint did
    const mapped = J.slice(0, nj).map((j, i) => [i, S(j.name)] as const).filter(([, s]) => s >= 0);
    const tRot = mapped.map(([i]) => w.rot[i]), sRot = mapped.map(([, s]) => sw.rot[s]);
    if (prevT && prevS) mapped.forEach(([i], k) => {
      const dt = qang(qmul(tRot[k], qinv(prevT![k]))), ds = qang(qmul(sRot[k], qinv(prevS![k])));
      if (dt - ds > 15) { hit.pop = true; note("pop", dt - ds, `${J[i].name} +${(dt - ds).toFixed(0)}°@${f}`); }
    });
    prevT = tRot; prevS = sRot;
    for (const k of KINDS) if (hit[k]) n[k]++;
    frames++;
  }
  const share = Object.fromEntries(KINDS.map((k) => [k, n[k] / Math.max(1, frames)])) as Faults;
  return { name: src.name, frames, share, worst: Object.fromEntries(Object.entries(worst).map(([k, v]) => [k, v[1]])) };
}

const all = list(), rows: ReturnType<typeof check>[] = [];
const ik = method === "ik" ? await import("raea/clip").then((m) => (m as unknown as { retargetIK?: typeof retarget }).retargetIK) : undefined;
if (method === "ik" && !ik) throw new Error("retarget-check: raea/clip has no retargetIK yet");
for (const src of all) {
  const b = readBvh(readFileSync(src.path, "utf8"));
  try {
    const clip = (ik ?? retarget)(skel, b, { name: src.name, map: src.map, rest: src.rest, keepTPose: true });
    rows.push(check(src, clip, b));
    if (rows.length % 25 === 0) console.error(`${rows.length}/${all.length} clips`);
  } catch (e) { console.log(`${src.name}: ${(e as Error).message}`); }
}
// the summary: per source and over all, the mean share of frames with each fault and the clips with any fault on a
// tenth of their frames or more
const bySrc = (r: { name: string }) => (r.name.startsWith("a") && /^a(Male|Female)/.test(r.name) ? "accad" : /^c\d/.test(r.name) ? "cmu" : "100style");
const pct = (x: number) => `${(x * 100).toFixed(1)}%`.padStart(6);
console.log(`method ${method}: ${rows.length} clips, frames every ${every}`);
console.log(`${"".padEnd(9)} ${KINDS.map((k) => k.padStart(6)).join(" ")}   bad clips`);
for (const g of [...new Set(rows.map(bySrc)), "all"]) {
  const rs = rows.filter((r) => g === "all" || bySrc(r) === g);
  const mean = (k: keyof Faults) => rs.reduce((a, r) => a + r.share[k], 0) / Math.max(1, rs.length);
  const bad = rs.filter((r) => KINDS.some((k) => r.share[k] >= 0.1)).length;
  console.log(`${g.padEnd(9)} ${KINDS.map((k) => pct(mean(k))).join(" ")}   ${bad}/${rs.length}`);
}
const worstN = +(flag("--worst") ?? 12), score = (r: (typeof rows)[number]) => KINDS.reduce((a, k) => a + r.share[k], 0);
console.log(`\nworst ${worstN}:`);
for (const r of [...rows].sort((a, b) => score(b) - score(a)).slice(0, worstN))
  console.log(`  ${r.name.padEnd(34)} ${KINDS.filter((k) => r.share[k] > 0).map((k) => `${k} ${pct(r.share[k]).trim()} (${r.worst[k]})`).join("; ")}`);
if (flag("--out")) writeFileSync(flag("--out")!, JSON.stringify({ method, every, rows }, null, 1));
