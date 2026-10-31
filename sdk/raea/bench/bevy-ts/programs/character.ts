// gap: S6e (docs/sdk.feature)
// /character's motion engine as an App system, on the engine: a headless App opened with CharacterPlugin, the stick
// body's walk registered as a motion library, and an entity spawned with a goal (walk 2.5 m along +z, then wait),
// starting at frame 2. Bevy gives it a CharacterPose with the goal (a required component); the plugin's Update system
// writes it each frame. After 40 frames the pose is the performance's frame 38, and its place is perform's own; past
// the end it holds the last frame, near where it was sent. Prints one JSON line; a wrong answer prints FAIL and throws.
//   sdk/raea/bench/bevy-plugins/motion.sh run sdk/raea/bench/bevy-ts/programs/character.ts
import { BevyApp } from "raea/bevy";
import { print } from "raea/native";
import { readBvh, readClips, retarget, writeClips } from "raea/clip";
import { motionLibrary, perform } from "raea/matching";
import { CharacterGoal, CharacterPlugin, CharacterPose, goal, registerMotion } from "@raea/character/plugin";
import { skel, stickBvh } from "../../../conformance/stick";

function check(ok: boolean, what: string) {
  if (!ok) {
    print(`FAIL character: ${what}`);
    throw new Error(`character: ${what}`);
  }
}

export default function main() {
  try {
    run();
  } catch (e) {
    print(`FAIL: ${e instanceof Error ? e.message : String(e)}`);
    throw e;
  }
}

function run() {
  const walk = retarget(skel, readBvh(stickBvh(1201)), { name: "walk" }), stand = retarget(skel, readBvh(stickBvh(361, 120, true)), { name: "stand" });
  const lib = motionLibrary(skel, readClips(JSON.parse(JSON.stringify(writeClips(skel, [walk, stand])))));
  registerMotion("stick", lib);
  const spec = { at: [0, 0, 0] as [number, number, number], facing: 0, steps: [{ walk: [[0, 0], [0, 2.5]] as [number, number][] }, { wait: 0.5 }] };
  const ref = perform(lib, spec);

  const app = BevyApp.open({ plugins: [CharacterPlugin] });
  const e = app.spawn(goal("stick", spec, 2));
  // the goal brought its pose (Rust's #[require]) before any system ran
  check(app.get(e, CharacterPose) !== undefined, "a goal without its CharacterPose: is the build's raea-runtime-plugins older than S6e?");
  check(app.query({ data: [CharacterGoal] }).length === 1, "the goal isn't queryable by its path");

  app.step(40);
  const mid = app.get(e, CharacterPose)!;
  check(mid.frame === 38, `frame 40's pose is the performance's ${mid.frame}, want 38`);
  const want = ref.pose(38).root;
  check(Math.abs(mid.translation[0] - want[0]) < 1e-5 && Math.abs(mid.translation[2] - want[2]) < 1e-5, `frame 40's place ${mid.translation} isn't perform's ${want}`);

  app.step(ref.frames + 10);
  const end = app.get(e, CharacterPose)!;
  check(end.frame === ref.frames - 1, `past the end the pose is frame ${end.frame}, want the last, ${ref.frames - 1}`);
  check(Math.hypot(end.translation[0], end.translation[2] - 2.5) < 0.15, `it ended at ${end.translation}, not near [0, 0, 2.5]`);

  print(JSON.stringify({ character: "PASS", frames: ref.frames, mid: mid.translation, end: end.translation }));
}
