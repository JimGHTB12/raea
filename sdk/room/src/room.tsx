// @raea/room: rooms laid out by relation. A room is a box of walls, each wall a frame (where it starts, which
// way it runs, which way is in); a run lays units (cupboards, appliances, wall cabinets) along a wall one after
// another, each set back against the wall and against the one before from its own measured box (scene.bounds, turned
// so its front faces into the room), never by typed coordinates; openings cut windows and doors into walls. The plan
// (rooms, runs, islands, the work points) is checked against a kitchen's clearances: `checkLayout`.
//
//   const room = defineRoom({ width: 5.4, depth: 4.8, height: 2.7 });
//   const back = layRun(room, "back", 0, [{ src: PANTRY }, { src: CUPBOARD_120 }, { src: SINK, role: "sink" }]);
//   <Room room={room} openings={[{ wall: "back", at: 3.6, width: 1.0, bottom: 1.15, top: 2.2, kind: "window" }]} />
//   <Run run={back} />
//   <Place src={MUG} on={COUNTER} at={onRun(back, 2.0, 0.3)} />
export { defineRoom, onWall, onSurface, layRun, onRun, unitCentre, Run, backsplash, Backsplash, bay, Room, wallFacing, overUnits, HANGING, hang, Hang, checkDecor, seeded, plant, swayAt, Outside, runFootprint, modelFootprint, seatFacing, faceToward, seatFootprint, behind, gap, CLEARANCE, checkLayout } from "./room/all";
export type { WallName, RoomSpec, WallFrame, Facing, Unit, Laid, LaidRun, Splash, SplashCap, Opening, BayOptions, Over, Hung, Planting, Plant, Footprint, Finding, Plan } from "./room/all";


