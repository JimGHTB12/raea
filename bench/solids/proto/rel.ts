// Prototype of the brief's "SDK shape": units, anchors, relations, patterns, growth, assemblies, over today's
// manifold API. One frame, Bevy's: +y up, +z front, +x right, metres. Manifold is z-up, so primitives are turned here.
import { manifold } from "raea/3d";

type Solid = ReturnType<typeof manifold.box>;
export type Vec3 = [number, number, number];

// 1. units: branded, so a bare number or a radian can't go where a length or degrees belong
declare const unit: unique symbol;
export type Metres = number & { [unit]: "m" };
export type Degrees = number & { [unit]: "deg" };
export const m = (v: number) => v as Metres;
export const cm = (v: number) => (v / 100) as Metres;
export const mm = (v: number) => (v / 1000) as Metres;
export const deg = (v: number) => v as Degrees;

// directions as words
export type Side = "top" | "bottom" | "left" | "right" | "front" | "back";
const NORMAL: Record<Side, Vec3> = {
  top: [0, 1, 0], bottom: [0, -1, 0], right: [1, 0, 0], left: [-1, 0, 0], front: [0, 0, 1], back: [0, 0, -1],
};
const OPPOSITE: Record<Side, Side> = { top: "bottom", bottom: "top", left: "right", right: "left", front: "back", back: "front" };

// 2. anchors: a point and the way it faces
export interface Anchor { at: Vec3; facing: Vec3; of: string }

const add = (a: Vec3, b: Vec3): Vec3 => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
const sub = (a: Vec3, b: Vec3): Vec3 => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const scl = (a: Vec3, s: number): Vec3 => [a[0] * s, a[1] * s, a[2] * s];

export class Part {
  constructor(readonly name: string, readonly solid: Solid, readonly own: Record<string, Anchor> = {}, readonly labels: Labels = {}) {}
  /** design-level metadata that travels with the part through every relation and into the world */
  label(l: Labels): Part { return new Part(this.name, this.solid, this.own, { ...this.labels, ...l }); }

  bounds() { return this.solid.bounds() as { min: Vec3; max: Vec3 }; }
  get center(): Vec3 { const b = this.bounds(); return scl(add(b.min, b.max), 0.5); }
  get size(): Vec3 { const b = this.bounds(); return sub(b.max, b.min); }

  /** the centre of a side of the bounds, facing out */
  side(s: Side): Anchor {
    const b = this.bounds(), c = this.center, n = NORMAL[s];
    const at: Vec3 = [
      n[0] > 0 ? b.max[0] : n[0] < 0 ? b.min[0] : c[0],
      n[1] > 0 ? b.max[1] : n[1] < 0 ? b.min[1] : c[1],
      n[2] > 0 ? b.max[2] : n[2] < 0 ? b.min[2] : c[2],
    ];
    return { at, facing: n, of: `${this.name}.${s}` };
  }
  get top() { return this.side("top"); }
  get bottom() { return this.side("bottom"); }
  anchor(name: string): Anchor {
    const a = this.own[name];
    if (!a) throw new Error(`${this.name} has no anchor "${name}" (it has ${Object.keys(this.own).join(", ") || "none"})`);
    return a;
  }

  moved(d: Vec3): Part {
    const own: Record<string, Anchor> = {};
    for (const [k, a] of Object.entries(this.own)) own[k] = { ...a, at: add(a.at, d) };
    return new Part(this.name, this.solid.move(d[0], d[1], d[2]), own, this.labels);
  }
  named(name: string) { return new Part(name, this.solid, this.own, this.labels); }
  with(name: string, at: Vec3, facing: Vec3 = [0, 1, 0]): Part {
    return new Part(this.name, this.solid, { ...this.own, [name]: { at, facing, of: `${this.name}.${name}` } }, this.labels);
  }

  // 3. relations: the part moves so that its side opposite the anchor's facing lands on the anchor. Parts sink JOIN
  // into each other by default, so a union fuses them (faces that only touch stay two pieces); the ground doesn't.
  placeOn(a: Anchor, gap: Metres = (a.of === "ground" ? m(0) : m(-JOIN))): Part {
    const side = sideOf(a.facing);
    const mine = this.side(OPPOSITE[side]).at;
    return this.moved(add(sub(a.at, mine), scl(a.facing, gap)));
  }
  beside(other: Part, s: Side, gap: Metres = m(0)): Part { return this.placeOn(other.side(s), gap); }
  /** flush with the other's side, keeping the other two axes */
  alignTo(other: Part, s: Side): Part {
    const d = sub(other.side(s).at, this.side(s).at), n = NORMAL[s];
    return this.moved([d[0] * Math.abs(n[0]), d[1] * Math.abs(n[1]), d[2] * Math.abs(n[2])]);
  }
  centerOn(p: Vec3 | Anchor, axes: "xz" | "xyz" = "xz"): Part {
    const at = Array.isArray(p) ? p : p.at, d = sub(at, this.center);
    return this.moved(axes === "xz" ? [d[0], 0, d[2]] : d);
  }
  /** its bottom at the given anchor's height, keeping x and z (a trunk on the ground) */
  standOn(a: Anchor): Part { return this.moved([0, a.at[1] - this.bottom.at[1], 0]); }
  /** its top at the anchor's point, sunk JOIN into what it hangs from (a leg under a seat's corner) */
  hangFrom(a: Anchor): Part { return this.moved(add(sub(a.at, this.top.at), [0, JOIN, 0])); }
  /** hollowed with walls this thick, open on one side (a mug, a box, a planter) */
  hollow(wall: Metres, open: Side = "top"): Part {
    const inner = this.solid.minkowskiDiff(manifold.sphere(wall));
    const n = NORMAL[open], s = this.size, d: Vec3 = [n[0] * s[0] * 0.5, n[1] * s[1] * 0.5, n[2] * s[2] * 0.5]; // half: the copy overlaps, so no lid is left
    const anchors = { ...this.own, inside: { at: add(this.side(OPPOSITE[open]).at, scl(n, wall)), facing: n, of: `${this.name}.inside` } };
    return new Part(this.name, this.solid.minus(inner.union(inner.move(d[0], d[1], d[2]))), anchors, this.labels);
  }
}

export const JOIN = 0.001;
export const ground: Anchor = { at: [0, 0, 0], facing: [0, 1, 0], of: "ground" };

function sideOf(n: Vec3): Side {
  let best: Side = "top", d = -2;
  for (const [s, v] of Object.entries(NORMAL) as [Side, Vec3][]) {
    const dot = v[0] * n[0] + v[1] * n[1] + v[2] * n[2];
    if (dot > d) { d = dot; best = s; }
  }
  return best;
}

// primitives in the y-up frame, sized in metres, sitting with their centre at the origin
export const box = (name: string, w: Metres, h: Metres, d: Metres) => new Part(name, manifold.box(w, h, d));
export const roundedBox = (name: string, w: Metres, h: Metres, d: Metres, r: Metres) =>
  new Part(name, manifold.roundedBox(w, h, d, r));
/** a cylinder standing up (manifold's runs along z: turned to y) */
export const post = (name: string, r: Metres, h: Metres, rTop: Metres = r) =>
  new Part(name, (rTop === r ? manifold.cylinder(r, h) : manifold.cone(r, rTop, h)).rotate(-90, 0, 0));
export const ball = (name: string, r: Metres) => new Part(name, manifold.sphere(r));

// 4. patterns by meaning: points, not arithmetic in the program
/** the four corners of a face (a top or bottom anchor's rectangle), pulled in by inset */
export function corners(p: Part, s: "top" | "bottom", inset: Metres): Anchor[] {
  const b = p.bounds(), y = s === "top" ? b.max[1] : b.min[1], f = NORMAL[s];
  return ([[b.min[0] + inset, b.min[2] + inset], [b.max[0] - inset, b.min[2] + inset],
    [b.max[0] - inset, b.max[2] - inset], [b.min[0] + inset, b.max[2] - inset]] as [number, number][])
    .map(([x, z], i) => ({ at: [x, y, z] as Vec3, facing: f, of: `${p.name}.${s}.corner${i}` }));
}
export function around(center: Vec3, n: number, radius: Metres, y = center[1]): Anchor[] {
  return Array.from({ length: n }, (_, i) => {
    const t = (i / n) * Math.PI * 2;
    return { at: [center[0] + Math.cos(t) * radius, y, center[2] + Math.sin(t) * radius] as Vec3, facing: [0, -1, 0] as Vec3, of: `around${i}` };
  });
}

/** a straight member with flat ends exactly at a and b (a splayed leg, a rung, a strut) */
export function strut(name: string, a: Vec3, b: Vec3, r0: Metres, r1: Metres = r0): Part {
  const disc = (p: Vec3, r: number, below: boolean) =>
    manifold.cylinder(r, 0.001).rotate(-90, 0, 0).move(p[0], p[1] + (below ? 0.0005 : -0.0005), p[2]);
  const up = b[1] >= a[1];
  return new Part(name, manifold.hull(disc(a, r0, up), disc(b, r1, !up)), {
    start: { at: a, facing: [0, up ? -1 : 1, 0], of: `${name}.start` }, end: { at: b, facing: [0, up ? 1 : -1, 0], of: `${name}.end` },
  });
}

// 5. growth: a tapered tube along points (a hull chain of spheres), and branching from a tip
export function tube(name: string, path: Vec3[], radii: number[]): Part {
  const balls = path.map((p, i) => manifold.sphere(radii[i] ?? radii[radii.length - 1]).move(p[0], p[1], p[2]));
  const links = balls.slice(1).map((b, i) => manifold.hull(balls[i], b));
  const s = manifold.union(...links);
  const end = path[path.length - 1], prev = path[path.length - 2] ?? [0, 0, 0];
  const dir = sub(end, prev), len = Math.hypot(...dir) || 1;
  return new Part(name, s, { tip: { at: end, facing: scl(dir, 1 / len), of: `${name}.tip` } });
}
/** a limb from an anchor: its own length, turned from the parent's facing by yaw (about up) and pitch (away from up) */
export function limb(name: string, from: Anchor, length: Metres, r0: Metres, r1: Metres, yaw: Degrees, pitch: Degrees, bend = 0.15): Part {
  const y = (yaw * Math.PI) / 180, p = (pitch * Math.PI) / 180;
  const f = from.facing, up = Math.atan2(Math.hypot(f[0], f[2]), f[1]), az = Math.atan2(f[2], f[0]);
  const tilt = p, a = az + y; // tilt from vertical, not added to the parent's: nested tilts sum to sideways growth
  const dir: Vec3 = [Math.sin(tilt) * Math.cos(a), Math.cos(tilt), Math.sin(tilt) * Math.sin(a)];
  const mid = add(add(from.at, scl(dir, length * 0.5)), [0, length * bend, 0]);
  const end = add(from.at, scl(dir, length));
  return tube(name, [from.at, mid, end], [r0, (r0 + r1) / 2, r1]);
}

// 6. assemblies with a joint graph: the checks read the graph
export type Join = { kind: "rests" | "attached"; a: string; b: string };
export const rests = (a: Part, b: Part): Join => ({ kind: "rests", a: a.name, b: b.name });
export const attached = (a: Part, b: Part): Join => ({ kind: "attached", a: a.name, b: b.name });
export class Assembly {
  constructor(readonly name: string, readonly parts: Part[], readonly joins: Join[]) {}
  get solid(): Solid { return manifold.union(...this.parts.map((p) => p.solid)); }
  part(name: string) {
    const p = this.parts.find((q) => q.name === name);
    if (!p) throw new Error(`${this.name} has no part ${name}`);
    return p;
  }
}
export const assembly = (name: string, parts: Part[], joins: Join[] = []) => new Assembly(name, parts, joins);

// 7. labels: what a part is and what it's for, at the design level. They travel through every relation above and
// become each part's row in the world (Bevy's ECS through raea's native world), so rules and games query them.
export type Affordance = "sit" | "support" | "grasp" | "contain" | "walk" | "climb" | "open" | "hang" | "light";
export interface Labels {
  role?: string;            // "seat", "leg", "handle", "trunk", "foliage"
  affords?: Affordance[];   // what a person or a game can do with it
  material?: string;        // a material or recipe name, picked up by the material layer
  [key: string]: unknown;   // anything a game wants (hp, team, loot, sound)
}
