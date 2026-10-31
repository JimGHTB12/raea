// @raea/character/ragdoll: a Character going limp. From a motion (a pose function of frame) up to frame `from`, the body
// falls under gravity onto the ground (y = 0) and any boxes given, carrying the motion's momentum, and comes to rest.
//
//   const fall = ragdoll(kid, (f) => live.pose(f), 188, { push: [0.8, 0, 0], solids: course.solids });
//   <Character body="kid" pose={fall(frame)} />
//
// The body is the 14 parts of the heros-journey game's ragdoll (apps/site/games/heros-journey/src/ragdoll.ts): pelvis,
// abdomen, chest, head, upper and lower arms, thighs, shins and feet, each rigid, joined at CMU's joints with the same
// anatomical limits, and solved by the same XPBD step (physics.ts there, ported here). Each part leads with one of the
// capture's joints (pelvis Hips, abdomen LowerBack, chest Spine1, head Neck, upper arm LeftArm, lower arm LeftForeArm,
// thigh LeftUpLeg, shin LeftLeg, foot LeftFoot, and the right side's); the joints between (the hip joints, Spine, the
// shoulders, Neck1 and Head, the hand and fingers, the toes) keep the turns they had at `from`, riding their part.
// A part's frame is its lead joint's: the body's frame (y up, facing +z, x its left) turned as the joint is, so a part
// at rest is unturned and the pose back out is each lead's turn from its parent's.
//
// What touches the ground and the boxes is the skin: samples of the body's surface (skeleton.json's `surface`), a few
// dozen a part, so a back or a shoulder lies on the floor where the skin does. The limbs keep out of each other and of
// the trunk by capsules fitted to the same samples.
//
// In the air (`terminal`, `buffet`): each part is dragged back as the square of its speed by its cross-section, so a
// body falling free comes up to its terminal speed smoothly and no faster, the light, wide parts trailing, and seeded
// gusts across the fall push each part its own way, turning it over (a skydiver's tumble, not a vacuum's).
//
// Frame-pure: the simulation is stepped from `from` with fixed steps and memoised, so frame f is the same whichever
// frame a render worker starts at. It stops: once every part has been still for a while the body sleeps, and every
// later frame is that pose.
export { ragdoll } from "./ragdoll/all";
export type { RagdollOptions, Fall } from "./ragdoll/all";


