// Things hung on the walls, and checked.
import { Model, type Vec3, Entity } from "raea/3d";
import { modelBounds } from "@raea/render/scene";
import { FACE_YAW, Facing, Finding, LaidRun, Opening, Room, WallName, gap, onWall, roomFinding } from "./all";
import { Quat, Transform } from "raea/transform";

// ---------------------------------------------------------------------------------------------------- on the walls

/** Which way a flat thing that hangs (a picture, a mirror, a shelf) faces, read from its box: its thinnest level axis
 * is its depth, and its back is the side its origin is on (a pack hangs its pictures by their backs). */
export function wallFacing(src: string): Facing {
  const b = modelBounds(src);
  const [dx, dz] = [b.max[0] - b.min[0], b.max[2] - b.min[2]];
  if (dx <= dz) return Math.abs(b.min[0]) < Math.abs(b.max[0]) ? "+x" : "-x";
  return Math.abs(b.min[2]) < Math.abs(b.max[2]) ? "+z" : "-z";
}

/** What decor hangs over: a span along the wall and the height of its top (a counter's stone, a sideboard, a bench). */
export interface Over {
  name: string;
  u0: number;
  u1: number;
  top: number;
}

/** A laid unit (or a span of a run from one unit to another) as something to hang over. */
export function overUnits(run: LaidRun, first: string, last = first): Over {
  const a = run.units.find((u) => u.name === first), b = run.units.find((u) => u.name === last);
  if (!a || !b) throw new Error(`overUnits: no unit ${!a ? first : last} in the ${run.wall.name} run`);
  return { name: a === b ? a.name! : `${a.name} to ${b.name}`, u0: a.u0, u1: b.u1, top: Math.max(a.height, b.height) };
}

/** A hung thing: on a wall, its span along it and up it, its model in the wall's frame. */
export interface Hung {
  name: string;
  src: string;
  wall: WallName;
  u0: number;
  u1: number;
  y0: number;
  y1: number;
  /** how far its back stands off the wall */
  off: number;
  over?: Over;
  /** its height given by the caller (a shelf, a clock over a door), not the art rule: its height isn't checked */
  given?: boolean;
  position: Vec3;
  rotation: Vec3;
}

/** The decorator's rules (metres): art centred 1.45-1.55 m up on a bare wall, its bottom 0.20-0.25 m over the furniture
 * it hangs over and centred on it, a group's pieces a hand apart on one centre line. */
export const HANGING = { centre: 1.5, centreTol: 0.05, over: 0.22, overMin: 0.18, overMax: 0.3, apart: 0.08, offCentre: 0.03 };

/** Decor hung on a wall by the rules: one piece or a group side by side, a hand apart (`apart`), on one centre line.
 * Over furniture (`over`), centred on it with its bottom `HANGING.over` above it; else centred at `at` along the wall,
 * its centre `HANGING.centre` up. Each piece faces into the room by its own measured front (wallFacing). */
export function hang(room: Room, wall: WallName, items: { src: string; name?: string; front?: Facing }[], o: { at?: number; over?: Over; centre?: number; apart?: number; off?: number } = {}): Hung[] {
  const apart = o.apart ?? HANGING.apart, off = o.off ?? 0.003;
  const boxes = items.map((it) => {
    const rotation: Vec3 = [0, FACE_YAW[it.front ?? wallFacing(it.src)], 0];
    return { it, rotation, b: modelBounds(it.src, rotation) };
  });
  const widths = boxes.map(({ b }) => b.max[0] - b.min[0]);
  const tall = Math.max(...boxes.map(({ b }) => b.max[1] - b.min[1]));
  const total = widths.reduce((a, w) => a + w, 0) + apart * (items.length - 1);
  const mid = o.over ? (o.over.u0 + o.over.u1) / 2 : o.at;
  if (mid === undefined) throw new Error("hang: say where along the wall (at) or what it hangs over (over)");
  const yc = o.centre ?? (o.over ? o.over.top + HANGING.over + tall / 2 : HANGING.centre);
  let u = mid - total / 2;
  return boxes.map(({ it, rotation, b }, i) => {
    const w = widths[i], h = b.max[1] - b.min[1];
    const h0: Hung = {
      name: it.name ?? `${wall} decor ${i + 1}`, src: it.src, wall, u0: u, u1: u + w, y0: yc - h / 2, y1: yc + h / 2, off, over: o.over, given: o.centre !== undefined,
      position: [u - b.min[0], yc - (b.min[1] + b.max[1]) / 2, off - b.min[2]], rotation,
    };
    u += w + apart;
    return h0;
  });
}

/** Hung decor drawn, each piece in its wall's frame. */
export function Hang({ room, hung }: { room: Room; hung: Hung[] }) {
  return (
    <>
      {hung.map((h, i) => (
        <Entity key={i} transform={Transform.fromTranslation(onWall(room.walls[h.wall], 0)).withRotation(Quat.fromEuler("XYZ", 0, room.walls[h.wall].yaw, 0))}>
          <Model name={h.name} src={h.src} position={h.position} rotation={h.rotation} />
        </Entity>
      ))}
    </>
  );
}

/** Hung decor checked: on its wall (not standing off it: floating), clear of the wall's openings and its wall
 * cabinets and counters, centred on what it hangs over at the right height above it, or at eye height on a bare wall. */
export function checkDecor(room: Room, hung: Hung[], { openings = [], runs = [] }: { openings?: Opening[]; runs?: LaidRun[] } = {}): Finding[] {
  const out: Finding[] = [];
  const H = HANGING;
  const cross = (a0: number, a1: number, b0: number, b1: number) => Math.min(a1, b1) - Math.max(a0, b0) > 0.01;
  // a group shares one centre line and one anchor: judge it once
  const groups = new Map<string, Hung[]>();
  for (const h of hung) {
    const key = `${h.wall}|${h.over?.name ?? ""}|${((h.y0 + h.y1) / 2).toFixed(3)}`;
    groups.set(key, [...(groups.get(key) ?? []), h]);
  }
  for (const h of hung) {
    if (h.off > 0.02) out.push(roomFinding("decor-floats", true, "error", [h.name], `${h.name} floats ${h.off.toFixed(2)} m off the ${h.wall} wall`, h.off, 0.02));
    for (const o of openings.filter((o) => o.wall === h.wall)) {
      if (cross(h.u0, h.u1, o.at - o.width / 2 - 0.05, o.at + o.width / 2 + 0.05) && cross(h.y0, h.y1, o.bottom - 0.05, o.top + 0.05)) out.push(roomFinding("decor-opening", true, "error", [h.name, `${h.wall} ${o.kind}`], `${h.name} hangs over the ${h.wall} wall's ${o.kind}`));
    }
    for (const r of runs.filter((r) => r.wall.name === h.wall)) {
      for (const l of r.units) {
        if (h.over && l.u0 >= h.over.u0 - 0.01 && l.u1 <= h.over.u1 + 0.01 && l.height <= h.over.top + 0.01) continue;
        if (cross(h.u0, h.u1, l.u0, l.u1) && cross(h.y0, h.y1, l.bottom, l.height)) out.push(roomFinding("decor-unit", true, "error", [h.name, l.name ?? "a unit"], `${h.name} runs into ${l.name ?? "a unit"} on the ${h.wall} wall`));
      }
    }
  }
  for (const g of groups.values()) {
    const [u0, u1, y0, y1] = [Math.min(...g.map((h) => h.u0)), Math.max(...g.map((h) => h.u1)), Math.min(...g.map((h) => h.y0)), Math.max(...g.map((h) => h.y1))];
    const what = g.length > 1 ? `the group of ${g.length} (${g.map((h) => h.name).join(", ")})` : g[0].name;
    const over = g[0].over;
    if (over) {
      const gap = y0 - over.top, off = (u0 + u1) / 2 - (over.u0 + over.u1) / 2;
      const names = [...g.map((h) => h.name), over.name];
      if (!g[0].given) out.push(roomFinding("decor-height", gap < H.overMin || gap > H.overMax, "warn", names, `${what}: ${gap.toFixed(2)} m over ${over.name} (${H.overMin}-${H.overMax})`, gap));
      else out.push(roomFinding("decor-height", false, "warn", names, `${what}: hung ${gap.toFixed(2)} m over ${over.name} (its height given)`, gap));
      if (Math.abs(off) > H.offCentre) out.push(roomFinding("decor-centre", true, "warn", names, `${what}: ${off.toFixed(2)} m off ${over.name}'s centre`, off, H.offCentre));
      if (u1 - u0 > over.u1 - over.u0) out.push(roomFinding("decor-width", true, "warn", names, `${what} is wider than ${over.name}`, u1 - u0, over.u1 - over.u0));
    } else if (!g[0].given) {
      const c = (y0 + y1) / 2;
      out.push(roomFinding("decor-height", Math.abs(c - H.centre) > H.centreTol, "warn", g.map((h) => h.name), `${what}: centred ${c.toFixed(2)} m up (${H.centre - H.centreTol}-${H.centre + H.centreTol})`, c));
    }
  }
  return out;
}
