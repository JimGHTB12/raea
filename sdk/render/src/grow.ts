// Organic solids and generated growth, built natively (raea-core-sdf, raea-core-grow) and returned as Solids, so they
// draw with <Mesh shape={solid}> and cut, union, measure and place like any other.
//
//   sdf.smoothUnion(0.05, sdf.sphere(0.3), sdf.capsule([0, 0, 0], [0, 0.6, 0], 0.1)).solid()   a blended body
//   grow.tree({ height: 6, crown: "dome" })     grow.loft([[0, "circle 0.2"], [0.3, "circle 0.3"], [0.6, "circle 0.1"]])
//   grow.terrain({ size: 20, height: 2 })       grow.boulder({ size: 1, seed: 2 })     grow.rosette({ leaves: 21 })
//
// A field is text until .solid() meshes it: building one costs nothing, meshing it is one native call.

import { native } from "@raea/app/native";
import { Solid, type Vec3 } from "./solid";

/** A number as the expressions read it: plain decimals, never an exponent. */
function num(owner: string, n: number): string {
  if (typeof n !== "number" || !Number.isFinite(n)) throw new TypeError(`${owner}: numbers only, got ${JSON.stringify(n)}`);
  const s = n.toFixed(6).replace(/\.?0+$/, "");
  return s === "-0" ? "0" : s;
}

function nums(owner: string, ns: number[]): string {
  return ns.map((n) => num(owner, n)).join(", ");
}

function made(expr: string): Solid {
  const out = native("solid.expr", { expr });
  return new Solid(out.id, out.expr);
}

/** A signed distance field (negative inside), as the text raea-core-sdf reads. Y is up; sizes are metres. */
export class Field {
  readonly text: string;
  constructor(text: string) {
    this.text = text;
  }
  move(x: number, y: number, z: number): Field {
    return new Field(`move(${this.text}, ${nums("move", [x, y, z])})`);
  }
  /** Degrees about x, then y, then z (as a Solid's rotate). */
  rotate(x: number, y: number, z: number): Field {
    return new Field(`rotate(${this.text}, ${nums("rotate", [x, y, z])})`);
  }
  scale(s: number): Field {
    return new Field(`scale(${this.text}, ${num("scale", s)})`);
  }
  /** Both sides: the field and its mirror across x = 0 (a body's left and right from one). */
  mirrorX(): Field {
    return new Field(`mirror_x(${this.text})`);
  }
  /** Grown outward by r, every edge rounded by it. */
  round(r: number): Field {
    return new Field(`round(${this.text}, ${num("round", r)})`);
  }
  /** Hollow: a skin 2t thick on the surface. */
  shell(t: number): Field {
    return new Field(`shell(${this.text}, ${num("shell", t)})`);
  }
  /** Roughened by fractal noise: `amp` metres, `freq` cycles a metre (bark, rock, a lumpy creature). */
  displace(amp: number, freq: number, octaves = 4, seed = 1): Field {
    return new Field(`displace(${this.text}, ${nums("displace", [amp, freq, octaves, seed])})`);
  }
  /** The field meshed (marching tetrahedra, `step` metres a cell: by default a 96th of its size) as a closed solid. */
  solid(step?: number): Solid {
    return made(step === undefined ? `sdf("${this.text}")` : `sdf("${this.text}", ${num("solid", step)})`);
  }
}

const fields = (owner: string, fs: Field[]) => {
  if (fs.length === 0 || fs.some((f) => !(f instanceof Field))) throw new TypeError(`${owner}: give fields (sdf.sphere(...) and the like)`);
  return fs.map((f) => f.text).join(", ");
};

export const sdf = {
  sphere: (r: number) => new Field(`sphere(${num("sphere", r)})`),
  /** w x h x d, centred, its edges rounded by r. */
  box: (w: number, h: number, d: number, r = 0) => new Field(`box(${nums("box", [w, h, d, r])})`),
  /** Radius r, height h, standing along y. */
  cylinder: (r: number, h: number) => new Field(`cylinder(${nums("cylinder", [r, h])})`),
  /** Lying flat in x-z: R to the tube's middle, r the tube's. */
  torus: (R: number, r: number) => new Field(`torus(${nums("torus", [R, r])})`),
  ellipsoid: (rx: number, ry: number, rz: number) => new Field(`ellipsoid(${nums("ellipsoid", [rx, ry, rz])})`),
  /** A rod from a to b with round ends (a limb, a finger, a handle). */
  capsule: (a: Vec3, b: Vec3, r: number) => new Field(`capsule(${nums("capsule", [...a, ...b, r])})`),
  /** A round cone from a (radius ra) to b (radius rb): a tapering limb, a horn, a tail. */
  cone: (a: Vec3, b: Vec3, ra: number, rb: number) => new Field(`cone(${nums("cone", [...a, ...b, ra, rb])})`),
  /** Round cones through points, each [x, y, z, radius]: a tentacle, a root, a cable. */
  tube: (points: [number, number, number, number][]) => {
    if (points.length < 2) throw new TypeError("tube: 2 or more [x, y, z, radius] points");
    return new Field(`tube(${nums("tube", points.flat())})`);
  },
  union: (...fs: Field[]) => new Field(`union(${fields("union", fs)})`),
  /** A union blended within k metres of each join: what makes parts read as one body. */
  smoothUnion: (k: number, ...fs: Field[]) => new Field(`smooth_union(${num("smoothUnion", k)}, ${fields("smoothUnion", fs)})`),
  subtract: (a: Field, b: Field) => new Field(`subtract(${fields("subtract", [a, b])})`),
  /** b carved out of a, the cut's edge rounded by k (an eye socket, a mouth). */
  smoothSubtract: (k: number, a: Field, b: Field) => new Field(`smooth_subtract(${num("smoothSubtract", k)}, ${fields("smoothSubtract", [a, b])})`),
  intersect: (...fs: Field[]) => new Field(`intersect(${fields("intersect", fs)})`),
  smoothIntersect: (k: number, ...fs: Field[]) => new Field(`smooth_intersect(${num("smoothIntersect", k)}, ${fields("smoothIntersect", fs)})`),
};

/** Settings as the generators read them: name=value words, numbers as plain decimals. */
function settings(owner: string, o: Record<string, number | string | undefined>): string {
  return Object.entries(o)
    .filter(([, v]) => v !== undefined)
    .map(([k, v]) => {
      if (typeof v === "string") {
        if (!/^[a-z]+$/.test(v)) throw new TypeError(`${owner}: ${k} is a word, got ${JSON.stringify(v)}`);
        return `${k}=${v}`;
      }
      return `${k}=${num(owner, v as number)}`;
    })
    .join(" ");
}

export interface TreeOptions {
  /** metres (6) */
  height?: number;
  /** the clear trunk below the crown, a share of the height (0.35) */
  trunk?: number;
  /** the crown's width over its height (0.9) */
  width?: number;
  crown?: "dome" | "sphere" | "cone" | "column" | "umbrella" | "vase";
  /** attraction points: more is a denser, twiggier crown (1200) */
  points?: number;
  /** a growth step, a share of the height (0.03) */
  step?: number;
  /** the trunk's radius at the ground (a fortieth of the height) */
  base?: number;
  /** the outer twigs' radius (0.006) */
  tip?: number;
  /** branches turn up (+) or droop (-) as they grow (0.15) */
  tropism?: number;
  /** leaves per outer twig node (3; 0 is bare) */
  leaves?: number;
  /** a leaf's length (0.09) */
  leaf?: number;
  sides?: number;
  seed?: number;
}

export type LoftOutline = string | [number, number][];

/** Generated growth as Solids, built natively: `tree`, `loft`, `terrain`, `boulder`, `rosette` and the rest. Draw one
 * with `<Mesh shape={solid}>`; cut, join, measure and place it like any other Solid. */
export const grow = {
  /** A tree grown by space colonization in its crown, thinned by the pipe model; `part` picks the wood or the leaves
   * (draw them as two meshes with two materials). */
  tree: (o: TreeOptions = {}, part: "all" | "wood" | "leaves" = "all") =>
    made(`tree("${settings("tree", o as Record<string, number | string | undefined>)}", "${part}")`),
  /** Outlines at heights, skinned and capped: each [y, outline], the outline "circle r", "ellipse rx rz", "rect w d",
   * "ngon n r", or [x, z] points. A vase, a bottle, a lamp base, a column. */
  loft: (sections: [number, LoftOutline][], around = 0) => {
    if (sections.length < 2) throw new TypeError("loft: 2 or more [y, outline] sections");
    const text = sections.map(([y, s]) => `${num("loft", y)}: ${typeof s === "string" ? s : s.map((p) => nums("loft", p).replace(",", "")).join(", ")}`).join("; ");
    if (/["]/.test(text)) throw new TypeError("loft: outlines hold no quotes");
    return made(`loft("${text}", ${num("loft", around)})`);
  },
  /** A fractal heightfield as a closed slab, centred, its surface from y 0 up to `height`; `flat` clears a disc in the
   * middle that share of the size wide. */
  terrain: (o: { size?: number; depth?: number; height?: number; cells?: number; freq?: number; octaves?: number; seed?: number; base?: number; flat?: number } = {}) =>
    made(`terrain("${settings("terrain", o)}")`),
  /** A rock about `size` across, `squash` as tall as wide, lumped by noise, resting on y 0. */
  boulder: (o: { size?: number; squash?: number; rough?: number; freq?: number; octaves?: number; seed?: number; step?: number } = {}) =>
    made(`boulder("${settings("boulder", o)}")`),
  /** Leaves at the golden angle, blended from one heart: a succulent, an agave, a lettuce, a flower's petals. */
  rosette: (o: { leaves?: number; length?: number; width?: number; thick?: number; tilt?: number; rise?: number; blend?: number; step?: number } = {}) =>
    made(`rosette("${settings("rosette", o)}")`),
};
