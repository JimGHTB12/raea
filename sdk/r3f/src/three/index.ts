// raea's 3D in three.js (docs/r3f.feature): Scene3d draws the parts a raea page renders (its `data-3d` elements) into
// a three.js scene, for any three.js app, React Three Fiber's <Raea> and sdk/playable's host; kinds.ts is its coverage.
export { Scene3d, type Settings } from "./scene";
export { KINDS, PRIMITIVES, type Status } from "./kinds";
export { loadShapes, shapesReady } from "./geometry";
