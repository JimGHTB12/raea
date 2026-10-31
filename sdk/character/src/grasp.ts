// Grasp guides for raea/character: simple shapes on a thing that say how a hand takes it, so a hand is put on
// it from opposed sides (the palm on one, the fingers and the thumb round the far one) rather than touching one side.
// They follow the grasp-primitives idea (Miller et al. 2003, "Automatic grasp planning using shape primitives"): a
// thing is held as the primitive it is nearest, and each primitive has its own few ways of being held.
//
//   cylinder   an axis, a radius and a length: the fingers wrap round it, the palm on one side, the thumb along the axis
//              closing over the fingers ("wrap", a can, a bar) or lying along it ("along", a knife, a racket)
//   pinch      two opposed pads, the thumb's and the fingers', `width` apart across `across` (a pencil, a glass's stem)
//   sphere     a centre and a radius: the palm on it, the fingers spread over it and the thumb opposing them
//   slab       two parallel faces `width` apart along `normal`: two hands on opposite faces (a box), or one hand's thumb
//              on one face and its fingers on the other (a book)
//
// Every guide has tags ("body", "handle", "stem", "lid", "rim", ...), which a pick can ask for. A thing with no guides
// of its own gets them from its shape and size (autoGuides): a round thing a cylinder round its body (and a pinch on a
// stem, a cylinder along a handle), a ball a sphere, a box a slab across each pair of its upright faces, a pole a
// cylinder along it. A scene author or an agent places guides by hand on anything (Thing.guides), in the thing's own
// frame (its bottom's middle at the origin, x across, y up, z its front, metres):
//
//   const mug: Thing = { at, size: [0.085, 0.1, 0.085], shape: "round", guides: [
//     { kind: "cylinder", at: [0.06, 0.05, 0], axis: [0, 1, 0], radius: 0.006, length: 0.06, tags: ["handle"] },
//     { kind: "cylinder", at: [0, 0.05, 0], axis: [0, 1, 0], radius: 0.0425, length: 0.1, tags: ["body"] }] };
//
// handOn(guide, side, approach) says where the hand goes on it (the palm's middle, the way the palm faces and the way
// the fingers point), and holds() whether a hand's contacts can hold a thing up against gravity (bench/character).
import type { Vec3 } from "raea/3d";
import { add, cross, dot, len, norm, scale3, sub, surface, type Posed, type Side, type Skeleton } from "raea/character";
import type { Thing } from "./things";

interface GuideBase {
  /** what it is ("body", "handle", "stem", "lid", "rim", "side"...): a pick may ask for one */
  tags?: string[];
  /** taken by one hand or two (a slab: two when wider than a hand spans, by default) */
  hands?: 1 | 2;
}
/** fingers wrapped round an axis: a can's body, a mug's handle bar, a bar, a pole */
export interface CylinderGuide extends GuideBase {
  kind: "cylinder";
  /** the middle of its axis */
  at: Vec3;
  axis: Vec3;
  radius: number;
  /** how long it is along the axis (the span the hand may take it in) */
  length: number;
  /** the thumb closed over the fingers (wrap, the default) or lying straight along the axis (a knife, a racket) */
  thumb?: "wrap" | "along";
}
/** two opposed pads, the thumb's on one side and the fingers' on the other */
export interface PinchGuide extends GuideBase {
  kind: "pinch";
  /** the middle between the pads */
  at: Vec3;
  /** from the thumb's pad to the fingers' (either way round: the hand takes whichever suits it) */
  across: Vec3;
  /** the pads' distance apart (the thing's thickness there) */
  width: number;
  /** thumb and index (pinch, the default), thumb, index and middle on a thin thing written with (tripod), or round a
   * wine glass's stem, the bowl over the hand (stem) */
  fingers?: "pinch" | "tripod" | "stem";
}
/** the palm on a ball, the fingers spread over it */
export interface SphereGuide extends GuideBase {
  kind: "sphere";
  at: Vec3;
  radius: number;
}
/** two parallel faces: a box's sides, a book's covers */
export interface SlabGuide extends GuideBase {
  kind: "slab";
  /** the middle between the faces */
  at: Vec3;
  /** across the faces (either way round) */
  normal: Vec3;
  /** the faces' distance apart */
  width: number;
  /** the faces' extent: along `up`, and across the face */
  span: [number, number];
  /** the faces' up (the thing's up by default) */
  up?: Vec3;
}
export type Guide = CylinderGuide | PinchGuide | SphereGuide | SlabGuide;

/**
 * A thing's guides from its shape and size, as a hand would take it: a round thing a cylinder round its body, and a
 * pinch on a stem or a cylinder along a handle (its grips, first, so a pick by default takes the handle); a ball a
 * sphere; a box a slab across each pair of its upright faces (two hands where it is wider than `span`, a hand's
 * spread); a pole a cylinder along it.
 */
export function autoGuides(t: Thing, span = 0.18): Guide[] {
  const [w, h, d] = t.size, s = t.shape ?? "box", out: Guide[] = [];
  let bodyFrom = 0;
  for (const g of t.grips ?? []) {
    if (g.kind === "stem") {
      out.push({ kind: "pinch", at: g.at, across: [1, 0, 0], width: 0.009, fingers: "stem", tags: ["stem"] });
      bodyFrom = 2 * g.at[1];
    } else out.push({ kind: "cylinder", at: g.at, axis: norm(g.along ?? [0, 1, 0]), radius: 0.006, length: 0.06, tags: ["handle"] });
  }
  if (s === "round") out.push({ kind: "cylinder", at: [0, (bodyFrom + h) / 2, 0], axis: [0, 1, 0], radius: w / 2, length: h - bodyFrom, tags: bodyFrom ? ["body", "bowl"] : ["body"] });
  else if (s === "ball") out.push({ kind: "sphere", at: [0, w / 2, 0], radius: w / 2, tags: ["body"] });
  else if (s === "pole") out.push({ kind: "cylinder", at: [0, h / 2, 0], axis: [0, 0, 1], radius: w / 2, length: d, tags: ["body"] });
  else {
    out.push({ kind: "slab", at: [0, h / 2, 0], normal: [1, 0, 0], width: w, span: [h, d], hands: w > span ? 2 : 1, tags: ["side"] });
    out.push({ kind: "slab", at: [0, h / 2, 0], normal: [0, 0, 1], width: d, span: [h, w], hands: d > span ? 2 : 1, tags: ["side"] });
  }
  return out;
}

/** A thing's guides: its own (Thing.guides), else autoGuides. */
export const guidesOf = (t: Thing, span?: number): Guide[] => t.guides ?? autoGuides(t, span);

/**
 * The guide a pick takes: the first with the tag asked for, else the one the grip calls for (a handle's cylinder, a
 * stem's pinch, a body's cylinder or a ball's sphere for a hand round it or on it from above, a slab for two hands).
 */
export function pickGuide(t: Thing, grip: string, tag?: string): Guide | undefined {
  const gs = guidesOf(t);
  if (tag) return gs.find((g) => g.tags?.includes(tag));
  const has = (g: Guide, x: string) => g.tags?.includes(x);
  switch (grip) {
    case "handle": return gs.find((g) => g.kind === "cylinder" && has(g, "handle"));
    case "stem": return gs.find((g) => g.kind === "pinch" && (g.fingers === "stem" || has(g, "stem")));
    case "wrap": return gs.find((g) => g.kind === "cylinder" && !has(g, "handle")) ?? gs.find((g) => g.kind === "sphere");
    case "palm": case "top": return gs.find((g) => g.kind === "sphere") ?? gs.find((g) => g.kind === "cylinder" && !has(g, "handle"));
    case "sides": case "edges": return gs.find((g) => g.kind === "slab");
  }
  return undefined;
}

/** Where a hand goes on a guide, in the thing's frame: the palm's middle (`at`, where its skin rests), the way the
 * palm faces and the fingers point (a HandGoal's), and the contacts it makes when the fingers close (for checking
 * opposition): the palm's, the thumb's and the fingertips'. */
export interface HandOn { at: Vec3; facing: Vec3; point: Vec3; contacts: { palm: Vec3; thumb: Vec3; fingers: Vec3 }; wrap?: number }

// a wrap's diagonal across the palm (radians; Anny's three bodies' cans held on 96-100% of carried frames from 0.7 to
// 0.8, under a third at 0.4) and how far down the axis the hand goes from its middle (of the hand's length)
const TILT = 0.8, SHIFT = 0.28;
// taken by its end: the hand turned about the axis toward the thumb's side (radians), and the palm's middle back from
// the end's middle, away from the fingers (of the radius: at 0.3 the man's and woman's cans from above held on 13% and
// 0% of frames, at 1.2 on 96% and 99%)
const END_TURN = 0.6, END_BACK = 1.2;
const perp = (v: Vec3, a: Vec3): Vec3 => sub(v, scale3(a, dot(v, a)));
const rotAbout = (v: Vec3, a: Vec3, th: number): Vec3 => add(add(scale3(v, Math.cos(th)), scale3(cross(a, v), Math.sin(th))), scale3(a, dot(a, v) * (1 - Math.cos(th))));

/**
 * A hand on a guide, coming at it from `approach` (the way from the guide toward the hand, in the thing's frame: the
 * palm goes on that side), `hand` its length (wrist to fingertips, about a tenth of the body's height). The hand's
 * frame is planned in its own terms, so the same guide is taken from beside it or from above it:
 *
 *   cylinder  the palm on the approach's side of the axis, flat on it, the axis across the palm toward the knuckles
 *             (where a hand closes on a bar); the thumb side along the axis toward `thumbUp` (up, by default), the
 *             fingers round it the far way. The fingers' wrap: their length against the radius (a thin bar nearly
 *             all the way round, a can about half), the thumb closing over them from the other side.
 *   sphere    the palm on the approach's side, the fingers pointing toward `ahead` over it, spread; the thumb across.
 *   pinch     the palm over the middle from the approach's side, the thumb toward one pad and the fingers the other.
 *   slab      the palm flat on the face toward the approach (each hand of two on its own face), its fingers down.
 */
export function handOn(g: Guide, side: Side, approach: Vec3, o: { hand: number; thumbUp?: Vec3; ahead?: Vec3 }): HandOn {
  const hand = o.hand, up = o.thumbUp ?? [0, 1, 0];
  // the frame from the palm's facing n and the thumb's side `across`: R's across is point x facing, L's facing x point
  const pointFor = (n: Vec3, across: Vec3): Vec3 => norm(side === "R" ? cross(n, across) : cross(across, n));
  if (g.kind === "cylinder") {
    const a0 = norm(g.axis), td = dot(a0, up) < 0 ? scale3(a0, -1) : a0;
    if (len(perp(norm(approach), a0)) < 0.5) {
      // from its end (a can from above): the palm on the end, the fingers over its far edge and down its side, the
      // hand turned about the axis toward the thumb so the thumb goes down the near side, across from them (square,
      // they went down sides a quarter turn apart and pushed at right angles)
      const e = dot(approach, a0) >= 0 ? a0 : scale3(a0, -1), n = scale3(e, -1);
      let f1 = perp(o.ahead ?? [0, 0, 1], e);
      if (len(f1) < 1e-6) f1 = perp([1, 0, 0], e);
      f1 = norm(f1);
      // (R's thumb is on the left of fingers pointing ahead with the palm down: f x n)
      const thumbSide = side === "R" ? cross(f1, n) : cross(n, f1);
      const f = norm(add(scale3(f1, Math.cos(END_TURN)), scale3(thumbSide, Math.sin(END_TURN))));
      const touch = add(g.at, scale3(e, g.length / 2)), at = add(touch, scale3(f, -END_BACK * g.radius));
      const down = scale3(e, -g.length / 4);
      return { at, facing: n, point: f, contacts: { palm: touch, fingers: add(add(g.at, scale3(f, g.radius)), down), thumb: add(add(g.at, scale3(f, -g.radius)), down) } };
    }
    let u = perp(approach, a0);
    if (len(u) < 1e-6) u = perp([0, 1, 0], a0);
    if (len(u) < 1e-6) u = perp([1, 0, 0], a0);
    u = norm(u);
    const n = scale3(u, -1), f0 = pointFor(n, td), r = g.radius;
    // the axis diagonally across the palm (a power grip's: from the index's root to the heel of the hand), the fingers
    // turned TILT toward the thumb's end of it, so the thumb, closing, goes round the far way from the fingers rather
    // than over the end (square across the palm, a can's thumb lay on its top and the fingers reached a third of the
    // way round: one side). A thumb lying along it keeps the hand square.
    const psi = g.thumb === "along" ? 0 : TILT;
    const f = norm(add(scale3(f0, Math.cos(psi)), scale3(td, Math.sin(psi))));
    // (the axis across the palm toward the knuckles: the palm's middle back toward the wrist from where it touches,
    // more for a thin bar, which sits in the fingers' roots, than for a can the palm rests on)
    const back = (r < 0.15 * hand ? 0.22 : 0.12) * hand;
    const touch = add(g.at, scale3(u, r));
    // (and the hand down the axis from its middle, away from the thumb's end, so the thumb's root is on its side, not
    // past its end: a man's hand is longer than a can)
    const at = add(add(touch, scale3(f, -back)), scale3(td, -Math.min(SHIFT * hand, 0.4 * g.length)));
    // (the fingers' wrap round it: their reach from the knuckles over the radius, a full turn at most)
    const wrap = Math.min(2 * Math.PI, (0.55 * hand + back) / Math.max(r, 1e-3));
    const tip = Math.min(wrap, 1.15 * Math.PI);
    const fingers = add(g.at, scale3(rotAbout(u, td, side === "R" ? tip : -tip), r));
    // (the thumb closed over the fingers from the other way round, or lying along the axis by the palm)
    const thumb = g.thumb === "along" ? add(touch, scale3(td, 0.25 * hand)) : add(g.at, scale3(rotAbout(u, td, side === "R" ? -Math.min(Math.PI / 2, 0.35 * hand / r) : Math.min(Math.PI / 2, 0.35 * hand / r)), r));
    return { at, facing: n, point: f, contacts: { palm: touch, thumb, fingers }, wrap };
  }
  if (g.kind === "sphere") {
    const u = norm(len(approach) > 1e-6 ? approach : [0, 1, 0]), n = scale3(u, -1), r = g.radius;
    let f = perp(o.ahead ?? [0, 0, 1], u);
    if (len(f) < 1e-6) f = perp([0, 1, 0], u);
    f = norm(f);
    const across = side === "R" ? cross(f, n) : cross(n, f), touch = add(g.at, scale3(u, r));
    const tip = Math.min(Math.PI * 0.9, (0.45 * hand) / r);
    return {
      at: touch, facing: n, point: f, contacts: {
        palm: touch,
        fingers: add(g.at, scale3(rotAbout(u, norm(cross(u, f)), -tip), r)),
        thumb: add(g.at, scale3(rotAbout(u, norm(cross(u, across)), -tip), r)),
      },
    };
  }
  if (g.kind === "pinch") {
    const c = norm(g.across), u = norm(perp(len(approach) > 1e-6 ? approach : [0, 1, 0], c)), n = scale3(u, -1);
    // (the thumb's pad on whichever side its own side is: R's across is point x facing)
    const thumbSide = dot(c, side === "R" ? [-1, 0, 0] : [1, 0, 0]) >= 0 ? c : scale3(c, -1);
    const f = norm(cross(thumbSide, u)), at = add(g.at, scale3(u, 0.012));
    return { at, facing: n, point: f, contacts: { palm: at, thumb: add(g.at, scale3(thumbSide, -g.width / 2)), fingers: add(g.at, scale3(thumbSide, g.width / 2)) } };
  }
  // slab: the face toward the approach, the palm on it, the fingers down (round the lower edge for two hands)
  const nn = norm(g.normal), u = dot(approach, nn) >= 0 ? nn : scale3(nn, -1), n = scale3(u, -1), upv = norm(g.up ?? [0, 1, 0]);
  const touch = add(g.at, scale3(u, g.width / 2)), f = norm(perp(scale3(upv, -1), u));
  return { at: add(touch, scale3(upv, -0.15 * g.span[0])), facing: n, point: f, contacts: { palm: touch, thumb: add(g.at, scale3(u, -g.width / 2)), fingers: add(g.at, scale3(u, -g.width / 2)) } };
}

// ---------------------------------------------------------------- holding against gravity

/** A skin contact with a thing: where, and the way it pushes (into the thing, along the surface's normal). */
export interface Contact { part: string; at: Vec3; push: Vec3 }

/** The skin touching a thing (its distance field in the scene): for each part (a palm, a phalanx, a forearm, the
 * chest...) within `near` metres of its surface, its sample nearest it, pushing along the field's inward normal.
 * `parts` picks the skin's parts (the hands', by default). */
export function contactsOf(skel: Skeleton, w: Posed, field: (q: Vec3) => number, o: { parts?: RegExp; near?: number } = {}): Contact[] {
  const near = o.near ?? 0.003, best = new Map<string, { at: Vec3; d: number }>();
  for (const x of surface(skel, w, o.parts ?? /^(palm|thumb|index|middle|ring|pinky)\d?$/)) {
    const d = field(x.at), k = x.side + x.part;
    if (d < near && (!best.has(k) || d < best.get(k)!.d)) best.set(k, { at: x.at, d });
  }
  const e = 5e-4, out: Contact[] = [];
  for (const [part, { at: p }] of best) {
    const g: Vec3 = [field(add(p, [e, 0, 0])) - field(sub(p, [e, 0, 0])), field(add(p, [0, e, 0])) - field(sub(p, [0, e, 0])), field(add(p, [0, 0, e])) - field(sub(p, [0, 0, e]))];
    if (len(g) < 1e-9) continue;
    out.push({ part, at: p, push: scale3(norm(g), -1) });
  }
  return out;
}

/**
 * Whether contacts can hold a thing (its middle `mid`) up against gravity: each a point contact with friction (`mu`,
 * 0.5 by default, its cone as `edges` forces), some pushes along them adding up to the thing's weight upward with no
 * turn about its middle (turns over `lever` metres, 5 cm by default, count as forces). The nonnegative least squares
 * fit of the cones' edges' wrenches to the one that holds it (Lawson-Hanson); it holds when what is left is under
 * `tol` of the weight (0.1). One-sided contacts (a palm and fingertips on the same side of a can) can't: their pushes
 * all go one way.
 */
export function holds(cs: Contact[], mid: Vec3, o: { mu?: number; edges?: number; lever?: number; tol?: number; down?: Vec3 } = {}): boolean {
  return holdResidual(cs, mid, o) < (o.tol ?? 0.1);
}

/** What is left of the weight when the contacts hold all they can (holds' fit): 0 held, 1 none of it. */
export function holdResidual(cs: Contact[], mid: Vec3, o: { mu?: number; edges?: number; lever?: number; down?: Vec3 } = {}): number {
  const mu = o.mu ?? 0.5, E = o.edges ?? 6, lever = o.lever ?? 0.05, down = norm(o.down ?? [0, -1, 0]);
  const cols: number[][] = [];
  for (const c of cs) {
    const n = norm(c.push), a: Vec3 = Math.abs(n[0]) < 0.9 ? [1, 0, 0] : [0, 1, 0];
    const u = norm(cross(n, a)), v = cross(n, u);
    for (let k = 0; k < E; k++) {
      const th = (2 * Math.PI * k) / E, f = norm(add(n, add(scale3(u, mu * Math.cos(th)), scale3(v, mu * Math.sin(th)))));
      const tq = scale3(cross(sub(c.at, mid), f), 1 / lever);
      cols.push([...f, ...tq]);
    }
  }
  if (!cols.length) return 1;
  return nnls(cols, [-down[0], -down[1], -down[2], 0, 0, 0]).res;
}

/** min |A x - b| over x >= 0, A by its columns (Lawson and Hanson's active set): the fit and what is left */
export function nnls(cols: number[][], b: number[], iters = 200): { x: number[]; res: number } {
  const n = cols.length, m = b.length, x = new Array(n).fill(0), P = new Set<number>();
  const resid = (xx: number[]) => b.map((bj, j) => bj - cols.reduce((a, c, i) => a + c[j] * xx[i], 0));
  // (the least squares over the passive columns, by their normal equations)
  const lsq = (idx: number[]): number[] => {
    const k = idx.length, M = Array.from({ length: k }, () => new Array(k + 1).fill(0));
    for (let a = 0; a < k; a++) {
      for (let c = 0; c < k; c++) { let s = 0; for (let j = 0; j < m; j++) s += cols[idx[a]][j] * cols[idx[c]][j]; M[a][c] = s + (a === c ? 1e-12 : 0); }
      let s = 0; for (let j = 0; j < m; j++) s += cols[idx[a]][j] * b[j]; M[a][k] = s;
    }
    for (let c = 0; c < k; c++) {
      let piv = c;
      for (let r = c + 1; r < k; r++) if (Math.abs(M[r][c]) > Math.abs(M[piv][c])) piv = r;
      [M[c], M[piv]] = [M[piv], M[c]];
      const d = M[c][c] || 1e-18;
      for (let r = 0; r < k; r++) if (r !== c) { const f = M[r][c] / d; for (let q = c; q <= k; q++) M[r][q] -= f * M[c][q]; }
    }
    return idx.map((_, a) => M[a][k] / (M[a][a] || 1e-18));
  };
  for (let it = 0; it < iters; it++) {
    const r = resid(x), wv = cols.map((c) => c.reduce((a, v, j) => a + v * r[j], 0));
    let t = -1, best = 1e-10;
    for (let i = 0; i < n; i++) if (!P.has(i) && wv[i] > best) { best = wv[i]; t = i; }
    if (t < 0) break;
    P.add(t);
    for (let inner = 0; inner < 50; inner++) {
      const idx = [...P], z = lsq(idx);
      if (z.every((v) => v > 0)) { idx.forEach((i, a) => (x[i] = z[a])); break; }
      let alpha = Infinity;
      idx.forEach((i, a) => { if (z[a] <= 0) alpha = Math.min(alpha, x[i] / (x[i] - z[a])); });
      idx.forEach((i, a) => { x[i] = x[i] + alpha * (z[a] - x[i]); if (x[i] <= 1e-12) { x[i] = 0; P.delete(i); } });
      if (!P.size) break;
    }
  }
  return { x, res: Math.hypot(...resid(x)) };
}
