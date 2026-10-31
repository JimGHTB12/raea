// Footprints and the layout's clearances, checked.
import { type Vec3 } from "raea/3d";
import { hasNative, native } from "@raea/app/native";
import { modelBounds } from "@raea/render/scene";
import { LaidRun, Room, onWall, unitCentre } from "./all";

// ---------------------------------------------------------------------------------------------------- the checks

/** A box on the floor plan: x and z, metres. */
export interface Footprint {
  name: string;
  min: [number, number];
  max: [number, number];
  /** a seat's: the way a diner faces, on the plan (x, z) */
  faces?: [number, number];
}

/** A laid run's footprint: from the wall to its deepest unit, along its length. */
export function runFootprint(run: LaidRun, name = `${run.wall.name} run`): Footprint {
  const depth = Math.max(...run.units.map((l) => l.depth));
  const a = onWall(run.wall, run.units[0]?.u0 ?? 0, 0), b = onWall(run.wall, run.end, depth);
  return { name, min: [Math.min(a[0], b[0]), Math.min(a[2], b[2])], max: [Math.max(a[0], b[0]), Math.max(a[2], b[2])] };
}

/** A model standing at `at` turned by `yaw`, as a footprint. A seat says which way a diner on it faces before it is
 * turned (`faces`, on the plan; a chair that faces +z, [0, 1]): turned by `yaw` with it. */
export function modelFootprint(name: string, src: string, at: Vec3, yaw = 0, faces?: [number, number]): Footprint {
  const b = modelBounds(src, [0, yaw, 0]);
  const f: [number, number] | undefined = faces && [faces[0] * Math.cos(yaw) + faces[1] * Math.sin(yaw), -faces[0] * Math.sin(yaw) + faces[1] * Math.cos(yaw)];
  return { name, min: [at[0] + b.min[0], at[2] + b.min[2]], max: [at[0] + b.max[0], at[2] + b.max[2]], faces: f };
}

const facings = new Map<string, [number, number] | null>();

/** Which way a seat faces as it comes, on the plan (x, z): read from its triangles by the engine, from its back's
 * centre across its seat's; null for a seat with nothing over it (a backless stool faces any way). */
export function seatFacing(src: string): [number, number] | null {
  let f = facings.get(src);
  if (f === undefined) {
    if (!hasNative()) throw new Error("seatFacing: reading a glTF needs the raea engine (raea render)");
    f = (native("scene.facing", { src }) as { faces: [number, number] | null }).faces;
    facings.set(src, f);
  }
  return f;
}

/** The yaw that turns a seat standing at `at` to face `target` (a table's or a counter's middle): never typed by hand,
 * a model's own front is read from it (seatFacing); a seat that faces any way stays as it comes. */
export function faceToward(src: string, at: Vec3, target: Vec3): number {
  const f = seatFacing(src);
  if (!f) return 0;
  return Math.atan2(target[0] - at[0], target[2] - at[2]) - Math.atan2(f[0], f[1]);
}

/** A seat's footprint, its facing read from the model and turned with it. */
export function seatFootprint(name: string, src: string, at: Vec3, yaw = 0): Footprint {
  return modelFootprint(name, src, at, yaw, seatFacing(src) ?? undefined);
}

/** The clear distance from a seat's back to the first thing behind it, across the seat's width (a seat that faces
 * neither along x nor z is read along the nearer one; a thing the seat is tucked into, 0); undefined when nothing is
 * behind it. */
export function behind(seat: Footprint, others: Footprint[]): { o: Footprint; g: number } | undefined {
  const [fx, fz] = seat.faces!;
  const ax = Math.abs(fx) >= Math.abs(fz) ? 0 : 1, across = 1 - ax, back = -Math.sign(ax === 0 ? fx : fz);
  let best: { o: Footprint; g: number } | undefined;
  for (const o of others) {
    if (o === seat || o.max[across] <= seat.min[across] || o.min[across] >= seat.max[across]) continue;
    const g = back > 0 ? o.min[ax] - seat.max[ax] : seat.min[ax] - o.max[ax];
    // on that side of the seat: its middle past the seat's
    const side = back * ((o.min[ax] + o.max[ax]) / 2 - (seat.min[ax] + seat.max[ax]) / 2) > 0;
    if (side && (!best || g < best.g)) best = { o, g: Math.max(0, g) };
  }
  return best;
}

/** The clear distance between two footprints on the plan (0 when they overlap). */
export function gap(a: Footprint, b: Footprint): number {
  const dx = Math.max(0, a.min[0] - b.max[0], b.min[0] - a.max[0]);
  const dz = Math.max(0, a.min[1] - b.max[1], b.min[1] - a.max[1]);
  return Math.hypot(dx, dz);
}

/** A kitchen's clearances (the NKBA's guidelines, in metres): a work aisle in front of a work run, a walkway
 * elsewhere, room behind a seated diner to walk past, and the work triangle between sink, stove and fridge. */
export const CLEARANCE = { workAisle: 1.07, walkway: 0.91, behindSeat: 0.91, legMin: 1.22, legMax: 2.74, triangleMax: 7.92 };

export interface Finding {
  level: "error" | "warning" | "ok";
  rule: string;
  says: string;
}

export interface Plan {
  room: Room;
  runs: LaidRun[];
  /** free-standing work pieces, held to a work aisle from the runs and a walkway from the walls: an island */
  islands: Footprint[];
  /** furniture that is in the way but owes no aisle: a table, a bench */
  furniture?: Footprint[];
  /** seats (with `faces`, room is kept behind a diner; without, to the nearest wall or run) */
  seats?: Footprint[];
}

/** The plan checked: every aisle between an island and a run or a wall, the walkways, room behind the seats, and the
 * work triangle. */
export function checkLayout(plan: Plan): Finding[] {
  const out: Finding[] = [];
  const c = CLEARANCE;
  const { width: w, depth: d } = plan.room;
  const walls: Footprint[] = [
    { name: "back wall", min: [0, -1], max: [w, 0] }, { name: "front wall", min: [0, d], max: [w, d + 1] },
    { name: "left wall", min: [-1, 0], max: [0, d] }, { name: "right wall", min: [w, 0], max: [w + 1, d] },
  ];
  const runs = plan.runs.map((r) => runFootprint(r));
  for (const isl of plan.islands) {
    for (const r of runs) {
      const g = gap(isl, r);
      if (g < 3) out.push({ level: g < c.workAisle ? "error" : "ok", rule: "work aisle", says: `${isl.name} to the ${r.name}: ${g.toFixed(2)} m (a work aisle wants ${c.workAisle})` });
    }
    for (const wl of walls) {
      const g = gap(isl, wl);
      if (g < 2 && !runs.some((r) => gap(r, wl) < 0.01 && overlapsAlong(r, wl, isl))) {
        out.push({ level: g < c.walkway ? "error" : "ok", rule: "walkway", says: `${isl.name} to the ${wl.name}: ${g.toFixed(2)} m (a walkway wants ${c.walkway})` });
      }
    }
  }
  const things = [...walls, ...runs, ...plan.islands, ...(plan.furniture ?? []), ...(plan.seats ?? [])];
  // a seat faces what it is sat at: an island or a table within half a metre before it
  for (const s of plan.seats ?? []) {
    if (!s.faces) continue;
    const front = behind({ ...s, faces: [-s.faces[0], -s.faces[1]] }, [...plan.islands, ...(plan.furniture ?? [])]);
    const ok = front && front.g < 0.5;
    out.push({ level: ok ? "ok" : "error", rule: "facing", says: ok ? `${s.name} faces the ${front.o.name}` : `${s.name} faces nothing to sit at${front ? ` (the ${front.o.name} is ${front.g.toFixed(2)} m before it)` : ""}: turn it with faceToward` });
  }
  for (const s of plan.seats ?? []) {
    const near = s.faces ? behind(s, things) : [...walls, ...runs].map((o) => ({ o, g: gap(s, o) })).sort((a, b) => a.g - b.g)[0];
    // a bench or a chair with its back to a wall: nobody walks behind it
    if (near && s.faces && walls.includes(near.o) && near.g < 0.15) out.push({ level: "ok", rule: "behind a seat", says: `${s.name}: its back to the ${near.o.name}` });
    else if (near) out.push({ level: near.g < c.behindSeat ? "error" : "ok", rule: "behind a seat", says: `behind ${s.name}: ${near.g.toFixed(2)} m to the ${near.o.name} (to walk past a diner: ${c.behindSeat})` });
  }
  const at = (role: string) => plan.runs.flatMap((r) => r.units.filter((u) => u.role === role).map((u) => unitCentre(r, u)))[0];
  const [sink, stove, fridge] = [at("sink"), at("stove"), at("fridge")];
  if (sink && stove && fridge) {
    const leg = (a: Vec3, b: Vec3) => Math.hypot(a[0] - b[0], a[2] - b[2]);
    const legs: [string, number][] = [["sink to stove", leg(sink, stove)], ["stove to fridge", leg(stove, fridge)], ["fridge to sink", leg(fridge, sink)]];
    for (const [n, l] of legs) {
      out.push({ level: l < c.legMin || l > c.legMax ? "warning" : "ok", rule: "work triangle", says: `${n}: ${l.toFixed(2)} m (${c.legMin}-${c.legMax})` });
    }
    const total = legs.reduce((a, [, l]) => a + l, 0);
    out.push({ level: total > c.triangleMax ? "error" : "ok", rule: "work triangle", says: `the triangle: ${total.toFixed(2)} m (at most ${c.triangleMax})` });
  }
  return out;
}

/** whether a run along a wall covers the stretch of it beside `isl` (so the island faces the run, not bare wall) */
function overlapsAlong(r: Footprint, wl: Footprint, isl: Footprint): boolean {
  const horizontal = wl.max[0] - wl.min[0] > wl.max[1] - wl.min[1];
  const k = horizontal ? 0 : 1;
  return r.min[k] < isl.max[k] && r.max[k] > isl.min[k];
}
