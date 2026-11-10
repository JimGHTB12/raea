// raea/check: a check is a system over the world (docs/direction.feature DR4). `raea check` reads each composition's
// frames into worlds holding the components `raea world` prints (raea-core-facts; their types are facts.gen.ts,
// generated from the Rust), joins the composition's newest render, and runs every check that applies on the frames it
// says it ranges over. A check finds entities, measures, and calls `found`; it never prints. raea's own checks are
// the same thing written in Rust, run under the same names, and a check reads the same for a mesh, a caption, a cut
// or a cue: nothing here is 3D's.
//
//   // checks/clear.ts, run by a rule a controller holds:
//   //   { label: "Captions clear the button band", type: "rule", check: "../checks/clear.ts", values: { band: 0.8 } }
//   import { check, found, type Text } from "raea/check";
//   export default check({ id: "caption-clear", text: "Captions clear the button band" }, (world, frame, values) => {
//     for (const t of world.find({ with: ["Text", "Layer"] })) {
//       const box = world.component<Text>(t, "Text")?.box;
//       if (box && box[3] > Number(values.band ?? 0.8)) found({ about: [t], says: "the caption runs into the button band" });
//     }
//   });

import { createWorld, type World } from "@raea/app/ecs";
import type { About, Finding, Level, Measure } from "./facts.gen";

export type * from "./facts.gen";
export type { World };

/** The frames a check runs on: every frame; the first frame of each shot after the first, and the frame before it;
 * every frame a line is said or sung on; once, on the first frame (for what is the piece's, as its mix); or these. */
export type Frames = "all" | "cuts" | "lines" | "once" | number[];

export interface CheckMeta {
  /** what `raea check --only` names, and its findings' check */
  id: string;
  /** the line it holds the work to */
  text: string;
  /** "render": it reads what only a render has (Contrast, Picture, Mix, Onset, Covers); with none, it waits for one */
  needs?: "render";
  /** how bad what it finds is, where a finding doesn't say (error when unset) */
  level?: Level;
  /** where it runs ("all" when unset) */
  frames?: Frames;
}

/** A rule's values for its check (a controller's `values`). */
export type Values = Record<string, number | string | boolean>;
/** The system: a frame's world, the frame, and the rule's values. */
export type CheckFn = (world: World, frame: number, values: Values) => void;
export interface Check extends CheckMeta {
  run: CheckFn;
}

/** A check: what it holds the work to, and the system that finds where the work doesn't hold. */
export function check(meta: CheckMeta, run: CheckFn): Check {
  return { ...meta, run };
}

/** What a check found: the entities it is about, what it says, and, where they aren't the check's or the frame's,
 * its level and frames (its first, and the frame after its last); the number it turns on and the fix, when there
 * are. */
export interface Found {
  about?: number[];
  says: string;
  level?: Level;
  frames?: [number, number];
  measure?: Omit<Measure, "limit"> & { limit?: number };
  fix?: string;
}

let running: { world: World; frame: number; check: Check; plan: Planned; out: Finding[] } | null = null;

/** Says what the running check found. Only inside a check's system, as raea check runs it. */
export function found(f: Found): void {
  if (!running) throw new Error("found() is for a check's system: raea check runs it on each frame");
  const { world, frame, check: c, plan, out } = running;
  out.push({
    check: c.id,
    // the finding's own level, else its rule's, else the check's
    level: f.level ?? plan.level ?? c.level ?? "error",
    about: (f.about ?? []).map((id) => aboutOf(world, id)),
    frames: f.frames ?? [frame, frame + 1],
    says: f.says,
    measure: f.measure ? { ...f.measure, limit: f.measure.limit ?? null } : null,
    fix: f.fix ?? null,
    by: plan.by ?? "",
  } as Finding);
}

/** An entity as a finding names it: its words, its part's or shot's name, its sound, its component; its layer key. */
function aboutOf(world: World, id: number): About {
  const c = world.components(id) as Record<string, Record<string, unknown> | undefined>;
  const name = [c.Text?.words, c.Part?.name, c.Shot?.name, c.Cue?.source, c.Clip?.source, c.Source?.component].find((v) => typeof v === "string" && v);
  const key = c.Source?.key;
  return { name: String(name ?? `entity ${id}`), ...(typeof key === "string" ? { key } : {}) } as About;
}

/** A frame's facts as `raea world --json` prints them. */
export interface Facts {
  frame: number;
  width: number;
  height: number;
  fps: number;
  entities: { id: number; parent?: number | null; components: Record<string, unknown> }[];
  resources?: Record<string, unknown>;
}

/** A frame's facts as a raea/ecs world: the same components, the frame's size and rate as the `Frame` resource, and
 * what the render added as resources. An entity's references to others (a Covered's `layer`, a relation's `to`)
 * are its id in this world. */
export function worldOf(facts: Facts): World {
  const w = createWorld({ native: false });
  const ids = new Map<number, number>();
  for (const e of facts.entities) ids.set(e.id, w.add({}, e.parent != null && ids.has(e.parent) ? { parent: ids.get(e.parent) } : {}));
  const to = (v: unknown): unknown => {
    if (Array.isArray(v)) return v.map(to);
    if (!v || typeof v !== "object") return v;
    const o = { ...(v as Record<string, unknown>) };
    for (const k of ["layer", "to"]) if (typeof o[k] === "number" && ids.has(o[k] as number)) o[k] = ids.get(o[k] as number);
    return o;
  };
  for (const e of facts.entities) w.insert(ids.get(e.id)!, Object.fromEntries(Object.entries(e.components).map(([k, v]) => [k, to(v)])));
  w.resource("Frame", { frame: facts.frame, width: facts.width, height: facts.height, fps: facts.fps });
  for (const [k, v] of Object.entries(facts.resources ?? {})) w.resource(k, v);
  return w;
}

/** Each check's declaration, as raea check reads it before it reads any frame. */
export function declared(checks: Check[]): CheckMeta[] {
  return checks.map((c, i) => {
    if (!c || typeof c.run !== "function") throw new Error(`check ${i + 1}: its file's default export isn't a check({ id, text }, system)`);
    return { id: c.id, text: c.text, needs: c.needs, level: c.level ?? "error", frames: c.frames ?? "all" };
  });
}

/** Which check runs on which frames with which values, as raea check plans it from a rule: the rule's level (none:
 * the check's own) and where the rule was written. */
export interface Planned {
  check: number;
  frames: number[];
  values: Values;
  level?: Level | null;
  by?: string;
}

/** Every planned check run on its frames' worlds: what they found, a run of frames finding the same thing as one
 * finding over those frames, and each check that threw, with what it threw. */
export function runChecks(checks: Check[], plan: Planned[], worlds: Facts[]): { findings: Finding[]; threw: { check: string; says: string }[] } {
  const out: Finding[] = [];
  const threw: { check: string; says: string }[] = [];
  for (const facts of worlds) {
    const here = plan.filter((p) => p.frames.includes(facts.frame));
    if (!here.length) continue;
    const world = worldOf(facts);
    for (const p of here) {
      const c = checks[p.check];
      running = { world, frame: facts.frame, check: c, plan: p, out };
      try {
        c.run(world, facts.frame, p.values);
      } catch (e) {
        if (!threw.some((t) => t.check === c.id)) threw.push({ check: c.id, says: `at frame ${facts.frame}: ${e instanceof Error ? e.message : String(e)}` });
      } finally {
        running = null;
      }
    }
  }
  return { findings: joined(out), threw };
}

/** Findings that say the same about the same on frames one after another, as one over all of them. */
function joined(fs: Finding[]): Finding[] {
  const out: Finding[] = [];
  const last = new Map<string, Finding>();
  for (const f of fs) {
    const k = JSON.stringify([f.check, f.level, f.says, f.about, f.measure, f.fix, f.by]);
    const prev = last.get(k);
    const [a, b] = (f.frames ?? [0, 0]) as [number, number];
    const pf = prev?.frames as [number, number] | undefined;
    // a run's end is the frame after its last, so the next frame starts where it ends
    if (prev && pf && a <= pf[1]) {
      prev.frames = [pf[0], Math.max(pf[1], b)] as Finding["frames"];
      continue;
    }
    out.push(f);
    last.set(k, f);
  }
  return out;
}
