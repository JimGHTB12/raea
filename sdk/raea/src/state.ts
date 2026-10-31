// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/state (bevy_state): states and their transitions, stepped by frame.
export * from "@raea/app/machine";
export { useGameState, useGameEvents, stepAll, type Playtest } from "@raea/app/games";
