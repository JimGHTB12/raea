// <Compute>: a compute shader written here, run once a frame before the cameras draw (docs/shaders.feature SH10).
import { node3d, shaderDefs, type ShaderDefs } from "./all";
import { assetPath, samplerSpec, type SamplerSpec } from "./materials";

/** A compute shader (raea-sys-compute). Its source has one `@compute` entry point and reads `time()` (seconds),
 * `u_NAME()` for each uniform, each buffer as `NAME: array<f32>` (read and written) and each image as
 * `NAME: texture_storage_2d<rgba8unorm, write>`. */
export interface ComputeProps {
  /** a shader file compiled as written, staticFile("shaders/x.wesl") (docs/control.feature FC2d), in place of wesl: its
   * `@group(0)` storage textures and uniforms its own, as a Bevy example's render-graph node binds them */
  file?: string;
  /** a file's entry point this frame (null or left out: nothing dispatched), as a node picks its pipeline */
  entry?: string | null;
  /** a file's storage textures this frame, each an image's key ({ input: "a", output: "b" }), as a node picks its bind
   * group (a ping-pong swaps them frame by frame) */
  bind?: Record<string, string>;
  /** the source in WESL (a superset of WGSL) */
  wesl?: string;
  /** the same in plain WGSL */
  wgsl?: string;
  /** how many workgroups a frame dispatches: [x], [x, y] or [x, y, z] */
  workgroups: number[];
  /** up to 16, as a surface shader's */
  uniforms?: Record<string, number | number[] | string | Record<string, number | number[] | string>>;
  /** up to 4, the compute's own, kept from frame to frame: { size } (that many f32s, zeroed) or its first contents */
  buffers?: Record<string, { size: number } | number[]>;
  /** up to 4 "render:NAME" (or "render:NAME:WxH") images it writes, which any shader's texture reads by that name (a
   * file's may be { src, sampler }: the sampler it is drawn with) */
  images?: Record<string, string | { src: string; sampler?: SamplerSpec }>;
  /** shader defs, as a surface shader's */
  defs?: ShaderDefs;
}

function checkWorkgroups(workgroups: number[]) {
  if (!Array.isArray(workgroups) || workgroups.length < 1 || workgroups.length > 3 || !workgroups.every((n) => Number.isInteger(n) && n >= 1 && n <= 65535)) throw new TypeError(`Compute: workgroups are [x, y, z], each 1 to 65535, got ${JSON.stringify(workgroups)}`);
}

/** A compute as written (raea-sys-compute written.rs): its file, images by key, and this frame's entry and bind. */
function written(props: ComputeProps) {
  const { file, entry = null, bind = {}, workgroups, uniforms = {}, images = {}, defs } = props;
  if (typeof file !== "string" || !file) throw new TypeError(`Compute: file is a shader file, staticFile("shaders/x.wesl"), got ${JSON.stringify(file)}`);
  if (props.wesl !== undefined || props.wgsl !== undefined || props.buffers !== undefined) throw new TypeError("Compute: a file is its own source and bindings: no wesl, wgsl or buffers beside it");
  checkWorkgroups(workgroups);
  for (const [k, im] of Object.entries(images)) {
    const src = typeof im === "object" && im !== null ? im.src : im;
    if (typeof src !== "string" || !src.startsWith("render:")) throw new TypeError(`Compute: image ${k} is a "render:NAME" (or "render:NAME:WxH"), or { src, sampler }, got ${JSON.stringify(im)}`);
    if (typeof im === "object" && im.sampler !== undefined) samplerSpec(`Compute: image ${k}'s sampler`, im.sampler);
  }
  if (entry !== null && (typeof entry !== "string" || !IDENT.test(entry))) throw new TypeError(`Compute: entry is an entry point's name or null, got ${JSON.stringify(entry)}`);
  for (const [k, key] of Object.entries(bind)) if (typeof key !== "string" || !(key in images)) throw new TypeError(`Compute: bind ${k} is one of its images' keys (${Object.keys(images).join(", ")}), got ${JSON.stringify(key)}`);
  const shader = { file: assetPath(file), workgroups, images, uniforms, ...(defs !== undefined ? { defs: shaderDefs("Compute", defs) } : {}) };
  return node3d("Compute", { shader, entry: entry ?? "", bind: JSON.stringify(bind) });
}

const IDENT = /^[A-Za-z_][A-Za-z0-9_]*$/;

/** A compute over the frame, in the 3D beside the camera; several run in the order written, before the cameras draw.
 * @bevy raea_sys_compute::Computes */
export function Compute(props: ComputeProps) {
  const { wesl, wgsl, workgroups, uniforms = {}, buffers = {}, images = {}, defs } = props;
  if (props.file !== undefined) return written(props);
  if ((wesl === undefined) === (wgsl === undefined)) throw new TypeError("Compute: give its source as wesl (or wgsl), one of them");
  const src = wesl ?? wgsl;
  if (typeof src !== "string" || (src.match(/@compute/g) ?? []).length !== 1) throw new TypeError("Compute: its source has one @compute entry point");
  checkWorkgroups(workgroups);
  if (Object.keys(uniforms).length > 16) throw new RangeError("Compute: at most 16 uniforms");
  const bs = Object.entries(buffers);
  if (bs.length > 4) throw new RangeError(`Compute: at most 4 buffers, got ${bs.length}`);
  for (const [k, b] of bs) {
    if (!IDENT.test(k)) throw new TypeError(`Compute: buffer ${JSON.stringify(k)} isn't a WGSL name`);
    const ok = Array.isArray(b) ? b.length > 0 && b.every(Number.isFinite) : typeof b === "object" && b !== null && Number.isInteger(b.size) && b.size >= 1;
    if (!ok) throw new TypeError(`Compute: buffer ${k} is { size } or its first contents (numbers), got ${JSON.stringify(b)}`);
  }
  const is = Object.entries(images);
  if (is.length > 4) throw new RangeError(`Compute: at most 4 images, got ${is.length}`);
  for (const [k, im] of is) {
    if (!IDENT.test(k)) throw new TypeError(`Compute: image ${JSON.stringify(k)} isn't a WGSL name`);
    if (typeof im !== "string" || !im.startsWith("render:")) throw new TypeError(`Compute: image ${k} is a "render:NAME", got ${JSON.stringify(im)}`);
  }
  const shader = { ...(wesl !== undefined ? { wesl } : { wgsl }), workgroups, uniforms, buffers, images, ...(defs !== undefined ? { defs: shaderDefs("Compute", defs) } : {}) };
  return node3d("Compute", { shader });
}
