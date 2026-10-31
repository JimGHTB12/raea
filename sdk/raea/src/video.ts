// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/video (raea's own; Bevy has no video crate): time-shifted sequences and footage.
export { Sequence, Series, Loop, Freeze, type SequenceProps, type SequenceLayout, type SeriesSequenceProps, type LoopProps, type FreezeProps } from "@raea/video/timeline";
export { Video, type VideoProps, type MediaProps } from "@raea/video/media";
