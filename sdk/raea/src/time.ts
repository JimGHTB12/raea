// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/time (bevy_time): the frame and the composition's time. useCurrentFrame is Time<Virtual> at the composition's fps.
export { Clock, useCurrentFrame, useVideoConfig, useLocalTime, type ClockProps, type TimelineWord, type VideoConfig, type FrameInput, type InputTimeline, type Perception, type LocalTime } from "@raea/app/clock";
