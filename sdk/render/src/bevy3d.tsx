// Bevy's own names for 3D (docs/sdk.feature S2a): Transform, Quat, Mesh3d, Camera3d with Projection, and
// StandardMaterial, and the lights (PointLight, SpotLight, DirectionalLight, AmbientLight, RectLight), each taking its Bevy fields as Bevy reflects them (snake_case, radians, quaternions, Bevy's
// defaults, Bevy's units) and drawn by the same components as 3d.tsx's three.js-shaped names, so the two vocabularies render alike.
//
//   <Camera3d transform={Transform.fromXyz(0, 2, 7).lookingAt([0, 0.5, 0])} />
//   <Mesh3d transform={Transform.fromXyz(0, 0.5, 0)}>
//     <Cuboid half_size={[0.5, 0.5, 0.5]} />
//     <StandardMaterial base_color="#c84" perceptual_roughness={0.4} />
//   </Mesh3d>
import { eulerOf } from "./3d";
export { Quat, Affine2, Transform, Mesh3d, Camera3d, Srgba, LinearRgba, StandardMaterial, GridExtension, PaintExtension, WearExtension, ToonExtension, ShaderExtension, ExtendedMaterial, ToonMaterial, Mesh2d, Circle, Rectangle, Ellipse, RegularPolygon, ColorMaterial, PointLight, SpotLight, DirectionalLight, AmbientLight, GlobalAmbientLight, RectLight } from "./bevy3d/all";
export type { EulerRot, TransformValue, TransformBuilt, Mesh3dProps, Projection, Camera3dProps, AlphaMode, StandardMaterialProps, StandardMaterialAnimate, ToonExtensionAnimate, SampledImage, MetallicRoughnessMaps, GridExtensionProps, PaintExtensionProps, WearExtensionProps, ToonExtensionProps, ShaderExtensionProps, ExtendedMaterialProps, ToonMaterialProps, Mesh2dProps, CircleProps, RectangleProps, EllipseProps, RegularPolygonProps, ColorMaterialProps, PointLightProps, SpotLightProps, DirectionalLightProps, AmbientLightProps, RectLightProps } from "./bevy3d/all";

export { eulerOf };
// ray-traced lighting: Bevy's Solari (docs/raytracing.feature RT3)
export { SolariLighting, Pathtracer, type SolariLightingProps, type PathtracerProps } from "./bevy3d/raytracing";
export { ConicalFrustum, type ConicalFrustumProps } from "./3d/shapes";
// a Bevy texture (a camera's target) drawn in an element (B3 H7)
export { TextureView } from "./layers";
