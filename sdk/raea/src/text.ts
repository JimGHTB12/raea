// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/text (bevy_text): text in the world, and time anchored to the words said.
export { Text3D, RichText, Span, type Text3DProps, type RichTextProps, type SpanProps, type GlyphInfo, type GlyphStyle } from "./3d";
export * from "@raea/video/words";
