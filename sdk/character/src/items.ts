// Items for raea/character: a body walks to a thing, reaches for it, takes it, carries it, and puts it down,
// drops it or throws it. The walking is motion-matched (perform); the hands, the lean, the squat, the trunk, the gaze,
// the grip and the thing itself are laid over the performance procedurally, so the hand lands on the thing wherever
// it is, and the thing goes where the hand goes until it is let go, then falls or flies and lands.
//
//   const items = { cup: { at: placeOn(table), size: [0.08, 0.1, 0.08], shape: "round", mass: 0.3 } };
//   const plan = planItems(lib, { at: [0, 0, 0], facing: 0 }, items, [{ pick: "cup" }, { throw: "cup", to: "bin" }], { things });
//   const perf = perform(lib, plan.spec, { effects });
//   const live = interact(skel, perf, plan);
//   live.pose(frame)            the body, hands on the cup
//   live.item("cup", frame)     where the cup is: its bottom's middle, and its turn
//
// Every item is a Thing (things.ts), and how it is taken follows from its shape, size and weight (handling): a pinch
// from above for a small thing, a hand round a cup or by its handle (when the handle faces the hand), a ball in the
// palm, two hands on a box's sides, a heavy or big thing hugged against the belly, a pole from above. The heavier it
// is to the body, the slower every moment of the reach, lift and put-down, the deeper the squat (the back staying
// straighter), the lower and closer the hold, the more the back leans back to carry it and the slower the walk.
//
// A pick: the body walks to where it can reach the thing (standAt, at whatever it stands on) and stands while it
// looks, reaches, takes it and lifts it in to the hold. A thing too heavy to lift is strained at and let be. A place:
// it walks to reach where the thing goes and lowers it there. A drop: the hands open and the thing falls onto whatever
// is under it (a light ball bounces and rolls; into a bin, it stays). A throw: the body turns to the target and winds up, the trunk twisting
// and the knees dipping by the style (one hand overhand or underhand, two hands from the chest, a two-hand toss from
// low, a full-body heave for a heavy thing), lets go at the speed that lands the thing on the target (as hard as the
// body can throw a thing that weight, which may fall short) and follows through.
export { planItems, interact, thingSdf, graspThing } from "./items/all";
export type { Item, ThrowStyle, ItemTask, ItemClip, PickManner, ItemPlan, Interaction } from "./items/all";


