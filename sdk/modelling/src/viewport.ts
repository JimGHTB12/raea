// The viewport's fixed pieces.
import { crossSection, manifold, Solid } from "@raea/render/solid";
import { writeGlb, linearRgba } from "@raea/asset/glb";
import { V3, lerp, lerp3, smooth } from "./all";

// ---------------------------------------------------------------- the viewport's fixed pieces

const BG_TOP = "#4a4a4a", BG_BOTTOM = "#2a2a2a";

let viewportFiles: { bg: string; grid: string } | null = null;

export function viewportOf() {
  if (viewportFiles) return viewportFiles;
  // a sphere inside out, its colour by height: the dark grey gradient behind everything
  const pos: number[] = [], col: number[] = [], idx: number[] = [];
  const R = 80, NS = 24, NR = 24;
  const top = linearRgba(BG_TOP), bot = linearRgba(BG_BOTTOM);
  for (let i = 0; i <= NR; i++) {
    const phi = (i / NR) * Math.PI;
    const y = Math.cos(phi);
    const k = smooth((y + 0.35) / 0.9);
    for (let j = 0; j <= NS; j++) {
      const th = (j / NS) * Math.PI * 2;
      pos.push(R * Math.sin(phi) * Math.cos(th), R * y, R * Math.sin(phi) * Math.sin(th));
      col.push(lerp(bot[0], top[0], k), lerp(bot[1], top[1], k), lerp(bot[2], top[2], k), 1);
    }
  }
  for (let i = 0; i < NR; i++)
    for (let j = 0; j < NS; j++) {
      const a = i * (NS + 1) + j, b = a + 1, c = a + NS + 1, d = c + 1;
      idx.push(a, b, c, b, d, c);
    }
  const bg = writeGlb({ primitives: [{ positions: pos, colors: col, indices: idx, material: { color: "#ffffff", unlit: true, doubleSided: true } }] });
  // the floor grid: minor lines every 0.25 near the middle, major every 1 out to 40, axis lines red (x) and green
  // (z, Blender's y); every line's colour fades into the floor's grey with distance (the vertex colours)
  const gp: number[] = [], gc: number[] = [];
  const floor = linearRgba("#383838");
  const line = (a: V3, b: V3, c: string, fade: number, segs: number) => {
    const cc = linearRgba(c);
    for (let s = 0; s < segs; s++) {
      for (const t of [s / segs, (s + 1) / segs]) {
        const p = lerp3(a, b, t);
        const k = smooth(Math.hypot(p[0], p[2]) / fade);
        gp.push(p[0], 0, p[2]);
        gc.push(lerp(cc[0], floor[0], k), lerp(cc[1], floor[1], k), lerp(cc[2], floor[2], k), 1);
      }
    }
  };
  for (let i = -24; i <= 24; i++) {
    if (i === 0) continue;
    const x = i * 0.25;
    if (i % 4 === 0) continue;
    line([x, 0, -6], [x, 0, 6], "#434343", 6, 12);
    line([-6, 0, x], [6, 0, x], "#434343", 6, 12);
  }
  for (let i = -40; i <= 40; i++) {
    if (i === 0) continue;
    line([i, 0, -40], [i, 0, 40], "#555555", 22, 40);
    line([-40, 0, i], [40, 0, i], "#555555", 22, 40);
  }
  line([-40, 0, 0], [40, 0, 0], "#a8404c", 30, 60);
  line([0, 0, -40], [0, 0, 40], "#6f9a2c", 30, 60);
  const grid = writeGlb({ primitives: [{ positions: gp, colors: gc, mode: "lines", material: { color: "#ffffff", unlit: true } }] });
  viewportFiles = { bg, grid };
  return viewportFiles;
}

let finalBgs = new Map<string, string>();

export function flatBackdrop(color: string): string {
  let src = finalBgs.get(color);
  if (src) return src;
  const pos: number[] = [], idx: number[] = [];
  const R = 80, N = 16;
  for (let i = 0; i <= N; i++)
    for (let j = 0; j <= N; j++) {
      const phi = (i / N) * Math.PI, th = (j / N) * Math.PI * 2;
      pos.push(R * Math.sin(phi) * Math.cos(th), R * Math.cos(phi), R * Math.sin(phi) * Math.sin(th));
    }
  for (let i = 0; i < N; i++)
    for (let j = 0; j < N; j++) {
      const a = i * (N + 1) + j, b = a + 1, c = a + N + 1, d = c + 1;
      idx.push(a, b, c, b, d, c);
    }
  src = writeGlb({ primitives: [{ positions: pos, indices: idx, material: { color, unlit: true, doubleSided: true } }] });
  finalBgs.set(color, src);
  return src;
}

// gizmo parts, unit size: arrows, rings and scale handles along +x (turned onto each axis)
let gizmoParts: Record<string, Solid> | null = null;

export function gizmo() {
  if (gizmoParts) return gizmoParts;
  const shaft = manifold.cylinder(0.014, 0.78).move(0, 0, 0.39 + 0.12).rotate(0, 90, 0);
  gizmoParts = {
    arrow: shaft.union(manifold.cone(0.055, 0, 0.22).move(0, 0, 0.9).rotate(0, 90, 0)),
    handle: shaft.union(manifold.box(0.1, 0.1, 0.1).move(0.95, 0, 0)),
    ring: manifold.torus(0.95, 0.014).rotate(0, 90, 0),
    ball: manifold.sphere(0.045),
    // the 3D cursor: a ring of alternating red and white arcs (facing +z) and four black ticks
    cursorRed: crossSection.circle(1, 48).minus(crossSection.circle(0.8, 48)).intersect(crossSection.of([[[0, 0], [2, 0], [2, 2], [0, 2]], [[0, 0], [-2, 0], [-2, -2], [0, -2]]])).extrude(0.02),
    cursorWhite: crossSection.circle(1, 48).minus(crossSection.circle(0.8, 48)).intersect(crossSection.of([[[0, 0], [0, 2], [-2, 2], [-2, 0]], [[0, 0], [0, -2], [2, -2], [2, 0]]])).extrude(0.02),
    cursorTicks: manifold.union(
      manifold.box(0.7, 0.08, 0.02).move(1.45, 0, 0), manifold.box(0.7, 0.08, 0.02).move(-1.45, 0, 0),
      manifold.box(0.08, 0.7, 0.02).move(0, 1.45, 0), manifold.box(0.08, 0.7, 0.02).move(0, -1.45, 0)),
    // a bone: Blender's octahedron, length 1 along +y, its waist a tenth of the way up
    bone: manifold.hull([[0, 0, 0], [0, 1, 0], [0.1, 0.1, 0.1], [-0.1, 0.1, 0.1], [0.1, 0.1, -0.1], [-0.1, 0.1, -0.1]]),
    boneEnd: manifold.sphere(0.05),
  };
  return gizmoParts;
}

/** the module's state its files assign (each field was a top-level let) */ export const S: { boneWire: string | null; boneWireSel: string | null } = { boneWire: null, boneWireSel: null };

// axis colours as Blender draws them: x red, its z (our y, up) blue, its y (our z, depth) green
export const AXIS: [string, string, string] = ["#ff3352", "#2890ff", "#8bdc00"];

// each axis's turn of the +x part onto it (radians, Euler XYZ)
export const AXIS_ROT: V3[] = [[0, 0, 0], [0, 0, Math.PI / 2], [0, -Math.PI / 2, 0]];
