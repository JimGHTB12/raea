// A room, its walls, and runs of units along them.
import type React from "react";
import { Model, type Vec3, Entity } from "raea/3d";
import { modelBounds } from "@raea/render/scene";
import { Opening } from "./all";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Quat, Transform } from "raea/transform";

export type WallName = "back" | "right" | "front" | "left";

export interface RoomSpec {
  /** along x, metres (the back wall's length) */
  width: number;
  /** along z (the side walls' length) */
  depth: number;
  height: number;
}

/** A wall's frame: its interior face starts at `origin` (on the floor) and runs `length` along `along`; `inward` points
 * into the room. Walls go round clockwise seen from above: back (z = 0, along +x), right (x = width, along +z), front
 * (z = depth, along -x), left (x = 0, along -z), so a run turning a corner carries on along the next. */
export interface WallFrame {
  name: WallName;
  origin: Vec3;
  along: Vec3;
  inward: Vec3;
  length: number;
  /** the turn about +y that takes a unit's +x along the wall and its +z into the room */
  yaw: number;
}

export interface Room extends RoomSpec {
  walls: Record<WallName, WallFrame>;
}

export function defineRoom(spec: RoomSpec): Room {
  const { width: w, depth: d } = spec;
  const frame = (name: WallName, origin: Vec3, along: Vec3, length: number): WallFrame => {
    const inward: Vec3 = [-along[2], 0, along[0]];
    return { name, origin, along, inward, length, yaw: Math.atan2(-along[2], along[0]) };
  };
  return {
    ...spec,
    walls: {
      back: frame("back", [0, 0, 0], [1, 0, 0], w),
      right: frame("right", [w, 0, 0], [0, 0, 1], d),
      front: frame("front", [w, 0, d], [-1, 0, 0], w),
      left: frame("left", [0, 0, d], [0, 0, -1], d),
    },
  };
}

/** A point `u` along a wall, `v` into the room from its face, `y` up. */
export function onWall(f: WallFrame, u: number, v = 0, y = 0): Vec3 {
  return [f.origin[0] + f.along[0] * u + f.inward[0] * v, y, f.origin[2] + f.along[2] * u + f.inward[2] * v];
}

/** Where on a wall's face a <Decal> lies (`u` along it, `y` up) and the way the wall faces: spread into a Decal
 * (`<Decal src {...onSurface(room, "back", 2, 1.2)} size={0.3} />`), the picture upright on the wall. */
export function onSurface(room: Room, wall: WallName, u: number, y: number): { position: Vec3; normal: Vec3 } {
  const f = room.walls[wall];
  return { position: onWall(f, u, 0, y), normal: f.inward };
}

/** Which of a model's own axes is its front (the pack's cupboards face +z; its appliances +x or -x). */
export type Facing = "+x" | "-x" | "+z" | "-z";

export const FACE_YAW: Record<Facing, number> = { "+z": 0, "+x": -Math.PI / 2, "-x": Math.PI / 2, "-z": Math.PI };

export interface Unit {
  src: string;
  /** its front (default "+z") */
  front?: Facing;
  /** room along the wall it takes, if not its measured width (an appliance in a slot wider than itself) */
  width?: number;
  /** set back off the wall (metres) */
  off?: number;
  /** its part in the kitchen's work: the work triangle's corners, the landing space beside them */
  role?: "sink" | "stove" | "fridge";
  name?: string;
  /** dressing in the unit's own frame (x along the wall from its left edge, z into the room from the wall): its doors,
   * drawers, handles, an appliance in a housing */
  children?: React.ReactNode;
}

/** A unit laid in its run: where along the wall it starts and ends, how deep it stands, and its model's offset. */
export interface Laid extends Unit {
  u0: number;
  u1: number;
  depth: number;
  /** the model in the unit's frame: turned to face into the room, its box's back left corner at the frame's origin */
  rotation: Vec3;
  position: Vec3;
  /** its top and its bottom over the floor (a wall cabinet's bottom is its underside) */
  height: number;
  bottom: number;
}

export interface LaidRun {
  wall: WallFrame;
  units: Laid[];
  /** where the run ends along its wall */
  end: number;
}

/** Units one after another along a wall from `from`, each against the wall and the one before, from their boxes. */
export function layRun(room: Room, wall: WallName, from: number, units: Unit[]): LaidRun {
  const f = room.walls[wall];
  let u = from;
  const laid = units.map((it) => {
    const rotation: Vec3 = [0, FACE_YAW[it.front ?? "+z"], 0];
    const b = modelBounds(it.src, rotation);
    const w = it.width ?? b.max[0] - b.min[0];
    // centred in its slot along the wall, its back on the wall
    const slack = w - (b.max[0] - b.min[0]);
    const position: Vec3 = [-b.min[0] + slack / 2, 0, -b.min[2] + (it.off ?? 0)];
    const l: Laid = { ...it, u0: u, u1: u + w, depth: b.max[2] - b.min[2] + (it.off ?? 0), rotation, position, height: b.max[1], bottom: b.min[1] };
    u += w;
    return l;
  });
  if (u > f.length + 1e-3) throw new Error(`layRun: the ${wall} run reaches ${u.toFixed(3)} m, past the wall's ${f.length} m`);
  return { wall: f, units: laid, end: u };
}

/** A point on a run's counter: `u` along its wall, `v` in from the wall. */
export function onRun(run: LaidRun, u: number, v: number, y = 0): Vec3 {
  return onWall(run.wall, u, v, y);
}

/** A laid unit's centre on the floor, in the world. */
export function unitCentre(run: LaidRun, l: Laid): Vec3 {
  return onWall(run.wall, (l.u0 + l.u1) / 2, l.depth / 2);
}

/** A run drawn: each unit at its place, turned into the room, with its dressing. */
export function Run({ run }: { run: LaidRun }) {
  const f = run.wall;
  return (
    <>
      {run.units.map((l, i) => (
        <Entity key={i} transform={Transform.fromTranslation(onWall(f, l.u0)).withRotation(Quat.fromEuler("XYZ", 0, f.yaw, 0))}>
          <Model name={l.name} src={l.src} rotation={l.rotation} position={l.position} />
          {l.children}
        </Entity>
      ))}
    </>
  );
}

/** The room's shell: floor, ceiling and walls (outside its interior faces, `thick` deep), cut for its openings.
 * `ceiling={false}` leaves the room open above, to look down on it as a plan. */
export function Room({ room, openings = [], thick = 0.1, wall = "#ece4d4", floor = "#b08a62", ceiling = "#f6f1e6", children }: { room: Room; openings?: Opening[]; thick?: number; wall?: string; floor?: string; ceiling?: string | false; children?: React.ReactNode }) {
  const { width: w, depth: d, height: h } = room;
  return (
    <>
      <Mesh3d transform={Transform.fromXyz(w / 2, -0.01, d / 2)} name="floor"><Cuboid half_size={[(w + 2 * thick) / 2, 0.01, (d + 2 * thick) / 2]} /><StandardMaterial base_color={floor} perceptual_roughness={0.7} /></Mesh3d>
      {ceiling !== false && <Mesh3d transform={Transform.fromXyz(w / 2, h + 0.02, d / 2)} name="ceiling"><Cuboid half_size={[(w + 2 * thick) / 2, 0.02, (d + 2 * thick) / 2]} /><StandardMaterial base_color={ceiling} perceptual_roughness={0.95} /></Mesh3d>}
      {(Object.keys(room.walls) as WallName[]).map((name) => {
        const f = room.walls[name];
        const holes = openings.filter((o) => o.wall === name).map((o) => [o.at - o.width / 2, o.at + o.width / 2, o.bottom, o.top] as const).sort((p, q) => p[0] - q[0]);
        // slabs between and round the holes; the wall runs on past both ends by its thickness, closing the corners
        const slabs: [number, number, number, number][] = [];
        let a = -thick;
        for (const [h0, h1, y0, y1] of holes) {
          if (h0 > a) slabs.push([a, h0, 0, h]);
          slabs.push([h0, h1, 0, y0], [h0, h1, y1, h]);
          a = h1;
        }
        if (a < f.length + thick) slabs.push([a, f.length + thick, 0, h]);
        return (
          <Entity key={name} transform={Transform.fromTranslation(f.origin).withRotation(Quat.fromEuler("XYZ", 0, f.yaw, 0))}>
            {slabs.filter(([p, q, y0, y1]) => q - p > 1e-3 && y1 - y0 > 1e-3).map(([p, q, y0, y1], i) => (
              <Mesh3d key={i} transform={Transform.fromXyz((p + q) / 2, (y0 + y1) / 2, -thick / 2)} name={`${name}-wall-${i}`}>
                <Cuboid half_size={[(q - p) / 2, (y1 - y0) / 2, thick / 2]} />
                <StandardMaterial base_color={wall} perceptual_roughness={0.9} />
              </Mesh3d>
            ))}
            {openings.filter((o) => o.wall === name).map((o, i) => <Entity key={`o${i}`} transform={Transform.fromXyz(o.at, 0, 0)}>{o.children}</Entity>)}
          </Entity>
        );
      })}
      {children}
    </>
  );
}
