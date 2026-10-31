// Declared animation: moves, keyframe tracks and timelines, declared once and sampled by the engine each frame
// (raea-animate, a frame-pure fork of Animato, in Rust: animate/). An <Animate> sends its tracks once, as data, when it
// mounts; raea-runtime-react compiles them and writes each frame's opacity, translate, scale, rotate, clip, colours, sizes or
// counter text onto the element itself, from the composition's frame. Nothing here runs per frame but the check of
// whether the element is in the tree. Under plain bun (no engine) the same tracks are sampled here instead, so tests
// and renderFrame still see the animated style.
//
//   <Animate enter="rise" exit="fade" at={word("crack")} exitAt={90}>...</Animate>
//   <Animate enter={pop({ peak: 1.12 })} animate={{ backgroundColor: track({ 0: "#fff", 12: "#ffd400" }) }}>
//   const plan = timeline(sequence(["card", "rise"], parallel(["title", "pop"], ["cta", slide({ from: "right" })])));
//   <Animate timeline={plan} part="title">
//   <Stagger each={3} enter="rise">{chips}</Stagger>
export { curve, track, move, delay, reverse, rise, pop, slam, slide, wipe, fill, tick, strike, pushIn, fade, moves, toMove, sequence, sequenceGap, parallel, stagger, gridDelay, timeline, fingerprint, propsFingerprint, curveFn, sampleAnimation, Animate, Stagger, useAnimation3d, sampleMove } from "./animate/all";
export type { CurveName, Curve, Keys, TrackSpec, Move, MoveName, MoveLike, Step, Group, GridFrom, Plan, AnimateOwnProps, AnimateProps, StaggerProps } from "./animate/all";

export type { AnimKey, AnimTrack, Animation } from "./animate.gen";
