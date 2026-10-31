// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/animation (bevy_animation): declared animation and a model's clips.
export * from "@raea/math/animate";
export { clipEvents, type AnimGraphNode } from "./3d";
