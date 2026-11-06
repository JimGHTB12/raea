// Motion matching for raea/character: a character walks where the scene says (paths round the furniture,
// stops facing things, waits), and each moment of its motion is the moment of the motion capture library that best
// continues what it is doing towards where it is going. Frame-pure: the whole performance is simulated once from
// frame 0 for its inputs (a deterministic function of them) and each frame reads its pose, so a render and a live
// play agree.
//
//   const lib = motionLibrary(kid, readClips(clips), { locomotion: ["c16_15", ...] });
//   const plan = perform(lib, { at: [0.5, 0, 4], facing: Math.PI, steps: [
//     { walk: route(room, [0.5, 4], [2.6, 1.2], obstacles), face: FRIDGE }, { wait: 2 } ] });
//   <Character skeleton={kid} src={SRC} pose={plan.pose(frame)} />
//
// The library: every frame of the locomotion clips (and their mirror images, so every turn goes both ways), each with
// features (Clavet's): the feet's places and speeds and the hips' speed now, and where the body will be and which way
// it will face a third, two thirds and a whole second on, all in the body's own ground frame. A query asks for the
// pose now and the route's next second; the nearest frame plays on from there. A switch is blended out over a fifth of
// a second (inertialization), the body is pulled onto the route so it arrives where it was sent, and the feet are
// pinned to the floor while they stand on it.
export { FEATURE_COUNT, STATE, STATE_COUNT, mirror, libraryPose, libraryRows, motionLibrary, libraryTable, stateMix, stateQuery, phaseCost, crouched, swingClear, swingClearance, swingCost, stepTags, nearest, nearestZ, clipFloor, levelClip, fitAction, planFloor, through, before, walkPlan, route, seatApproach, perform } from "./matching/all";
export type { MotionLibrary, SearchQuery, SearchBackend, TagCond, TagSpec, LibraryOptions, LibraryTable, MotionState, PhaseGoal, ObstacleKind, ActionFit, Obstacle, Mark, Step, Limbs, Seat, PerformanceSpec, PerformOptions, Performance } from "./matching/all";


export { MotionDebug, type MotionDebugProps } from "./matching/debug";
