// The frames the attract match's player sets off from (its shoe on the floor), for the sounds.
import { INPUT } from "./input";

/** the frames the player sets off: a shoe on the floor, spaced so they read as steps */
export function stepsOf(timeline: Record<number, string[]>): number[] {
  const out: number[] = [];
  const frames = Object.keys(timeline)
    .map(Number)
    .sort((a, b) => a - b);
  let prev = "";
  let prevF = -99;
  let last = -99;
  for (const f of frames) {
    const dir = timeline[f][0] || "";
    if (dir && (dir !== prev || f - prevF > 6) && f - last >= 16) {
      out.push(f);
      last = f;
    }
    prev = dir;
    prevF = f;
  }
  return out;
}
export const STEPS = stepsOf(INPUT);
