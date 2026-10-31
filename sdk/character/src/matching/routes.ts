// Routes through the scene's room.
import type { Vec3 } from "raea/3d";
import { obstacleDistance } from "../things";
import { onWall, runFootprint, unitCentre, type LaidRun, type Opening, type Plan, type Room } from "@raea/room/room";
import { Step } from "./all";

// ---------------------------------------------------------------- routes

/** A box on the floor plan the route keeps clear of (the scene SDK's Footprint). */
export interface Obstacle {
  min: [number, number];
  max: [number, number];
  /** its floor exactly, a convex polygon (x, z), when it is not its box (a turned table, a round one: things.ts) */
  poly?: [number, number][];
  /** how high it stands (things.ts says, for a thing): a walk goes round it whatever, an elbow passes over it if low */
  top?: number;
}

// ---------------------------------------------------------------- the scene's room

/** What a plan (the scene SDK's) puts in a walker's way, its runs, islands, furniture and seats, and the room's walls
 * as the route's bounds. */
export function planFloor(plan: Plan): { obstacles: Obstacle[]; bounds: Obstacle } {
  const obstacles = [...plan.runs.map((r) => runFootprint(r)), ...plan.islands, ...(plan.furniture ?? []), ...(plan.seats ?? [])].map((f) => ({ min: f.min, max: f.max }));
  return { obstacles, bounds: { min: [0, 0], max: [plan.room.width, plan.room.depth] } };
}

/** A spot on the floor and a point to face there. */
export interface Mark {
  at: [number, number];
  face: Vec3;
}

/** Just inside an opening (a doorway), `inset` metres into the room, facing on into it. */
export function through(room: Room, o: Opening, inset = 0.45): Mark {
  const f = room.walls[o.wall], p = onWall(f, o.at, inset), q = onWall(f, o.at, inset + 1, 1);
  return { at: [p[0], p[2]], face: q };
}

/** Standing before a run's unit (by its role, "fridge", "sink", "stove", or its name), `gap` metres out from its front,
 * facing it. */
export function before(run: LaidRun, unit: string, gap = 0.4): Mark {
  const l = run.units.find((u) => u.role === unit || u.name === unit);
  if (!l) throw new Error(`before: the ${run.wall.name} run has no unit ${unit} (it has ${run.units.map((u) => u.role ?? u.name).join(", ")})`);
  const p = onWall(run.wall, (l.u0 + l.u1) / 2, l.depth + gap), c = unitCentre(run, l);
  return { at: [p[0], p[2]], face: [c[0], 1, c[2]] };
}

/** A walk step from one spot to another round everything the plan puts in the way, ending facing the second's point
 * (`clear`: how far the route keeps from things, 0.3 m by default). */
export function walkPlan(plan: Plan, from: Mark | [number, number], to: Mark, o: { clear?: number; speed?: number } = {}): Step {
  const { obstacles, bounds } = planFloor(plan);
  const a = Array.isArray(from) ? from : from.at;
  return { walk: route(a, to.at, obstacles, { bounds, clear: o.clear }), face: to.face, speed: o.speed };
}

/** A route on the floor from `from` to `to` (x, z) round the obstacles, `clear` metres from them and inside the room's
 * walls (`bounds`, x and z): the grid's shortest path (8 cm cells), pulled taut, its corners rounded into arcs as wide
 * as the clearance allows (a person walks round a corner, not to it and off at an angle; `smooth: false` keeps the
 * corners). `arrive` (a yaw) brings it in to the end along that way for its last `lead` metres (0.6), so the body comes
 * up to a counter facing it rather than turning on the spot. */
export function route(from: [number, number], to: [number, number], obstacles: Obstacle[], o: { bounds?: Obstacle; clear?: number; cell?: number; smooth?: boolean; arrive?: number; lead?: number } = {}): [number, number][] {
  const clear = o.clear ?? 0.3, cell = o.cell ?? 0.08;
  if (o.arrive !== undefined) {
    // the way in: from a point `lead` back along the arrival if it is clear, else as far back as is
    for (let l = o.lead ?? 0.6; l > 0.15; l -= 0.15) {
      const pre: [number, number] = [to[0] - Math.sin(o.arrive) * l, to[1] - Math.cos(o.arrive) * l];
      const inBounds = !o.bounds || (pre[0] > o.bounds.min[0] + clear && pre[0] < o.bounds.max[0] - clear && pre[1] > o.bounds.min[1] + clear && pre[1] < o.bounds.max[1] - clear);
      if (!inBounds || obstacles.some((b) => obstacleDistance(pre, b) < clear)) continue;
      const way = Math.hypot(from[0] - pre[0], from[1] - pre[1]) < 0.05 ? [from] : route(from, pre, obstacles, { ...o, arrive: undefined, smooth: false });
      return o.smooth === false ? [...way, to] : rounded([...way, to], obstacles, clear, o.bounds);
    }
  }
  const pts = [from, to, ...obstacles.flatMap((b) => [b.min, b.max])];
  const bx = o.bounds ?? { min: [Math.min(...pts.map((p) => p[0])) - 1, Math.min(...pts.map((p) => p[1])) - 1], max: [Math.max(...pts.map((p) => p[0])) + 1, Math.max(...pts.map((p) => p[1])) + 1] };
  const W = Math.ceil((bx.max[0] - bx.min[0]) / cell) + 1, H = Math.ceil((bx.max[1] - bx.min[1]) / cell) + 1;
  const at = (i: number, j: number): [number, number] => [bx.min[0] + i * cell, bx.min[1] + j * cell];
  const blocked = (p: [number, number]) => {
    if (o.bounds && (p[0] < bx.min[0] + clear || p[0] > bx.max[0] - clear || p[1] < bx.min[1] + clear || p[1] > bx.max[1] - clear)) return true;
    return obstacles.some((b) => obstacleDistance(p, b) < clear);
  };
  const free = new Uint8Array(W * H);
  for (let j = 0; j < H; j++) for (let i = 0; i < W; i++) free[j * W + i] = blocked(at(i, j)) ? 0 : 1;
  const cellOf = (p: [number, number]): [number, number] => [Math.round((p[0] - bx.min[0]) / cell), Math.round((p[1] - bx.min[1]) / cell)];
  // the ends may stand close to a thing (at a counter, by a chair): they are let free, with the straight way out to them
  const [si, sj] = cellOf(from), [ti, tj] = cellOf(to);
  const openNear = (ci: number, cj: number) => {
    for (let r = 0; r < 12; r++) for (let dj = -r; dj <= r; dj++) for (let di = -r; di <= r; di++) {
      const i = ci + di, j = cj + dj;
      if (i >= 0 && j >= 0 && i < W && j < H && free[j * W + i]) return [i, j] as [number, number];
    }
    return [ci, cj] as [number, number];
  };
  const s0 = free[sj * W + si] ? [si, sj] : openNear(si, sj), t0 = free[tj * W + ti] ? [ti, tj] : openNear(ti, tj);
  // A*, 8 neighbours
  const g = new Float32Array(W * H).fill(Infinity), came = new Int32Array(W * H).fill(-1);
  const open: number[] = [s0[1] * W + s0[0]];
  g[open[0]] = 0;
  const h = (k: number) => Math.hypot((k % W) - t0[0], Math.floor(k / W) - t0[1]);
  const goal = t0[1] * W + t0[0];
  const done = new Uint8Array(W * H);
  while (open.length) {
    let bi = 0;
    for (let k = 1; k < open.length; k++) if (g[open[k]] + h(open[k]) < g[open[bi]] + h(open[bi])) bi = k;
    const cur = open.splice(bi, 1)[0];
    if (cur === goal) break;
    if (done[cur]) continue;
    done[cur] = 1;
    const ci = cur % W, cj = Math.floor(cur / W);
    for (let dj = -1; dj <= 1; dj++) for (let di = -1; di <= 1; di++) {
      if (!di && !dj) continue;
      const i = ci + di, j = cj + dj;
      if (i < 0 || j < 0 || i >= W || j >= H || !free[j * W + i]) continue;
      const nk = j * W + i, ng = g[cur] + Math.hypot(di, dj);
      if (ng < g[nk]) { g[nk] = ng; came[nk] = cur; open.push(nk); }
    }
  }
  if (came[goal] < 0 && goal !== open[0]) throw new Error(`route: no way from ${from} to ${to} ${clear} m clear of the obstacles`);
  const cells: [number, number][] = [];
  for (let k = goal; k >= 0; k = came[k]) cells.unshift(at(k % W, Math.floor(k / W)));
  // pulled taut: from each kept point, the farthest point it sees in a straight clear line
  const sees = (a: [number, number], b: [number, number]) => {
    const n = Math.ceil(Math.hypot(b[0] - a[0], b[1] - a[1]) / (cell / 2));
    for (let k = 1; k < n; k++) {
      const p: [number, number] = [a[0] + ((b[0] - a[0]) * k) / n, a[1] + ((b[1] - a[1]) * k) / n];
      const [i, j] = cellOf(p);
      if (!free[j * W + i]) return false;
    }
    return true;
  };
  const taut: [number, number][] = [cells[0]];
  let k = 0;
  while (k < cells.length - 1) {
    let far = k + 1;
    for (let m = cells.length - 1; m > k + 1; m--) if (sees(cells[k], cells[m])) { far = m; break; }
    taut.push(cells[far]);
    k = far;
  }
  const way: [number, number][] = [from, ...taut.slice(1, -1), to];
  return o.smooth === false ? way : rounded(way, obstacles, clear, o.bounds);
}

/** A route's corners rounded: each an arc tangent to both legs, as wide as half the shorter leg allows (0.8 m at most)
 * and narrowed until it keeps 85% of the clearance from every obstacle, drawn every 8 cm. */
export function rounded(way: [number, number][], obstacles: Obstacle[], clear: number, bounds?: Obstacle): [number, number][] {
  if (way.length < 3) return way;
  const k85 = 0.85 * clear;
  const ok = (p: [number, number]) => obstacles.every((b) => obstacleDistance(p, b) >= k85) &&
    (!bounds || (p[0] >= bounds.min[0] + k85 && p[0] <= bounds.max[0] - k85 && p[1] >= bounds.min[1] + k85 && p[1] <= bounds.max[1] - k85));
  const out: [number, number][] = [way[0]];
  for (let k = 1; k < way.length - 1; k++) {
    const a = way[k - 1], c = way[k], b = way[k + 1];
    const la = Math.hypot(c[0] - a[0], c[1] - a[1]), lb = Math.hypot(b[0] - c[0], b[1] - c[1]);
    if (la < 1e-6 || lb < 1e-6) { out.push(c); continue; }
    const ua: [number, number] = [(c[0] - a[0]) / la, (c[1] - a[1]) / la], ub: [number, number] = [(b[0] - c[0]) / lb, (b[1] - c[1]) / lb];
    const turn = Math.acos(Math.max(-1, Math.min(1, ua[0] * ub[0] + ua[1] * ub[1])));
    if (turn < 0.03) { out.push(c); continue; }
    // an arc of radius r starts r tan(turn / 2) before the corner on each leg; each leg gives its two arcs half itself
    const tanH = Math.tan(turn / 2), left = ua[0] * ub[1] - ua[1] * ub[0] > 0 ? 1 : -1;
    let r = Math.min(0.8, (0.5 * Math.min(la, lb)) / tanH), arc: [number, number][] = [];
    for (; r > 0.04; r *= 0.8) {
      const cut = r * tanH, p0: [number, number] = [c[0] - ua[0] * cut, c[1] - ua[1] * cut];
      // the centre: off the first leg, toward the turn
      const ctr: [number, number] = [p0[0] - ua[1] * left * r, p0[1] + ua[0] * left * r];
      const a0 = Math.atan2(p0[1] - ctr[1], p0[0] - ctr[0]), m = Math.max(2, Math.ceil((r * turn) / 0.08));
      arc = Array.from({ length: m + 1 }, (_, i) => { const th = a0 + left * turn * (i / m); return [ctr[0] + r * Math.cos(th), ctr[1] + r * Math.sin(th)] as [number, number]; });
      if (arc.every(ok)) break;
    }
    if (r <= 0.04) out.push(c);
    else out.push(...arc);
  }
  out.push(way[way.length - 1]);
  return out;
}
