// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/input (bevy_input, bevy_input_focus): the frame's input.
export { useInput, keyEdges, wireKeys } from "@raea/app/games";
export type { FrameInput, InputTimeline } from "@raea/app/clock";
// InputRegion (input-region.tsx) joins when it is committed
