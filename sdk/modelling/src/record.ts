// Recording a build: modelling().
import { manifold, Solid } from "@raea/render/solid";
import { Bone, Built, Modeller, Op, Pose, Step, StepBase, V3, Xf, applyXf, autoWeights, clamp01, easeOut, gizmo, lerp, meshOf } from "./all";

// ---------------------------------------------------------------- recording a build

const vecOf = (axis: "x" | "y" | "z"): V3 => (axis === "x" ? [1, 0, 0] : axis === "y" ? [0, 1, 0] : [0, 0, 1]);

/** Record a model's build: `fn` is called once with a Modeller whose calls are the steps, in order. */
export function modelling(fn: (m: Modeller) => void, defaults: { beats?: number } = {}): Built {
  const steps: Step[] = [];
  const objects = new Map<string, Solid>();
  const colors = new Map<string, string>();
  const bones: Bone[] = [];
  let cursor: V3 = [0, 0, 0];
  let t = 0;
  const D = defaults.beats ?? 2;
  const push = (op: Op, o: Partial<StepBase> | undefined, beats = D) => {
    const b = o?.beats ?? beats;
    steps.push({ ...op, beats: b, view: o?.view, start: t, index: steps.length } as Step);
    t += b;
  };
  const need = (name: string) => {
    const s = objects.get(name);
    if (!s) throw new Error(`modelling: no object ${JSON.stringify(name)} yet (add it first)`);
    return s;
  };
  const transform = (object: string, xf: Xf, o?: { gizmo?: "move" | "rotate" | "scale" } & Partial<StepBase>) => {
    const before = need(object);
    const md = meshOf(before);
    const x: Xf = { ...xf, pivot: xf.pivot ?? (xf.rotate || xf.scale !== undefined ? md.center : undefined) };
    const after = applyXf(before, x);
    const gizmo = o?.gizmo ?? (xf.rotate ? "rotate" : xf.scale !== undefined ? "scale" : "move");
    objects.set(object, after);
    push({ kind: "transform", object, before, after, xf: x, gizmo }, o);
  };
  const boolean = (fn: "minus" | "union" | "intersect") => (object: string, tool: Solid, o?: Partial<StepBase>) => {
    const before = need(object);
    const after = before[fn](tool);
    objects.set(object, after);
    push({ kind: "boolean", object, fn, before, tool, after }, o);
  };
  const m: Modeller = {
    add(object, solid, o) {
      const at = o?.at ?? cursor;
      const after = solid.move(at[0], at[1], at[2]);
      const prev = objects.get(object);
      objects.set(object, prev ? prev.union(after) : after);
      push({ kind: "add", object, after, from: cursor, at }, o);
      cursor = at;
    },
    transform,
    move: (object, by, o) => transform(object, { move: by }, o),
    rotate: (object, degrees, o) => transform(object, { rotate: degrees, pivot: o?.pivot }, o),
    scale: (object, by, o) => transform(object, { scale: by, pivot: o?.pivot }, o),
    minus: boolean("minus"),
    union: boolean("union"),
    intersect: boolean("intersect"),
    mirror(object, axis, o) {
      const before = need(object);
      const n = vecOf(axis);
      const copy = before.mirror(n[0], n[1], n[2]);
      const after = before.union(copy);
      objects.set(object, after);
      push({ kind: "mirror", object, before, copy, after, axis: axis === "x" ? 0 : axis === "y" ? 1 : 2 }, o);
    },
    extrude(object, profile, height, o) {
      const before = objects.get(object) ?? null;
      const local = profile.extrude(height, { twist: o?.twist, scaleTop: o?.scaleTop, divisions: o?.twist ? 8 : 1 });
      const place = o?.place ?? {};
      const placed = applyXf(local, place);
      const after = before ? before.union(placed) : placed;
      objects.set(object, after);
      push({ kind: "extrude", object, before, profile, local, after, place }, o, 3);
    },
    revolve(object, profile, o) {
      const before = objects.get(object) ?? null;
      const place = o?.place ?? {};
      const seg = o?.segments;
      const stages = [60, 120, 180, 240, 300].map((deg) => applyXf(manifold.revolve(profile, { degrees: deg, ...(seg ? { segments: Math.max(3, Math.round((seg * deg) / 360)) } : {}) }), place));
      const full = applyXf(manifold.revolve(profile, seg ? { segments: seg } : {}), place);
      const after = before ? before.union(full) : full;
      objects.set(object, after);
      push({ kind: "revolve", object, before, stages: [...stages, full], after, place, profile }, o, 3);
    },
    sculpt(object, sdf, o) {
      const before = objects.get(object) ?? null;
      const n = o.stages ?? 5, lv = o.level ?? 0, from = o.from ?? -0.4;
      const stages: Solid[] = [];
      for (let i = 0; i < n; i++) {
        const level = lerp(from, lv, easeOut((i + 1) / n));
        try {
          stages.push(manifold.levelSet(sdf, { min: o.min, max: o.max, resolution: o.resolution ?? 48, level }));
        } catch {
          // a level with no surface yet: skip it
        }
      }
      const after = stages[stages.length - 1];
      objects.set(object, after);
      push({ kind: "sculpt", object, before, stages, after }, o, 4);
    },
    smooth(object, o) {
      const before = need(object);
      const after = before.smooth({ angle: o?.angle ?? 180, smoothness: o?.smoothness ?? 0, refine: o?.refine ?? 3 });
      objects.set(object, after);
      push({ kind: "smooth", object, before, after }, o);
    },
    refine(object, n, o) {
      const before = need(object);
      const after = before.refine(n);
      objects.set(object, after);
      push({ kind: "smooth", object, before, after }, o);
    },
    array(object, from, copies, o) {
      const base = need(from);
      const made = copies.map((xf) => applyXf(base, xf));
      const prev = objects.get(object);
      const after = manifold.union(...(prev ? [prev] : []), ...made.length > 1 || prev ? made : [made[0], made[0]]);
      if (from !== object) objects.delete(from);
      objects.set(object, after);
      if (colors.has(from) && !colors.has(object)) colors.set(object, colors.get(from)!);
      push({ kind: "array", object, before: base, copies: made, after, template: from !== object ? from : null }, o, 4);
    },
    color(object, hex) {
      colors.set(object, hex);
    },
    bone(name, head, tail, o) {
      bones.push({ name, head, tail, parent: o?.parent ?? null });
      push({ kind: "bone", bones: [name] }, o, 1);
    },
    bones(list, o) {
      for (const b of list) bones.push({ name: b.name, head: b.head, tail: b.tail, parent: b.parent ?? null });
      push({ kind: "bone", bones: list.map((b) => b.name) }, o, Math.max(2, list.length * 0.5));
    },
    weights(o) {
      push({ kind: "weights", bones: o?.bones ?? bones.map((b) => b.name) }, o, Math.max(2, (o?.bones ?? bones).length));
    },
    pose(fn, o) {
      push({ kind: "pose", fn }, o, 4);
    },
    finish(o) {
      push({ kind: "finish" }, o, 0);
    },
    animate(fn, o) {
      push({ kind: "animate", fn }, o, 16);
    },
    wait(beats) {
      push({ kind: "wait" }, { beats }, beats);
    },
  };
  fn(m);
  for (const b of bones) if (b.parent && !bones.some((x) => x.name === b.parent)) throw new Error(`modelling: bone ${b.name}'s parent ${b.parent} isn't a bone`);

  // the objects as they stand after each step (step i's "after" state), for playback
  const states: Map<string, Solid>[] = [];
  {
    const cur = new Map<string, Solid>();
    for (const s of steps) {
      if ("object" in s && "after" in s) {
        if (s.kind === "array" && s.template) cur.delete(s.template);
        cur.set(s.object, s.after);
      }
      states.push(new Map(cur));
    }
  }
  const finalObjects = new Map<string, { solid: Solid; color: string }>();
  for (const [k, s] of objects) finalObjects.set(k, { solid: s, color: colors.get(k) ?? "#d8d8d8" });

  const poseSteps = steps.filter((s) => s.kind === "pose" || s.kind === "animate") as (Step & ({ kind: "pose"; fn: (u: number) => Pose } | { kind: "animate"; fn: (b: number) => Pose }))[];
  const poseAt = (at: number): Pose => {
    let last: Pose = {};
    for (const s of poseSteps) {
      if (at < s.start) break;
      const u = s.beats > 0 ? clamp01((at - s.start) / s.beats) : 1;
      last = s.kind === "pose" ? s.fn(u) : s.fn(Math.min(at, s.start + s.beats) - s.start);
    }
    return last;
  };

  let w: Map<string, { joints: Uint16Array; weights: Float32Array }> | null = null;
  const built: Built = {
    steps,
    duration: t,
    objects: finalObjects,
    bones,
    stateAt(at) {
      let i = -1;
      for (const s of steps) if (s.start + s.beats <= at) i = s.index;
      return i < 0 ? new Map() : states[i];
    },
    poseAt,
    weights() {
      if (!w) w = autoWeights(finalObjects, bones);
      return w;
    },
  };
  return built;
}
