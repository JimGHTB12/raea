// The object bench's hidden checks, prototyped: everything from the solid itself (y up, metres), nothing from the
// program that made it. One sampling (a grid of vertical rays) gives the material as columns: centre of mass,
// footprint, area by height, seat-like surfaces and cavities all come from it.
import { manifold } from "raea/3d";
import type { Vec3 } from "./rel.ts";

type Solid = ReturnType<typeof manifold.box>;
export interface Result { check: string; ok: boolean; got: string }

interface Column { x: number; z: number; segs: [number, number][] } // [bottom, top] heights of material
export interface Sample { cols: Column[]; cell: number; min: Vec3; max: Vec3 }

export function sample(s: Solid, n = 48): Sample {
  const { min, max } = s.bounds() as { min: Vec3; max: Vec3 };
  const span = Math.max(max[0] - min[0], max[2] - min[2]), cell = span / n;
  const cols: Column[] = [];
  const top = max[1] + 1, bot = min[1] - 1;
  for (let i = 0; i < n; i++) for (let j = 0; j < n; j++) {
    const x = min[0] + (i + 0.5) * cell, z = min[2] + (j + 0.5) * cell;
    if (x > max[0] || z > max[2]) continue;
    const hits = s.rayCast([x, top, z], [x, bot, z]).map((h) => h.position[1]).sort((a, b) => b - a);
    const segs: [number, number][] = [];
    for (let k = 0; k + 1 < hits.length; k += 2) segs.push([hits[k + 1], hits[k]]);
    cols.push({ x, z, segs });
  }
  return { cols, cell, min, max };
}

const fmt = (v: number, d = 2) => v.toFixed(d);
const len = (c: Column) => c.segs.reduce((a, [b, t]) => a + t - b, 0);

export function centerOfMass(S: Sample): [number, number, number] {
  let w = 0, x = 0, y = 0, z = 0;
  for (const c of S.cols) for (const [b, t] of c.segs) {
    const l = t - b; w += l; x += l * c.x; z += l * c.z; y += l * (b + t) / 2;
  }
  return [x / w, y / w, z / w];
}
/** the area of material at height h */
export function areaAt(S: Sample, h: number) {
  return S.cols.filter((c) => c.segs.some(([b, t]) => b <= h && h <= t)).length * S.cell * S.cell;
}
/** the xz half-width of material at height h (how far it spreads from its centre) */
function spreadAt(S: Sample, h: number) {
  const xs = S.cols.filter((c) => c.segs.some(([b, t]) => b <= h && h <= t));
  if (!xs.length) return 0;
  return (Math.max(...xs.map((c) => c.x)) - Math.min(...xs.map((c) => c.x))) / 2;
}

function hull2(pts: [number, number][]): [number, number][] {
  const p = [...pts].sort((a, b) => a[0] - b[0] || a[1] - b[1]);
  if (p.length < 3) return p;
  const cross = (o: number[], a: number[], b: number[]) => (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
  const lo: [number, number][] = [], up: [number, number][] = [];
  for (const q of p) { while (lo.length >= 2 && cross(lo[lo.length - 2], lo[lo.length - 1], q) <= 0) lo.pop(); lo.push(q); }
  for (const q of p.reverse()) { while (up.length >= 2 && cross(up[up.length - 2], up[up.length - 1], q) <= 0) up.pop(); up.push(q); }
  return lo.slice(0, -1).concat(up.slice(0, -1));
}
/** signed distance inside a convex polygon (negative outside) */
function inside(poly: [number, number][], p: [number, number]) {
  let d = Infinity;
  for (let i = 0; i < poly.length; i++) {
    const a = poly[i], b = poly[(i + 1) % poly.length];
    const ex = b[0] - a[0], ez = b[1] - a[1], l = Math.hypot(ex, ez) || 1;
    d = Math.min(d, ((p[0] - a[0]) * ez - (p[1] - a[1]) * ex) / -l);
  }
  return d;
}

// --- the checks ---------------------------------------------------------------------------------------------------

export function connected(s: Solid): Result {
  const n = s.decompose().length;
  return { check: "one piece", ok: n === 1, got: `${n} piece${n === 1 ? "" : "s"}` };
}
export function grounded(S: Sample): Result {
  return { check: "on the ground", ok: Math.abs(S.min[1]) <= 0.002, got: `bottom at ${fmt(S.min[1], 3)} m` };
}
export type Range = [number, number];
export function scale(S: Sample, spec: { h?: Range; w?: Range }): Result {
  const h = S.max[1] - S.min[1], w = Math.max(S.max[0] - S.min[0], S.max[2] - S.min[2]);
  const okH = !spec.h || (h >= spec.h[0] && h <= spec.h[1]), okW = !spec.w || (w >= spec.w[0] && w <= spec.w[1]);
  return { check: "real-world size", ok: okH && okW, got: `h ${fmt(h)} m, w ${fmt(w)} m` };
}
/** the centre of mass over the footprint, with a margin (a stand-in for the physics drop) */
export function stands(S: Sample): Result {
  const foot = S.cols.filter((c) => c.segs.length && Math.min(...c.segs.map((s) => s[0])) <= S.min[1] + 0.01);
  const poly = hull2(foot.map((c) => [c.x, c.z]));
  const com = centerOfMass(S), d = poly.length >= 3 ? inside(poly, [com[0], com[2]]) : -1;
  return { check: "stands (mass over footprint)", ok: d > 0.01, got: `margin ${fmt(d * 100, 1)} cm` };
}
/** somewhere to sit: a level surface at seat height, open above, covering a body-sized patch */
export function seat(S: Sample, h: Range = [0.38, 0.55], patch = 0.3): Result {
  const tops = S.cols.map((c) => (c.segs.length ? c.segs[0][1] : -1)); // the first surface from above
  const at = S.cols.filter((_, i) => tops[i] >= h[0] && tops[i] <= h[1]);
  if (!at.length) return { check: "a seat", ok: false, got: "no surface at seat height" };
  const xs = at.map((c) => c.x), zs = at.map((c) => c.z);
  const wx = Math.max(...xs) - Math.min(...xs), wz = Math.max(...zs) - Math.min(...zs);
  const ok = wx >= patch && wz >= patch && at.length * S.cell * S.cell >= patch * patch * 0.8;
  return { check: "a seat", ok, got: `${fmt(wx)} x ${fmt(wz)} m at ${fmt(Math.min(...at.map((c) => c.segs[0][1])))} m` };
}
/** holds water: a floor above the ground under the opening, and walls all round up to near the rim */
export function holdsWater(s: Solid, S: Sample): Result {
  // the opening: columns whose first surface from above is well below the rim, or that go straight through
  const rim = S.max[1], H = rim - S.min[1];
  // only columns walled in on all four sides count (air beside the cup or through a handle's loop doesn't)
  const solidCols = S.cols.filter((q) => q.segs.length), e = S.cell / 2;
  const walled = (q: Column) =>
    solidCols.some((o) => Math.abs(o.z - q.z) < e && o.x < q.x) && solidCols.some((o) => Math.abs(o.z - q.z) < e && o.x > q.x) &&
    solidCols.some((o) => Math.abs(o.x - q.x) < e && o.z < q.z) && solidCols.some((o) => Math.abs(o.x - q.x) < e && o.z > q.z);
  const open = S.cols.filter((q) => (!q.segs.length || q.segs[0][1] < rim - H * 0.3) && walled(q));
  if (!open.length) return { check: "holds water", ok: false, got: "no cavity" };
  const cx = open.reduce((a, q) => a + q.x, 0) / open.length, cz = open.reduce((a, q) => a + q.z, 0) / open.length;
  const holes = open.filter((q) => !q.segs.length);
  if (holes.length) return { check: "holds water", ok: false, got: `${holes.length} columns go straight through the floor` };
  const col = open.reduce((a, q) => (Math.hypot(q.x - cx, q.z - cz) < Math.hypot(a.x - cx, a.z - cz) ? q : a));
  const floor = col.segs[0][1], depth = rim - floor;
  let leaks = 0, tried = 0;
  for (let k = 1; k <= 4; k++) {
    const y = floor + (depth * k) / 5;
    for (let a = 0; a < 12; a++) {
      const t = (a / 12) * Math.PI * 2, far = 10;
      const hits = s.rayCast([col.x, y, col.z], [col.x + Math.cos(t) * far, y, col.z + Math.sin(t) * far]);
      tried++; if (!hits.length) leaks++;
    }
  }
  return { check: "holds water", ok: leaks === 0, got: `${fmt(depth * 100, 1)} cm deep, ${leaks}/${tried} side rays escape` };
}
/** a tree: a trunk at the ground narrower than the crown, thinner going up, the crown above the trunk */
export function treeForm(s: Solid, S: Sample): Result {
  // exact horizontal slices (y turned to manifold's z) for the trunk, the column grid for the crown's spread
  const zUp = s.rotate(90, 0, 0), H = S.max[1] - S.min[1], at = (f: number) => S.min[1] + H * f;
  const cut = (f: number) => zUp.slice(at(f));
  const s0 = cut(0.03), s1 = cut(0.2);
  const trunk = 2 * Math.sqrt(s0.area / Math.PI), crown = 2 * Math.max(...[0.6, 0.7, 0.8].map((f) => spreadAt(S, at(f))));
  // the crown: how tall the leafy part is against how wide (an oak's is a dome, not a flat table)
  const levels = Array.from({ length: 20 }, (_, i) => (i + 0.5) / 20).filter((f) => spreadAt(S, at(f)) * 2 > trunk * 3);
  const crownH = levels.length ? (levels[levels.length - 1] - levels[0] + 0.05) * H : 0, aspect = crownH / crown;
  const ok = s0.contours === 1 && crown >= trunk * 4 && s0.area >= s1.area && s1.contours === 1 && aspect >= 0.4 && aspect <= 1.6;
  return { check: "tree form", ok, got: `trunk ${fmt(trunk)} m in ${s0.contours} piece, crown ${fmt(crown)} m wide x ${fmt(crownH)} m tall (${fmt(aspect)}), trunk area ${fmt(s0.area, 3)} -> ${fmt(s1.area, 3)} m2` };
}
/** each join in the graph touches */
export function joins(parts: { name: string; solid: Solid }[], js: { a: string; b: string; kind: string }[]): Result {
  const by = Object.fromEntries(parts.map((p) => [p.name, p.solid]));
  const bad = js.filter((j) => by[j.a].minGap(by[j.b], 0.05) > 0.002).map((j) => `${j.a}-${j.b}`);
  return { check: "joins touch", ok: bad.length === 0, got: bad.length ? `apart: ${bad.join(", ")}` : `${js.length} joins` };
}
