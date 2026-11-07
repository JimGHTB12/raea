// docs/sdk.feature S1b (M1): a Bevy-named entry point, re-exporting what today lives in the files named. Sources move
// into it at M5; until then the old subpaths and this one export the same values.
// raea/pbr (bevy_pbr, bevy_material): materials, fog, volumes and the paint look's things.
export { SurfaceShader, ThreeShaderMaterial, type ThreeShaderMaterialProps, Refinish, Fog, VolumetricFog, FogVolume, Fire, Decal, ShaderMesh, AmbientOcclusion, type SurfaceShaderSpec, type SurfaceDrawProps, type ToonBand, type ToonRim, type ToonHighlight, type ToonOutline, type FogProps, type VolumetricFogProps, type FogVolumeProps, type FireProps, type DecalProps, type ShaderMeshProps, type AmbientOcclusionProps } from "./3d";
export * from "@raea/render/splat";
export * from "@raea/render/paint";
export { StandardMaterial, ExtendedMaterial, ToonMaterial, type ToonMaterialProps, GridExtension, PaintExtension, WearExtension, ToonExtension, ShaderExtension, type StandardMaterialProps, type StandardMaterialAnimate, type ToonExtensionAnimate, type ExtendedMaterialProps, type GridExtensionProps, type PaintExtensionProps, type WearExtensionProps, type ToonExtensionProps, type ShaderExtensionProps, type AlphaMode, type SampledImage, type MetallicRoughnessMaps } from "@raea/render/bevy3d";
