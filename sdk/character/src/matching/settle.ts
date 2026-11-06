// settle: a performance's passes after it is played (perform.ts): the speed carried through switches, still stretches
// and turns smoothed, an action's feet held, the feet planted or locked, the elbows and hands relaxed, the arms cleared
// of the trunk, the gaze led and the head steadied; then its trace, its lint, and the cache.
import { relaxHands, softElbows, type Pose } from "raea/character";
import { plantFeet, type PlantLog } from "../feet";
import { lockFeet, lockLog } from "../lock";
import { lintReport, motionLint } from "../lint";
import { MotionLibrary, PerformOptions, Performance, PerformanceSpec, carrySpeed, clearArms, holdActionFeet, leadGaze, lintEnv, performances, smoothStill, smoothTurns, stages, steadyHead } from "./all";

/** what playing a performance gives: the poses as played, each frame's foot contacts, the switches, the clips played
 * (the body's and the upper body's own), and for a trace the clips' own frames and the rates */
export interface Played {
  raw: Pose[]; contacts: [number, number][]; switches: number[];
  played: Performance["played"]; playedU: ({ clip: string; f: number } | null)[];
  clipStage: Pose[]; rates: number[];
}

/** the performance from what was played, settled; cached under `key` */
export function settle(lib: MotionLibrary, spec: PerformanceSpec, o: PerformOptions, G: Performance["guide"], N: number, key: string, s: Played): Performance {
  const skel = lib.skel, fps = lib.fps, { raw, contacts, switches, played, playedU, clipStage, rates } = s;
  carrySpeed(raw, switches, fps);
  smoothStill(raw, G);
  if (o.smooth !== 0) smoothTurns(raw, o.smooth ?? 2);
  holdActionFeet(skel, raw, G, contacts);
  // relaxed elbows: the library's arms can swing locked straight
  const log: PlantLog | undefined = o.trace ? { state: [[], []], events: [], drop: [], lift: [[], []] } : undefined;
  const lockL = o.trace ? lockLog() : undefined;
  const feetBy = o.feet ?? (process.env.RAEA_FEET === "plant" ? "plant" : "lock");
  const planted = o.pin === false ? raw : feetBy === "lock" ? lockFeet(skel, raw, contacts, fps, { log: lockL }) : plantFeet(skel, raw, contacts, fps, log);
  const elbowed = planted.map((p) => relaxHands(skel, softElbows(skel, p)));
  // (arms cleared of the trunk again after play, matching/arms.ts; RAEA_CLEAR_ARMS=0 not)
  const relaxed = process.env.RAEA_CLEAR_ARMS !== "0" ? clearArms(skel, elbowed) : elbowed;
  const gazed = o.gaze === false ? relaxed : leadGaze(skel, relaxed, G);
  // (the head held steady (RAEA_STEADY_HEAD=0 not) in the world, matching/head.ts)
  const poses = process.env.RAEA_STEADY_HEAD !== "0" ? steadyHead(skel, gazed, fps) : gazed;
  const { phase, step } = stages(G, spec, fps);
  const perf: Performance = { frames: N, pose: (frame) => poses[Math.min(N - 1, Math.max(0, Math.round(frame)))], guide: G, played, phase, step };
  if (playedU.some(Boolean)) perf.upper = playedU;
  if (o.trace) perf.trace = { stages: { clip: clipStage, blend: raw, plant: planted, elbows: relaxed, gaze: poses }, switches, rate: rates, plant: log, lock: lockL };
  // RAEA_LINT=1: every performance lints itself and says what it found (its trace kept for that)
  if (o.trace && lintEnv() && !o.quiet) console.warn(`[motion lint] ${spec.steps.length} steps, ${N} frames\n` + lintReport(motionLint(lib, perf)));
  performances.set(key, perf);
  if (performances.size > 64) performances.delete(performances.keys().next().value!);
  return perf;
}
