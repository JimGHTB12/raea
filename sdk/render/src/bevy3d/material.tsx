// Your own Material (docs/control.feature FC2a): Bevy's Material with its fragment_shader(), a file compiled as written.
import { ExtendedMaterial, ShaderExtension, StandardMaterial } from "./materials";
import type { SurfaceShaderSpec } from "../3d";

/** Your own Material (docs/control.feature FC2a, its first step): Bevy's `impl Material` with a fragment_shader(), the
 *  file compiled as written (its imports, its own @fragment, its own @binding(100) uniform struct if it has one, read
 *  from `uniforms`). Today it draws as an ExtendedMaterial whose own fragment replaces the surface, the base unused.
 *  @bevy bevy_pbr::material::Material */
export interface MaterialProps {
  /** the shader file, staticFile("shaders/x.wesl"), as Bevy's Material::fragment_shader names one */
  fragment: string;
  uniforms?: SurfaceShaderSpec["uniforms"];
  textures?: SurfaceShaderSpec["textures"];
  defs?: SurfaceShaderSpec["defs"];
}

export function Material({ fragment, uniforms, textures, defs }: MaterialProps) {
  if (typeof fragment !== "string" || !fragment) throw new TypeError(`Material: fragment is the shader file, staticFile("shaders/x.wesl"), got ${JSON.stringify(fragment)}`);
  return <ExtendedMaterial base={<StandardMaterial />} extension={<ShaderExtension fragment={fragment} uniforms={uniforms} textures={textures} defs={defs} />} />;
}
