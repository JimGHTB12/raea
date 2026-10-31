// perform: the performance simulated once from frame 0.
import type { Vec3 } from "raea/3d";
import { softElbows, qinv, qmul, qnorm, qrot, qyaw, type Pose, type Quat } from "raea/character";
import { framePose } from "../clip";
import { plantFeet, type PlantLog } from "../feet";
import { lintReport, motionLint } from "../lint";
import { applyEffect, clipMeans, mirrorEffect, mixEffects, qexp, qlog, type RigEffect } from "../effects";
import { Act, FUTURE, MotionLibrary, NF, PerformOptions, Performance, PerformanceSpec, PhaseGoal, S, before, between, carrySpeed, clipFloor, guide, handCost, holdActionFeet, idOf, labelOf, leadGaze, libraryPose, lintEnv, mirror, nearest, nearestHands, paceOf, performances, phaseCost, seatYaw, smooth, smoothStill, stages, stateCost, stateMix, stepLookups, strideEffect, tagCost, wrap } from "./all";

/** A performance simulated once for its library and spec: the guide along the steps, each moment's best library frame,
 * blended switches, the body pulled onto the guide, and the feet pinned while they stand. */
export function perform(lib: MotionLibrary, spec: PerformanceSpec, o: PerformOptions = {}): Performance {
  const { effects, ...opts } = o;
  if (lintEnv() && !o.trace) return perform(lib, spec, { ...o, trace: true });
  const key = JSON.stringify([lib.skel.name, lib.size, lib.search?.name ?? "", spec, opts, effects ? idOf(effects) : 0]);
  const hit = performances.get(key);
  if (hit) return hit;
  const skel = lib.skel, fps = lib.fps;
  // a walk in a state goes at the state's pace (an old man's is a third of neutral's) unless the step says its own
  const paced = effects ? { ...spec, steps: spec.steps.map((st) => {
    const s = ("state" in st && st.state) || spec.state, mix = s ? stateMix(s) : null;
    if (!("walk" in st) || st.speed !== undefined || !mix) return st;
    // the heaviest style the library has frames of sets the pace (its walks' own); the others scale it by their
    // effects' (a library with none of them: its Neutral's pace, or the default walk's, scaled)
    const own = Object.entries(mix).filter(([l]) => paceOf(lib, l) > 0).sort((x, y) => y[1] - x[1])[0];
    let k = own ? paceOf(lib, own[0]) : paceOf(lib, "Neutral") || 1;
    for (const [l, w] of Object.entries(mix)) if (!own || l !== own[0]) k *= (effects[l]?.speed ?? 1) ** w;
    return { ...st, speed: Math.max(0.2, k) };
  }) } : spec;
  const G = guide(paced, fps, lib);
  const N = spec.frames ?? G.length;
  const every = o.every ?? 6, bf = Math.max(1, Math.round((o.blend ?? 0.2) * fps)), bo = Math.max(bf, Math.round((o.blendOut ?? 0.45) * fps)), pull = o.pull ?? 0.12;
  // the body's ground frame in the scene, and the library frame it plays
  let x = spec.at[0], z = spec.at[2], yaw = spec.facing;
  const gAt = (k: number) => G[Math.min(G.length - 1, Math.max(0, k))];
  const ws = o.stateWeight ?? 2;
  const { wantAt, penAt, maskAt, swingAt, condAt, upperAt, upperHands, mixAt } = stepLookups(lib, spec, G, ws);
  let tw: Record<string, number> = { ...mixAt(0) };
  const ease = 1 - Math.exp(-3 / fps);
  // the pose library frame kk shows (a fraction on toward the next), with the effects laid over: the target less what
  // the frame's own style already is (a frame of Old's walk needs no Old effect)
  const mixed = new Map<string, RigEffect | null>();
  // the effects' rate (library frames a frame) and stride (the ground covered a library frame, against the library's):
  // a slow state walks the library's walk slower and shorter, not a slow walk the library may not have
  let rate = 1, stride = 1, effRate = 1, effStride = 1, paceK = 1;
  // the library's own walking pace: a walk asked to go faster or slower than it plays its walk faster (a quicker
  // cadence) and longer (a longer stride), split so the legs stay in reach, rather than the body being dragged over
  // feet that then slide (asked 1 m/s of a walk that goes 0.52, a child's feet slid 18 cm a step)
  const libPace = paceOf(lib, typeof spec.state === "string" && paceOf(lib, spec.state) > 0 ? spec.state : "Neutral");
  const ownPaced = libPace <= 0 && !effects;
  const ownPace = (kk: number) => {
    const r = lib.clips[lib.frameClip[kk]], a = Math.max(r.start, kk - 5), b = Math.min(r.end - 1, kk + 5), G0 = lib.ground;
    return b > a ? (Math.hypot(G0[b * 3] - G0[a * 3], G0[b * 3 + 1] - G0[a * 3 + 1]) * lib.fps) / (b - a) : 0;
  };
  const shownAt = (kk: number, frac: number): Pose => {
    const r = lib.clips[lib.frameClip[kk]], f = lib.frameF[kk];
    let p = frac > 0 && f + 1 < r.clip.frames ? between(r.clip, f, frac) : framePose(r.clip, f);
    if (effects) {
      const own = labelOf(r.name), parts: [RigEffect, number][] = [];
      for (const l of new Set([...Object.keys(tw), own])) {
        const w = (tw[l] ?? 0) - (l === own ? 1 : 0);
        if (effects[l] && Math.abs(w) > 0.01) parts.push([effects[l], Math.round(w * 100) / 100]);
      }
      const mk = parts.map(([e, w]) => `${idOf(e)}:${w}`).join(",") + (r.mirrored ? "~" : "");
      if (!mixed.has(mk)) mixed.set(mk, parts.length ? (r.mirrored ? mirrorEffect(skel, mixEffects(parts)) : mixEffects(parts)) : null);
      const e = mixed.get(mk);
      if (e) p = applyEffect(skel, p, e, { base: clipMeans(skel, r.clip) });
      // the rate and stride only from the styles the library has no frames of: a style it has walks at its own frames'
      // pace (from another style's frames, the difference's rate would make those frames fit the route as well as its own)
      let lr = 0, ls = 0;
      for (const [l, w2] of Object.entries(tw)) if (effects[l] && paceOf(lib, l) === 0) { lr += w2 * Math.log(effects[l].rate); ls += w2 * Math.log(effects[l].speed ?? 1); }
      effRate = Math.exp(lr);
      effStride = Math.exp(ls - lr);
    }
    // a stride the pace match lengthened, swung for: the hips' swing fore and aft wider by it and the knees' by its
    // square root, so the legs reach the longer step instead of the planted foot falling behind
    const sk = paceK ** 0.4;
    if (Math.abs(sk - 1) > 0.02) p = applyEffect(skel, p, strideEffect(skel, sk), { base: clipMeans(skel, r.clip) });
    return r.mirrored ? mirror(skel, p) : p;
  };
  let u = 0;
  // the query: the pose's own features from frame k (feet, hips) and the guide's next second in the body's frame
  // (in the library's time and stride, so the effects' rate and stride ask for the library's own walk)
  const q = new Float32Array(NF);
  const query = (k: number, t: number) => {
    // (a clip's last second has no features: its last frame that has)
    const F = lib.features, c0 = lib.clips[lib.frameClip[k]].start;
    let kf = k;
    while (kf > c0 && Number.isNaN(F[kf * NF])) kf--;
    const o2 = kf * NF;
    for (let d = 0; d < 15; d++) q[d] = F[o2 + d] / lib.scale[d] + lib.mean[d];
    let c = 15;
    const inv = qyaw(-yaw);
    // (played against the clip's own pace, the route as it is: asked against the playing, a slower capture played
    // quicker asked for slower still)
    const qr = ownPaced ? effRate : rate, qs = ownPaced ? effStride : stride;
    for (const dt of FUTURE) {
      const g = gAt(t + Math.round(dt / qr));
      const p = qrot(inv, [(g.x - x) / qs, 0, (g.z - z) / qs]);
      q[c++] = p[0];
      q[c++] = p[2];
    }
    for (const dt of FUTURE) {
      const y = gAt(t + Math.round(dt / qr)).yaw - yaw;
      q[c++] = Math.sin(y);
      q[c++] = Math.cos(y);
    }
    return q;
  };
  // where it starts: the frame nearest standing still facing ahead
  const start = (() => {
    const s = new Float32Array(NF);
    for (let d = 0; d < 15; d++) s[d] = lib.mean[d];
    for (let d = 6; d < 15; d++) s[d] = 0;
    let c = 15;
    for (const _ of FUTURE) { s[c++] = 0; s[c++] = 0; }
    for (const _ of FUTURE) { s[c++] = 0; s[c++] = 1; }
    const k0 = nearest(lib, s, undefined, wantAt(0), ws, penAt(0), undefined, undefined, condAt(0)).k;
    // (a walk entered at speed starts mid-stride: the frame whose way on is the guide's, its pose any)
    const first = spec.steps[0];
    if (!(first && "walk" in first && (first.enter ?? 0) > 0.5)) return k0;
    const g = query(k0, 0), F = lib.features;
    let best = k0, bd = Infinity;
    for (let k2 = 0; k2 < lib.size; k2++) {
      if (lib.jump[k2] !== 1) continue;
      let dd = 0;
      for (let d = 15; d < NF; d++) dd += (F[k2 * NF + d] - (g[d] - lib.mean[d]) * lib.scale[d]) ** 2;
      if (dd < bd) { bd = dd; best = k2; }
    }
    return best;
  })();
  let k = start;
  // the blend out of a switch (inertialized): each joint's offset and how fast it was changing, and the hips' height's,
  // fading to nothing over the blend on a Hermite curve, so a switch keeps the body's speed as well as its place
  let off: { x: Vec3[]; v: Vec3[]; y: number; vy: number } | null = null, since = 0;
  // (longer at an action's edges: its run-in and run-out lean into the obstacle, a capture's runs stand up, and the
  // lean taken off in a fifth of a second read as a hitch. The hips' tilt too, the lean is mostly theirs, but the legs
  // turned back by its difference from the short blend: legs blended as long slid the feet and floated the strides)
  let bl = bf;
  // (the legs: from the hips' children that lead down to a thigh, LHipJoint and on)
  const legOf = (i: number): boolean => i > 0 && (/UpLeg/.test(skel.joints[i].name) || skel.joints.some((j, c) => j.parent === i && legOf(c)));
  const thighs = skel.joints.flatMap((j, i) => (j.parent === 0 && legOf(i) ? [i] : []));
  const upper = skel.joints.map((_, i) => { for (let a = i; a > 0; a = skel.joints[a].parent) if (thighs.includes(a)) return false; return true; });
  const hermite = (s: number, n = bf): [number, number] => { if (s >= n) return [0, 0]; const a = s / n; return [2 * a ** 3 - 3 * a ** 2 + 1, (a ** 3 - 2 * a ** 2 + a) * n]; };
  const offAt = (O: NonNullable<typeof off>, i: number, [a, b]: [number, number]) => qexp([O.x[i][0] * a + O.v[i][0] * b, O.x[i][1] * a + O.v[i][1] * b, O.x[i][2] * a + O.v[i][2] * b]);
  /** each joint's blend offset `s` frames into it; `fr`, the frame it goes onto (its turns and ground heading), to keep
   * the legs: the thighs turned back by the hips' difference, through their hip joints (which hold still in every
   * capture: turned themselves, they jerked past the library's) */
  const blendAt = (s: number, fr?: { turns: Quat[]; g: number }): Quat[] | null => {
    if (!off || s >= Math.max(bf, bl)) return null;
    const lo = hermite(s), hi = hermite(s, bl), O = off;
    const out = O.x.map((_, i) => offAt(O, i, upper[i] ? hi : lo));
    if (fr && bl > bf) {
      const local = qmul(qyaw(-fr.g), fr.turns[0]), d = qmul(qinv(local), qmul(qmul(qinv(out[0]), offAt(O, 0, lo)), local));
      for (const i of thighs) {
        const H = qmul(out[i], fr.turns[i]), D = qmul(qinv(H), qmul(d, H));
        for (const c of skel.joints.flatMap((j, ci) => (j.parent === i ? [ci] : []))) out[c] = qnorm(qmul(D, out[c]));
      }
    }
    return out;
  };
  const yAt = (s: number) => { if (!off || s >= bf) return 0; const [a, b] = hermite(s); return off.y * a + off.vy * b; };
  const raw: Pose[] = [], played: Performance["played"] = [], contacts: [number, number][] = [];
  const groundOf = (ci: number) => clipFloor(skel, lib.clips[ci].clip);
  // for checks (o.trace): the clip's own frame each frame, the switches, the rate
  const clipStage: Pose[] = [], switches: number[] = [], rates: number[] = [];
  const valid = (j: number) => lib.jump[j] === 1;
  // the upper body's own stream (limbs.upper): its library frame and how far on, the offsets of its switches (from what
  // showed, eased out as the body's are), and the joints it plays (the hips' children but the legs, and on up)
  const upperJ = upper.map((x, i) => x && i > 0);
  let kU = -1, uU = 0, offU: { x: Vec3[]; v: Vec3[] } | null = null, sinceU = 0, wasU = false;
  const playedU: ({ clip: string; f: number } | null)[] = [];
  /** a frame's pose for the upper body (the effects' rate and stride stay the body's) */
  const upperShown = (kk: number, frac: number) => { const a = effRate, b = effStride, p = shownAt(kk, frac); effRate = a; effStride = b; return p; };
  /** the offsets from what showed (the last two frames' upper body, carried on one frame and two) onto `to` and a frame on */
  const offOnto = (to: Quat[], to1: Quat[]): { x: Vec3[]; v: Vec3[] } | null => {
    if (raw.length < 2) return null;
    const a = raw[raw.length - 1].turns, p = raw[raw.length - 2].turns, x: Vec3[] = [], v: Vec3[] = [];
    for (let i = 0; i < a.length; i++) {
      if (!upperJ[i]) { x.push([0, 0, 0]); v.push([0, 0, 0]); continue; }
      const d = qlog(qnorm(qmul(qinv(p[i]), a[i]))), A = qmul(a[i], qexp(d)), B = qmul(a[i], qexp([2 * d[0], 2 * d[1], 2 * d[2]]));
      const xa = qlog(qnorm(qmul(A, qinv(to[i])))), xb = qlog(qnorm(qmul(B, qinv(to1[i]))));
      x.push(xa); v.push([xb[0] - xa[0], xb[1] - xa[1], xb[2] - xa[2]]);
    }
    return { x, v };
  };
  /** the ground frame u of the way on from library frame k to the next: the library's (x, z, heading) there, and how far
   * the body has come from its place at k (scaled by the effects' stride, in the body's heading) */
  const onward = () => {
    const nk = k + 1 < lib.clips[lib.frameClip[k]].end ? k + 1 : k, G0 = lib.ground, a = k * 3, b = nk * 3;
    const d = qrot(qyaw(yaw - G0[a + 2]), [G0[b] - G0[a], 0, G0[b + 1] - G0[a + 1]]), dy = wrap(G0[b + 2] - G0[a + 2]);
    return { gx: G0[a] + (G0[b] - G0[a]) * u, gz: G0[a + 1] + (G0[b + 1] - G0[a + 1]) * u, gy: G0[a + 2] + dy * u, dx: d[0] * stride * u, dz: d[2] * stride * u, dyaw: dy * u };
  };
  /** library frame j, u of the way on, a frame's worth of playing (r library frames) later */
  const ahead = (j: number, uu: number, r: number): [number, number] => {
    uu += r;
    while (uu >= 1 && j + 1 < lib.clips[lib.frameClip[j]].end) { j++; uu--; }
    return [j, uu >= 1 ? 0 : uu];
  };
  /** play library frame `to` next, blending out of what shows now */
  const switchTo = (to: number) => {
    switches.push(raw.length);
    // what shows now and a frame on, and the new frame and a frame on of it
    const [k1, u1] = ahead(k, u, rate);
    let fromA = shownAt(k, u), fromB = shownAt(k1, u1);
    // (at a clip's last frame, an action handing back to the run, there is no frame on and the last is what showed:
    // carried on from the frame before, a frame and two on, or the blend showed it again and every limb stopped dead a
    // frame and started again from rest, Shane 10-22: "the fluidity of animations isn't really fluid")
    if (k + 1 >= lib.clips[lib.frameClip[k]].end && k > lib.clips[lib.frameClip[k]].start) {
      const P = shownAt(k - 1, u), gA = lib.ground[k * 3 + 2], gP = lib.ground[(k - 1) * 3 + 2];
      // (the legs at a quarter of their speed: a foot just landed, carried on at its full swing, scraped along the floor)
      const legs = new Set(skel.joints.flatMap((j, i) => (/Leg|Foot|Toe/.test(j.name) ? [i] : []))), sl = 0.25;
      const on = (n: number): Pose => ({
        root: [0, 1, 2].map((c) => fromA.root[c] + n * (fromA.root[c] - P.root[c])) as Vec3,
        turns: fromA.turns.map((q, i) => {
          // (the hips' turn in each frame's own ground frame, so the heading carries on)
          const a = i === 0 ? qmul(qyaw(-gA), q) : q, b = i === 0 ? qmul(qyaw(-gP), P.turns[0]) : P.turns[i];
          const m = legs.has(i) ? n * sl : n, d = qlog(qnorm(qmul(qinv(b), a))), r = qnorm(qmul(a, qexp([d[0] * m, d[1] * m, d[2] * m])));
          return i === 0 ? qmul(qyaw(gA), r) : r;
        }),
      });
      [fromA, fromB] = [on(1), on(2)];
    }
    shownAt(to, 0);
    const [t1, v1] = ahead(to, 0, rate);
    const nextB = shownAt(t1, v1), nextA = shownAt(to, 0);
    // the root's turn is compared in each frame's own ground frame, so the heading carries on
    const offOf = (fr: Pose, kf: number, nx: Pose, kt: number, s: number): Vec3[] => {
      const prev = blendAt(s, { turns: fr.turns, g: lib.ground[kf * 3 + 2] });
      return nx.turns.map((q2, i) => {
        const sh = i === 0 ? qmul(qyaw(-lib.ground[kf * 3 + 2]), fr.turns[0]) : fr.turns[i];
        const q3 = i === 0 ? qmul(qyaw(-lib.ground[kt * 3 + 2]), q2) : q2;
        return qlog(qnorm(qmul(prev ? qmul(prev[i], sh) : sh, qinv(q3))));
      });
    };
    const xA = offOf(fromA, k, nextA, to, since), xB = offOf(fromB, k1, nextB, t1, since + 1);
    const yA = yAt(since) + fromA.root[1] - nextA.root[1], yB = yAt(since + 1) + fromB.root[1] - nextB.root[1];
    off = { x: xA, v: xA.map((x, i) => [xB[i][0] - x[0], xB[i][1] - x[1], xB[i][2] - x[2]] as Vec3), y: yA, vy: yB - yA };
    // (a switch inside a long blend keeps what is left of it)
    { const t = raw.length, out = [1, 2, 3].some((d) => G[t - d]?.act?.play) && !G[t]?.act;
      bl = Math.max(bf, bl - since, out ? bo : 0); }
    // the way come since frame k carried into the body's place, as the new frame starts from none of it
    const w = onward();
    x += w.dx; z += w.dz; yaw += w.dyaw;
    u = 0;
    since = 0;
    k = to;
  };
  // an action's alignment: the clip's seat onto the scene's, and the error at its start, eased out over sitting down
  let align: { rot: number; e: [number, number]; ey: number } | null = null;
  const actFrame = (a: Act): number => {
    const S = lib.sit!;
    if (a.j < a.len[0]) return S.down[0] + a.j;
    if (a.j < a.len[0] + a.len[1]) {
      // the sat stretch to and fro as long as it is sat
      const m = Math.max(1, S.sat[1] - S.sat[0]), u = (a.j - a.len[0]) % (2 * m);
      return S.sat[0] + (u < m ? u : 2 * m - u);
    }
    return S.up[0] + (a.j - a.len[0] - a.len[1]);
  };
  for (let t = 0; t < N; t++) {
    const g0 = gAt(t), act = t < G.length ? g0.act : undefined;
    // the pace asked of the walk against the library's, eased over a third of a second
    // (a library with no walking style's pace: against the playing clip's own, a little either way, so a run matched
    // into a slower capture cycles its legs quicker rather than the hips being pulled on over standing feet)
    const own = libPace > 0 || act ? 0 : ownPace(k);
    const want = g0.moving && g0.speed && libPace > 0 && !effects ? Math.max(0.5, Math.min(2.5, g0.speed / libPace))
      : g0.moving && g0.speed && own > 0.2 && !effects ? Math.max(0.7, Math.min(1.6, g0.speed / own)) : 1;
    paceK += (want - paceK) * 0.1;
    // (against the clip's own pace, all of it in the cadence: a longer stride than the capture's drags the hips on past
    // the standing foot's reach, the foot pulled up off the floor)
    rate = effRate * paceK ** (ownPaced ? 1 : 0.6);
    stride = effStride * paceK ** (ownPaced ? 0 : 0.4);
    if (act?.play) {
      // an action clip plays straight through; its first frame blends in from what shows
      const P = act.play, want = P.from + act.j;
      if (act.j > 0 && want === k + 1) k = want;
      else switchTo(want);
      if (act.j === 0) {
        // the clip's anchor onto the scene's; what is left between where the body is and where the clip starts, eased
        // out by the anchor
        const rot = P.rot, rA = libraryPose(lib, P.anchor).root, r0 = libraryPose(lib, k).root;
        const p0 = qrot(qyaw(rot), [r0[0] - rA[0], 0, r0[2] - rA[2]]);
        // (where the hips would be a frame on at the speed they came in with: the clip's first frame is the next frame)
        const last = raw.length ? raw[raw.length - 1].root : [x, 0, z], before = raw.length > 1 ? raw[raw.length - 2].root : last;
        const shownHips = [2 * last[0] - before[0], last[1], 2 * last[2] - before[2]];
        align = { rot, e: [shownHips[0] - (P.at[0] + p0[0]), shownHips[2] - (P.at[1] + p0[2])], ey: wrap(yaw - (lib.ground[k * 3 + 2] + rot)) };
      }
    } else if (act) {
      const want = actFrame(act);
      // the clip plays on (or turns back, to and fro while sat) with no blend; a jump in it blends
      if (act.j > 0 && Math.abs(want - k) <= 1 && lib.frameClip[want] === lib.frameClip[k]) k = want;
      else switchTo(want);
      if (act.j === 0) {
        // the clip's seat onto the scene's; what is left between where the body is and where the clip starts, eased
        const S = lib.sit!;
        const rot = seatYaw(act.seat!) - S.yaw;
        const r0 = libraryPose(lib, k).root;
        const p0 = qrot(qyaw(rot), [r0[0] - S.seat[0], 0, r0[2] - S.seat[2]]);
        const shownHips = raw.length ? raw[raw.length - 1].root : [x, 0, z];
        align = { rot, e: [shownHips[0] - (act.seat!.at[0] + p0[0]), shownHips[2] - (act.seat!.at[2] + p0[2])], ey: wrap(yaw - (lib.ground[k * 3 + 2] + rot)) };
      }
    } else {
      align = null;
      // search: every few frames, or when the clip runs out
      if (t > 0 && (t % every === 0 || !valid(k))) {
        const c = lib.clips[lib.frameClip[k]];
        const cur = query(k, t);
        // (and none of the last second of the same clip: played quicker than the capture, it jumped back 8 frames and
        // replayed them, the feet dragged back with it)
        const near: [number, number] = [Math.max(c.start, k - 30), Math.min(c.end, k + 8)];
        // the current frame's own cost: staying unless something is clearly better
        let stay = Infinity;
        const want = wantAt(t);
        // an action starting within 0.8 s: the walk steered to arrive in its phase (its first frame's feet), more so the
        // nearer it is
        // (none in its last 6 frames: a switch blending out as the action's own blends in jerks the legs)
        let phase: PhaseGoal | undefined, late = false;
        for (let d = 1; d <= 36 && t + d < G.length; d++) {
          const a2 = G[t + d].act;
          if (a2?.play && a2.j === 0) { phase = { to: a2.play.from, m: Math.round(d * rate), w: 1.5 }; late = d <= 6; break; }
          if (a2) break;
        }
        if (valid(k)) {
          const z0 = lib.features.subarray(k * NF, (k + 1) * NF);
          const pen = penAt(t);
          const tc = condAt(t);
          stay = stateCost(lib, k, want, ws) + (pen ? pen[lib.frameClip[k]] : 0) + (phase ? phaseCost(lib, k, phase) : 0) + (maskAt(t)?.[k] ? Infinity : 0) + (swingAt(t)?.[k] ?? 0) + (tc ? tagCost(lib, k, tc) : 0);
          for (let d = 0; d < NF; d++) stay += (z0[d] - (cur[d] - lib.mean[d]) * lib.scale[d]) ** 2;
        }
        const best = late && valid(k) ? { k: -1, d: Infinity } : nearest(lib, cur, near, want, ws, penAt(t), phase, maskAt(t), condAt(t), swingAt(t));
        if (best.k >= 0 && best.d < stay * 0.9) switchTo(best.k);
      }
    }
    if (effects) {
      const m = mixAt(t), next: Record<string, number> = {};
      for (const l of new Set([...Object.keys(tw), ...Object.keys(m)])) {
        const v = (tw[l] ?? 0) + ((m[l] ?? 0) - (tw[l] ?? 0)) * ease;
        if (Math.abs(v) > 0.002 || m[l]) next[l] = v;
      }
      tw = next;
    }
    // the upper body's own match (limbs.upper): on entering, every few frames, or as its clip runs out, the frame the
    // legs' query finds among frames with the upper's tags, kept unless something is clearly better
    const cu = act ? undefined : upperAt(t);
    if (cu && (!wasU || t % every === 0 || !valid(kU))) {
      const cur = query(k, t), want = wantAt(t), pen = penAt(t), hu = upperHands(t);
      let stay = Infinity, near: [number, number] | undefined;
      if (wasU && valid(kU)) {
        const z0 = lib.features.subarray(kU * NF, (kU + 1) * NF), c = lib.clips[lib.frameClip[kU]];
        stay = stateCost(lib, kU, want, ws) + (pen ? pen[lib.frameClip[kU]] : 0) + tagCost(lib, kU, cu) + (hu ? handCost(lib, kU, hu) : 0) + (swingAt(t)?.[kU] ?? 0);
        for (let d = 0; d < NF; d++) stay += (z0[d] - (cur[d] - lib.mean[d]) * lib.scale[d]) ** 2;
        near = [Math.max(c.start, kU - 30), Math.min(c.end, kU + 8)];
      }
      const sw = swingAt(t), zq = () => Float32Array.from(cur, (v, d) => (v - lib.mean[d]) * lib.scale[d]);
      let best = hu ? nearestHands(lib, zq(), hu, near, want, ws, pen, cu, sw) : nearest(lib, cur, near, want, ws, pen, undefined, undefined, cu, sw);
      if (best.k < 0 && sw && hu) best = nearestHands(lib, zq(), hu, near, want, ws, pen, cu);
      if (best.k >= 0 && (!wasU || !(best.d >= stay * 0.9))) {
        const [b1, v1] = ahead(best.k, 0, rate);
        offU = offOnto(upperShown(best.k, 0).turns, upperShown(b1, v1).turns);
        sinceU = 0; kU = best.k; uU = 0;
      }
    }
    const onU = !!cu && kU >= 0, leaveU = wasU && !onU;
    // (an action takes the whole body: what the upper body had is let go)
    if (act) { offU = null; kU = -1; }
    const fr = { clip: lib.frameClip[k], f: lib.frameF[k], ...shownAt(k, act ? 0 : u) };
    played.push({ clip: lib.clips[fr.clip].name + (lib.clips[fr.clip].mirrored ? "~" : ""), f: fr.f });
    if (o.trace) { clipStage.push({ root: fr.root, turns: fr.turns }); rates.push(rate); }
    const f = blendAt(since, { turns: fr.turns, g: lib.ground[k * 3 + 2] });
    const gy = lib.ground[k * 3 + 2];
    const yOff = yAt(since);
    if (act?.play && align) {
      const P = act.play, rA = libraryPose(lib, P.anchor).root, lead = Math.max(1, P.anchor - P.from);
      const ease = act.j < lead ? 1 - smooth(act.j / lead) : 0;
      // (the heading the body came in with eased into the clip's on the body's turn only: turning the clip's path by it
      // would swing its start, metres before the anchor, off to the side)
      const rot = align.rot + align.ey * ease;
      const p = qrot(qyaw(align.rot), [fr.root[0] - rA[0], 0, fr.root[2] - rA[2]]);
      const turns = fr.turns.map((q2, i) => (i === 0 ? qnorm(qmul(qyaw(rot), f ? qmul(qmul(qyaw(gy), qmul(f[0], qyaw(-gy))), q2) : q2)) : f ? qnorm(qmul(f[i], q2)) : q2));
      // (stood on the floor: the clip's own floor taken off, eased in and out over its ends)
      const gr = groundOf(lib.frameClip[k]), len = act.len[0], down = Math.min(1, smooth(act.j / 8), smooth((len - 1 - act.j) / 8));
      raw.push({ root: [P.at[0] + p[0] + align.e[0] * ease, fr.root[1] + yOff - gr.floor * down, P.at[1] + p[2] + align.e[1] * ease], turns });
      const gp = qrot(qyaw(align.rot), [lib.ground[k * 3] - rA[0], 0, lib.ground[k * 3 + 1] - rA[2]]);
      x = P.at[0] + gp[0] + align.e[0] * ease;
      z = P.at[1] + gp[2] + align.e[1] * ease;
      yaw = gy + rot;
    } else if (act && align) {
      // the action's frame in the scene: the clip's place about its seat turned onto the seat, raised to its height
      const S = lib.sit!;
      const ease = act.j < act.len[0] ? 1 - smooth(act.j / act.len[0]) : 0;
      const rot = align.rot + align.ey * ease;
      const p = qrot(qyaw(rot), [fr.root[0] - S.seat[0], 0, fr.root[2] - S.seat[2]]);
      const up = act.j < act.len[0] ? smooth(act.j / act.len[0]) : act.j < act.len[0] + act.len[1] ? 1 : 1 - smooth((act.j - act.len[0] - act.len[1]) / act.len[2]);
      const dy = (act.seat!.at[1] - S.seat[1]) * up;
      const turns = fr.turns.map((q2, i) => (i === 0 ? qnorm(qmul(qyaw(rot), f ? qmul(qmul(qyaw(gy), qmul(f[0], qyaw(-gy))), q2) : q2)) : f ? qnorm(qmul(f[i], q2)) : q2));
      raw.push({ root: [act.seat!.at[0] + p[0] + align.e[0] * ease, fr.root[1] + dy + yOff, act.seat!.at[2] + p[2] + align.e[1] * ease], turns });
      // the body's ground frame follows the clip's, so the walk after carries on from where it stood up
      const gp = qrot(qyaw(rot), [lib.ground[k * 3] - S.seat[0], 0, lib.ground[k * 3 + 1] - S.seat[2]]);
      x = act.seat!.at[0] + gp[0] + align.e[0] * ease;
      z = act.seat!.at[2] + gp[2] + align.e[1] * ease;
      yaw = gy + rot;
    } else {
      // the pose in the scene: the frame's turns with its heading swapped for the body's, the hips over the body's place
      // (both u of the way on to the next library frame, so a rate other than 1 moves the body smoothly)
      const w = act ? { gx: lib.ground[k * 3], gz: lib.ground[k * 3 + 1], gy, dx: 0, dz: 0, dyaw: 0 } : onward(), by = yaw + w.dyaw;
      const turns = fr.turns.map((q2, i) => {
        if (i === 0) {
          const local = qmul(qyaw(-w.gy), q2);
          return qnorm(qmul(qyaw(by), f ? qmul(f[0], local) : local));
        }
        return f ? qnorm(qmul(f[i], q2)) : q2;
      });
      // the upper body from its own stream, and out of it back onto the body's own, blended
      if (leaveU) {
        const [k1, u1] = ahead(k, u, rate), n1 = shownAt(k1, u1), f1 = blendAt(since + 1, { turns: n1.turns, g: lib.ground[k1 * 3 + 2] });
        offU = offOnto(turns, n1.turns.map((q2, i) => (f1 ? qnorm(qmul(f1[i], q2)) : q2)));
        sinceU = 0; kU = -1;
      }
      if (onU || (offU && sinceU < bf)) {
        const src = onU ? upperShown(kU, uU).turns : undefined, O = offU && sinceU < bf ? { ...offU, y: 0, vy: 0 } : null, hs = hermite(sinceU);
        for (let i = 0; i < turns.length; i++) {
          if (!upperJ[i]) continue;
          const base = src ? src[i] : turns[i];
          turns[i] = O ? qnorm(qmul(offAt(O, i, hs), base)) : base;
        }
      }
      const hipOff = qrot(qyaw(by - w.gy), [fr.root[0] - w.gx, 0, fr.root[2] - w.gz]);
      raw.push({ root: [x + w.dx + hipOff[0], fr.root[1] + yOff, z + w.dz + hipOff[2]], turns });
    }
    // (a mirror image's feet swapped: its clip's contacts are the unmirrored capture's, and a throw played for the other
    // hand held down the foot it steps with)
    const acted = act?.play ? (groundOf(lib.frameClip[k]).contact[lib.frameF[k]].map((c) => (c === 1 ? 3 : c)) as [number, number]) : undefined;
    contacts.push(acted ? (lib.clips[lib.frameClip[k]].mirrored ? [acted[1], acted[0]] : acted) : [lib.contact[k * 2], lib.contact[k * 2 + 1]]);
    playedU.push(onU && !act ? { clip: lib.clips[lib.frameClip[kU]].name + (lib.clips[lib.frameClip[kU]].mirrored ? "~" : ""), f: lib.frameF[kU] } : null);
    if (onU && !act) [kU, uU] = ahead(kU, uU, rate);
    sinceU++;
    wasU = onU && !act;
    since++;
    if (act) {
      // the last frame of a clip played whole: the walk after starts a frame on, at the speed the clip ended with (from
      // where it ended it would show the same place twice)
      const nx = gAt(t + 1);
      if (act.play && raw.length > 1 && (!nx.act || nx.act.play !== act.play)) {
        const a0 = raw[raw.length - 2].root, a1 = raw[raw.length - 1].root;
        x += a1[0] - a0[0];
        z += a1[2] - a0[2];
      }
      continue;
    }
    // advance: the library's own steps (as many as the effects' rate makes this frame), turned into the body's
    // heading, then pulled toward the guide
    for (u += rate; u >= 1; u--) {
      const nk = k + 1 < lib.clips[lib.frameClip[k]].end ? k + 1 : k;
      if (nk === k) { u = 0; break; }
      const g0 = lib.ground[k * 3 + 2];
      const d = qrot(qyaw(-g0), [lib.ground[nk * 3] - lib.ground[k * 3], 0, lib.ground[nk * 3 + 1] - lib.ground[k * 3 + 1]]);
      const dw = qrot(qyaw(yaw), d);
      x += dw[0] * stride;
      z += dw[2] * stride;
      yaw += wrap(lib.ground[nk * 3 + 2] - g0);
      k = nk;
    }
    const g = gAt(t + 1);
    if (!g.act) {
      // against where the body shows, u of the way on to the next library frame (at a rate other than 1 the ground
      // frame moves in whole library frames, and pulling that against the guide shakes the body)
      const w = onward();
      // (a wait with slack pulled only for how far it is past it: a stop at a thing taken in passing overran its spot
      // 8 cm, was pulled back over standing feet, and at the start after them the feet were 10 cm from the clip's)
      let ex = g.x - x - w.dx, ez = g.z - z - w.dz;
      if (g.slack) { const e = Math.hypot(ex, ez), k2 = e > g.slack ? 1 - g.slack / e : 0; ex *= k2; ez *= k2; }
      x += ex * pull;
      z += ez * pull;
      yaw += wrap(g.yaw - yaw) * pull;
    }
  }
  carrySpeed(raw, switches, fps);
  smoothStill(raw, G);
  holdActionFeet(skel, raw, G, contacts);
  // relaxed elbows: the library's arms can swing locked straight
  const log: PlantLog | undefined = o.trace ? { state: [[], []], events: [], drop: [], lift: [[], []] } : undefined;
  const planted = o.pin === false ? raw : plantFeet(skel, raw, contacts, fps, log);
  const relaxed = planted.map((p) => softElbows(skel, p));
  const poses = o.gaze === false ? relaxed : leadGaze(skel, relaxed, G);
  const { phase, step } = stages(G, spec, fps);
  const perf: Performance = { frames: N, pose: (frame) => poses[Math.min(N - 1, Math.max(0, Math.round(frame)))], guide: G, played, phase, step };
  if (playedU.some(Boolean)) perf.upper = playedU;
  if (o.trace) perf.trace = { stages: { clip: clipStage, blend: raw, plant: planted, elbows: relaxed, gaze: poses }, switches, rate: rates, plant: log };
  // RAEA_LINT=1: every performance lints itself and says what it found (its trace kept for that)
  if (o.trace && lintEnv() && !o.quiet) console.warn(`[motion lint] ${spec.steps.length} steps, ${N} frames\n` + lintReport(motionLint(lib, perf)));
  performances.set(key, perf);
  if (performances.size > 64) performances.delete(performances.keys().next().value!);
  return perf;
}
