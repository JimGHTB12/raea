// Prototype for docs/direction.feature DR1-DR9: one world in raea/ecs's form (plain bun, no engine), holding a 2D
// lyric layer, a 3D rider and seat, a footage cut and a sound cue; checks as systems; findings and a note as entities;
// cheats that must fail. Nothing here imports Bevy.
import { createWorld, type World } from "../../../sdk/app/src/ecs.ts";

type Finding = { check: string; level: "error" | "warn" | "note"; about: number[]; says: string; measure?: { name: string; value: number; limit: number; unit: string } };
const found = (w: World, f: Finding) => w.add({ Finding: f as never });
const findings = (w: World) => w.find({ with: ["Finding"] }).map((id) => w.component<Finding>(id, "Finding")!);

function build(opts: { extruded?: boolean; masked?: boolean; riderTurn?: number; riderInBack?: number; cutAt?: number; cueAt?: number } = {}): World {
  const w = createWorld({ native: false });
  // a 3D layer with a seat and its rider
  const l3 = w.add({ Layer: { medium: "3d", order: 0 } });
  const seat = w.add({ Part: { component: "Chair", kind: "furniture/chair", yaw: 1.2 }, Source: { file: "components/Chair.tsx", line: 1 } }, { parent: l3 });
  const rider = w.add({ Part: { component: "Computer", kind: "cast/computer", yaw: opts.riderTurn ?? 1.2 }, Rides: { seat, socket: "seat", offsetMm: 0, inSeatMm: opts.riderInBack ?? 0 } }, { parent: l3 });
  // a 2D type layer over it, one sung line
  const l2 = w.add({ Layer: { medium: opts.extruded ? "3d" : "2d-type", order: 1 } });
  const line = w.add({ Line: { from: 2.0, to: 4.0 } });
  const text = w.add({ Text: { words: "OUT OF THE GREY", extruded: opts.extruded ? 0.05 : 0, x0: 0.55, x1: 0.93, y0: 0.2, y1: 0.4 }, SungAt: { line } }, { parent: l2 });
  if (opts.masked ?? true) w.insert(text, { MaskedBy: { part: rider } });
  // a footage cut against a word's end, and a cue against its cause
  w.add({ Clip: { source: "take1.mov", out: opts.cutAt ?? 3.2 }, Word: { end: 3.2 } });
  w.add({ Cue: { source: "thud.wav", at: opts.cueAt ?? 5.0 }, CausedBy: { at: 5.0, what: rider } });
  return w;
}

// ---- checks: each a system over the world; the same shape whatever the medium
const checks: Record<string, (w: World) => void> = {
  "type-2d-masked": (w) => {
    for (const id of w.find({ with: ["Text", "SungAt"] })) {
      const t = w.component<{ extruded: number; x0: number; x1: number; y0: number; y1: number }>(id, "Text")!;
      const layer = w.component<{ medium: string }>(w.parent(id)!, "Layer")!;
      if (t.extruded > 0 || layer.medium !== "2d-type") found(w, { check: "type-2d-masked", level: "error", about: [id], says: `the line is ${layer.medium} type with ${t.extruded} m of depth: the ask said 2D` });
      if (!w.component(id, "MaskedBy")) found(w, { check: "type-2d-masked", level: "error", about: [id], says: "nothing masks the line: it sits over the subject" });
      if (t.x0 < 0.03 || t.x1 > 0.97 || t.y0 < 0.03 || t.y1 > 0.97) found(w, { check: "type-2d-masked", level: "error", about: [id], says: "the line runs off the frame" });
    }
  },
  "rider-with-seat": (w) => {
    for (const id of w.find({ with: ["Rides"] })) {
      const r = w.component<{ seat: number; inSeatMm: number }>(id, "Rides")!, me = w.component<{ yaw: number }>(id, "Part")!, seat = w.component<{ yaw: number }>(r.seat, "Part")!;
      const turn = Math.abs(me.yaw - seat.yaw) * 180 / Math.PI;
      if (turn > 0.5) found(w, { check: "rider-with-seat", level: "error", about: [id, r.seat], says: `the rider is turned ${turn.toFixed(1)} degrees from its seat`, measure: { name: "turn", value: turn, limit: 0.5, unit: "deg" } });
      if (r.inSeatMm > 0) found(w, { check: "rider-with-seat", level: "error", about: [id, r.seat], says: `the rider is ${r.inSeatMm} mm inside its seat`, measure: { name: "inside", value: r.inSeatMm, limit: 0, unit: "mm" } });
    }
  },
  "cut-on-word-end": (w) => {
    for (const id of w.find({ with: ["Clip", "Word"] })) {
      const c = w.component<{ out: number }>(id, "Clip")!, wd = w.component<{ end: number }>(id, "Word")!;
      if (Math.abs(c.out - wd.end) > 1 / 30) found(w, { check: "cut-on-word-end", level: "error", about: [id], says: `the cut is ${(c.out - wd.end).toFixed(2)} s off the word's end` });
    }
  },
  "cue-on-cause": (w) => {
    for (const id of w.find({ with: ["Cue", "CausedBy"] })) {
      const c = w.component<{ at: number }>(id, "Cue")!, by = w.component<{ at: number }>(id, "CausedBy")!;
      if (Math.abs(c.at - by.at) > 1 / 30) found(w, { check: "cue-on-cause", level: "error", about: [id], says: `the cue is ${(c.at - by.at).toFixed(2)} s from its cause` });
    }
  },
};
const run = (w: World) => { for (const c of Object.values(checks)) c(w); return findings(w); };

// ---- a note is an entity too, related to its asks' checks and to what it is about
function note(w: World, words: string, asks: { text: string; check: string; medium?: string }[]) {
  const n = w.add({ Note: { round: 1, words } });
  for (const a of asks) w.add({ Ask: a as never }, { parent: n });
  return n;
}
const noteState = (w: World, n: number) => w.children(n).map((a) => { const ask = w.component<{ text: string; check: string }>(a, "Ask")!; return { ask: ask.text, failing: findings(w).filter((f) => f.check === ask.check).length }; });

// ---- the cases
let bad = 0;
const expect = (name: string, got: number, want: (n: number) => boolean) => { const ok = want(got); if (!ok) bad++; console.log(`${ok ? "ok  " : "FAIL"} ${name}: ${got} finding(s)`); };
expect("as it should be", run(build()).length, (n) => n === 0);
expect("the work before the note (extruded type, unmasked, rider turned and in the chair's back)", run(build({ extruded: true, masked: false, riderTurn: 1.5, riderInBack: 40 })).length, (n) => n === 4);
expect("cheat: the type made 2D by a hair of depth left on a 3D layer", run(build({ extruded: true })).length, (n) => n >= 1);
expect("cheat: seat angle matched, rider still inside the back", run(build({ riderInBack: 40 })).length, (n) => n === 1);
expect("a cut 0.2 s past the word", run(build({ cutAt: 3.4 })).length, (n) => n === 1);
expect("a cue 3 frames late", run(build({ cueAt: 5.1 })).length, (n) => n === 1);
// one note holding three asks, on the work before it
const w = build({ extruded: true, masked: false, riderInBack: 40 });
const n = note(w, "this text is broken and again white 2d text with occlusion, and the chair is misaligned", [
  { text: "the line is 2D type", check: "type-2d-masked", medium: "2d" }, { text: "the subject masks it", check: "type-2d-masked" }, { text: "the rider sits in its seat", check: "rider-with-seat" },
]);
run(w);
console.log("note:", JSON.stringify(noteState(w, n)));
expect("the note's asks all fail before the work changes", noteState(w, n).filter((a) => a.failing > 0).length, (k) => k === 3);
// everything found about one entity, whoever found it
const rider = w.find({ with: ["Rides"] })[0];
console.log("about the rider:", findings(w).filter((f) => f.about.includes(rider)).map((f) => f.says));
console.log(bad === 0 ? "prototype: all cases as expected" : `prototype: ${bad} case(s) wrong`);
