// Every three.js class an R3F scene can hold that puts something on screen (every Object3D and Material in three's
// core), and what raea draws for it: the coverage of R3F as raea's input. test/coverage.test.ts walks three's exports
// and fails on one missing here, so a class added to three (or one the version raea is tested on gains) is decided
// here. Statuses as the three host's (../three/kinds.ts): drawn, near (note says what differs), setting, engine; and
//   none       nothing to draw (a bone, an audio source, a camera that isn't the one drawn with)
export type InputStatus = "drawn" | "near" | "none";

export const CLASSES: Record<string, { status: InputStatus; raea: string; note?: string }> = {
  // the scene graph
  Object3D: { status: "drawn", raea: "Group (with children), its transform composed as three does" },
  Group: { status: "drawn", raea: "Group" },
  Scene: { status: "drawn", raea: "the composition's 3D; background colour behind it, environment, fog" },
  LOD: { status: "drawn", raea: "the level three would draw from the camera this frame" },
  Bone: { status: "none", raea: "a skinned mesh's pose (baked into its vertices)" },
  // drawn things
  Mesh: { status: "drawn", raea: "Mesh + BufferMesh (its triangles, normals, uvs, vertex colours; a range per geometry group)" },
  SkinnedMesh: { status: "drawn", raea: "Mesh + BufferMesh posed on the CPU each frame (bones and morph targets)" },
  InstancedMesh: { status: "drawn", raea: "Instances while its copies hold still; a Mesh a copy when they move or have colours" },
  BatchedMesh: { status: "drawn", raea: "a Mesh per instance: its geometry's range, matrix and colour" },
  Points: { status: "drawn", raea: "one Mesh of quads facing the camera, three's point size (attenuated or in pixels), vertex colours" },
  Line: { status: "drawn", raea: "Gizmo strip in world space (vertex colours)", note: "lines are drawn 1 pixel wide in WebGL whatever linewidth says; raea draws linewidth" },
  LineSegments: { status: "drawn", raea: "Gizmo lines (helpers too: AxesHelper, GridHelper, BoxHelper ...)" },
  LineLoop: { status: "drawn", raea: "Gizmo strip, closed" },
  Sprite: { status: "drawn", raea: "a Mesh quad facing the camera, its centre and rotation" },
  ClippingGroup: { status: "near", raea: "Group", note: "its clipping planes are WebGPU-renderer only in three; raea draws the whole group" },
  // lights
  AmbientLight: { status: "drawn", raea: "AmbientLight (units.ts)" },
  DirectionalLight: { status: "drawn", raea: "DirectionalLight with its target, shadows when the canvas has them" },
  PointLight: { status: "drawn", raea: "PointLight, its reach where three's falls under half a level", note: "decay other than 2 falls off as the inverse square" },
  SpotLight: { status: "drawn", raea: "SpotLight with its target, angle and penumbra", note: "decay other than 2 falls off as the inverse square; a `map` (projected picture) isn't drawn" },
  RectAreaLight: { status: "drawn", raea: "RectAreaLight facing its -z, its size times its scale" },
  HemisphereLight: { status: "near", raea: "AmbientLight of its two colours' average", note: "raea has no sky-and-ground light" },
  LightProbe: { status: "near", raea: "AmbientLight of its average colour", note: "its directional bands are left out" },
  Light: { status: "none", raea: "the base class: its subclasses are drawn" },
  // cameras
  Camera: { status: "none", raea: "the base class" },
  PerspectiveCamera: { status: "drawn", raea: "PerspectiveCamera (zoom as field of view, view offset as `view`)" },
  OrthographicCamera: { status: "drawn", raea: "OrthographicCamera (height over zoom)" },
  ArrayCamera: { status: "drawn", raea: "PerspectiveCamera (the array's own view)" },
  CubeCamera: { status: "none", raea: "a render to a cube texture (raea makes its own reflections)" },
  // sound
  AudioListener: { status: "none", raea: "sound: raea/tone and <Sound3D>" },
  Audio: { status: "none", raea: "sound: raea/tone" },
  PositionalAudio: { status: "none", raea: "sound: <Sound3D>" },
  // materials
  MeshStandardMaterial: { status: "drawn", raea: "MeshStandardMaterial (maps, alpha test and blending, sides, uv placement, samplers)" },
  MeshPhysicalMaterial: { status: "drawn", raea: "MeshPhysicalMaterial (clearcoat, transmission, ior, thickness, attenuation, anisotropy, specular)", note: "sheen and iridescence are three's" },
  MeshBasicMaterial: { status: "drawn", raea: "MeshBasicMaterial" },
  MeshLambertMaterial: { status: "drawn", raea: "MeshStandardMaterial, roughness 1, metalness 0" },
  MeshPhongMaterial: { status: "near", raea: "MeshStandardMaterial with its shininess as a roughness", note: "its specular colour is left out" },
  MeshToonMaterial: { status: "near", raea: "MeshToonMaterial, a band per texel of its gradient map" },
  MeshNormalMaterial: { status: "near", raea: "MeshBasicMaterial coloured by its normals per vertex", note: "three colours per pixel" },
  MeshMatcapMaterial: { status: "near", raea: "MeshBasicMaterial in its colour", note: "the matcap's lighting is left out" },
  MeshDepthMaterial: { status: "near", raea: "MeshBasicMaterial grey", note: "three's depth pass, not a surface" },
  MeshDistanceMaterial: { status: "near", raea: "MeshBasicMaterial grey", note: "three's shadow pass, not a surface" },
  ShadowMaterial: { status: "drawn", raea: "Mesh shadow=\"catch\": only the shadows falling on it, in its colour and opacity" },
  SpriteMaterial: { status: "drawn", raea: "MeshBasicMaterial on the sprite's quad" },
  PointsMaterial: { status: "drawn", raea: "MeshBasicMaterial on the points' quads" },
  LineBasicMaterial: { status: "drawn", raea: "the Gizmo's colours" },
  LineDashedMaterial: { status: "near", raea: "the Gizmo's colours", note: "drawn solid" },
  ShaderMaterial: { status: "near", raea: "MeshBasicMaterial in its colour uniform", note: "GLSL doesn't run in Bevy: port it to <SurfaceShader> (WESL)" },
  RawShaderMaterial: { status: "near", raea: "MeshBasicMaterial in its colour uniform", note: "GLSL doesn't run in Bevy: port it to <SurfaceShader> (WESL)" },
  Material: { status: "none", raea: "the base class" },
  // helpers: lines (and a plane's mesh), drawn as any lines are
  ...Object.fromEntries(["ArrowHelper", "AxesHelper", "Box3Helper", "BoxHelper", "CameraHelper", "DirectionalLightHelper", "GridHelper", "HemisphereLightHelper", "PlaneHelper", "PointLightHelper", "PolarGridHelper", "SkeletonHelper", "SpotLightHelper"].map((k) => [k, { status: "drawn" as const, raea: "its lines as Gizmo lines (an arrow's head and a plane's face as meshes)" }])),
};
