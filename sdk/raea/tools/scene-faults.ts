// Faults of a body among things (docs/locomotion.md, I-class): what clips into furniture, what the hands do to the
// thing they take, and how a sit plays. For a plan case (errands): its things are boxes, a seat is errands.tsx's chair
// (a seat top at 0.46 m, a leg block under it, a back behind the sitter).
//
//   clip     I1  frames with a body surface point more than 1.5 cm inside furniture or the chair (%), the deepest (cm)
//                and where; the hips, waist and thighs on the seat's top while sitting are not in it
//   inThing  I2  frames with a palm or finger point more than 0.5 cm inside the thing being taken or held (%), deepest
//   gripGap  I2  held frames on which no palm or finger point is within 1 cm of the thing's surface (%): a float
//   sitLoop  I4  turns back of the seat clip's frame while sat, a second: a sit's held part played to and fro
import type { Vec3 } from "raea/3d";
import { pose, qinv, qrot, surface, type Pose, type Quat, type Skeleton } from "raea/character";
import type { Performance } from "raea/matching";
import { thingSdf, type Interaction } from "raea/items";
import type { Seat } from "raea/matching";

const seatYaw = (s: Seat) => (typeof s.facing === "number" ? s.facing : Math.atan2(s.facing[0] - s.at[0], s.facing[2] - s.at[2]));

interface Box { name: string; c: Vec3; h: Vec3; rot: Quat }
const I: Quat = [0, 0, 0, 1];
const yawQ = (a: number): Quat => [0, Math.sin(a / 2), 0, Math.cos(a / 2)];

/** how far inside a box a point is (metres; negative outside, the distance to the nearest face either way) */
function depth(b: Box, p: Vec3): number {
  const l = qrot(qinv(b.rot), [p[0] - b.c[0], p[1] - b.c[1], p[2] - b.c[2]]);
  const d = [0, 1, 2].map((k) => b.h[k] - Math.abs(l[k]));
  if (d.every((x) => x > 0)) return Math.min(...d);
  const o = [0, 1, 2].map((k) => Math.max(0, -d[k]));
  return -Math.hypot(...o);
}

export function sceneFaults(skel: Skeleton, perf: Performance, live: Interaction, things: Record<string, any>, items: Record<string, any>, seat?: Seat) {
  // (the plan's own box for a seat sat on is left out: the chair below is drawn as errands.tsx draws it)
  const boxes: Box[] = Object.entries(things).filter(([, t]) => !seat || Math.hypot(t.at[0] - seat.at[0], t.at[2] - seat.at[2]) > 0.01).map(([name, t]) => ({ name, c: [t.at[0], t.size[1] / 2, t.at[2]], h: [t.size[0] / 2, t.size[1] / 2, t.size[2] / 2], rot: yawQ(t.yaw ?? 0) }));
  if (seat) {
    const r = yawQ(seatYaw(seat) + Math.PI), at = (x: number, y: number, z: number): Vec3 => { const v = qrot(r, [x, 0, z]); return [seat.at[0] + v[0], y, seat.at[2] + v[2]]; };
    const top = seat.at[1];
    boxes.push({ name: "seat", c: at(0, top - 0.02, 0), h: [0.24, 0.02, 0.24], rot: r }, { name: "chair legs", c: at(0, (top - 0.04) / 2, 0), h: [0.2, (top - 0.04) / 2, 0.2], rot: r }, { name: "chair back", c: at(0, top + 0.29, 0.25), h: [0.24, 0.3, 0.02], rot: r });
  }
  const n = perf.frames, sitting = (f: number) => !!perf.phase(f)?.stage;
  const by: Record<string, number> = {}, deepBy: Record<string, { d: number; at: string }> = {};
  const inBy: Record<string, number> = {}, gapBy: Record<string, number> = {}, gapMax: Record<string, number> = {}, sdfs: Record<string, (p: Vec3) => number> = {};
  let clipN = 0, clipDeep = 0, clipAt = "", inN = 0, inDeep = 0, inAt = "", heldN = 0, gapN = 0;
  for (let f = 0; f < n; f++) {
    const w = pose(skel, live.pose(f) as Pose), pts = surface(skel, w);
    // I1: the body in furniture
    let worst = 0, wAt = "";
    for (const p of pts) for (const b of boxes) {
      if (b.name === "seat" && sitting(f) && /^(hips|waist|thigh)$/.test(p.part)) continue;
      const d = depth(b, p.at);
      if (d > worst) { worst = d; wAt = `${p.part}${p.side}@${f} in ${b.name}`; }
    }
    if (worst > 0.015) { clipN++; const k = `${wAt.split(" in ")[1]}/${perf.phase(f)?.stage ?? (live.held("cup", f) || live.held("ball", f) ? "carry" : "walk")}`; by[k] = (by[k] ?? 0) + 1; }
    if (worst > clipDeep) { clipDeep = worst; clipAt = wAt; }
    if (worst > 0.015) { const bn = wAt.split(" in ")[1]; if (!(bn in deepBy) || worst > deepBy[bn].d) deepBy[bn] = { d: worst, at: wAt.split(" in ")[0] }; }
    // I2: the hands and the things
    const hand = pts.filter((p) => /^(palm|thumb|index|middle|ring|pinky)\d?$/.test(p.part));
    for (const [id, it] of Object.entries(items)) {
      const s = live.item(id, f), held = live.held(id, f);
      // (in the thing's own frame, its own distance field: a cup's box had corners out past the cup)
      const sdf = (sdfs[id] ??= thingSdf(it));
      const dOf = (p: Vec3) => -sdf(qrot(qinv(s.rot), [p[0] - s.at[0], p[1] - s.at[1], p[2] - s.at[2]]));
      let deep = 0, dAt = "", near = Infinity;
      for (const p of hand) { const d = dOf(p.at); if (d > deep) { deep = d; dAt = `${p.part}${p.side}@${f} in ${id}`; } near = Math.min(near, Math.abs(d)); }
      const key = `${id}/${held ? "held" : "free"}`;
      if (deep > 0.005) { inN++; inBy[key] = (inBy[key] ?? 0) + 1; }
      if (deep > inDeep) { inDeep = deep; inAt = dAt; }
      if (held) { heldN++; if (near > 0.01) { gapN++; gapBy[id] = (gapBy[id] ?? 0) + 1; gapMax[id] = Math.max(gapMax[id] ?? 0, near); } }
    }
  }
  // I4: the seat clip's frame turning back while sat
  let backs = 0, satN = 0, dir = 0;
  for (let f = 1; f < n; f++) {
    if (perf.phase(f)?.stage !== "sat" || perf.phase(f - 1)?.stage !== "sat") continue;
    satN++;
    const d = Math.sign(perf.played[f].f - perf.played[f - 1].f);
    if (d && dir && d !== dir) backs++;
    if (d) dir = d;
  }
  return {
    clip: (100 * clipN) / n, clipDeep: clipDeep * 100, clipAt, clipBy: by, deepBy: Object.fromEntries(Object.entries(deepBy).map(([k, v]) => [k, `${(v.d * 100).toFixed(1)} cm ${v.at}`])),
    inThing: (100 * inN) / n, inDeep: inDeep * 100, inAt, inBy,
    gripGap: heldN ? (100 * gapN) / heldN : 0, gapBy, gapMax: Object.fromEntries(Object.entries(gapMax).map(([k, v]) => [k, `${(v * 100).toFixed(1)} cm`])),
    sitLoop: satN ? (backs * 30) / satN : 0,
  };
}
