// Fingers: hand shapes and closing on a thing.
import { type Vec3 } from "@raea/render/3d";
import { B, FINGERS, Finger, FingerSegment, alongPath, captured, capturedHand, I, pathHand, Pose, Posed, Quat, Side, Skeleton, add, cross, dot, fingerSegments, handRest, joint, norm, place, pose, qaxis, qmul, qrot, scale3, slerp, sub, surface, withFingers } from "./all";

// ---------------------------------------------------------------- fingers

/** Each finger's own joints on a side (thumb first), phalanx 1 to 3; empty for a skeleton without them. */
export function fingerJoints(skel: Skeleton, s: Side): number[][] {
  if (!skel.joints.some((j) => j.name === B.finger("index", 1, s))) return [];
  return FINGERS.map((f) => ([1, 2, 3] as const).map((k) => joint(skel, B.finger(f, k, s))));
}

/** A hand's shape: each finger's curl at its three joints (radians; + bends toward the palm, about each bone's own
 * hinge, so a fist is about [1.4, 1.6, 1.1] on the fingers), how far the fingers fan apart (radians at the index and
 * pinky, + spreads), and the thumb's opposition (radians about the hand's length, + swings it under the palm toward
 * the fingers, as a pinch or a grip round a glass's stem wants). A finger left out is straight. */
/** A hand at ease, as a hanging hand rests: the fingers a little curled, more toward the pinky, the thumb in a little
 * (for clips without fingers: 100STYLE's and most of CMU's) */
export const RELAXED_HAND: HandShape = {
  index: [0.2, 0.3, 0.15], middle: [0.28, 0.38, 0.2], ring: [0.34, 0.44, 0.24], pinky: [0.4, 0.5, 0.28],
  thumb: [0.1, 0.2, 0.1], spread: 0.04, thumbOpp: 0.25,
};

/** the captured hand a hanging hand rests in (Godot XR Tools' "Rounded": the fingers curled at all three joints, more
 * toward the pinky; RELAXED_HAND's curls left the middle and end joints near straight, a flat hand) */
export const RELAXED_CAPTURE = "Rounded";

/** a pose with both hands at ease (RELAXED_CAPTURE on a body with fingers, else RELAXED_HAND) where it has no finger
 * turns of its own */
export function relaxHands(skel: Skeleton, p: Pose): Pose {
  if (p.turns.length > captured(skel)) return p;
  if (fingerJoints(skel, "L").length) return capturedHand(skel, capturedHand(skel, p, "L", RELAXED_CAPTURE), "R", RELAXED_CAPTURE);
  return handPose(skel, handPose(skel, p, "L", RELAXED_HAND), "R", RELAXED_HAND);
}

export interface HandShape {
  thumb?: [number, number, number];
  index?: [number, number, number];
  middle?: [number, number, number];
  ring?: [number, number, number];
  pinky?: [number, number, number];
  spread?: number;
  thumbOpp?: number;
}

const handAxes = new WeakMap<Skeleton, Partial<Record<Side, { hinge: Vec3[][]; spread: Vec3; opp: Vec3 }>>>();

/** each phalanx's hinge at rest (across its bone and the palm's normal, signed so + curls it toward the palm; the
 * thumb's toward the palm and the fingers both, the way its pad faces), the fan's axis and the thumb's opposition's */
function axesOf(skel: Skeleton, s: Side) {
  let m = handAxes.get(skel);
  if (!m) handAxes.set(skel, (m = {}));
  if (m[s]) return m[s]!;
  const J = skel.joints, r = handRest(skel, s), F = fingerJoints(skel, s);
  const dir = (i: number) => norm(sub(J[i].tail, J[i].head));
  const hinge = F.map((ch, f) => ch.map((i) => norm(cross(dir(i), f === 0 ? norm(sub(r.palm, r.across)) : r.palm))));
  // the fan about the palm's normal, + taking the index toward the thumb's side; the opposition about the hand's length,
  // + taking the thumb's tip toward the palm
  const spread = scale3(r.palm, Math.sign(dot(cross(r.palm, dir(F[1][0])), r.across)) || 1);
  const opp = scale3(r.along, Math.sign(dot(cross(r.along, dir(F[0][0])), r.palm)) || 1);
  return (m[s] = { hinge, spread, opp });
}

/** How far a hand's joints turn (radians; from straight, + toward the palm), a hand's own: the fingers' knuckle (MCP)
 * 90° bent and 30° back, the middle joint (PIP) 105° and 5° back, the end joint (DIP) 80° and 15° back; the thumb's
 * base (CMC) 50° and 20° back, its knuckle (MCP) 55° and 10° back, its end joint (IP) 80° and 20° back; the fingers
 * fanned 20° either way at the index and pinky; the thumb's opposition from 40° back past the hand's side (as far as a
 * hand flat under a box swings it out) to 70° under the palm. A shape past them (a grip's search closing a finger 1.7
 * at each joint with nothing in the way, or swinging the thumb 86° out behind the hand) drove the fingertips through
 * the palm and out the back of the hand. */
export const HAND_LIMITS = {
  finger: [[-0.52, 1.57], [-0.09, 1.83], [-0.26, 1.4]] as [number, number][],
  thumb: [[-0.35, 0.87], [-0.17, 0.96], [-0.35, 1.4]] as [number, number][],
  spread: [-0.35, 0.35] as [number, number],
  thumbOpp: [-0.7, 1.22] as [number, number],
};

const clampTo = (x: number, [lo, hi]: [number, number]) => Math.max(lo, Math.min(hi, x));

/** A hand shaped: each finger's and the thumb's joints curled as `h` says (held inside HAND_LIMITS), in place of the
 * capture's own fingers (the finger base, finger and thumb it turns are set straight, so the shape is the hand's
 * whatever the clip had them doing). For a skeleton without fingers of its own, the pose as it was. */
export function handPose(skel: Skeleton, p: Pose, s: Side, h: HandShape, only?: readonly Finger[]): Pose {
  const F = fingerJoints(skel, s);
  if (!F.length) return p;
  const ax = axesOf(skel, s), turns = p.turns.slice();
  while (turns.length < skel.joints.length) turns.push(I);
  // (the fan: the index out the most, the middle barely, the ring and pinky the other way)
  const fan = [0, 1, 0.15, -0.55, -1];
  // the thumb `t` of the way to its shape (its curl and its swing under the palm; a thumb curled back or swung out
  // stays so)
  const thumbAt = (t: number) => F[0].forEach((i, ph) => {
    const c = Math.min(t * (h.thumb?.[ph] ?? 0), h.thumb?.[ph] ?? 0), o = Math.min(t * (h.thumbOpp ?? 0), h.thumbOpp ?? 0);
    let q = qaxis(ax.hinge[0][ph], clampTo(c, HAND_LIMITS.thumb[ph]));
    if (ph === 0 && o) q = qmul(qaxis(ax.opp, clampTo(o, HAND_LIMITS.thumbOpp)), q);
    turns[i] = q;
  });
  FINGERS.forEach((f, k) => {
    if (k === 0 || (only && !only.includes(f))) return;
    const c = h[f] ?? [0, 0, 0];
    F[k].forEach((i, ph) => {
      let q = qaxis(ax.hinge[k][ph], clampTo(c[ph], HAND_LIMITS.finger[ph]));
      if (ph === 0 && h.spread) q = qmul(qaxis(ax.spread, fan[k] * clampTo(h.spread, HAND_LIMITS.spread)), q);
      turns[i] = q;
    });
  });
  if (only && !only.includes("thumb")) return { ...p, turns };
  thumbAt(1);
  // the thumb rests on the fingers, never in them: closed as far as its skin clears the fingers' (a full wrap swung it
  // under the palm through the index's knuckle, its tip out the back of the hand; Shane, 10-22: "column 2 of the grips
  // is clipping the thumb")
  if (skel.surface && (h.thumbOpp ?? 0) > 0 && thumbInFingers(skel, { ...p, turns }, s)) {
    let lo = 0, hi = 1;
    for (let k = 0; k < 8; k++) { const m = (lo + hi) / 2; thumbAt(m); if (thumbInFingers(skel, { ...p, turns }, s)) hi = m; else lo = m; }
    thumbAt(lo);
  }
  return { ...p, turns };
}

/** whether a hand's thumb skin goes into its fingers' (more than a millimetre into a finger's capsule) */
function thumbInFingers(skel: Skeleton, p: Pose, s: Side): boolean {
  const w = pose(skel, p), segs = fingerSegments(skel, w, s).slice(1).flatMap((f) => f.segments);
  for (const x of surface(skel, w, /^thumb[23]$/, s)) {
    for (const g of segs) {
      const ab = sub(g.tail, g.head), t = Math.max(0, Math.min(1, dot(sub(x.at, g.head), ab) / Math.max(1e-9, dot(ab, ab))));
      const q = add(g.head, scale3(ab, t)), d = sub(x.at, q);
      if (dot(d, d) < (g.radius - 0.001) ** 2) return true;
    }
  }
  return false;
}

/** How a hand closes on a thing (closeOn; items.ts's grips): the fingers that close on it (`on`), the shape the
 * others hold (`rest`), how far one may curl at a joint (`max`, radians), the fan and the thumb's opposition. */
export interface Closing {
  on: readonly Finger[]; rest?: Partial<Record<Finger, [number, number, number]>>; max: number; spread: number; opp: number;
  /** captured hand poses, open to closed (HAND_POSES' names): the fingers close along them, each joint's number its
   * place on the path (0 the first pose, 1 the next, ...) rather than a curl, `rest` and `max` too, and the fan and
   * the thumb's swing are the poses' own (Shane, 11-01: the made-up curls looked wrong; captured hands don't) */
  path?: readonly string[];
  /** with a path, the thumb closing as a curl and a swing under the palm instead (radians: how far it curls, and its
   * opposition): a power grip's thumb goes round the far side of a can, where the captured grips lay it along the
   * fingers and nothing held the can against them */
  thumb?: { max: number; opp: number };
}

// how a finger curls as one while it closes: the middle joint the most, the tip the least (the thumb's knuckle most)
const CURL: [number, number, number] = [1, 1.1, 0.75], THUMB_CURL: [number, number, number] = [0.45, 0.8, 0.6];
// (along a captured path the poses already move the joints together)
const EVEN: [number, number, number] = [1, 1, 1];

/** how far a finger's skin may press into a thing it rests on (soft skin gives a few millimetres: ContactOpt's
 * 2-3 mm), metres */
export const SKIN_GIVE = 0.0015;

// how far a held finger's joint (or the thumb's opposition) turns a frame at most toward the frame's fit, radians
// (where that is in the thing it opens at once, as little as clears it)
const STEP = 0.1;

// each finger's skin samples (thumb first) and the skin joints they ride, to try a finger's curl on without posing
// and skinning the whole body
const fingerSkin = new WeakMap<Skeleton, Partial<Record<Side, { pts: { ph: number; j: number[]; w: number[]; o: Vec3; tw: number }[]; sj: number[] }[]>>>();

function fingerSkinOf(skel: Skeleton, s: Side) {
  let m = fingerSkin.get(skel);
  if (!m) fingerSkin.set(skel, (m = {}));
  if (m[s]) return m[s]!;
  return (m[s] = FINGERS.map((f) => {
    const pts = skel.surface!.filter((x) => x.side === s && (x.part === `${f}1` || x.part === `${f}2` || x.part === `${f}3`))
      .map((x) => ({ ph: Number(x.part.slice(-1)) - 1, j: x.j, w: x.w, o: x.o, tw: x.w.reduce((a, b) => a + b, 0) }));
    return { pts, sj: [...new Set(pts.flatMap((x) => x.j))] };
  }));
}

/**
 * A hand's fingers closed on a thing (its signed distance field `field` in the body's frame, metres, < 0 inside):
 * each finger the grip closes curls from the knuckle until its skin touches the thing, then on from the joint past
 * the one touching until that phalanx touches too, so it wraps the thing rather than stopping at the first knuckle
 * that meets it; each stops where its skin rests on the surface (pressed in no more than SKIN_GIVE) or at the grip's
 * `max` or the hand's limits (HAND_LIMITS). The fingers close first and then the thumb, which also stops on the
 * fingers' skin (it rests on them, never in them). A finger in the thing before it closes (the thumb swung under a
 * ball's palm, a pencil's tucked ring finger) opens until it only just touches. Returns the shape (handPose poses it);
 * a skeleton without fingers or a skin, the grip's shape as it is.
 *
 * Each phalanx's contact is the nearest of its skin samples to the surface, and each joint's curl a scan and a
 * bisection over that, a finger at a time with the rest of the hand as it is, its skin skinned alone (not the whole
 * body), so it is cheap enough per frame. Given the hand's shape the frame before (`prev`), each finger moves from it
 * toward this frame's fit by STEP a joint at most, so a fit that jumps (the thumb swung out of a mug's way and back)
 * eases rather than flicks, and where that is in the thing, opens from there as little as clears it.
 */
export function closeOn(skel: Skeleton, p: Pose, s: Side, field: (q: Vec3) => number, g: Closing, close = 1, prev?: HandShape): HandShape {
  // (along the path or not, finger by finger: a power grip's thumb curls as ever)
  const onPath = (k: number) => !!g.path && !(k === 0 && g.thumb);
  const shape: HandShape = { spread: g.spread * close, thumbOpp: (g.thumb?.opp ?? g.opp) * close };
  for (const [f, c] of Object.entries(g.rest ?? {}) as [Finger, [number, number, number]][]) shape[f] = c.map((v) => v * close) as [number, number, number];
  for (const f of g.on) shape[f] = [0, 0, 0];
  const F = fingerJoints(skel, s);
  if (!F.length || !skel.surface) return shape;
  const P = g.path ? pathHand(skel, p, s, g.path, shape, g.thumb ? 1 : 0) : handPose(skel, p, s, shape), turns = P.turns;
  const w = withFingers(skel, pose(skel, P)), wp: Posed = { pos: w.pos.slice(), rot: w.rot.slice() };
  const ax = axesOf(skel, s), skin = fingerSkinOf(skel, s), sk = skel.joints;
  const fan = [0, 1, 0.15, -0.55, -1];
  const order = [1, 2, 3, 4, 0];
  const end = g.path ? g.path.length - 1 : 0, along: [number, number][] = [[0, end], [0, end], [0, end]];
  const lim = (k: number) => (onPath(k) ? along : k === 0 ? HAND_LIMITS.thumb : HAND_LIMITS.finger);
  // finger k's skin at curl c (and the thumb's opposition o): each phalanx's nearest to the surface (metres, < 0 in
  // it), and for the thumb whether it is in the fingers
  let segs: FingerSegment[] = [];
  const at = (k: number, c: [number, number, number], o: number) => {
    F[k].forEach((i, ph) => {
      if (onPath(k)) { turns[i] = alongPath(skel, s, g.path!, k, ph, c[ph]); place(skel, turns, wp, i, i + 1, P.root); return; }
      let q = qaxis(ax.hinge[k][ph], clampTo(c[ph], lim(k)[ph]));
      if (ph === 0 && k === 0 && o) q = qmul(qaxis(ax.opp, clampTo(o, HAND_LIMITS.thumbOpp)), q);
      if (ph === 0 && k > 0 && shape.spread) q = qmul(qaxis(ax.spread, fan[k] * clampTo(shape.spread, HAND_LIMITS.spread)), q);
      turns[i] = q;
      place(skel, turns, wp, i, i + 1, P.root);
    });
    const sj: { pos: Vec3; rot: Quat }[] = [];
    for (const j of skin[k].sj) {
      sj[j] = { pos: wp.pos[j], rot: wp.rot[j] };
    }
    const near: [number, number, number] = [Infinity, Infinity, Infinity];
    let inFingers = false;
    for (const x of skin[k].pts) {
      let v: Vec3 = [0, 0, 0];
      for (let m = 0; m < x.j.length; m++) {
        const y = sj[x.j[m]], v0 = add(sk[x.j[0]].head, x.o);
        v = add(v, scale3(add(y.pos, qrot(y.rot, sub(v0, sk[x.j[m]].head))), x.w[m]));
      }
      v = scale3(v, 1 / x.tw);
      near[x.ph] = Math.min(near[x.ph], field(v));
      if (k === 0 && x.ph > 0 && !inFingers) {
        for (const gg of segs) {
          const ab = sub(gg.tail, gg.head), t = Math.max(0, Math.min(1, dot(sub(v, gg.head), ab) / Math.max(1e-9, dot(ab, ab))));
          const d = sub(v, add(gg.head, scale3(ab, t)));
          if (dot(d, d) < (gg.radius - 0.001) ** 2) { inFingers = true; break; }
        }
      }
    }
    return { near, inFingers };
  };
  // how far finger k's skin is in the thing (from phalanx `from` out) or the thumb in the fingers: 0 clear
  const depthOf = (k: number, c: [number, number, number], o: number, from: number) => {
    const r = at(k, c, o);
    return Math.max(0, -SKIN_GIVE - Math.min(...r.near.slice(from))) + (r.inFingers ? 0.01 : 0);
  };
  const clearOf = (k: number, c: [number, number, number], o: number, from: number) => depthOf(k, c, o, from) <= 0;
  type Fit = [[number, number, number], number];
  const lerp = (x: Fit, y: Fit, u: number): Fit => [x[0].map((v, m) => v + u * (y[0][m] - v)) as [number, number, number], x[1] + u * (y[1] - x[1])];
  // (out of it along the way from x to y: the least of the way that clears it, or the least deep)
  const outOf = (k: number, x: Fit, y: Fit): Fit => {
    let lo = -1, hi = 0, best = Infinity, bu = 0;
    for (let i = 1; i <= 8; i++) {
      const d = depthOf(k, ...lerp(x, y, i / 8), 0);
      if (d <= 0) { lo = i / 8; break; }
      hi = i / 8;
      if (d < best) { best = d; bu = i / 8; }
    }
    if (lo < 0) return lerp(x, y, bu);
    for (let n = 0; n < 6; n++) { const m = (lo + hi) / 2; if (clearOf(k, ...lerp(x, y, m), 0)) lo = m; else hi = m; }
    return lerp(x, y, lo);
  };
  // (held on in it: the least turn out of it of the end joint opened alone, the last two, or the whole finger and
  // the thumb's swing; a fist pressed at its tip opened all its joints at once flicked open)
  const outOfLeast = (k: number, x: Fit, open: Fit): Fit => {
    let best: Fit | undefined, bd = Infinity;
    for (const from of [2, 1, 0]) {
      const y: Fit = [x[0].map((v, m) => (m >= from ? open[0][m] : v)) as [number, number, number], from === 0 ? open[1] : x[1]];
      const z = outOf(k, x, y);
      if (!clearOf(k, ...z, 0)) continue;
      const d = z[0].reduce((a, v, m) => a + Math.abs(v - x[0][m]), Math.abs(z[1] - x[1]));
      if (d < bd) { bd = d; best = z; }
    }
    return best ?? outOf(k, x, open);
  };
  for (const k of order) {
    const f = FINGERS[k], on = g.on.includes(f), R = onPath(k) ? EVEN : k === 0 ? THUMB_CURL : CURL, L = lim(k);
    if (k === 0) segs = fingerSegments(skel, wp, s).slice(1).flatMap((x) => x.segments);
    const o0 = k === 0 ? shape.thumbOpp ?? 0 : 0, mx = k === 0 && g.thumb ? g.thumb.max : g.max, cap = (m: number) => Math.min(mx * close, L[m][1]);
    // where it closes to (a finger the grip holds as it is: that shape) and where it opens to: past straight as far
    // as the hand lets it, the thumb swung out from under the palm
    const want: Fit = [on ? [cap(0), cap(1), cap(2)] : shape[f] ?? [0, 0, 0], o0];
    const open: Fit = [[L[0][0], L[1][0], L[2][0]], k === 0 && !onPath(0) ? HAND_LIMITS.thumbOpp[0] : 0];
    let x: Fit;
    if (!on) {
      // (a finger the grip holds as it is: opened until it only just touches)
      x = clearOf(k, ...want, 0) ? want : outOf(k, want, open);
    } else {
      x = [[0, 0, 0], o0];
      // (in it before it closes at all, the hand's own place putting it there: opened past straight, the thumb swung
      // back out from under the palm, until it only just touches; or as far out of it as it comes)
      if (!clearOf(k, ...x, 0)) x = outOf(k, x, open);
      // closed from the knuckle until a phalanx touches, then on from the joint past it
      for (let j = 0; j < 3; j++) {
        const c = x[0], o = x[1];
        const curl = (t: number) => c.map((v, m) => (m >= j ? Math.max(L[m][0], Math.min(Math.max(v, cap(m)), v + t * R[m])) : v)) as [number, number, number];
        const clear = (t: number) => clearOf(k, curl(t), o, j);
        if (!clear(0)) break;
        const tmax = Math.max(0, ...[0, 1, 2].filter((m) => m >= j).map((m) => (cap(m) - c[m]) / R[m]));
        if (tmax <= 0) continue;
        let lo = 0, hi = -1;
        for (let i = 1; i <= 6; i++) { const t = (tmax * i) / 6; if (clear(t)) lo = t; else { hi = t; break; } }
        if (hi > 0) for (let n = 0; n < 6; n++) { const m = (lo + hi) / 2; if (clear(m)) lo = m; else hi = m; }
        x = [curl(lo), o];
      }
    }
    // held on from the frame before: moved toward this frame's fit by STEP a joint at most (the fit can jump, as the
    // thumb swinging out of a mug's way and back), or where that is in the thing, opened from there as little as clears
    // it, or the fit itself where nothing nearer does
    const was = prev?.[f];
    if (was) {
      const step = (a: number, b: number) => a + Math.max(-STEP, Math.min(STEP, b - a));
      const fit = x, y: Fit = [was.map((v, m) => step(v, fit[0][m])) as [number, number, number], k === 0 ? step(prev!.thumbOpp ?? o0, fit[1]) : fit[1]];
      if (clearOf(k, ...y, 0)) x = y;
      else {
        const z = outOfLeast(k, y, open);
        if (clearOf(k, ...z, 0)) x = z;
      }
    }
    shape[f] = x[0];
    if (k === 0) shape.thumbOpp = x[1];
    at(k, ...x);
  }
  return shape;
}
