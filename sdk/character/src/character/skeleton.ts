// The skeleton: joints, rest, posing.
import { type Vec3 } from "@raea/render/3d";
import { B, I, Pose, Posed, Quat, Skeleton, add, isFinger, qfromTo, qinv, qmul, qnorm, qrot, sub } from "./all";

// ---------------------------------------------------------------- the skeleton

const indexes = new WeakMap<Skeleton, Map<string, number>>();

/** a joint's index by name (throws on a name the skeleton hasn't) */
export function joint(skel: Skeleton, name: string): number {
  let m = indexes.get(skel);
  if (!m) indexes.set(skel, (m = new Map(skel.joints.map((j, i) => [j.name, i]))));
  const i = m.get(name);
  if (i === undefined) throw new Error(`character ${skel.name}: no joint ${name} (it has ${skel.joints.map((j) => j.name).join(", ")})`);
  return i;
}

/** A body's size against the 1.26 m child the motion library's centimetres and speeds were first set on, by the legs
 * (0.594 m, hip to ankle): a capture retargeted onto a body keeps its timing and scales its travel by the legs, so a
 * man (0.869 m legs, 1.46) runs half as fast again, stands his feet higher off a capture's floor and swings them
 * faster. */
export function bodyScale(skel: Skeleton): number {
  const J = (n: string) => skel.joints[joint(skel, n)].head, d = (a: number[], b: number[]) => Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]);
  return (d(J(B.thigh("L")), J(B.calf("L"))) + d(J(B.calf("L")), J(B.foot("L")))) / 0.5944;
}

/** How many of the skeleton's joints a capture turns: the body's (bones.ts BODY_JOINTS), the fingers' coming after
 * them, and a clip's frames have a turn for each of these only. */
const capturedOf = new WeakMap<Skeleton, number>();

export function captured(skel: Skeleton): number {
  let k = capturedOf.get(skel);
  if (k === undefined) {
    k = skel.joints.findIndex((j) => isFinger(j.name));
    capturedOf.set(skel, (k = k < 0 ? skel.joints.length : k));
  }
  return k;
}

/** The body at rest: the root where it stands, no joint turned (the fingers' own joints left out, as a clip leaves
 * them: unturned). */
export function rest(skel: Skeleton): Pose {
  return { root: [...skel.joints[0].head] as Vec3, turns: skel.joints.slice(0, captured(skel)).map(() => I) };
}

/** Each joint's place and turn (forward kinematics). Parents come before their children in the skeleton. A pose that
 * turns only the body's joints (a clip's) is drawn without the fingers (`withFingers` adds them, unturned): they cost
 * more than the body, and the solvers pose a body thousands of times a run. */
export function pose(skel: Skeleton, p: Pose): Posed {
  const nc = captured(skel), n = p.turns.length > nc ? skel.joints.length : nc;
  return place(skel, p.turns, { pos: new Array(n), rot: new Array(n) }, 0, n, p.root);
}

/** A drawn pose with the fingers' own joints too, unturned, where pose leaves them out. */
export function withFingers(skel: Skeleton, w: Posed): Posed {
  const n = skel.joints.length;
  return w.pos.length >= n ? w : place(skel, [], { pos: w.pos.slice(), rot: w.rot.slice() }, w.pos.length, n, w.pos[0]);
}

export function place(skel: Skeleton, turns: Quat[], w: Posed, from: number, to: number, root: Vec3): Posed {
  const J = skel.joints, { pos, rot } = w;
  for (let i = from; i < to; i++) {
    const t = turns[i] ?? I, par = J[i].parent;
    if (par < 0) {
      rot[i] = t;
      pos[i] = root;
    } else {
      rot[i] = qmul(rot[par], t);
      pos[i] = add(pos[par], qrot(rot[par], sub(J[i].head, J[par].head)));
    }
  }
  return w;
}

/** A copy of a pose with joint i's turn in the body's frame made `world` (its children ride with it). */
export function setWorld(skel: Skeleton, p: Pose, w: Posed, i: number, world: Quat): Pose {
  const par = skel.joints[i].parent;
  if (par >= w.rot.length || i >= w.rot.length) w = withFingers(skel, w);
  const turns = p.turns.slice();
  turns[i] = qnorm(qmul(qinv(par < 0 ? I : w.rot[par]), world));
  return { ...p, turns };
}

/** A held pose from bone aims: each named bone pointed along a direction in the body's frame (x its left, y up, z
 * ahead), parents before children, so a pose reads as plain directions. A bone's direction is from its head to its
 * tail. */
export function held(skel: Skeleton, aims: Record<string, Vec3>, from: Pose = rest(skel)): Pose {
  let p = from;
  for (const [name, dir] of Object.entries(aims).sort((a, b) => joint(skel, a[0]) - joint(skel, b[0]))) p = aim(skel, p, joint(skel, name), dir);
  return p;
}

/** Joint i turned (the shortest way, from how it is now) so its bone points along `dir` in the body's frame. */
export function aim(skel: Skeleton, p: Pose, i: number, dir: Vec3): Pose {
  const w = pose(skel, p);
  const j = skel.joints[i];
  const now = qrot(w.rot[i], sub(j.tail, j.head));
  return setWorld(skel, p, w, i, qmul(qfromTo(now, dir), w.rot[i]));
}
