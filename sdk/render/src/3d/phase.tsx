// <PhaseItem>: your own draw in the camera's opaque phase (docs/control.feature FC2d), as Bevy's
// shader_advanced/custom_phase_item queues one: raea-sys-phase's PhaseItem on an <Entity>, with its Aabb.
import { BevyComponent, Entity } from "./all";
import { assetPath } from "./materials";

/** A vertex attribute: Bevy's VertexFormat by name, its byte offset in a vertex and its shader @location. */
export interface PhaseAttribute { format: "Float32" | "Float32x2" | "Float32x3" | "Float32x4" | "Uint32" | "Uint32x2" | "Uint32x3" | "Uint32x4" | "Sint32" | "Sint32x2" | "Sint32x3" | "Sint32x4" | "Unorm8x4"; offset: number; location: number }

export interface PhaseItemProps {
  /** the shader file, compiled as written: staticFile("shaders/x.wesl") */
  shader: string;
  /** its entry points (Bevy's default names) */
  vertex?: string;
  fragment?: string;
  /** the vertex buffer: each vertex `stride` bytes, its 32-bit words in order (a u32 pad of 0 is a 0) */
  stride: number;
  attributes: PhaseAttribute[];
  words: number[];
  /** the index buffer (u32), drawn whole */
  indices: number[];
  /** its depth state: whether it writes depth, and its compare (Bevy's CompareFunction by name; Bevy's opaque
   * default, Greater, when left out) */
  depth?: { write?: boolean; compare?: "Never" | "Less" | "Equal" | "LessEqual" | "Greater" | "NotEqual" | "GreaterEqual" | "Always" };
  /** its bounds, so the camera sees it (Bevy's Aabb): a centre and half extents */
  aabb: { center: [number, number, number]; half_extents: [number, number, number] };
  /** where it is (a bare Entity's place) */
  position?: [number, number, number];
}

/** Your own pipeline drawing your own buffers in each camera's Opaque3d phase.
 * @bevy raea_sys_phase::PhaseItem */
export function PhaseItem(p: PhaseItemProps) {
  const { shader, vertex = "vertex", fragment = "fragment", stride, attributes, words, indices, depth = {}, aabb, position } = p;
  if (typeof shader !== "string" || !shader) throw new TypeError(`PhaseItem: shader is a shader file, staticFile("shaders/x.wesl"), got ${JSON.stringify(shader)}`);
  if (!Number.isInteger(stride) || stride < 4 || stride % 4 !== 0) throw new TypeError(`PhaseItem: stride is a vertex's bytes, a multiple of 4, got ${stride}`);
  if (!Array.isArray(words) || words.length % (stride / 4) !== 0 || !words.every(Number.isFinite)) throw new TypeError(`PhaseItem: words are each vertex's ${stride / 4} numbers, got ${words?.length}`);
  const n = words.length / (stride / 4);
  if (!Array.isArray(indices) || !indices.length || !indices.every((i) => Number.isInteger(i) && i >= 0 && i < n)) throw new TypeError(`PhaseItem: indices are vertices 0 to ${n - 1}, got ${JSON.stringify(indices)}`);
  for (const a of attributes) if (!(Number.isInteger(a.offset) && a.offset >= 0 && a.offset < stride && Number.isInteger(a.location) && a.location >= 0)) throw new TypeError(`PhaseItem: an attribute is { format, offset (under the stride), location }, got ${JSON.stringify(a)}`);
  const value = { shader: assetPath(shader), vertex, fragment, stride, attributes, words, indices, depth_write: depth.write ?? true, depth_compare: depth.compare ?? "Greater" };
  return (
    <Entity {...(position !== undefined ? { position } : {})}>
      <BevyComponent type="raea_sys_phase::PhaseItem" value={value} />
      <BevyComponent type="bevy_camera::primitives::Aabb" value={{ center: aabb.center, half_extents: aabb.half_extents }} />
    </Entity>
  );
}

export interface MeshPassProps {
  /** the shader file, compiled as written: staticFile("shaders/x.wesl"). Its vertex input is the mesh's position at
   * location 0 (a vec4 `compressed_position` under VERTEX_POSITIONS_COMPRESSED), its bind groups Bevy's mesh view (0)
   * and mesh (2) */
  shader: string;
  /** its entry points (Bevy's default names) */
  vertex?: string;
  fragment?: string;
}

/** The mesh it's inside drawn again by your shader, in a phase of its own after the camera's main opaque pass: onto
 * the colour target, with no depth test, back faces culled (Bevy's shader_advanced/custom_render_phase).
 * @bevy raea_sys_phase::MeshPass */
export function MeshPass(p: MeshPassProps) {
  const { shader, vertex = "vertex", fragment = "fragment" } = p;
  if (typeof shader !== "string" || !shader) throw new TypeError(`MeshPass: shader is a shader file, staticFile("shaders/x.wesl"), got ${JSON.stringify(shader)}`);
  return <BevyComponent type="raea_sys_phase::MeshPass" value={{ shader: assetPath(shader), vertex, fragment }} />;
}

export interface ShaderInstancesProps {
  /** the shader file, compiled as written: staticFile("shaders/x.wesl"). Its vertex reads the mesh's position, normal
   * and uv at locations 0 to 2 and the instance's attributes at theirs; its bind groups are Bevy's mesh pipeline's */
  shader: string;
  /** each instance's bytes (a multiple of 4) */
  stride: number;
  /** what the shader reads of an instance: Bevy's VertexFormat, its byte offset and @location (3 and up) */
  attributes: PhaseAttribute[];
  /** every instance's 32-bit words in order, `stride / 4` of them each */
  words: number[];
}

/** The mesh it's inside drawn once per instance in one draw, by your shader, in the camera's Transparent3d phase, a
 * vertex buffer of your words stepped per instance (Bevy's shader_advanced/custom_shader_instancing). Its instances
 * aren't in the mesh's bounds: give the mesh Bevy's NoFrustumCulling (<BevyComponent type="bevy_camera::visibility::NoFrustumCulling" value={{}} />).
 * @bevy raea_sys_phase::ShaderInstances */
export function ShaderInstances(p: ShaderInstancesProps) {
  const { shader, stride, attributes, words } = p;
  if (typeof shader !== "string" || !shader) throw new TypeError(`ShaderInstances: shader is a shader file, staticFile("shaders/x.wesl"), got ${JSON.stringify(shader)}`);
  if (!Number.isInteger(stride) || stride < 4 || stride % 4 !== 0) throw new TypeError(`ShaderInstances: stride is an instance's bytes, a multiple of 4, got ${stride}`);
  if (!Array.isArray(words) || words.length % (stride / 4) !== 0 || !words.every(Number.isFinite)) throw new TypeError(`ShaderInstances: words are each instance's ${stride / 4} numbers, got ${words?.length}`);
  for (const a of attributes) if (!(Number.isInteger(a.offset) && a.offset >= 0 && a.offset < stride && Number.isInteger(a.location) && a.location >= 0)) throw new TypeError(`ShaderInstances: an attribute is { format, offset (under the stride), location }, got ${JSON.stringify(a)}`);
  return <BevyComponent type="raea_sys_phase::ShaderInstances" value={{ shader: assetPath(shader), stride, attributes, words }} />;
}

export interface GBufferPassProps {
  /** the shader file, compiled as written: staticFile("shaders/x.wesl"). Its entry points take Bevy's
   * FullscreenVertexOutput; bind group 0 is the view uniform (binding 0) and Globals (binding 1) */
  shader: string;
  /** the entry point writing the G-buffer: its deferred vec4<u32> (location 0), its lighting pass id u32 (location 1)
   * and its frag_depth ("fragment") */
  fragment?: string;
  /** the entry point writing a shadow view's frag_depth, so it casts shadows; none when left out */
  shadow?: string;
}

/** Inside a Camera3d with Bevy's DepthPrepass and DeferredPrepass: your shader over the whole view, written into the
 * deferred G-buffer after the deferred prepass (depth compared GreaterEqual), so Bevy's deferred lighting lights it as
 * any surface, and into the shadow maps of the camera's lights (Bevy's shader_advanced/deferred_raymarch).
 * @bevy raea_sys_phase::GBufferPass */
export function GBufferPass(p: GBufferPassProps) {
  const { shader, fragment = "fragment", shadow = "" } = p;
  if (typeof shader !== "string" || !shader) throw new TypeError(`GBufferPass: shader is a shader file, staticFile("shaders/x.wesl"), got ${JSON.stringify(shader)}`);
  return <BevyComponent type="raea_sys_phase::GBufferPass" value={{ shader: assetPath(shader), fragment, shadow }} />;
}
