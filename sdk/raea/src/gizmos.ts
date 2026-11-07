// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/gizmos (bevy_gizmos): lines drawn for looking, and debug overlays.
export { Gizmo, type GizmoProps, type GizmoColor } from "./3d";
export * from "@raea/video/overlay";
