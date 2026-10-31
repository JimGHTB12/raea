// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/asset (bevy_asset): files and generated assets.
export { staticFile, MEDIA_ROOT } from "@raea/video/media";
export * from "@raea/asset";
