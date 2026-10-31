// @raea/render/3d: 3D in the same tree (spec: "3D in the same tree"). Every component renders as an element that
// carries its name (data-3d) and props (data-props, JSON) for Bevy to draw; a Mesh also carries its Manifold
// expression (data-shape). Engine-safe: this module imports nothing beyond React.
import { toHex } from "./colour";
export { Canvas3D, Cuboid, Sphere, Plane3d, Cylinder, Torus, Cone, Capsule, Capsule3d, BufferMesh, Text3D, Mesh, MeshStandardMaterial, MeshPhysicalMaterial, MeshBasicMaterial, MeshToonMaterial, SurfaceShader, surfaceShader, samplerSpec, FramePass, Compute, AmbientLight, DirectionalLight, PointLight, SpotLight, RectAreaLight, Atmosphere, PerspectiveCamera, OrthographicCamera, eulerOf, Entity, Refinish, Group, clipEvents, Model, ImagePlane, surfaceTurn, Decal, Plate, Bloom, Fog, ToneMapping, Grade, Environment, AmbientOcclusion, AntiAliasing, Shadows, VolumetricFog, FogVolume, Fire, dofOf, DepthOfField, Outline, Fisheye, LensDistortion, MotionBlur, checkPass, Look, Scene, Entities, BevyComponent, Gizmo, ShaderMesh, BevyResource, Span, RichText } from "./3d/all";
export type { Vec3, Canvas3DProps, CuboidProps, SphereProps, CylinderProps, TorusProps, Plane3dProps, ConeProps, CapsuleProps, Capsule3dProps, BufferMeshProps, Text3DProps, MeshProps, Animate3d, SurfaceDrawProps, MeshStandardMaterialProps, MeshPhysicalMaterialProps, MeshBasicMaterialProps, ToonBand, ToonRim, ToonHighlight, ToonOutline, MeshToonMaterialProps, SurfaceShaderSpec, SamplerSpec, SamplerWrap, FramePassProps, ComputeProps, AmbientLightProps, DirectionalLightProps, PointLightProps, SpotLightProps, RectAreaLightProps, AtmosphereProps, PerspectiveCameraProps, BevyParts, OrthographicCameraProps, EntityProps, GroupProps, ModelProps, AnimGraphNode, ImagePlaneProps, DecalProps, PlateProps, BloomProps, FogProps, ToneMappingProps, GradeSection, GradeProps, EnvironmentProps, AmbientOcclusionProps, AntiAliasingProps, ShadowsProps, VolumetricFogProps, FogVolumeProps, FireProps, DepthOfFieldProps, OutlineProps, LensCommon, FisheyeProps, LensDistortionProps, MotionBlurProps, LookProps, LookPass, SceneProps, BevyComponentProps, GizmoProps, ShaderMeshProps, SpanProps, GlyphInfo, GlyphStyle, RichTextProps } from "./3d/all";

export { CrossSection, crossSection, manifold, outline, Solid, type Bounds, type Polygons, type Section } from "./solid";

export { Volume, type VolumeProps, type VolumeSource, type VolumeCommon } from "./paint";

// footage with 3D standing in its world: the camera solved, the plate behind, the subject in front (matchmove.tsx)
// gaussian splats: a reconstructed scene or object, placed and animated by props (splat.tsx)
export { Splat, type SplatProps, type SplatWave, type SplatMorph } from "./splat";

// physics: rigid bodies baked by Rapier, glitches on parts, goo baked by MLS-MPM (physics.tsx)
// a live picture on a surface: a nested composition or game drawn as a CRT (screen.tsx)
export { Screen, type ScreenProps, type ScreenSource, type CrtSettings, type ModelScreens } from "./screen";

export { pick, pointerRay, type Hit, type PickCamera, type Pickable } from "./pick";

export { Matchmove, useMatchmove, matchmoveView, matchmoveIndex, checkMatchmove, type MatchmoveProps, type MatchmoveSolve, type MatchmoveState, type MatchmoveCamera } from "./matchmove";

export { toHex };
// 2D over the 3D (docs/sdk.feature S3c)
export { Sprite, Shape2D } from "./3d/all";
export type { Place2dProps, SpriteProps, Shape2DProps } from "./3d/all";
