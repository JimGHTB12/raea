// Openings (bays, doors, windows) and the room drawn.
import type React from "react";
import { Entity } from "raea/3d";
import { Footprint, Room, WallName, onWall } from "./all";
import { Srgba } from "raea/color";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Quat, Transform } from "raea/transform";

export interface Opening {
  wall: WallName;
  /** its middle along the wall */
  at: number;
  width: number;
  bottom: number;
  top: number;
  kind: "window" | "door" | "bay";
  /** drawn in the wall's frame at the opening's middle, on the floor (x along, z in): a frame, glass, a sill, a door */
  children?: React.ReactNode;
}

/** A bay window's make: `width` its mouth in the wall, `depth` how far it stands out, its sides at `angle` (degrees)
 * to the wall; windows from `sill` to `head` in each of its three faces; a window seat (`seat` high, cushioned) fills its
 * floor unless `seat` is false. */
export interface BayOptions {
  width: number;
  depth: number;
  angle?: number;
  sill?: number;
  head?: number;
  seat?: number | false;
  thick?: number;
  wall?: string;
  trim?: string;
  cushion?: string;
}

/** A bay window: the room's wall opened full height `width` wide at `at`, and out past it three faces in a trapezoid,
 * each a window between a panel under the sill and one over the head, mullion posts at its corners, its own floor and
 * ceiling, and a window seat to the trapezoid's shape. Its seat comes as a footprint that faces into the room, for the
 * plan's checks. */
export function bay(room: Room, wall: WallName, at: number, o: BayOptions): Opening & { seat?: Footprint } {
  const f = room.walls[wall], h = room.height;
  const { width: w, depth: d, angle = 45, sill = 0.55, head = 2.2, seat = 0.45, thick: t = 0.1 } = o;
  const [wallC, trim, cushion] = [o.wall ?? "#ece4d4", o.trim ?? "#f4efe4", o.cushion ?? "#c9b79c"];
  const s = d / Math.tan((angle * Math.PI) / 180);
  if (2 * s >= w) throw new Error(`bay: sides at ${angle} degrees out ${d} m meet before the back (mouth ${w} m)`);
  // the inside outline on the plan, in the wall's frame (x along, z into the room: the bay is at -z)
  const pts: [number, number][] = [[-w / 2, 0], [-w / 2 + s, -d], [w / 2 - s, -d], [w / 2, 0]];
  const faces = [0, 1, 2].map((i) => {
    const [a, b] = [pts[i], pts[i + 1]];
    const len = Math.hypot(b[0] - a[0], b[1] - a[1]);
    const u: [number, number] = [(b[0] - a[0]) / len, (b[1] - a[1]) / len];
    // the outward normal (to the left walking a to b, the outline goes round the bay clockwise seen from above)
    const out: [number, number] = [u[1], -u[0]];
    return { mid: [(a[0] + b[0]) / 2 + (out[0] * t) / 2, (a[1] + b[1]) / 2 + (out[1] * t) / 2] as [number, number], len, yaw: Math.atan2(-u[1], u[0]) };
  });
  const glassH = head - sill, m = 0.07;
  // the outline for extrude, counter-clockwise (a clockwise one extrudes to nothing), turned up by rotate(-90, 0, 0)
  const poly = (z0: number) => [...pts].reverse().map(([x, z]) => `${x.toFixed(4)} ${(-z + z0).toFixed(4)}`).join(", ");
  const seatTop = seat === false ? 0 : seat;
  const inward = onWall(f, at, 1), here = onWall(f, at, 0);
  const seatFoot: Footprint | undefined = seat === false ? undefined : (() => {
    const c = [onWall(f, at - w / 2, 0), onWall(f, at + w / 2, 0), onWall(f, at - w / 2, -d), onWall(f, at + w / 2, -d)];
    return { name: `${wall} bay seat`, min: [Math.min(...c.map((p) => p[0])), Math.min(...c.map((p) => p[2]))], max: [Math.max(...c.map((p) => p[0])), Math.max(...c.map((p) => p[2]))], faces: [inward[0] - here[0], inward[2] - here[2]] };
  })();
  return {
    wall, at, width: w, bottom: 0, top: h, kind: "bay", seat: seatFoot,
    children: (
      <Entity transform={Transform.IDENTITY}>
        {faces.map((fc, i) => (
          <Entity key={i} transform={Transform.fromXyz(fc.mid[0], 0, fc.mid[1]).withRotation(Quat.fromEuler("XYZ", 0, fc.yaw, 0))}>
            <Mesh3d transform={Transform.fromXyz(0, sill / 2, 0)} name={`bay-${i}-under`}><Cuboid half_size={[fc.len / 2, sill / 2, t / 2]} /><StandardMaterial base_color={wallC} perceptual_roughness={0.9} /></Mesh3d>
            <Mesh3d transform={Transform.fromXyz(0, (h + head) / 2, 0)} name={`bay-${i}-over`}><Cuboid half_size={[fc.len / 2, (h - head) / 2, t / 2]} /><StandardMaterial base_color={wallC} perceptual_roughness={0.9} /></Mesh3d>
            {/* the sash: a frame round the glass, a muntin across, the glass */}
            {[[0, head - m / 2, fc.len, m], [0, sill + m / 2, fc.len, m], [-fc.len / 2 + m / 2, (sill + head) / 2, m, glassH], [fc.len / 2 - m / 2, (sill + head) / 2, m, glassH], [0, sill + glassH * 0.62, fc.len, 0.035]].map(([x, y, bw, bh], k) => (
              <Mesh3d key={k} transform={Transform.fromXyz(x, y, 0)} name={`bay-${i}-sash-${k}`}><Cuboid half_size={[bw / 2, bh / 2, (t * 0.9) / 2]} /><StandardMaterial base_color={trim} perceptual_roughness={0.5} /></Mesh3d>
            ))}
            <Mesh3d transform={Transform.fromXyz(0, (sill + head) / 2, 0)} name={`bay-${i}-glass`} shadow={false}>
              <Cuboid half_size={[(fc.len - 2 * m) / 2, (glassH - 2 * m) / 2, 0.003]} />
              <StandardMaterial base_color={Srgba.hex("#dfe8ea").with_alpha(0.12)} perceptual_roughness={0.03} clearcoat={1} alpha_mode="Blend" />
            </Mesh3d>
          </Entity>
        ))}
        {/* posts where the faces meet and where the bay meets the wall */}
        {pts.map(([x, z], i) => (
          <Mesh3d key={`p${i}`} transform={Transform.fromXyz(x, h / 2, z - (i === 0 || i === 3 ? 0 : t / 2))} name={`bay-post-${i}`}><Cuboid half_size={[(t * 1.1) / 2, h / 2, (t * 1.1) / 2]} /><StandardMaterial base_color={trim} perceptual_roughness={0.5} /></Mesh3d>
        ))}
        <Mesh3d transform={Transform.fromXyz(0, -0.01, -(d + t) / 2)} name="bay-floor"><Cuboid half_size={[(w + 2 * t) / 2, 0.01, (d + t) / 2]} /><StandardMaterial base_color="#b08a62" perceptual_roughness={0.7} /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, h + 0.02, -(d + t) / 2)} name="bay-ceiling"><Cuboid half_size={[(w + 2 * t) / 2, 0.02, (d + t) / 2]} /><StandardMaterial base_color="#f6f1e6" perceptual_roughness={0.95} /></Mesh3d>
        {seat !== false && (
          <>
            <Mesh3d name="bay-seat" shape={`rotate(extrude("${poly(0)}", ${(seatTop - 0.07).toFixed(3)}), -90, 0, 0)`}><StandardMaterial base_color={trim} perceptual_roughness={0.55} /></Mesh3d>
            <Mesh3d transform={Transform.fromXyz(0, seatTop - 0.07, 0)} name="bay-cushion" shape={`rotate(extrude("${poly(0)}", 0.07), -90, 0, 0)`}><StandardMaterial base_color={cushion} perceptual_roughness={0.95} /></Mesh3d>
          </>
        )}
      </Entity>
    ),
  };
}
