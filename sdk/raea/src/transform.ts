// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/transform (bevy_transform): where things are, and nesting.
export { Group, surfaceTurn, type GroupProps, type Vec3 } from "./3d";
export { Transform, Quat, eulerOf, type EulerRot, type TransformValue, type TransformBuilt } from "@raea/render/bevy3d";
