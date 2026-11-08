// Your own Material (docs/control.feature FC2a): Bevy's Material, its shaders the author's files compiled as written.
import { ExtendedMaterial, ShaderExtension, StandardMaterial, type AlphaMode } from "./materials";
import { surfaceShader, type SurfaceShaderSpec } from "../3d";

/** Your own Material (docs/control.feature FC2a): Bevy's `impl Material`, not an extension of StandardMaterial. Its
 *  fragment_shader() (and vertex_shader(), when `vertex` names one) is the file compiled as written: its imports (a
 *  `super::NAME` one reads NAME.wesl beside it), its own @fragment and its own `@group(MATERIAL_BIND_GROUP)` bindings,
 *  up to 8 uniforms of any type (a struct's from { field: value }), 4 textures and their samplers, in its layout. A
 *  uniform or texture it doesn't declare is refused. `alpha_mode` and `depth_bias` are its Material methods', and
 *  `cull_mode` the faces its specialize culls (the pipeline's by default: Back in 3D, none in 2D; null culls none). In
 *  a Mesh2d it is Bevy's Material2d (its group 2), in place of the ColorMaterial.
 *  @bevy bevy_pbr::material::Material */
export interface MaterialProps {
  /** the shader file, staticFile("shaders/x.wesl"), as Bevy's Material::fragment_shader names one */
  fragment: string;
  /** a vertex shader file, as Material::vertex_shader names one (Bevy's mesh vertex shader when not given) */
  vertex?: string;
  uniforms?: Record<string, number | number[] | string | Record<string, number | number[] | string>>;
  textures?: SurfaceShaderSpec["textures"];
  defs?: SurfaceShaderSpec["defs"];
  alpha_mode?: AlphaMode;
  depth_bias?: number;
  cull_mode?: "Front" | "Back" | null;
}

const ALPHA = ["Opaque", "Blend", "Premultiplied", "Add", "Multiply", "AlphaToCoverage"];

/** A Material's props checked, as the shader spec it is drawn by (in a Mesh3d an OwnMaterial, in a Mesh2d its
 *  Material2d). */
function ownSpec({ fragment, vertex, uniforms, textures, defs, alpha_mode, depth_bias, cull_mode }: MaterialProps): SurfaceShaderSpec {
  if (typeof fragment !== "string" || !fragment) throw new TypeError(`Material: fragment is the shader file, staticFile("shaders/x.wesl"), got ${JSON.stringify(fragment)}`);
  if (vertex !== undefined && (typeof vertex !== "string" || !vertex)) throw new TypeError(`Material: vertex is a shader file, got ${JSON.stringify(vertex)}`);
  const mask = typeof alpha_mode === "object" && alpha_mode !== null && Number.isFinite((alpha_mode as { Mask?: number }).Mask);
  if (alpha_mode !== undefined && !mask && !ALPHA.includes(alpha_mode as string)) throw new TypeError(`Material: alpha_mode is ${ALPHA.join(", ")} or { Mask: cutoff }, got ${JSON.stringify(alpha_mode)}`);
  if (depth_bias !== undefined && !Number.isFinite(depth_bias)) throw new TypeError(`Material: depth_bias is a number, got ${depth_bias}`);
  if (cull_mode !== undefined && cull_mode !== null && cull_mode !== "Front" && cull_mode !== "Back") throw new TypeError(`Material: cull_mode is "Front", "Back" or null, got ${JSON.stringify(cull_mode)}`);
  const own = { ...(vertex !== undefined ? { vertex } : {}), ...(alpha_mode !== undefined ? { alpha_mode } : {}), ...(depth_bias !== undefined ? { depth_bias } : {}), ...(cull_mode !== undefined ? { cull_mode } : {}) };
  return { fragment, uniforms: uniforms as SurfaceShaderSpec["uniforms"], textures, defs, own };
}

export function Material(p: MaterialProps) {
  // drawn by the engine as raea-sys-shader's OwnMaterial; the StandardMaterial is only what the rows carry it on
  return <ExtendedMaterial base={<StandardMaterial />} extension={<ShaderExtension {...ownSpec(p)} />} />;
}

/** A Material in a Mesh2d: the spec its Shape2D carries as `shader` (drawn as its own Material2d). */
export function materialShader(p: MaterialProps): Record<string, unknown> {
  return surfaceShader("Material", ownSpec(p));
}
