// The body's trunk turned, crouched and stooped, and its fingers.
import type { Vec3 } from "raea/3d";
import { B, FINGERS, I, handRest, add, joint, len, twoBone, palmAt, pose, qaxis, qmul, qrot, qyaw, reach, scale3, setWorld, slerp, sub, type HandGoal, type Pose, type Side, type Skeleton } from "raea/character";
import { lerp } from "./all";

/** The trunk turned: twisted about up (`twist`, + to the body's left), pitched about the body's left (`pitch`, +
 * forward) and bent sideways about its forward (`roll`, + toward its right), shared up the back from the lower back,
 * so the shoulders and arms ride with it. */
export function trunkTurn(skel: Skeleton, p: Pose, yaw: number, twist: number, pitch: number, roll = 0): Pose {
  if (!twist && !pitch && !roll) return p;
  const chain = [B.spine[0], B.spine[1], B.chest].map((n) => joint(skel, n)), share = [0.25, 0.35, 0.4];
  const left = qrot(qyaw(yaw), [1, 0, 0]), ahead = qrot(qyaw(yaw), [0, 0, 1]);
  for (let c = 0; c < 3; c++) {
    const w = pose(skel, p);
    const turn = qmul(qmul(qaxis([0, 1, 0], twist * share[c]), qaxis(left, pitch * share[c])), qaxis(ahead, roll * share[c]));
    p = setWorld(skel, p, w, chain[c], qmul(turn, w.rot[chain[c]]));
  }
  return p;
}

/** The hands on their goals, the back bent toward them (up to `lean` radians), and the knees bent too (the hips
 * lowered, the feet where they stand) as far as the hands need to reach a low thing: a little, and no more, for a
 * table; down to a squat for the floor. */
export function crouched(skel: Skeleton, p: Pose, hands: Partial<Record<Side, HandGoal>>, drop: number, lean = 0.6, stoopAt?: number, back = 0): Pose {
  return crouchedAt(skel, p, hands, drop, lean, stoopAt, back).pose;
}

/** crouched, and how far it stooped (`stoopAt` a stoop already found, else as far as the hands need up to `lean`), the
 * hips `back` metres further back over the feet (a load balanced before the body) */
export function crouchedAt(skel: Skeleton, p: Pose, hands: Partial<Record<Side, HandGoal>>, drop: number, lean: number, stoopAt?: number, back = 0): { pose: Pose; stoop: number } {
  // (on the floor as far as it squats: a carry at a run keeps its stride's feet, in the air over a hurdle too)
  const down = Math.min(1, Math.max(0, drop / 0.06));
  // (a foot mid-step, over 2.5 cm up, stays where the step has it: put on the floor, it was dragged along the floor
  // the whole step, errands' turn at the desk sliding 11 cm)
  const w0 = pose(skel, p), feet = (["L", "R"] as Side[]).map((s) => { const i = joint(skel, B.foot(s)), a = w0.pos[i], up = a[1] - skel.joints[i].head[1]; return [a[0], up > 0.025 ? up : up * (1 - down), a[2]] as Vec3; });
  // the hips go back as they go down (a squat sits back over the heels, balancing the knees ahead)
  let q: Pose = { ...p, root: add(p.root, add([0, -drop, 0], scale3(backOf(p), 0.25 * drop + back))) };
  let th = 0, axis: Vec3 = [1, 0, 0];
  // (a stoop already found is kept whatever the lean: eased out over the frames after a pick, it vanished in one the
  // moment the carry's lean of none took over)
  if (Object.keys(hands).length && (lean > 0 || (stoopAt ?? 0) > 0)) ({ pose: q, th, axis } = stoop(skel, q, hands, lean, stoopAt));
  // the feet held where they are (a foot in the air too: put on the floor, a step in the drop's ease snapped down),
  // turned as they are (laid flat they would turn the moment the hips start down)
  // (the knees forward over the toes and a little out, as in a squat: left to itself the leg can fold the other way
  // once the pelvis tips, into a kneel)
  if (drop > 0 || q !== p) {
    const ahead = qrot(p.turns[0], [0, 0, 1]), yaw = Math.atan2(ahead[0], ahead[2]);
    // the hips no higher than lets both feet stand flat on the floor (sat back and bent over, the legs would stretch
    // and leave the body on its toes)
    const legs = (["L", "R"] as Side[]).map((s, i) => {
      const [u, k, f] = B.leg(s).map((n) => joint(skel, n));
      const J = skel.joints, L = 0.985 * (len(sub(J[k].head, J[u].head)) + len(sub(J[f].head, J[k].head)));
      const hip = pose(skel, q).pos[u], ankle: Vec3 = [feet[i][0], J[f].head[1] + feet[i][1], feet[i][2]];
      const dh = Math.hypot(hip[0] - ankle[0], hip[2] - ankle[2]), dv = hip[1] - ankle[1];
      return dh < L ? Math.max(0, dv - Math.sqrt(L * L - dh * dh)) : dv;
    });
    const sink = Math.max(...legs);
    if (sink > 0) q = { ...q, root: [q.root[0], q.root[1] - sink, q.root[2]] };
    (["L", "R"] as Side[]).forEach((s, i) => {
      const fi = joint(skel, B.foot(s)), knee = qrot(qyaw(yaw), [s === "L" ? 0.25 : -0.25, 0, 1]);
      // (the ankle at its standing height over the floor: the foot flat)
      q = twoBone(skel, q, B.leg(s), [feet[i][0], skel.joints[fi].head[1] + feet[i][1], feet[i][2]], knee);
      // the foot flat on the floor, turned the way its toes point (not tipped onto its toes with the shin, nor up on
      // them as the walk had it mid-step)
      const ti = joint(skel, B.ball(s)), restDir = sub(skel.joints[ti].head, skel.joints[fi].head), now = qrot(w0.rot[fi], restDir);
      const wq = pose(skel, q);
      q = setWorld(skel, q, wq, fi, slerp(w0.rot[fi], qyaw(Math.atan2(now[0], now[2]) - Math.atan2(restDir[0], restDir[2])), down));
    });
  }
  for (const [s, g] of Object.entries(hands) as [Side, HandGoal][]) q = reach(skel, q, s, g);
  // a free arm keeps hanging the way it did (it would swing up behind with the stooping back): turned back by most
  // of the stoop
  if (th > 0) for (const s of ["L", "R"] as Side[]) {
    if (hands[s]) continue;
    const i = joint(skel, B.upperarm(s)), w = pose(skel, q);
    q = setWorld(skel, q, w, i, qmul(qaxis(axis, -0.85 * th), w.rot[i]));
  }
  return { pose: q, stoop: th };
}

/** backward along the floor from the hips' heading */
function backOf(p: Pose): Vec3 {
  const f = qrot(p.turns[0], [0, 0, 1]), l = Math.hypot(f[0], f[2]) || 1;
  return [-f[0] / l, 0, -f[2] / l];
}

/** The body bent toward the hands' goals until every palm is in easy reach (0.85 of the arm from its shoulder), up to
 * `most` radians, the way a body stoops: most of it at the hips (the pelvis tipping forward over the thighs), the rest
 * up the back. (leanToward bends the back alone, which for a reach to the floor brings the shoulders down too little.) */
export function stoop(skel: Skeleton, p: Pose, hands: Partial<Record<Side, HandGoal>>, most: number, fixed?: number): { pose: Pose; th: number; axis: Vec3 } {
  const goals = Object.entries(hands) as [Side, HandGoal][];
  const J = skel.joints, chain = [B.spine[0], B.spine[1], B.chest].map((n) => joint(skel, n)), share = [0.2, 0.2, 0.15], hipShare = 0.45;
  const arm = (s: Side) => joint(skel, B.upperarm(s));
  const armLen = (s: Side) => { const [a, b, c] = B.arm(s).map((n) => J[joint(skel, n)].head); return len(sub(b, a)) + len(sub(c, b)); };
  const w0 = pose(skel, p);
  const mid = scale3(add(w0.pos[arm("L")], w0.pos[arm("R")]), 0.5);
  const toward = goals.reduce<Vec3>((acc, [, g]) => add(acc, sub(g.at, mid)), [0, 0, 0]);
  const flat = Math.hypot(toward[0], toward[2]);
  if (flat < 1e-6) return { pose: p, th: 0, axis: [1, 0, 0] };
  // about the level axis across the way to the goals
  const axis: Vec3 = [toward[2] / flat, 0, -toward[0] / flat];
  // and further back the further it bends over (the weight of the back leaning ahead balanced by the hips behind)
  const trunk = len(sub(J[arm("L")].head, J[0].head)), back = backOf(p);
  const bent = (th: number) => {
    let q: Pose = { ...p, root: add(p.root, scale3(back, 0.2 * trunk * Math.sin(th))), turns: [qmul(qaxis(axis, th * hipShare), p.turns[0]), ...p.turns.slice(1)] };
    for (let c = 0; c < 3; c++) { const w = pose(skel, q); q = setWorld(skel, q, w, chain[c], qmul(qaxis(axis, th * share[c]), w.rot[chain[c]])); }
    return q;
  };
  const far = (q: Pose) => { const w = pose(skel, q); return Math.max(...goals.map(([s, g]) => len(sub(g.at, w.pos[arm(s)])) / armLen(s))); };
  if (fixed !== undefined) return { pose: fixed > 0 ? bent(fixed) : p, th: fixed, axis };
  if (far(p) <= 0.85) return { pose: p, th: 0, axis };
  let lo = 0, hi = most;
  if (far(bent(hi)) > 0.85) return { pose: bent(hi), th: hi, axis };
  for (let k = 0; k < 12; k++) { const m = (lo + hi) / 2; if (far(bent(m)) > 0.85) lo = m; else hi = m; }
  return { pose: bent(hi), th: hi, axis };
}

/** The least drop of the hips that brings the hands to their goals (a third of the body's height at most, a full
 * squat), found on one frame: interact smooths it over the
 * frames, since the least drop jumps as the walk under it moves. */
export function crouchDrop(skel: Skeleton, p: Pose, hands: Partial<Record<Side, HandGoal>>, lean: number, bend = 0.5): { drop: number; stoop: number } {
  // the knees bend by the `bend` share of how low the hands go below the hips, whether the back could have done it
  // alone or not, so a squat is a squat and not a stoop that runs out of back
  const hipsY = skel.joints[0].head[1], lowest = Math.min(...Object.values(hands).map((g) => g!.at[1] - p.root[1] + hipsY));
  // (from a little above the hips: a table at the waist is bent to at the knees as well as leaned over)
  const least = Math.max(0, Math.min(0.38 * skel.height, bend * 0.6 * (1.0 * hipsY - lowest)));
  // and the bend's hold on the back only for a thing well down; at a table the lean stops short of folding over it
  const low = Math.min(1, Math.max(0, (0.9 * hipsY - lowest) / (0.9 * hipsY - 0.05)));
  lean = lerp(Math.min(1.1, lean + 0.4), lean, low);
  let at = (d: number) => crouchedAt(skel, p, hands, Math.max(least, d), lean);
  const miss = (q: Pose) => { const w = pose(skel, q); return Math.max(...(Object.entries(hands) as [Side, HandGoal][]).map(([s, g]) => { const pa = palmAt(skel, w, s); return Math.hypot(pa[0] - g.at[0], pa[1] - g.at[1], pa[2] - g.at[2]); })); };
  const found = (d: number) => ({ drop: Math.max(least, d), stoop: at(d).stoop });
  const m0 = miss(at(0).pose);
  // (nor for hands held above the hips, a thing carried: a squat takes the shoulders away from them; a man's hold
  // just short at one frame of a run found its reach at a full squat, and smoothed over the frames round it sank him
  // 10 cm through a hurdle)
  if (m0 < 0.015 || lowest > hipsY) return found(0);
  let lo = 0, hi = 0.38 * skel.height;
  // still out of reach at the lowest: lean over further before giving up, a squat that reaches forward
  for (let more = 0.1; more <= 0.2 && miss(at(hi).pose) > 0.015; more += 0.1) {
    const l2 = Math.min(1.5, lean + more);
    at = (d: number) => crouchedAt(skel, p, hands, Math.max(least, d), l2);
  }
  if (miss(at(hi).pose) > 0.015) {
    // out of reach however low: as low as brings the hands nearest (none at all if bending the knees doesn't help: a
    // hold just out of reach before the chest)
    let best = 0, bm = m0;
    for (let k = 1; k <= 6; k++) { const m = miss(at((hi * k) / 6).pose); if (m < bm - 0.01) { bm = m; best = (hi * k) / 6; } }
    return found(best);
  }
  for (let k = 0; k < 8; k++) { const m = (lo + hi) / 2; if (miss(at(m).pose) > 0.015) lo = m; else hi = m; }
  return found(hi);
}

/** The fingers and thumb closed round what each hand holds, `amount` of a loose fist's curl. */
export function fingers(skel: Skeleton, p: Pose, amounts: Partial<Record<Side, number>>): Pose {
  const turns = p.turns.slice();
  for (const [s, amount] of Object.entries(amounts) as [Side, number][]) {
    if (!(amount > 0)) continue;
    // the fingers curl about the hand's across axis at rest (the knuckles a little, the two joints past them more),
    // the thumb in about the hand's length
    const { along, across } = handRest(skel, s), sign = s === "L" ? -1 : 1;
    while (turns.length < skel.joints.length) turns.push(I);
    for (const f of FINGERS) for (const k of [1, 2, 3] as const) {
      const i = joint(skel, B.finger(f, k, s));
      turns[i] = qmul(turns[i], f === "thumb" ? qaxis(along, -sign * 0.4 * amount) : qaxis(across, sign * (k === 1 ? 0.5 : 0.55) * amount));
    }
  }
  return { ...p, turns };
}
