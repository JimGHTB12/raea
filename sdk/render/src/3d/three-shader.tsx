// A three.js shader on a mesh (docs/r3f.feature R3F11): a ShaderMaterial's GLSL or a node material's WGSL, drawn by
// raea's engine with three's built-ins (raea-sys-three-shader). <R3F> makes one from a ShaderMaterial,
// RawShaderMaterial or node material in an R3F scene; written by hand, it runs a three.js shader as it is:
//   <Mesh><Sphere /><ThreeShaderMaterial glsl={{ vertex, fragment }} uniforms={{ uTime: t }} /></Mesh>
import { node3d } from "./all";

/** A uniform's value: a number, a boolean, or its numbers in order (a vector's, a matrix's column by column as three's
 * `elements`, an array's elements one after another) */
export type ThreeUniform = number | boolean | number[];

export interface ThreeShaderMaterialProps {
  /** a ShaderMaterial's GLSL, its `#include`s expanded and its `defines` as #define lines in front; `raw` for a
   * RawShaderMaterial (which declares three's built-ins itself), `vertexColors` as the material's */
  glsl?: { vertex: string; fragment: string; raw?: boolean; vertexColors?: boolean };
  /** a node material's WGSL, as three's TSL builder writes it for raea (sdk/r3f's input/tsl.ts): the contract's
   * `three_vertex` and `three_fragment`, its uniforms' types, the mesh attributes it reads beyond the standard four */
  wgsl?: { vertex: string; fragment: string; uniforms?: { name: string; type: string; array?: number }[]; attributes?: string[] };
  /** each uniform's value this frame */
  uniforms?: Record<string, ThreeUniform>;
  /** each sampler's picture: a file, a data URL, "render:NAME" (at most four); `colorSpace: "linear"` for one three
   * samples raw (NoColorSpace: a distance field, a data map), "srgb" (the default) for a colour picture */
  textures?: Record<string, string | { src: string; colorSpace?: "srgb" | "linear"; sampler?: Record<string, unknown> }>;
  /** "display": its colour is what three's canvas shows (a ShaderMaterial's); "linear": scene light, tone-mapped as
   * the rest of the frame is (a node material's) */
  output?: "display" | "linear";
  /** blended over what's behind it, as three's \`transparent\` */
  transparent?: boolean;
  /** how it blends: normal, additive (add), multiply or none */
  blending?: "normal" | "additive" | "add" | "multiply" | "none";
  /** which faces draw: front, back or double */
  side?: "front" | "back" | "double";
  /** it writes the depth buffer */
  depthWrite?: boolean;
  /** it's hidden behind what's nearer */
  depthTest?: boolean;
  /** pixels less opaque than this are dropped */
  alphaTest?: number;
  /** three's clock this frame: [seconds, delta, frame] */
  time?: [number, number, number];
  /** the colour it's drawn in where its shader can't run (named in the render's warnings) */
  color?: string;
}

/** A three.js shader on the mesh it's in, as it is: a ShaderMaterial's GLSL (or a node material's WGSL) with its
 * uniforms, drawn by raea's engine with three's built-ins. `<R3F>` makes these from an R3F scene's materials. */
export function ThreeShaderMaterial(props: ThreeShaderMaterialProps) {
  const { glsl, wgsl } = props;
  if (!glsl === !wgsl) throw new TypeError("ThreeShaderMaterial: give glsl {vertex, fragment} or wgsl {vertex, fragment}, one of them");
  const src = (glsl ?? wgsl)!;
  if (typeof src.vertex !== "string" || typeof src.fragment !== "string") throw new TypeError("ThreeShaderMaterial: its vertex and fragment are source strings");
  const textures = Object.keys(props.textures ?? {});
  if (textures.length > 4) throw new RangeError(`ThreeShaderMaterial: at most 4 textures, got ${textures.length}`);
  for (const [k, v] of Object.entries(props.uniforms ?? {})) {
    const ok = typeof v === "boolean" || (typeof v === "number" && Number.isFinite(v)) || (Array.isArray(v) && v.every((x) => typeof x === "number" && Number.isFinite(x)));
    if (!ok) throw new TypeError(`ThreeShaderMaterial: uniform ${k} is a number, a boolean or a list of numbers, got ${JSON.stringify(v)}`);
  }
  return node3d("ThreeShaderMaterial", props as unknown as Record<string, unknown>);
}
