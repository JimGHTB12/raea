// raea → React Three Fiber as source (docs/r3f.feature R3F9): a composition's frames baked by the three.js host and
// written as an R3F module (tools/export.ts is its command line).
import { readFileSync } from "node:fs";
import type { ComponentType } from "react";
import { loadShapes, shapesReady } from "../three/geometry";
import { emit } from "./emit";
import { record } from "./record";

export interface SourceOptions {
  width: number;
  height: number;
  fps: number;
  /** the first frame and how many (1: a still scene) */
  from?: number;
  frames?: number;
  /** the component's props */
  props?: Record<string, unknown>;
  /** the default export's name */
  name?: string;
  /** the header's title */
  title?: string;
  /** the shape module's script (tools/shape-wasm.sh builds sdk/r3f/wasm/raea_shape_web.js, the default): solids and
   * Text3D are meshed with it */
  shapes?: string;
}

export interface R3FSource {
  /** the TSX module: `Scene` for any <Canvas>, a default export that is a <Canvas> set up as raea's */
  source: string;
  /** raea-effects.ts, to write beside it, when the scene has frame effects or raea's Tony tone mapping: the composer
   * <Raea> draws them with, needing only three */
  effects?: string;
}

/** a raea composition's 3D as React Three Fiber source: TSX needing only react, three and @react-three/fiber */
export async function toR3FSource(component: ComponentType<Record<string, unknown>>, opts: SourceOptions): Promise<R3FSource> {
  if (!shapesReady()) {
    const shapes = opts.shapes ?? new URL("../../wasm/raea_shape_web.js", import.meta.url).pathname;
    await loadShapes(shapes).catch((e: unknown) => console.warn(`raea/r3f: the shape module (${shapes}) didn't load, so solids aren't in the export: ${String(e)}`));
  }
  const source = emit(record(component, opts), opts.name ?? "RaeaScene", opts.title ?? "a raea composition");
  return { source, effects: source.includes(`from "./raea-effects"`) ? effectsModule() : undefined };
}

/** three/effects.ts with its Tony table inline: one file, three's own imports only */
export function effectsModule(): string {
  const read = (f: string) => readFileSync(new URL(`../three/${f}`, import.meta.url), "utf8");
  const lut = read("tony-lut.ts").replace(/^export const/gm, "const");
  // the table goes where effects.ts imported it, after the imports, since the shaders use its size
  const body = read("effects.ts").replace(/^import \{ TONY_B64, TONY_N \} from "\.\/tony-lut";\n/m, () => lut + "\n");
  return `// raea's frame effects for three.js, copied from raea/r3f (sdk/r3f/src/three/effects.ts) by raea's source export.\n${body}`;
}
