// The motion check's fixed suite: turbo's library (ACCAD Male2's locomotion, CMU's pick-up, throw and sit) and the
// performances it is judged on: walk, jog and run on a curve, a 90 and a 180 degree turn, a start and a stop, a run
// turning, and errands (a pick, a throw, a sit, a cup carried and put down).
import { readFileSync, readdirSync } from "node:fs";
import type { Skeleton } from "raea/character";
import { readClips, type Clip, type ClipFile } from "raea/clip";
import { levelClip, motionLibrary, seatApproach, type MotionLibrary, type Seat } from "raea/matching";
import { planItems } from "raea/items";
import type { Vec3 } from "raea/3d";

const LIB = process.env.RAEA_MOTION_LIB ?? `${process.env.HOME}/.cache/raea-mocap/lib/turbo`;
const json = (p: string) => JSON.parse(readFileSync(p, "utf8")) as ClipFile;

/** ACCAD Male2's forward locomotion: walks, turns, starts and stops, runs and a stand */
export function locoClips(): Map<string, Clip> {
  const dir = `${LIB}/accad`;
  const files = readdirSync(dir).filter((f) => /^Male2_([BC]\d+|A1)_/.test(f) && !/back|box|hop|leap|crouch|skip|bow|ready|jump/i.test(f));
  return new Map(files.flatMap((f) => [...readClips(json(`${dir}/${f}`))]));
}

export const PICK = { name: "c64_27", from: 39, to: 105 }, THROW = { name: "c111_33", from: 176, to: 218 };

export function suiteLibrary(skel: Skeleton): MotionLibrary {
  // (each clip on a level floor, as the product's database stands them: clips/library.ts levels every clip it loads)
  const lv = (m: [string, Clip][]) => m.map(([n, c]) => [n, levelClip(skel, c)] as [string, Clip]);
  const loco = new Map(lv([...locoClips()]));
  const acts = lv(["64_27", "111_33", "113_15"].flatMap((id) => [...readClips(json(`${LIB}/cmu/cmu/${id}.json`))]));
  return motionLibrary(skel, new Map([...loco, ...acts]), {
    locomotion: [...loco.keys()], actions: [PICK.name, THROW.name],
    sit: { clip: "c113_15", down: [42, 80], sat: [80, 140], up: [140, 178] },
  });
}

export interface Case { name: string; spec?: any; plan?: ReturnType<typeof planItems>; seat?: Seat }

/** turbo's hip height (its thigh joints over the floor), what the suite's speeds were set for */
const TURBO_HIP = 0.975;

/** The cases. With `skel`, the locomotion cases' speeds are asked at the same Froude number as turbo's, v scaled by
 * sqrt of the body's hip height over turbo's (RAEA_SUITE_ABS=1 for the same metres a second for every body): 3.6 m/s
 * is turbo's run and mai's sprint (Froude 1.07, where capture runs at 0.87). Errands keep their speeds. */
export function suiteCases(lib: MotionLibrary, skel?: Skeleton): Case[] {
  const at: Vec3 = [0, 0, 0], curve: [number, number][] = [[0, 3], [2, 7], [6, 9]];
  const th = skel?.joints.find((j) => j.name === "thigh_l"), fk = th && process.env.RAEA_SUITE_ABS !== "1" ? Math.sqrt(th.head[1] / TURBO_HIP) : 1;
  const walk = (name: string, steps: any[]): Case => ({ name, spec: { at, facing: 0, steps: steps.map((st) => (st.speed ? { ...st, speed: st.speed * fk } : st)) } });
  const cases: Case[] = [
    walk("walk curve", [{ walk: curve, speed: 1.3 }, { wait: 0.5 }]),
    walk("jog curve", [{ walk: curve, speed: 2.6 }, { wait: 0.5 }]),
    walk("run curve", [{ walk: curve, speed: 3.6 }, { wait: 0.5 }]),
    walk("turn 90", [{ walk: [[0, 3], [3, 3]], speed: 1.3 }, { wait: 0.5 }]),
    walk("turn 180", [{ walk: [[0, 3], [0, 0.5]], speed: 1.3 }, { wait: 0.5 }]),
    walk("run turn 90", [{ walk: [[0, 4], [4, 4]], speed: 3.6 }, { wait: 0.5 }]),
    walk("start and stop", [{ wait: 1 }, { walk: [[0, 3]], speed: 1.3 }, { wait: 1.5 }]),
  ];
  // errands, as ~/Studio/internal/projects/psx/raea/errands.tsx runs them
  const things = {
    bin: { at: [8.6, 0, 5.4] as Vec3, size: [0.45, 0.4, 0.45] as Vec3, surfaces: [{ y: 0.03, min: [-0.2, -0.2] as [number, number], max: [0.2, 0.2] as [number, number] }] },
    side: { at: [7.6, 0, 0.6] as Vec3, size: [0.5, 0.74, 0.5] as Vec3 },
    desk: { at: [0.2, 0, -3.4] as Vec3, size: [1.2, 0.74, 0.6] as Vec3 },
  };
  const items = {
    ball: { at: [2.4, 0, 5.2] as Vec3, size: [0.11, 0.11, 0.11] as Vec3, shape: "ball" as const, mass: 0.15 },
    cup: { at: [7.6, 0.74, 0.6] as Vec3, size: [0.08, 0.1, 0.08] as Vec3, mass: 0.3, upright: true },
  };
  const SEAT: Seat = { at: [5.0, 0.46, 2.2], facing: Math.PI }, seat = seatApproach(lib, SEAT);
  const plan = planItems(lib, { at: [0, 0, 0], facing: 0 }, items, [
    { walk: [[0, 2.5], [1.2, 4]], speed: 0.8 },
    { pick: "ball", clip: PICK },
    { walk: [[3.8, 7], [6.2, 7.2]], speed: 2.6 },
    { throw: "ball", to: "bin", clip: THROW },
    { walk: [[6.2, 5.2], [seat.at[0], seat.at[1]]], speed: 1.3 },
    { step: { sit: SEAT, for: 3.5 }, at: seat.at, face: seat.face },
    { pick: "cup" },
    { walk: [[8.2, -1.2], [5.5, -2.8], [2.2, -2.6]], speed: 1.9 },
    { place: "cup", on: "desk", spot: [0.3, 0] },
    { walk: [[-1.6, -1.5], [-1.8, 2.5], [0.5, 4.5]], speed: 3.6 },
    { wait: 1.5 },
  ] as any, { things } as any);
  cases.push({ name: "errands", plan, seat: SEAT });
  return cases;
}
