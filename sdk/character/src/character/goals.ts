// Goals (inverse kinematics): two-bone reaches, elbows, hands.
import { type Vec3 } from "@raea/render/3d";
import { I, Pose, Posed, Quat, Skeleton, add, cross, dot, fingerJoints, joint, len, norm, pose, qaxis, qbasis, qfromTo, qinv, qmul, qrot, scale3, setWorld, slerp, sub } from "./all";

// ---------------------------------------------------------------- goals (inverse kinematics)

export type Side = "L" | "R";

export const LIMB = {
  arm: (s: Side) => (s === "L" ? ["LeftArm", "LeftForeArm", "LeftHand"] : ["RightArm", "RightForeArm", "RightHand"]),
  leg: (s: Side) => (s === "L" ? ["LeftUpLeg", "LeftLeg", "LeftFoot"] : ["RightUpLeg", "RightLeg", "RightFoot"]),
};

/** Two bones (a hip, knee, ankle or a shoulder, elbow, wrist) bent so the end comes to `target`. The bend stays in the
 * plane it is in (so a clip's knees keep bending its way), unless `pole` says which way the middle joint points (a
 * knee ahead, an elbow down). Out of reach, the limb points straight at the target. The end joint keeps its turn in the
 * body's frame. */
export function twoBone(skel: Skeleton, p: Pose, names: string[], target: Vec3, pole?: Vec3, o: { keep?: boolean } = {}): Pose {
  const [a, b, c] = names.map((n) => joint(skel, n));
  let w = pose(skel, p);
  const endRot = w.rot[c];
  const pa = w.pos[a], pb = w.pos[b], pc = w.pos[c];
  const la = len(sub(pb, pa)), lb = len(sub(pc, pb));
  const t = sub(target, pa);
  // soft near full stretch (from 97% eased toward 99.5%, about 11° short of straight), so a target moving about a
  // straight limb doesn't snap the middle joint and the limb never locks. With `keep`, only past the limb's own reach
  // now: a clip's leg straight at heel strike stays straight (softened from 97%, every heel strike was bent 15° more)
  const L = la + lb, now = len(sub(pc, pa)), soft = o.keep ? Math.max(0.97 * L, Math.min(now, 0.9995 * L)) : 0.97 * L;
  const top = Math.max(0.995 * L, soft + 1e-4), raw = len(t);
  const reachD = raw > soft ? soft + (top - soft) * (1 - Math.exp(-(raw - soft) / (top - soft))) : raw;
  const d = Math.min(Math.max(reachD, Math.abs(la - lb) + 1e-4), L - 1e-4);
  const tn = norm(t);
  // the bend's side: the pole, or where the middle joint is now, off the line to the target. A nearly straight limb's
  // middle joint is off the line by noise, so that eases over to the way it bends (a knee ahead, an elbow behind, turned
  // with the limb's first bone)
  const off = (v: Vec3) => sub(v, scale3(tn, dot(v, tn)));
  let side = off(pole ?? sub(pb, pa));
  if (!pole) {
    const anat = off(qrot(w.rot[a], [0, 0, /Arm$/.test(skel.joints[a].name) ? -1 : 1]));
    const u = Math.min(1, len(side) / (0.12 * la)), k = u * u * (3 - 2 * u);
    side = add(scale3(norm(side), k), scale3(norm(anat), 1 - k));
  }
  if (len(side) < 1e-6) side = sub([0, 0, 1], scale3(tn, tn[2]));
  side = norm(side);
  const cosA = (la * la + d * d - lb * lb) / (2 * la * d);
  const sinA = Math.sqrt(Math.max(0, 1 - cosA * cosA));
  const mid = add(pa, add(scale3(tn, la * cosA), scale3(side, la * sinA)));
  p = setWorld(skel, p, w, a, qmul(qfromTo(sub(pb, pa), sub(mid, pa)), w.rot[a]));
  w = pose(skel, p);
  p = setWorld(skel, p, w, b, qmul(qfromTo(sub(w.pos[c], w.pos[b]), sub(add(pa, scale3(tn, d)), w.pos[b])), w.rot[b]));
  w = pose(skel, p);
  return setWorld(skel, p, w, c, endRot);
}

/** A pose with its elbows never quite straight: a bend under twice `least` degrees eases up toward `least` (a straight
 * arm gets `least`, one bent twice that or more is left alone), about the way the elbow bends, so arms a clip swings
 * locked straight (100STYLE's Neutral walks at 6° on average) hang relaxed. */
export function softElbows(skel: Skeleton, p: Pose, least = 12): Pose {
  for (const s of ["L", "R"] as const) {
    const [a, b, c] = LIMB.arm(s).map((n) => joint(skel, n));
    const w = pose(skel, p);
    const up = sub(w.pos[b], w.pos[a]), fore = sub(w.pos[c], w.pos[b]);
    // the bend about the elbow's hinge only (it bends the forearm ahead), signed: past straight the other way is less
    // than none. (Taking the axis from the arm's own bend, it flipped as the arm swung through straight and the forearm
    // twisted 16-20° a frame: the motion lint's finding.)
    const un = norm(up), hinge = norm(cross(un, qrot(w.rot[a], [0, 0, 1])));
    const fp = sub(fore, scale3(hinge, dot(fore, hinge)));
    const bent = (Math.atan2(dot(cross(un, fp), hinge), dot(un, fp)) * 180) / Math.PI;
    if (bent >= 2 * least) continue;
    const want = bent <= 0 ? least : least + (bent * bent) / (4 * least);
    p = setWorld(skel, p, w, b, qmul(qaxis(hinge, ((want - bent) * Math.PI) / 180), w.rot[b]));
  }
  return p;
}

/** The hand's own frame at rest: its length (wrist to knuckles), its across (thumb side out) and its palm's normal. */
export function handRest(skel: Skeleton, s: Side) {
  const J = skel.joints;
  const h = J[joint(skel, s === "L" ? "LeftHand" : "RightHand")];
  const knuckles = J[joint(skel, s === "L" ? "LeftFingerBase" : "RightFingerBase")].tail;
  const thumb = J[joint(skel, s === "L" ? "LThumb" : "RThumb")].head;
  const along = norm(sub(knuckles, h.head));
  const across = norm(sub(sub(thumb, h.head), scale3(along, dot(sub(thumb, h.head), along))));
  // the palm faces away from the back of the hand: for the left hand, across x along is the palm side (checked on a
  // render of a hand laid flat)
  const palm = norm(s === "L" ? cross(along, across) : cross(across, along));
  return { along, across, palm, centre: scale3(add(h.head, knuckles), 0.5) };
}

/** A hand laid flat: fingers along `point` (in the body's frame, levelled), the palm facing `facing` (down by
 * default). */
export function flatHand(skel: Skeleton, p: Pose, s: Side, point: Vec3, facing: Vec3 = [0, -1, 0]): Pose {
  const i = joint(skel, s === "L" ? "LeftHand" : "RightHand");
  const r = handRest(skel, s);
  const n = norm(facing);
  const f = norm(sub(point, scale3(n, dot(point, n))));
  // the frame (along, palm, along x palm) at rest goes to (f, n, f x n)
  const R0 = qbasis(r.along, r.palm, cross(r.along, r.palm));
  const R1 = qbasis(f, n, cross(f, n));
  const w = pose(skel, p);
  p = setWorld(skel, p, w, i, qmul(R1, qinv(R0)));
  // the fingers and thumb straight along the hand (as they are at rest), whatever the clip had them doing
  const turns = p.turns.slice();
  for (const n of s === "L" ? ["LeftFingerBase", "LeftHandFinger1", "LThumb"] : ["RightFingerBase", "RightHandFinger1", "RThumb"]) turns[joint(skel, n)] = I;
  for (const i of fingerJoints(skel, s).flat()) if (i < turns.length) turns[i] = I;
  return { ...p, turns };
}

/** A hand cupped round a ball of radius `r` (metres), by `w` (0 open, 1 closed on it): the fingers curled toward the
 * palm at their base and again along them, the thumb turned in across the palm. CMU's hands are captured flat, and a
 * ball laid on a flat palm read as floating off it in an overhand throw (Shane, 10-22). */
export function cupHand(skel: Skeleton, p: Pose, s: Side, r: number, w = 1): Pose {
  const { along, across } = handRest(skel, s), len = 2 * Math.hypot(...sub(handRest(skel, s).centre, skel.joints[joint(skel, s === "L" ? "LeftHand" : "RightHand")].head));
  // (a smaller ball against the same hand closes it further: about 60° at the knuckles for a ball a hand long across)
  const k = Math.min(1.3, Math.max(0.6, len / (2 * r)));
  const sg = s === "L" ? -1 : 1, turns = p.turns.slice();
  // (in place of the capture's own finger turns, by w: CMU's fingers are a marker or two, and a throw's flick bent them
  // back past straight with the ball in them)
  const at = (n: string, q: Quat) => { const i = joint(skel, n); turns[i] = slerp(turns[i], q, Math.min(1, w)); };
  const L = s === "L" ? "Left" : "Right";
  at(`${L}FingerBase`, qaxis(across, sg * 0.9 * k));
  at(`${L}HandFinger1`, qaxis(across, sg * 0.7 * k));
  at(s === "L" ? "LThumb" : "RThumb", qaxis(along, -sg * 0.6 * k));
  return { ...p, turns };
}

/** Where a ball of radius `r` sits in a cupped hand (cupHand): off the palm by its radius and toward the fingers that
 * close on it. */
export function ballInHand(skel: Skeleton, w: Posed, s: Side, r: number): Vec3 {
  const i = joint(skel, s === "L" ? "LeftHand" : "RightHand"), hr = handRest(skel, s);
  const len = 2 * Math.hypot(...sub(hr.centre, skel.joints[i].head));
  return add(palmAt(skel, w, s), add(scale3(palmFacing(skel, w, s), r + 0.004), scale3(qrot(w.rot[i], hr.along), 0.2 * len)));
}

/** where the middle of a hand's palm is, posed */
export function palmAt(skel: Skeleton, w: Posed, s: Side): Vec3 {
  const i = joint(skel, s === "L" ? "LeftHand" : "RightHand");
  const r = handRest(skel, s);
  return add(w.pos[i], qrot(w.rot[i], sub(r.centre, skel.joints[i].head)));
}

/** the way a hand's fingers point (from the wrist to the knuckles), posed */
export function handAlong(skel: Skeleton, w: Posed, s: Side): Vec3 {
  return qrot(w.rot[joint(skel, s === "L" ? "LeftHand" : "RightHand")], handRest(skel, s).along);
}

/** the way a hand's palm faces, posed */
export function palmFacing(skel: Skeleton, w: Posed, s: Side): Vec3 {
  return qrot(w.rot[joint(skel, s === "L" ? "LeftHand" : "RightHand")], handRest(skel, s).palm);
}
