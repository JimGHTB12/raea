// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/camera (bevy_camera): cameras and the views they make.
export { Canvas3D, PerspectiveCamera, OrthographicCamera, type Canvas3DProps, type PerspectiveCameraProps, type OrthographicCameraProps } from "./3d";
export { Matchmove, useMatchmove, matchmoveView, matchmoveIndex, checkMatchmove, type MatchmoveProps, type MatchmoveSolve, type MatchmoveState, type MatchmoveCamera } from "@raea/render/matchmove";
export { Camera3d, type Camera3dProps, type Projection } from "@raea/render/bevy3d";
// a Bevy texture (a camera's target) drawn in an element (B3 H7)
export { TextureView } from "@raea/render/bevy3d";
