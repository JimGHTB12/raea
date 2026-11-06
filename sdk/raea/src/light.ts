// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/light (bevy_light): Bevy's lights in Bevy's units (lux, lumens, cd/m²; bevy3d.tsx), environments and
// daylight. 3d.tsx's lights, in raea's relative units, stay under /3d.
export { AmbientLight, GlobalAmbientLight, DirectionalLight, PointLight, SpotLight, RectLight, type AmbientLightProps, type DirectionalLightProps, type PointLightProps, type SpotLightProps, type RectLightProps } from "@raea/render/bevy3d";
export { Atmosphere, Environment, Shadows, type AtmosphereProps, type EnvironmentProps, type ShadowsProps } from "./3d";
export * from "@raea/render/daylight";
// ray-traced lighting (Bevy's Solari, docs/raytracing.feature RT3)
export { SolariLighting, Pathtracer, type SolariLightingProps, type PathtracerProps } from "@raea/render/bevy3d";
