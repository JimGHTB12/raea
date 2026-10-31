// @raea/character/plugin (docs/sdk.feature S6e): the motion engine as an App system. A character is an entity with a
// CharacterGoal (a registered motion library's name, a PerformanceSpec and the frame it starts at); each frame the
// one motion engine (raea/matching's perform) moves it and writes where it is into its CharacterPose, which Bevy
// gives it with the goal (a required component). The joints come from poseAt(goal, frame), not the World: perform
// is frame-pure, so a pose is a function of the goal and the frame, and a headless App, a render and a live play (and
// a render that seeks straight to a frame) agree.
//
//   registerMotion("kid", motionLibrary(kid, clips, { locomotion: [...] }));
//   const app = BevyApp.open({ plugins: [CharacterPlugin] });
//   const e = app.spawn(goal("kid", { at: [0, 0, 4], facing: Math.PI, steps: [{ walk: route(...) }, { wait: 2 }] }));
//   app.step(30);
//   app.get(e, CharacterPose);                      // { frame: 29, translation: [x, 0, z], yaw }
//
// In a page there's no BevyApp to build a plugin into: useCharacters() runs the same step as the page's Update system
// (hooks.ts useSystem). A page system runs on React's thread and its writes land at the next frame's PreUpdate, a
// frame behind a Rust system's; that is the same in a render and a play, so they still agree.
import { Update, definePlugin, with as withValue, type With } from "@raea/app/bevy";
import { entity, read, write } from "@raea/app/typed";
import { useSystem } from "@raea/app/hooks";
import { perform, type MotionLibrary, type Performance, type PerformanceSpec } from "./matching";
import type { Pose } from "./character";

// the components' handles and types, generated from the registry by character's declaration in package.json (S6b)
import { character, type CharacterGoal as Goal, type CharacterPose as Placed } from "../bevy/character.gen";
export { character };
export type { Goal, Placed };

/** A character's goal: the motion library's name (registerMotion), the performance as JSON, the frame it starts */
export const CharacterGoal = character.CharacterGoal;
/** Where the motion engine has a character on a frame: the performance's frame, its place on the floor, its yaw */
export const CharacterPose = character.CharacterPose;

const libraries = new Map<string, MotionLibrary>();

/** A motion library by name, for goals to name (a body's: registerMotion("kid", motionLibrary(kid, ...))) */
export function registerMotion(name: string, lib: MotionLibrary): void {
  libraries.set(name, lib);
}

/** A goal's component value: `spec` performed with the library named `motion`, from frame `start` */
export function goal(motion: string, spec: PerformanceSpec, start = 0): With {
  return withValue(CharacterGoal, { motion, spec: JSON.stringify(spec), start });
}

// each goal planned once (perform simulates the whole performance), by what it is
const plans = new Map<string, Performance>();

function plan(g: Goal): Performance {
  const key = `${g.motion}\u0000${g.spec}`;
  let p = plans.get(key);
  if (!p) {
    const lib = libraries.get(g.motion);
    if (!lib) throw new Error(`CharacterGoal: no motion library ${JSON.stringify(g.motion)} (registerMotion it; there are: ${[...libraries.keys()].join(", ") || "none"})`);
    p = perform(lib, JSON.parse(g.spec) as PerformanceSpec);
    plans.set(key, p);
  }
  return p;
}

/** The performance's frame for the App's `frame`: from the goal's start, held at the last once it's over */
const local = (p: Performance, g: Goal, frame: number) => Math.min(Math.max(frame - g.start, 0), p.frames - 1);

/** The joints at `frame` (the World holds only the place: a pose is a function of the goal and the frame) */
export function poseAt(g: Goal, frame: number): Pose {
  const p = plan(g);
  return p.pose(local(p, g, frame));
}

/** Where the goal has the character at `frame` */
export function placedAt(g: Goal, frame: number): Placed {
  const p = plan(g);
  const f = local(p, g, frame);
  const root = p.pose(f).root;
  return { frame: f, translation: [root[0], 0, root[2]], yaw: p.guide[f].yaw };
}

/** The motion engine as a plugin: every character with a goal moved each frame, in the App's Update */
export const CharacterPlugin = definePlugin({
  name: "character",
  build(app) {
    app.addSystems(Update, (a, frame) => {
      for (const row of a.query({ data: [CharacterGoal] })) {
        a.insert(row.entity, withValue(CharacterPose, placedAt(row.components[CharacterGoal.path] as Goal, frame)));
      }
    });
  },
});

/** The same step in a page: its characters moved by the page's Update system */
export function useCharacters(): void {
  useSystem("Update", [entity, read(CharacterGoal), write(CharacterPose)], (rows, frame) => {
    for (const [, g, pose] of rows) Object.assign(pose, placedAt(g as Goal, frame));
  });
}
