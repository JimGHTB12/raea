// Placement by relations, solved natively (raea-core-place): say how things relate and get their places, measured,
// instead of typing coordinates. Call it once at module level; the same input gives the same layout every frame.
//
//   const lay = place({
//     room: { min: [-2, -2], max: [2, 2] },
//     objects: { desk: [1.4, 0.75, 0.7], chair: [0.5, 0.9, 0.5], lamp: [0.2, 0.45, 0.2] },
//     rules: [{ against: "desk", wall: "+z" }, { near: "chair", of: "desk", dist: 0.15 }, { facing: "chair", of: "desk" },
//             { on: "lamp", of: "desk" }],
//   });
//   <Mesh3d transform={{ translation: lay.desk.center, rotation: Quat.fromEuler(0, lay.desk.yaw, 0) }}>...
//
// Each object's `base` is the centre of its bottom face, `center` its middle (where a centred Cuboid goes), `yaw` its
// turn about y in radians (its front is +z before the turn). `found` says what couldn't hold, in scene lint's shape.

import { native } from "@raea/app/native";

type V3 = [number, number, number];
type Side = "+x" | "-x" | "+z" | "-z";

export type PlaceRule =
  | { on: string; of: string }
  | { against: string; wall: Side }
  | { beside: string; of: string; side: Side; gap?: number }
  | { near: string; of: string; dist?: number }
  | { apart: string; of: string; min?: number }
  | { facing: string; of: string }
  | { align: string; of: string; axis: "x" | "z" }
  | { clear: string; depth?: number };

export interface PlaceObject {
  size: V3;
  /** a start (or, with fixed, the place) for its bottom centre */
  at?: V3;
  /** degrees */
  yaw?: number;
  fixed?: boolean;
}

export interface Placed {
  base: V3;
  center: V3;
  yaw: number;
  size: V3;
}

export interface PlaceFinding {
  part: string;
  check: string;
  level: string;
  says: string;
}

export type Layout = Record<string, Placed> & { readonly found: PlaceFinding[]; readonly errors: number };

export function place(o: { objects: Record<string, V3 | PlaceObject>; rules: PlaceRule[]; room?: { min: [number, number]; max: [number, number] } }): Layout {
  const objects = Object.entries(o.objects).map(([id, v]) => (Array.isArray(v) ? { id, size: v } : { id, ...v }));
  const out = native("place.solve", { objects, rules: o.rules as unknown as Record<string, string | number>[], ...(o.room ? { room: o.room } : {}) });
  const lay: Record<string, Placed> = {};
  for (const p of out.objects) {
    lay[p.id] = { base: p.at, center: [p.at[0], p.at[1] + p.size[1] / 2, p.at[2]], yaw: (p.yaw * Math.PI) / 180, size: p.size };
  }
  const found = out.found.filter((f) => f.level !== "ok");
  Object.defineProperty(lay, "found", { value: found, enumerable: false });
  Object.defineProperty(lay, "errors", { value: out.errors, enumerable: false });
  return lay as Layout;
}
