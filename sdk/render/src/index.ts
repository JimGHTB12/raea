// @raea/render: raea's 3D drawing, as bevy_render, bevy_pbr, bevy_light and bevy_camera are Bevy's (docs/sdk.feature S1d).
export * from "./3d";
export * from "./daylight";
export * from "./bevy3d";
export * from "./paint";
export * from "./matchmove";
export * from "./scene";
export * from "./solid";
export * from "./grow";
export * from "./place";
export * from "./pick";
export * from "./splat";
// scene and solid each declare the same Bounds; /3d has always given solid's
export type { Bounds } from "./solid";
// /3d and Bevy's both name the lights: @raea/render's are Bevy's, in Bevy's units (lux, lumens, cd/m²), as the prelude's
export { AmbientLight, DirectionalLight, PointLight, SpotLight, type AmbientLightProps, type DirectionalLightProps, type PointLightProps, type SpotLightProps } from "./bevy3d";
// 3d and solid each declare the same Vec3 tuple
export type { Vec3 } from "./3d";
