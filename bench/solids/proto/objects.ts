// Objects in two styles. A: coordinates, as an agent writes with today's API (it must know manifold is z-up and
// work out every centre). B: relations over rel.ts. Plus red-team variants each check has to catch.
import { manifold } from "raea/3d";
import {
  Assembly, Part, assembly, attached, ball, box, cm, corners, deg, ground, limb, m, post, rests, roundedBox, strut, JOIN, type Anchor,
} from "./rel.ts";

type Solid = ReturnType<typeof manifold.box>;
export interface Obj { name: string; kind: "chair" | "stool" | "mug" | "tree"; solid: Solid; asm?: Assembly; redTeam?: string; arm?: "A" | "B" }

// ---- A: coordinates. The seat's centre is at y = 0.45 - 0.02, a leg's centre at its half height, and so on.
function chairA(o: { legGap?: number; seatY?: number } = {}): Solid {
  const seatY = o.seatY ?? 0.45, legH = seatY - 0.04;
  const seat = manifold.box(0.45, 0.04, 0.45).move(0, seatY - 0.02, 0);
  const legs = [[-0.2, -0.2], [0.2, -0.2], [0.2, 0.2], [-0.2, 0.2]].map(([x, z], i) =>
    manifold.cylinder(0.02, legH).rotate(-90, 0, 0).move(x, legH / 2 + (i === 0 ? (o.legGap ?? 0) : 0), z));
  const back = manifold.box(0.45, 0.45, 0.03).move(0, seatY + 0.225, -0.21);
  return manifold.union(seat, back, ...legs);
}

// ---- B: relations. No centre is computed in the program.
function chairB(o: { seatHeight?: number; legs?: "under" | "one side"; lie?: boolean } = {}): Assembly {
  const seatHeight = cm((o.seatHeight ?? 45) * 1);
  let seat = roundedBox("seat", cm(45), cm(4), cm(45), cm(1)).label({ role: "seat", affords: ["sit"], material: "oak" });
  const legH = m(seatHeight - cm(4));
  seat = seat.placeOn(ground, legH);
  const spots = o.legs === "one side"
    ? corners(seat, "bottom", cm(3)).map((c, i) => ({ ...c, at: [c.at[0] * 0.15 - 0.17, c.at[1], c.at[2]] as typeof c.at }))
    : corners(seat, "bottom", cm(3));
  const legs = spots.map((c, i) => strut(`leg${i}`, [c.at[0], 0, c.at[2]], [c.at[0], c.at[1] + JOIN, c.at[2]], cm(2)).label({ role: "leg", affords: ["support"], material: "oak" }));
  const back = box("back", cm(45), cm(45), cm(3)).placeOn(seat.top).alignTo(seat, "back")
    .label({ role: "backrest", material: "oak", ...(o.lie ? { affords: ["sit" as const] } : {}) });
  return assembly("chair", [seat, ...legs, back], [...legs.map((l) => rests(seat, l)), attached(back, seat)]);
}

function stoolB(): Assembly {
  const legH = cm(60);
  const top = post("top", cm(17), cm(4)).placeOn(ground, legH).label({ role: "seat", affords: ["sit"], material: "walnut" });
  // three legs splayed from under the top's rim out to the floor, flat at both ends
  const legs = [0, 120, 240].map((a, i) => {
    const t = (a * Math.PI) / 180;
    return strut(`leg${i}`, [Math.cos(t) * 0.2, 0, Math.sin(t) * 0.2], [Math.cos(t) * 0.12, legH + 0.001, Math.sin(t) * 0.12], cm(1.8), cm(1.5)).label({ role: "leg", affords: ["support"], material: "walnut" });
  });
  return assembly("stool", [top, ...legs], legs.map((l) => rests(top, l)));
}

function mugB(o: { hole?: boolean } = {}): Assembly {
  let body = post("body", cm(4), cm(10)).placeOn(ground).hollow(cm(0.4), "top")
    .label({ role: "body", affords: ["contain"], material: "glazed ceramic" });
  if (o.hole) body = new Part("body", body.solid.minus(manifold.cylinder(0.008, 0.05).rotate(-90, 0, 0)), body.own, body.labels);
  // the handle: a ring (manifold's torus already stands upright, facing front), its inner side sunk into the body's right wall
  const ring = new Part("handle", manifold.torus(0.028, 0.006)).label({ role: "handle", affords: ["grasp"], material: "glazed ceramic" });
  const handle = ring.centerOn([0, 0.055, 0], "xyz").beside(body, "right", cm(-1.5));
  return assembly("mug", [body, handle], [attached(handle, body)]);
}

function oakB(o: { float?: number; flat?: boolean } = {}): Assembly {
  const base: Anchor = { ...ground, at: [0, o.float ?? 0, 0] };
  const trunk = limb("trunk", base, m(4), cm(30), cm(18), deg(0), deg(0), 0).standOn(base).label({ role: "trunk", material: "bark", affords: ["climb"] })
  const parts: Part[] = [trunk];
  const grow = (parent: Part, depth: number, len: number, r: number) => {
    if (depth === 0) return;
    for (let k = 0; k < 3; k++) {
      const b = limb(`${parent.name}.${k}`, parent.anchor("tip"), m(len), cm(r), cm(r * 0.6), deg(k * 120 + depth * 37), deg(o.flat ? 85 : 20 + (3 - depth) * 15), 0.1).label({ role: "branch", material: "bark" });
      parts.push(b);
      grow(b, depth - 1, len * 0.7, r * 0.6);
    }
  };
  grow(trunk, 3, 3, 16);
  const tips = parts.filter((p) => !parts.some((q) => q.name.startsWith(p.name + ".")));
  for (const t of tips) parts.push(ball(`${t.name}.leaves`, m(1.1)).centerOn(t.anchor("tip"), "xyz").label({ role: "foliage", material: "oak leaves" }));
  return assembly("oak", parts, []);
}

export function objects(): Obj[] {
  const asm = (kind: Obj["kind"], a: Assembly, redTeam?: string): Obj => ({ name: a.name, kind, solid: a.solid, asm: a, redTeam });
  return [
    { name: "chair (A, coordinates)", kind: "chair", solid: chairA(), arm: "A" },
    { ...asm("chair", chairB()), name: "chair (B, relations)" },
    { ...asm("stool", stoolB()), name: "three-legged stool (B)" },
    { ...asm("mug", mugB()), name: "mug (B)" },
    { ...asm("tree", oakB()), name: "oak (B)" },
    // red team: each should fail the check named
    { name: "chair, a leg 3 cm short", kind: "chair", solid: chairA({ legGap: 0.03 }), redTeam: "one piece" },
    { name: "chair, seat at 1 m", kind: "chair", solid: chairA({ seatY: 1.0 }), redTeam: "a seat" },
    { ...asm("chair", chairB({ legs: "one side" })), name: "chair, legs bunched at one side", redTeam: "stands (mass over footprint)" },
    { name: "chair, built 4x too big", kind: "chair", solid: chairA().scale(4), redTeam: "real-world size" },
    { ...asm("mug", mugB({ hole: true })), name: "mug, hole in the bottom", redTeam: "holds water" },
    { ...asm("tree", oakB({ float: 0.5 })), name: "oak, floating 0.5 m", redTeam: "on the ground" },
    { ...asm("tree", oakB({ flat: true })), name: "oak, branches grown flat (the umbrella)", redTeam: "tree form" },
    { ...asm("chair", chairB({ lie: true })), name: "chair, its backrest labelled sittable", redTeam: "back affords sit" },
  ];
}
