// Backsplashes.

import { Laid, LaidRun, Opening, Room, WallFrame, WallName, onWall } from "./all";
import { Affine2 } from "raea/math";
import { StandardMaterial } from "raea/pbr";
import { Quat, Transform } from "raea/transform";
import { Cuboid, Mesh3d } from "raea/mesh";

/** A stretch of backsplash: tile on one wall from `u0` to `u1` along it, `y0` to `y1` up. */
export interface Splash {
  wall: WallName;
  u0: number;
  u1: number;
  y0: number;
  y1: number;
}

/** What stops a backsplash's top over a stretch of wall: a wall cabinet's or a hood's underside. */
export interface SplashCap {
  wall: WallName;
  u0: number;
  u1: number;
  bottom: number;
}

/** A backsplash round the kitchen: tile on the wall over every counter of `runs`, on each wall a unit stands against (a
 * corner unit's two), from its top (the stone's back lip) up to what's over it, a wall cabinet of `over` or a `caps`
 * (a hood, the tile up behind it), else `height` over the stone; under a window, up to its sill's underside (`sill` below the hole; `trim` either
 * side of it for the frame). A unit standing more than `counter` + 0.15 m (a pantry, a fridge's housing) breaks it, a
 * stove doesn't (the tile runs behind it, from the stone's height).
 *
 *   <Backsplash pieces={backsplash(ROOM, [BACK, RIGHT], { counter: 1.0, over: [BACK_UP, RIGHT_UP], openings })}
 *     src={staticFile("tiles.jpg")} tile={0.6} /> */
export function backsplash(room: Room, runs: LaidRun[], o: { counter: number; over?: LaidRun[]; caps?: SplashCap[]; openings?: Opening[]; height?: number; trim?: number; sill?: number; against?: number }): Splash[] {
  const { counter, over = [], caps = [], openings = [], height = 0.55, trim = 0.1, sill = 0.05, against = 0.03 } = o;
  const tall = counter + 0.15;
  // a laid unit's footprint on a wall it stands against: its span along it
  const spanOn = (run: LaidRun, l: Laid, f: WallFrame): [number, number] | null => {
    const c = [onWall(run.wall, l.u0, 0), onWall(run.wall, l.u1, 0), onWall(run.wall, l.u0, l.depth), onWall(run.wall, l.u1, l.depth)];
    const uv = c.map((p) => {
      const d = [p[0] - f.origin[0], p[2] - f.origin[2]];
      return [d[0] * f.along[0] + d[1] * f.along[2], d[0] * f.inward[0] + d[1] * f.inward[2]];
    });
    if (Math.min(...uv.map((x) => x[1])) > against) return null;
    const u0 = Math.max(0, Math.min(...uv.map((x) => x[0]))), u1 = Math.min(f.length, Math.max(...uv.map((x) => x[0])));
    return u1 - u0 > 1e-3 ? [u0, u1] : null;
  };
  const out: Splash[] = [];
  for (const [name, f] of Object.entries(room.walls) as [WallName, WallFrame][]) {
    const low: { u0: number; u1: number; y: number }[] = [], high: [number, number][] = [];
    for (const run of runs) {
      for (const l of run.units) {
        const s = spanOn(run, l, f);
        if (!s) continue;
        if (l.height > tall) high.push(s);
        else low.push({ u0: s[0], u1: s[1], y: Math.max(l.height, counter) });
      }
    }
    if (!low.length) continue;
    const tops: { u0: number; u1: number; y: number }[] = [...caps.filter((c) => c.wall === name).map((c) => ({ u0: c.u0, u1: c.u1, y: c.bottom }))];
    for (const run of over) {
      for (const l of run.units) {
        const s = spanOn(run, l, f);
        if (s) tops.push({ u0: s[0], u1: s[1], y: l.bottom });
      }
    }
    for (const op of openings.filter((p) => p.wall === name && p.kind === "window")) {
      tops.push({ u0: op.at - op.width / 2 - trim, u1: op.at + op.width / 2 + trim, y: op.bottom - sill });
    }
    // cut at every edge (edges within 2 cm one cut: a window's trim by a sink's side), and each piece between takes its
    // lowest top and its highest foot
    const cuts = [...low.flatMap((s) => [s.u0, s.u1]), ...high.flat(), ...tops.flatMap((t) => [t.u0, t.u1])].sort((a, b) => a - b)
      .filter((u, i, all) => i === 0 || u - all[i - 1] >= 0.02);
    for (let i = 0; i + 1 < cuts.length; i++) {
      const [a, b] = [cuts[i], cuts[i + 1]], m = (a + b) / 2;
      if (b - a < 1e-3 || high.some(([h0, h1]) => m > h0 && m < h1)) continue;
      const under = low.filter((s) => m > s.u0 && m < s.u1);
      if (!under.length) continue;
      const y0 = Math.max(...under.map((s) => s.y));
      const over = tops.filter((t) => m > t.u0 && m < t.u1).map((t) => t.y);
      const y1 = over.length ? Math.min(...over) : counter + height;
      if (y1 - y0 < 0.02) continue;
      const last = out[out.length - 1];
      if (last && last.wall === name && Math.abs(last.u1 - a) < 1e-4 && Math.abs(last.y0 - y0) < 1e-4 && Math.abs(last.y1 - y1) < 1e-4) last.u1 = b;
      else out.push({ wall: name, u0: a, u1: b, y0, y1 });
    }
  }
  return out;
}

/** A backsplash drawn: each piece a slab `thick` on its wall, its tile picture (`src`, `normalMap`) repeating every
 * `tile` metres from the piece's corner, glazed (`roughness`). */
export function Backsplash({ room, pieces, src, normalMap, tile, color, roughness = 0.25, thick = 0.008 }: { room: Room; pieces: Splash[]; src: string; normalMap?: string; tile: number; color?: string; roughness?: number; thick?: number }) {
  return (
    <>
      {pieces.map((p, i) => {
        const f = room.walls[p.wall], w = p.u1 - p.u0, h = p.y1 - p.y0;
        return (
          <Mesh3d key={i} transform={Transform.fromTranslation(onWall(f, (p.u0 + p.u1) / 2, thick / 2, (p.y0 + p.y1) / 2)).withRotation(Quat.fromEuler("XYZ", 0, f.yaw, 0))} name={`backsplash-${p.wall}-${i}`}>
            <Cuboid half_size={[w / 2, h / 2, thick / 2]} />
            <StandardMaterial base_color={color} perceptual_roughness={roughness} base_color_texture={src} normal_map_texture={normalMap} uv_transform={Affine2.fromScale([w / tile, h / tile])} />
          </Mesh3d>
        );
      })}
    </>
  );
}
