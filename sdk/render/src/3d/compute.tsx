// <Compute>: a compute shader written here, run once a frame before the cameras draw (docs/shaders.feature SH10).
import { node3d, shaderDefs, type ShaderDefs } from "./all";

/** A compute shader (raea-sys-compute). Its source has one `@compute` entry point and reads `time()` (seconds),
 * `u_NAME()` for each uniform, each buffer as `NAME: array<f32>` (read and written) and each image as
 * `NAME: texture_storage_2d<rgba8unorm, write>`. */
export interface ComputeProps {
  /** the source in WESL (a superset of WGSL) */
  wesl?: string;
  /** the same in plain WGSL */
  wgsl?: string;
  /** how many workgroups a frame dispatches: [x], [x, y] or [x, y, z] */
  workgroups: number[];
  /** up to 16, as a surface shader's */
  uniforms?: Record<string, number | number[] | string>;
  /** up to 4, the compute's own, kept from frame to frame: { size } (that many f32s, zeroed) or its first contents */
  buffers?: Record<string, { size: number } | number[]>;
  /** up to 4 "render:NAME" (or "render:NAME:WxH") images it writes, which any shader's texture reads by that name */
  images?: Record<string, string>;
  /** shader defs, as a surface shader's */
  defs?: ShaderDefs;
}

const IDENT = /^[A-Za-z_][A-Za-z0-9_]*$/;

/** A compute over the frame, in the 3D beside the camera; several run in the order written, before the cameras draw.
 * @bevy raea_sys_compute::Computes */
export function Compute(props: ComputeProps) {
  const { wesl, wgsl, workgroups, uniforms = {}, buffers = {}, images = {}, defs } = props;
  if ((wesl === undefined) === (wgsl === undefined)) throw new TypeError("Compute: give its source as wesl (or wgsl), one of them");
  const src = wesl ?? wgsl;
  if (typeof src !== "string" || (src.match(/@compute/g) ?? []).length !== 1) throw new TypeError("Compute: its source has one @compute entry point");
  if (!Array.isArray(workgroups) || workgroups.length < 1 || workgroups.length > 3 || !workgroups.every((n) => Number.isInteger(n) && n >= 1 && n <= 65535)) throw new TypeError(`Compute: workgroups are [x, y, z], each 1 to 65535, got ${JSON.stringify(workgroups)}`);
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
