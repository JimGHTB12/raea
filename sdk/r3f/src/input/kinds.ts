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
  ClippingGroup: { status: "drawn", raea: "Group, the meshes inside cut at its planes (clip.ts)" },
  // lights
  AmbientLight: { status: "drawn", raea: "AmbientLight (units.ts)" },
  DirectionalLight: { status: "drawn", raea: "DirectionalLight with its target, shadows when the canvas has them" },
  PointLight: { status: "drawn", raea: "PointLight, its reach where three's falls under half a level", note: "decay other than 2 falls off as the inverse square" },
  SpotLight: { status: "drawn", raea: "SpotLight with its target, angle and penumbra", note: "decay other than 2 falls off as the inverse square; a `map` (projected picture) isn't drawn" },
  RectAreaLight: { status: "drawn", raea: "RectAreaLight facing its -z, its size times its scale" },
  HemisphereLight: { status: "near", raea: "AmbientLight of its two colours' average", note: "raea has no sky-and-ground light" },
  LightProbe: { status: "near", raea: "AmbientLight of its constant band and a DirectionalLight along its linear band", note: "exact for surfaces facing that light or sideways; its quadratic band is left out" },
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
  MeshPhongMaterial: { status: "near", raea: "MeshPhysicalMaterial: its shininess as a roughness, its specular colour as F0 (reflectance and specular tint)", note: "GGX's highlight, not Blinn-Phong's; an F0 above 0.16 (Bevy's highest reflectance) is held there, so a bright specular colour glints less" },
  MeshToonMaterial: { status: "near", raea: "MeshToonMaterial, a band per texel of its gradient map" },
  MeshNormalMaterial: { status: "drawn", raea: "ThreeShaderMaterial of three's ShaderLib.normal" },
  MeshMatcapMaterial: { status: "drawn", raea: "ThreeShaderMaterial of three's ShaderLib.matcap (its matcap, map, colour)" },
  MeshDepthMaterial: { status: "near", raea: "MeshBasicMaterial grey", note: "three's depth pass, not a surface" },
  MeshDistanceMaterial: { status: "near", raea: "MeshBasicMaterial grey", note: "three's shadow pass, not a surface" },
  ShadowMaterial: { status: "drawn", raea: "Mesh shadow=\"catch\": only the shadows falling on it, in its colour and opacity" },
  SpriteMaterial: { status: "drawn", raea: "MeshBasicMaterial on the sprite's quad" },
  PointsMaterial: { status: "drawn", raea: "MeshBasicMaterial on the points' quads" },
  LineBasicMaterial: { status: "drawn", raea: "the Gizmo's colours" },
  LineDashedMaterial: { status: "drawn", raea: "the Gizmo's colours, the line cut into its dashes by its line distances" },
  ShaderMaterial: { status: "drawn", raea: "ThreeShaderMaterial: its GLSL translated by naga and run with three's built-ins", note: "at most 4 textures, 62 vec4s of uniforms and 4 attributes of its own; a render target's texture is white; onBeforeCompile isn't read" },
  RawShaderMaterial: { status: "drawn", raea: "ThreeShaderMaterial: its GLSL translated by naga and run with three's built-ins", note: "at most 4 textures, 62 vec4s of uniforms and 4 attributes of its own; a render target's texture is white; onBeforeCompile isn't read" },
  Material: { status: "none", raea: "the base class" },
  // three/webgpu's node materials (TSL): one with no node of its own is the classic material it extends, drawn natively;
  // one with a graph (colorNode, positionNode, ...) on a mesh is compiled to WGSL by three's own WGSLNodeBuilder and run
  // by raea's engine (tsl.ts: ThreeShaderMaterial), lit by three's light nodes when it is a lit material
  ...Object.fromEntries(["NodeMaterial", "MeshBasicNodeMaterial", "MeshLambertNodeMaterial", "MeshPhongNodeMaterial", "MeshStandardNodeMaterial", "MeshPhysicalNodeMaterial", "MeshSSSNodeMaterial", "MeshToonNodeMaterial", "MeshMatcapNodeMaterial", "MeshNormalNodeMaterial", "VolumeNodeMaterial"].map((k) => [k, { status: "drawn" as const, raea: "its classic material, or its graph as WGSL (ThreeShaderMaterial)", note: "a graph needing more than 4 textures, 62 vec4s of uniforms (uniformArrays included) or 4 attributes of the mesh's own, a storage buffer or a render target is named and drawn flat" }])),
  ...Object.fromEntries(["LineBasicNodeMaterial", "LineDashedNodeMaterial", "Line2NodeMaterial", "PointsNodeMaterial", "SpriteNodeMaterial", "ShadowNodeMaterial"].map((k) => [k, { status: "near" as const, raea: "its classic material (lines, points, a sprite, a shadow catcher)", note: "a graph of its own on lines, points or a sprite is drawn flat" }])),
  // helpers: lines (and a plane's mesh), drawn as any lines are
  ...Object.fromEntries(["ArrowHelper", "AxesHelper", "Box3Helper", "BoxHelper", "CameraHelper", "DirectionalLightHelper", "GridHelper", "HemisphereLightHelper", "PlaneHelper", "PointLightHelper", "PolarGridHelper", "SkeletonHelper", "SpotLightHelper"].map((k) => [k, { status: "drawn" as const, raea: "its lines as Gizmo lines (an arrow's head and a plane's face as meshes)" }])),
};

/** @react-three/postprocessing's effects (pmndrs postprocessing's classes) inside an <EffectComposer>, and the raea
 * effect each becomes (post.ts). A composer's frame is tone-mapped only by its ToneMapping effect. */
export const EFFECTS: Record<string, { status: InputStatus; raea: string; note?: string }> = {
  BloomEffect: { status: "near", raea: "Bloom (intensity, luminanceThreshold as threshold)", note: "Bevy's energy-conserving bloom, not postprocessing's mipmap blur: a glow of its own size" },
  SelectiveBloomEffect: { status: "near", raea: "Bloom", note: "the whole frame's: its selection isn't kept" },
  VignetteEffect: { status: "near", raea: "Grade vignette (darkness as intensity, offset as radius)", note: "raea's falloff is softer at the corners" },
  ChromaticAberrationEffect: { status: "near", raea: "Grade aberration (the offset's length)" },
  HueSaturationEffect: { status: "near", raea: "Grade saturation (1 + saturation)", note: "hue isn't turned" },
  BrightnessContrastEffect: { status: "near", raea: "Grade midtones (contrast, brightness as lift)" },
  ToneMappingEffect: { status: "drawn", raea: "ToneMapping (its mode; ACES with three's 1/0.6 as exposure)" },
  DepthOfFieldEffect: { status: "near", raea: "DepthOfField (gaussian, worldFocusDistance as focus, bokehScale as its blur)" },
  BokehEffect: { status: "near", raea: "DepthOfField (bokeh)", note: "postprocessing's legacy bokeh, its focus as raea's" },
  RealisticBokehEffect: { status: "near", raea: "DepthOfField (bokeh)", note: "postprocessing's legacy bokeh, its focus as raea's" },
  GammaCorrectionEffect: { status: "none", raea: "left out: raea's output is already encoded for the screen" },
  SMAAEffect: { status: "drawn", raea: "AntiAliasing smaa" },
  FXAAEffect: { status: "drawn", raea: "AntiAliasing fxaa" },
  SSAOEffect: { status: "near", raea: "AmbientOcclusion (Bevy's SSAO)" },
  N8AOPostPass: { status: "near", raea: "AmbientOcclusion (Bevy's SSAO)" },
  PixelationEffect: { status: "near", raea: "Look pixel (granularity / 5 as the cell)" },
  LensDistortionEffect: { status: "near", raea: "LensDistortion (distortion.x as k1)" },
  ...Object.fromEntries(["NoiseEffect", "GlitchEffect", "ScanlineEffect", "SepiaEffect", "DotScreenEffect", "ColorAverageEffect", "ColorDepthEffect", "GridEffect", "GodRaysEffect", "ShockWaveEffect", "TiltShiftEffect", "LUT3DEffect", "LUT1DEffect", "LUTEffect", "TextureEffect", "DepthEffect", "OutlineEffect", "ASCIIEffect"].map((k) => [k, { status: "none" as const, raea: "left out, named once", note: "write it as a raea <FramePass> (WESL) outside <R3F>" }])),
};
