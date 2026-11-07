// The three.js-shaped forms raea/3d had before Bevy's names (docs/control.feature FC1e): <Mesh position rotation
// scale>, the Mesh*Material set, Group, PerspectiveCamera, OrthographicCamera and the lights with a relative
// `intensity`. Not public: sdk/r3f's <R3F> maps three.js's JSX onto them, bevy3d's Bevy-named components draw through
// them, and the conformance tests compare against them. Compositions use Camera3d, Mesh3d, StandardMaterial, Entity
// and raea/light's lights.
export { Mesh, MeshStandardMaterial, MeshPhysicalMaterial, MeshBasicMaterial, MeshToonMaterial, AmbientLight, DirectionalLight, PointLight, SpotLight, RectAreaLight, PerspectiveCamera, OrthographicCamera, Group } from "./3d/all";
export type { MeshProps, MeshStandardMaterialProps, MeshPhysicalMaterialProps, MeshBasicMaterialProps, MeshToonMaterialProps, AmbientLightProps, DirectionalLightProps, PointLightProps, SpotLightProps, RectAreaLightProps, PerspectiveCameraProps, OrthographicCameraProps, GroupProps } from "./3d/all";
