// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/audio (bevy_audio; raea plays it with seedling on its own mixer): sound.
export { Audio, type AudioProps, type VolumeProp } from "@raea/video/media";
export * from "@raea/audio/mix";
export * from "@raea/audio/song";
export * from "@raea/audio/seedling";
export * from "@raea/audio/speech";
export * as Tone from "@raea/audio/tone";
export * as music from "@raea/audio/music";
