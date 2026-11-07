// An assembly into raea's native ECS world: a row per part (its labels, place and size), per anchor and per join.
// Then the labels are claims the bench checks: a part that says it affords "sit" must pass the seat check itself.
import { createWorld } from "raea/ecs";
import * as C from "./checks.ts";
import type { Assembly } from "./rel.ts";

// affordances become marker components, as Bevy games mark things (a Sittable, a Graspable): found by a query
export const MARKER: Record<string, string> = {
  sit: "Sittable", support: "Support", grasp: "Graspable", contain: "Container", walk: "Walkable", climb: "Climbable",
  open: "Openable", hang: "Hangable", light: "LightSource",
};

/** the object as a parent entity, each part a child: Name, Part (role, material), Place, Size, a marker per affordance */
export function toWorld(a: Assembly, w = createWorld()) {
  const root = w.add({ Name: { name: a.name }, Object: { parts: a.parts.length, joins: a.joins.length } });
  const ids: Record<string, number> = {};
  for (const p of a.parts) {
    const c = p.center, s = p.size, l = p.labels;
    const comps: Record<string, unknown> = {
      Name: { name: p.name },
      Part: { role: String(l.role ?? ""), material: String(l.material ?? ""), volume: p.solid.volume() },
      Place: { x: c[0], y: c[1], z: c[2] }, Size: { w: s[0], h: s[1], d: s[2] },
    };
    for (const f of l.affords ?? []) comps[MARKER[f]] = {};
    for (const [k, v] of Object.entries(l)) if (!["role", "material", "affords"].includes(k)) comps[k] = v;
    ids[p.name] = w.add(comps, { parent: root });
  }
  for (const j of a.joins) w.add({ Join: { kind: j.kind, a: j.a, b: j.b } }, { parent: root });
  return { world: w, root, ids };
}

/** each label is a claim; the geometry has to back it */
export function claims(a: Assembly): C.Result[] {
  const out: C.Result[] = [];
  for (const p of a.parts) for (const f of p.labels.affords ?? []) {
    const S = C.sample(p.solid);
    if (f === "sit") out.push({ ...C.seat(S, [0.35, 0.85], 0.25), check: `${p.name} affords sit` }); // any sitting height: a chair to a bar stool
    if (f === "contain") out.push({ ...C.holdsWater(p.solid, S), check: `${p.name} affords contain` });
    if (f === "support") {
      const r = C.grounded(S);
      out.push({ ...r, check: `${p.name} affords support (reaches the ground)` });
    }
    if (f === "grasp") {
      // a hand fits: the part's smallest dimension is finger-sized and it has a hole a finger goes through
      const s = p.size, thin = Math.min(...s), ok = thin >= 0.005 && thin <= 0.04 && p.solid.genus() >= 1;
      out.push({ check: `${p.name} affords grasp`, ok, got: `thinnest ${(thin * 100).toFixed(1)} cm, genus ${p.solid.genus()}` });
    }
  }
  return out;
}
