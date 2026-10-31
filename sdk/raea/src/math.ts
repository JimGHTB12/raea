// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/math (bevy_math, bevy_curve): numbers through time: easing, interpolation, springs, curves.
export * from "@raea/math/easing";
export * from "@raea/math/interpolate";
export * from "@raea/math/spring";
export { curve, curveFn, type Curve, type CurveName } from "@raea/math/animate";
export { quatAxis, quatMul, hash01 } from "@raea/physics/physics";
export { Affine2 } from "@raea/render/bevy3d";
